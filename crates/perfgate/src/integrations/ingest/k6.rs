//! Parser for k6 end-of-test summary JSON.
//!
//! k6 `--summary-export` and `handleSummary()` JSON are aggregate summaries,
//! not raw per-request sample streams. perfgate imports the clear HTTP/load
//! metrics into `perfgate.run.v1` while keeping the receipt summary-only and
//! preserving k6-specific context in benchmark metadata.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, bail};
use perfgate_types::{
    BenchMeta, F64Summary, HostInfo, RUN_SCHEMA_V1, RunMeta, RunReceipt, Sample, Stats, ToolInfo,
    U64Summary,
};
use serde::Deserialize;
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct K6Summary {
    #[serde(default)]
    metrics: BTreeMap<String, K6Metric>,
    #[serde(default)]
    options: Option<Value>,
    #[serde(default)]
    state: Option<K6State>,
}

#[derive(Debug, Deserialize)]
struct K6State {
    #[serde(default, rename = "testRunDurationMs")]
    test_run_duration_ms: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct K6Metric {
    #[serde(default, rename = "type")]
    metric_type: Option<String>,
    #[serde(default)]
    contains: Option<String>,
    #[serde(default)]
    values: BTreeMap<String, f64>,
}

/// Parse k6 end-of-test summary JSON into a summary-only `RunReceipt`.
///
/// The adapter maps `http_req_duration` to lower-is-better `wall_ms` and
/// `http_reqs.rate` to higher-is-better `throughput_per_s` when present.
/// k6 error-rate and scenario metadata are preserved in `bench.command` because
/// `perfgate.run.v1` has no dedicated error-rate or scenario fields.
pub fn parse_k6_summary_json(input: &str, name: Option<&str>) -> anyhow::Result<RunReceipt> {
    let summary: K6Summary =
        serde_json::from_str(input).context("failed to parse k6 summary JSON")?;

    if summary.metrics.is_empty() {
        bail!("k6 summary JSON requires a non-empty metrics object");
    }

    let time_unit = summary_time_unit(summary.options.as_ref())?;
    let (latency_metric_name, latency_metric) = find_metric(&summary.metrics, "http_req_duration")
        .or_else(|| find_metric(&summary.metrics, "iteration_duration"))
        .context("k6 summary JSON requires http_req_duration or iteration_duration trend values")?;

    validate_metric_kind(
        latency_metric,
        latency_metric_name,
        Some("trend"),
        Some("time"),
    )?;
    let wall_ms = trend_wall_summary(latency_metric_name, latency_metric, time_unit)?;

    let throughput_metric = find_metric(&summary.metrics, "http_reqs")
        .or_else(|| find_metric(&summary.metrics, "iterations"));
    let throughput_per_s = throughput_metric
        .map(|(metric_name, metric)| {
            validate_metric_kind(metric, metric_name, Some("counter"), None)?;
            rate_summary(metric_name, metric)
        })
        .transpose()?;

    let request_count = throughput_metric.and_then(|(_, metric)| metric.values.get("count"));
    let repeat = request_count
        .copied()
        .and_then(f64_to_u32)
        .or_else(|| {
            latency_metric
                .values
                .get("count")
                .copied()
                .and_then(f64_to_u32)
        })
        .unwrap_or(0);
    let work_units = request_count.copied().and_then(f64_to_u64);

    let bench_name = name
        .map(str::to_string)
        .unwrap_or_else(|| "k6-http-summary".to_string());

    let command = k6_metadata_command(
        latency_metric_name,
        throughput_metric.map(|(name, _)| name),
        find_metric(&summary.metrics, "http_req_failed").map(|(_, metric)| metric),
        &summary.metrics,
        summary.state.as_ref(),
        time_unit,
    )?;

    let now = OffsetDateTime::now_utc();
    let timestamp = now
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string());

    Ok(RunReceipt {
        schema: RUN_SCHEMA_V1.to_string(),
        tool: ToolInfo {
            name: "perfgate-ingest".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
        run: RunMeta {
            id: Uuid::new_v4().to_string(),
            started_at: timestamp.clone(),
            ended_at: timestamp,
            host: HostInfo {
                os: "unknown".to_string(),
                arch: "unknown".to_string(),
                cpu_count: None,
                memory_bytes: None,
                hostname_hash: None,
            },
        },
        bench: BenchMeta {
            name: bench_name,
            cwd: None,
            command,
            repeat,
            warmup: 0,
            work_units,
            timeout_ms: None,
        },
        samples: Vec::<Sample>::new(),
        stats: Stats {
            wall_ms,
            cpu_ms: None,
            page_faults: None,
            ctx_switches: None,
            max_rss_kb: None,
            io_read_bytes: None,
            io_write_bytes: None,
            network_packets: None,
            energy_uj: None,
            binary_bytes: None,
            throughput_per_s,
        },
    })
}

fn find_metric<'a>(
    metrics: &'a BTreeMap<String, K6Metric>,
    base_name: &str,
) -> Option<(&'a str, &'a K6Metric)> {
    metrics
        .get_key_value(base_name)
        .map(|(name, metric)| (name.as_str(), metric))
        .or_else(|| {
            let prefix = format!("{base_name}{{");
            metrics
                .iter()
                .find(|(name, _)| name.starts_with(&prefix))
                .map(|(name, metric)| (name.as_str(), metric))
        })
}

fn validate_metric_kind(
    metric: &K6Metric,
    name: &str,
    expected_type: Option<&str>,
    expected_contains: Option<&str>,
) -> anyhow::Result<()> {
    if let Some(expected_type) = expected_type
        && metric
            .metric_type
            .as_deref()
            .is_some_and(|actual| !actual.eq_ignore_ascii_case(expected_type))
    {
        bail!(
            "k6 metric '{name}' has type '{}' but expected '{expected_type}'",
            metric.metric_type.as_deref().unwrap_or_default()
        );
    }

    if let Some(expected_contains) = expected_contains
        && metric
            .contains
            .as_deref()
            .is_some_and(|actual| !actual.eq_ignore_ascii_case(expected_contains))
    {
        bail!(
            "k6 metric '{name}' contains '{}' but expected '{expected_contains}'",
            metric.contains.as_deref().unwrap_or_default()
        );
    }

    Ok(())
}

fn trend_wall_summary(
    name: &str,
    metric: &K6Metric,
    time_unit: TimeUnit,
) -> anyhow::Result<U64Summary> {
    let median = value_any(&metric.values, &["med", "median", "p(50)", "avg"])
        .with_context(|| format!("k6 trend metric '{name}' requires med or avg"))?;
    let min = value_any(&metric.values, &["min"])
        .with_context(|| format!("k6 trend metric '{name}' requires min"))?;
    let max = value_any(&metric.values, &["max"])
        .with_context(|| format!("k6 trend metric '{name}' requires max"))?;
    let mean = value_any(&metric.values, &["avg", "mean"]);
    let stddev = value_any(&metric.values, &["stddev", "std_dev"]);

    let median = time_unit.to_ms(median)?;
    let min = time_unit.to_ms(min)?;
    let max = time_unit.to_ms(max)?;
    if min > max {
        bail!("k6 trend metric '{name}' min must be <= max");
    }

    Ok(U64Summary {
        median: f64_to_u64_checked(median, name)?,
        min: f64_to_u64_checked(min, name)?,
        max: f64_to_u64_checked(max, name)?,
        mean: mean.map(|value| time_unit.to_ms(value)).transpose()?,
        stddev: stddev.map(|value| time_unit.to_ms(value)).transpose()?,
    })
}

fn rate_summary(name: &str, metric: &K6Metric) -> anyhow::Result<F64Summary> {
    let rate = *metric
        .values
        .get("rate")
        .with_context(|| format!("k6 counter metric '{name}' requires rate"))?;
    validate_non_negative_finite(rate, name)?;
    Ok(F64Summary {
        median: rate,
        min: rate,
        max: rate,
        mean: Some(rate),
        stddev: None,
    })
}

fn k6_metadata_command(
    latency_metric: &str,
    throughput_metric: Option<&str>,
    error_metric: Option<&K6Metric>,
    metrics: &BTreeMap<String, K6Metric>,
    state: Option<&K6State>,
    time_unit: TimeUnit,
) -> anyhow::Result<Vec<String>> {
    let mut command = vec![
        "(ingested k6 summary JSON)".to_string(),
        format!("latency_metric={latency_metric}"),
        format!("summary_time_unit={}", time_unit.label()),
    ];

    if let Some(throughput_metric) = throughput_metric {
        command.push(format!("throughput_metric={throughput_metric}"));
    }

    if let Some(error_metric) = error_metric {
        validate_metric_kind(error_metric, "http_req_failed", Some("rate"), None)?;
        if let Some(rate) = error_metric.values.get("rate").copied() {
            validate_non_negative_finite(rate, "http_req_failed.rate")?;
            command.push(format!("http_req_failed_rate={rate:.6}"));
        }
        if let Some(fails) = error_metric
            .values
            .get("fails")
            .copied()
            .and_then(f64_to_u64)
        {
            command.push(format!("http_req_failed_fails={fails}"));
        }
        if let Some(passes) = error_metric
            .values
            .get("passes")
            .copied()
            .and_then(f64_to_u64)
        {
            command.push(format!("http_req_failed_passes={passes}"));
        }
    }

    for scenario in scenario_labels(metrics) {
        command.push(format!("scenario={scenario}"));
    }

    if let Some(duration_ms) = state.and_then(|state| state.test_run_duration_ms) {
        validate_non_negative_finite(duration_ms, "state.testRunDurationMs")?;
        command.push(format!("test_run_duration_ms={duration_ms:.0}"));
    }

    command.push("sample_model=summary_only".to_string());
    command.push("capacity_proof=not_production".to_string());
    Ok(command)
}

fn scenario_labels(metrics: &BTreeMap<String, K6Metric>) -> Vec<String> {
    let mut labels = BTreeSet::new();
    for metric_name in metrics.keys() {
        if let Some(tags) = metric_tags(metric_name) {
            for raw_tag in tags.split(',') {
                let tag = raw_tag.trim();
                let Some((key, value)) = split_tag(tag) else {
                    continue;
                };
                if key.trim().eq_ignore_ascii_case("scenario") {
                    let value = value.trim().trim_matches('"').trim_matches('\'');
                    if !value.is_empty() {
                        labels.insert(value.to_string());
                    }
                }
            }
        }
    }
    labels.into_iter().collect()
}

fn metric_tags(metric_name: &str) -> Option<&str> {
    let start = metric_name.find('{')?;
    let end = metric_name.rfind('}')?;
    if end <= start {
        return None;
    }
    Some(&metric_name[start + 1..end])
}

fn split_tag(tag: &str) -> Option<(&str, &str)> {
    tag.find(':')
        .or_else(|| tag.find('='))
        .map(|index| (&tag[..index], &tag[index + 1..]))
}

fn value_any(values: &BTreeMap<String, f64>, names: &[&str]) -> Option<f64> {
    names.iter().find_map(|name| values.get(*name).copied())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TimeUnit {
    Milliseconds,
    Seconds,
    Microseconds,
}

impl TimeUnit {
    fn to_ms(self, value: f64) -> anyhow::Result<f64> {
        validate_non_negative_finite(value, "k6 time value")?;
        Ok(match self {
            TimeUnit::Milliseconds => value,
            TimeUnit::Seconds => value * 1000.0,
            TimeUnit::Microseconds => value / 1000.0,
        })
    }

    fn label(self) -> &'static str {
        match self {
            TimeUnit::Milliseconds => "ms",
            TimeUnit::Seconds => "s",
            TimeUnit::Microseconds => "us",
        }
    }
}

fn summary_time_unit(options: Option<&Value>) -> anyhow::Result<TimeUnit> {
    let Some(options) = options else {
        return Ok(TimeUnit::Milliseconds);
    };
    let Some(unit) = options
        .get("summaryTimeUnit")
        .or_else(|| options.get("summary_time_unit"))
    else {
        return Ok(TimeUnit::Milliseconds);
    };
    if unit.is_null() {
        return Ok(TimeUnit::Milliseconds);
    }
    let unit = unit
        .as_str()
        .context("k6 options.summaryTimeUnit must be a string when present")?;
    match unit.trim().to_ascii_lowercase().as_str() {
        "" | "ms" | "millisecond" | "milliseconds" => Ok(TimeUnit::Milliseconds),
        "s" | "sec" | "second" | "seconds" => Ok(TimeUnit::Seconds),
        "us" | "microsecond" | "microseconds" => Ok(TimeUnit::Microseconds),
        other => bail!("unsupported k6 summaryTimeUnit '{other}'; expected ms, s, or us"),
    }
}

fn validate_non_negative_finite(value: f64, field: &str) -> anyhow::Result<()> {
    if !value.is_finite() || value < 0.0 {
        bail!("k6 field '{field}' must be finite and non-negative");
    }
    Ok(())
}

fn f64_to_u64(value: f64) -> Option<u64> {
    if !value.is_finite() || value < 0.0 || value > u64::MAX as f64 {
        return None;
    }
    Some(value.round() as u64)
}

fn f64_to_u32(value: f64) -> Option<u32> {
    if !value.is_finite() || value < 0.0 || value > u32::MAX as f64 {
        return None;
    }
    Some(value.round() as u32)
}

fn f64_to_u64_checked(value: f64, field: &str) -> anyhow::Result<u64> {
    validate_non_negative_finite(value, field)?;
    if value > u64::MAX as f64 {
        bail!("k6 field '{field}' is too large for perfgate.run.v1");
    }
    let rounded = value.round();
    if rounded == 0.0 && value > 0.0 {
        Ok(1)
    } else {
        Ok(rounded as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const K6_SUMMARY_JSON: &str = r#"{
      "metrics": {
        "http_req_duration": {
          "type": "trend",
          "contains": "time",
          "values": {
            "avg": 118.42,
            "min": 90.10,
            "med": 112.70,
            "max": 180.30,
            "p(90)": 160.00,
            "p(95)": 170.00
          }
        },
        "http_req_duration{scenario:checkout}": {
          "type": "trend",
          "contains": "time",
          "values": {"avg": 120.0, "min": 100.0, "med": 110.0, "max": 190.0}
        },
        "http_reqs": {
          "type": "counter",
          "contains": "default",
          "values": {"count": 34, "rate": 4.25}
        },
        "http_req_failed": {
          "type": "rate",
          "contains": "default",
          "values": {"rate": 0.0294117647, "passes": 33, "fails": 1}
        }
      },
      "state": {"testRunDurationMs": 8000}
    }"#;

    #[test]
    fn parses_http_summary_as_summary_only_receipt() {
        let receipt = parse_k6_summary_json(K6_SUMMARY_JSON, None).unwrap();

        assert_eq!(receipt.schema, RUN_SCHEMA_V1);
        assert_eq!(receipt.bench.name, "k6-http-summary");
        assert_eq!(receipt.bench.repeat, 34);
        assert_eq!(receipt.bench.work_units, Some(34));
        assert!(receipt.samples.is_empty());
        assert_eq!(receipt.stats.wall_ms.median, 113);
        assert_eq!(receipt.stats.wall_ms.min, 90);
        assert_eq!(receipt.stats.wall_ms.max, 180);
        assert_eq!(receipt.stats.wall_ms.mean, Some(118.42));
        assert_eq!(
            receipt.stats.throughput_per_s.as_ref().unwrap().median,
            4.25
        );
        assert_eq!(receipt.run.host.os, "unknown");
        assert!(
            receipt
                .bench
                .command
                .contains(&"http_req_failed_rate=0.029412".to_string())
        );
        assert!(
            receipt
                .bench
                .command
                .contains(&"scenario=checkout".to_string())
        );
        assert!(
            receipt
                .bench
                .command
                .contains(&"capacity_proof=not_production".to_string())
        );
    }

    #[test]
    fn accepts_name_override() {
        let receipt = parse_k6_summary_json(K6_SUMMARY_JSON, Some("checkout-load")).unwrap();

        assert_eq!(receipt.bench.name, "checkout-load");
    }

    #[test]
    fn converts_declared_seconds_time_unit() {
        let input = r#"{
          "options": {"summaryTimeUnit": "s"},
          "metrics": {
            "http_req_duration": {
              "type": "trend",
              "contains": "time",
              "values": {"avg": 0.120, "min": 0.100, "med": 0.110, "max": 0.160}
            }
          }
        }"#;
        let receipt = parse_k6_summary_json(input, None).unwrap();

        assert_eq!(receipt.stats.wall_ms.median, 110);
        assert_eq!(receipt.stats.wall_ms.min, 100);
        assert_eq!(receipt.stats.wall_ms.max, 160);
        assert_eq!(receipt.stats.wall_ms.mean, Some(120.0));
        assert!(
            receipt
                .bench
                .command
                .contains(&"summary_time_unit=s".to_string())
        );
    }

    #[test]
    fn rejects_missing_latency_metric() {
        let err = parse_k6_summary_json(
            r#"{"metrics":{"http_reqs":{"type":"counter","values":{"count":10,"rate":2.0}}}}"#,
            None,
        )
        .unwrap_err();

        assert!(err.to_string().contains("requires http_req_duration"));
    }

    #[test]
    fn rejects_negative_latency_value() {
        let err = parse_k6_summary_json(
            r#"{
              "metrics": {
                "http_req_duration": {
                  "type": "trend",
                  "contains": "time",
                  "values": {"avg": 1.0, "min": -1.0, "med": 1.0, "max": 2.0}
                }
              }
            }"#,
            None,
        )
        .unwrap_err();

        assert!(err.to_string().contains("finite and non-negative"));
    }
}
