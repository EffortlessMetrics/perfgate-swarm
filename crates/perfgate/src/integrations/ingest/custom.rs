//! Explicit custom JSON/CSV field mapping.
//!
//! This adapter is intentionally row-based. It does not infer arbitrary metric
//! meaning from field names; callers provide metric, field, unit, and direction
//! mappings explicitly.

use std::collections::BTreeMap;

use anyhow::{Context, bail};
use perfgate_types::{
    BenchMeta, Direction, F64Summary, HostInfo, Metric, RUN_SCHEMA_V1, RunMeta, RunReceipt, Sample,
    Stats, ToolInfo, U64Summary,
};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

/// Custom mapping options shared by custom JSON and custom CSV imports.
#[derive(Debug, Clone, Default)]
pub struct CustomMappingOptions {
    /// Metric mappings supplied by the caller.
    pub metrics: Vec<CustomMetricMapping>,
    /// Optional field that identifies rows/samples for review. The value is
    /// validated for every row but stored as metadata because run.v1 samples do
    /// not have a sample-id field.
    pub sample_id_field: Option<String>,
    /// Optional host field mapping.
    pub host: CustomHostMapping,
}

/// Explicit mapping from a source field to a perfgate metric.
#[derive(Debug, Clone, PartialEq)]
pub struct CustomMetricMapping {
    /// The target perfgate metric.
    pub metric: Metric,
    /// Dot-path for JSON or header name for CSV.
    pub field: String,
    /// Source unit.
    pub unit: String,
    /// Source direction.
    pub direction: Direction,
}

/// Optional host context field mapping.
#[derive(Debug, Clone, Default)]
pub struct CustomHostMapping {
    /// Field containing host OS.
    pub os_field: Option<String>,
    /// Field containing host architecture.
    pub arch_field: Option<String>,
    /// Field containing CPU count.
    pub cpu_count_field: Option<String>,
    /// Field containing memory bytes.
    pub memory_bytes_field: Option<String>,
    /// Field containing a pre-hashed hostname.
    pub hostname_hash_field: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CustomSourceKind {
    Json,
    Csv,
}

impl CustomSourceKind {
    fn label(self) -> &'static str {
        match self {
            CustomSourceKind::Json => "custom_json",
            CustomSourceKind::Csv => "custom_csv",
        }
    }

    fn command_label(self) -> &'static str {
        match self {
            CustomSourceKind::Json => "(ingested custom JSON)",
            CustomSourceKind::Csv => "(ingested custom CSV)",
        }
    }
}

#[derive(Debug)]
struct CustomDataset {
    root_fields: BTreeMap<String, String>,
    rows: Vec<CustomRow>,
}

#[derive(Debug)]
struct CustomRow {
    fields: BTreeMap<String, String>,
}

/// Parse a CLI `--metric` custom mapping.
///
/// Syntax:
///
/// ```text
/// metric_name=field.path,unit=ms,direction=lower_is_better
/// ```
pub fn parse_custom_metric_mapping_spec(raw: &str) -> anyhow::Result<CustomMetricMapping> {
    let mut parts = raw.split(',');
    let metric_part = parts
        .next()
        .context("custom metric mapping requires metric=field")?;
    let (metric_name, field) = split_key_value(metric_part)
        .with_context(|| format!("custom metric mapping '{raw}' requires metric=field"))?;
    let metric = Metric::parse_key(metric_name).with_context(|| {
        format!("custom metric mapping uses unsupported perfgate metric '{metric_name}'")
    })?;
    if field.trim().is_empty() {
        bail!(
            "custom metric mapping for '{}' requires a non-empty field",
            metric.as_str()
        );
    }

    let mut unit = None;
    let mut direction = None;
    for part in parts {
        let (key, value) = split_key_value(part).with_context(|| {
            format!("custom metric mapping '{raw}' has invalid segment '{part}'")
        })?;
        match normalize_label(key).as_str() {
            "unit" => unit = Some(value.trim().to_string()),
            "direction" => direction = Some(value.trim().to_string()),
            other => bail!("custom metric mapping '{raw}' has unsupported option '{other}'"),
        }
    }

    let unit = unit.with_context(|| {
        format!(
            "custom metric mapping for '{}' requires unit=...",
            metric.as_str()
        )
    })?;
    validate_unit(metric, &unit)?;

    let direction = direction.with_context(|| {
        format!(
            "custom metric mapping for '{}' requires direction=lower_is_better or direction=higher_is_better",
            metric.as_str()
        )
    })?;
    let direction = parse_direction(&direction).with_context(|| {
        format!(
            "custom metric mapping for '{}' has ambiguous direction; use lower_is_better or higher_is_better",
            metric.as_str()
        )
    })?;
    if direction != metric.default_direction() {
        bail!(
            "custom metric mapping for '{}' declares direction '{}' but perfgate expects '{}'",
            metric.as_str(),
            direction_label(direction),
            direction_label(metric.default_direction())
        );
    }

    Ok(CustomMetricMapping {
        metric,
        field: field.trim().to_string(),
        unit,
        direction,
    })
}

/// Parse mapped custom JSON into a `RunReceipt`.
pub fn parse_custom_json(
    input: &str,
    name: Option<&str>,
    options: &CustomMappingOptions,
) -> anyhow::Result<RunReceipt> {
    let value: Value = serde_json::from_str(input).context("failed to parse custom JSON")?;
    let dataset = custom_json_dataset(&value)?;
    custom_dataset_to_receipt(CustomSourceKind::Json, dataset, name, options)
}

/// Parse mapped custom CSV into a `RunReceipt`.
pub fn parse_custom_csv(
    input: &str,
    name: Option<&str>,
    options: &CustomMappingOptions,
) -> anyhow::Result<RunReceipt> {
    let dataset = custom_csv_dataset(input)?;
    custom_dataset_to_receipt(CustomSourceKind::Csv, dataset, name, options)
}

fn custom_dataset_to_receipt(
    source_kind: CustomSourceKind,
    dataset: CustomDataset,
    name: Option<&str>,
    options: &CustomMappingOptions,
) -> anyhow::Result<RunReceipt> {
    validate_custom_options(options)?;
    if dataset.rows.is_empty() {
        bail!("{} import contains no sample rows", source_kind.label());
    }

    let bench_name = name
        .map(str::to_string)
        .or_else(|| {
            field_value_any(
                &dataset,
                &dataset.rows[0],
                &["benchmark.name", "bench.name", "name"],
            )
        })
        .with_context(|| {
            format!(
                "{} import requires --name or a benchmark.name/name field",
                source_kind.label()
            )
        })?;

    let wall_mapping = options
        .metrics
        .iter()
        .find(|mapping| mapping.metric == Metric::WallMs)
        .with_context(|| {
            format!(
                "{} import requires a wall_ms metric mapping, for example --metric wall_ms=duration_ms,unit=ms,direction=lower_is_better",
                source_kind.label()
            )
        })?;

    if let Some(sample_id_field) = &options.sample_id_field {
        for (index, row) in dataset.rows.iter().enumerate() {
            row_field_value(row, sample_id_field).with_context(|| {
                format!(
                    "{} import sample row {} is missing sample identity field '{}'",
                    source_kind.label(),
                    index + 1,
                    sample_id_field
                )
            })?;
        }
    }

    let wall_values = metric_values(&dataset, wall_mapping)?;
    let mut samples = wall_samples(wall_mapping, &wall_values)?;
    let wall_ms = u64_summary_from_values(wall_mapping, &wall_values)?;

    let mut stats = Stats {
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
        throughput_per_s: None,
    };

    for mapping in &options.metrics {
        if mapping.metric == Metric::WallMs {
            continue;
        }

        let values = metric_values(&dataset, mapping)?;
        if mapping.metric == Metric::ThroughputPerS {
            stats.throughput_per_s = Some(f64_summary_from_values(mapping, &values)?);
            continue;
        }

        let summary = u64_summary_from_values(mapping, &values)?;
        assign_u64_summary(&mut stats, mapping.metric, summary)?;
        apply_u64_sample_values(&mut samples, mapping.metric, &values)?;
    }

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
            host: host_info(&dataset, &options.host)?,
        },
        bench: BenchMeta {
            name: bench_name,
            cwd: field_value_any(
                &dataset,
                &dataset.rows[0],
                &["benchmark.cwd", "bench.cwd", "cwd"],
            ),
            command: metadata_command(source_kind, options),
            repeat: samples.len() as u32,
            warmup: 0,
            work_units: None,
            timeout_ms: None,
        },
        samples,
        stats,
    })
}

fn validate_custom_options(options: &CustomMappingOptions) -> anyhow::Result<()> {
    if options.metrics.is_empty() {
        bail!("custom import requires at least one --metric mapping");
    }

    for mapping in &options.metrics {
        validate_unit(mapping.metric, &mapping.unit)?;
        if mapping.direction != mapping.metric.default_direction() {
            bail!(
                "custom metric mapping for '{}' declares direction '{}' but perfgate expects '{}'",
                mapping.metric.as_str(),
                direction_label(mapping.direction),
                direction_label(mapping.metric.default_direction())
            );
        }
    }

    Ok(())
}

fn custom_json_dataset(value: &Value) -> anyhow::Result<CustomDataset> {
    let mut root_fields = BTreeMap::new();
    flatten_json("", value, &mut root_fields);

    let rows = match value {
        Value::Array(samples) => samples
            .iter()
            .enumerate()
            .map(|(index, value)| json_row(value, index + 1))
            .collect::<anyhow::Result<Vec<_>>>()?,
        Value::Object(object) => {
            if let Some(samples) = object.get("samples") {
                let samples = samples
                    .as_array()
                    .context("custom JSON field 'samples' must be an array when present")?;
                samples
                    .iter()
                    .enumerate()
                    .map(|(index, value)| json_row(value, index + 1))
                    .collect::<anyhow::Result<Vec<_>>>()?
            } else {
                vec![json_row(value, 1)?]
            }
        }
        _ => bail!(
            "custom JSON must be an object, an array of sample objects, or an object with samples[]"
        ),
    };

    Ok(CustomDataset { root_fields, rows })
}

fn json_row(value: &Value, row_number: usize) -> anyhow::Result<CustomRow> {
    if !value.is_object() {
        bail!("custom JSON sample row {row_number} must be an object");
    }
    let mut fields = BTreeMap::new();
    flatten_json("", value, &mut fields);
    Ok(CustomRow { fields })
}

fn flatten_json(prefix: &str, value: &Value, fields: &mut BTreeMap<String, String>) {
    match value {
        Value::Object(object) => {
            for (key, value) in object {
                let key = if prefix.is_empty() {
                    key.to_string()
                } else {
                    format!("{prefix}.{key}")
                };
                flatten_json(&key, value, fields);
            }
        }
        Value::Array(_) | Value::Null => {}
        Value::Bool(value) => {
            fields.insert(prefix.to_string(), value.to_string());
        }
        Value::Number(value) => {
            fields.insert(prefix.to_string(), value.to_string());
        }
        Value::String(value) => {
            fields.insert(prefix.to_string(), value.to_string());
        }
    }
}

fn custom_csv_dataset(input: &str) -> anyhow::Result<CustomDataset> {
    let mut records = Vec::new();
    for (index, line) in input.lines().enumerate() {
        let line = line.trim_end_matches('\r');
        if line.trim().is_empty() {
            continue;
        }
        records.push(parse_csv_record(line, index + 1)?);
    }

    let header = records.first().context("custom CSV is empty")?;
    if header.is_empty() {
        bail!("custom CSV header is empty");
    }
    let mut rows = Vec::new();
    for (index, record) in records.iter().enumerate().skip(1) {
        if record.len() != header.len() {
            bail!(
                "custom CSV line {} has {} fields; expected {}",
                index + 1,
                record.len(),
                header.len()
            );
        }
        let fields = header.iter().cloned().zip(record.iter().cloned()).collect();
        rows.push(CustomRow { fields });
    }

    Ok(CustomDataset {
        root_fields: BTreeMap::new(),
        rows,
    })
}

fn parse_csv_record(line: &str, line_number: usize) -> anyhow::Result<Vec<String>> {
    let mut fields = Vec::new();
    let mut field = String::new();
    let mut chars = line.chars().peekable();
    let mut in_quotes = false;

    while let Some(ch) = chars.next() {
        match ch {
            '"' if in_quotes && chars.peek() == Some(&'"') => {
                field.push('"');
                chars.next();
            }
            '"' => in_quotes = !in_quotes,
            ',' if !in_quotes => {
                fields.push(field.trim().to_string());
                field.clear();
            }
            _ => field.push(ch),
        }
    }

    if in_quotes {
        bail!("custom CSV line {line_number} has an unterminated quoted field");
    }

    fields.push(field.trim().to_string());
    Ok(fields)
}

fn metric_values(
    dataset: &CustomDataset,
    mapping: &CustomMetricMapping,
) -> anyhow::Result<Vec<f64>> {
    dataset
        .rows
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let raw = row_field_value(row, &mapping.field).with_context(|| {
                format!(
                    "custom mapping for '{}' field '{}' is missing from sample row {}",
                    mapping.metric.as_str(),
                    mapping.field,
                    index + 1
                )
            })?;
            let value = raw.parse::<f64>().with_context(|| {
                format!(
                    "custom mapping for '{}' field '{}' in sample row {} is not a number",
                    mapping.metric.as_str(),
                    mapping.field,
                    index + 1
                )
            })?;
            normalize_metric_value(mapping.metric, &mapping.unit, value)
        })
        .collect()
}

fn wall_samples(mapping: &CustomMetricMapping, values: &[f64]) -> anyhow::Result<Vec<Sample>> {
    values
        .iter()
        .map(|value| {
            Ok(Sample {
                wall_ms: f64_to_u64(*value, mapping.metric.as_str())?,
                exit_code: 0,
                warmup: false,
                timed_out: false,
                cpu_ms: None,
                page_faults: None,
                ctx_switches: None,
                max_rss_kb: None,
                io_read_bytes: None,
                io_write_bytes: None,
                network_packets: None,
                energy_uj: None,
                binary_bytes: None,
                stdout: None,
                stderr: None,
            })
        })
        .collect()
}

fn u64_summary_from_values(
    mapping: &CustomMetricMapping,
    values: &[f64],
) -> anyhow::Result<U64Summary> {
    if values.is_empty() {
        bail!(
            "custom mapping for '{}' produced no sample values",
            mapping.metric.as_str()
        );
    }

    let values_u64 = values
        .iter()
        .map(|value| f64_to_u64(*value, mapping.metric.as_str()))
        .collect::<anyhow::Result<Vec<_>>>()?;
    let mut sorted = values_u64.clone();
    sorted.sort_unstable();
    let median = if sorted.len().is_multiple_of(2) {
        (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2
    } else {
        sorted[sorted.len() / 2]
    };
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance = values
        .iter()
        .map(|value| (*value - mean).powi(2))
        .sum::<f64>()
        / values.len() as f64;

    Ok(U64Summary {
        median,
        min: sorted[0],
        max: sorted[sorted.len() - 1],
        mean: Some(mean),
        stddev: Some(variance.sqrt()),
    })
}

fn f64_summary_from_values(
    mapping: &CustomMetricMapping,
    values: &[f64],
) -> anyhow::Result<F64Summary> {
    if values.is_empty() {
        bail!(
            "custom mapping for '{}' produced no sample values",
            mapping.metric.as_str()
        );
    }

    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median = if sorted.len().is_multiple_of(2) {
        (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0
    } else {
        sorted[sorted.len() / 2]
    };
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance = values
        .iter()
        .map(|value| (*value - mean).powi(2))
        .sum::<f64>()
        / values.len() as f64;

    Ok(F64Summary {
        median,
        min: sorted[0],
        max: sorted[sorted.len() - 1],
        mean: Some(mean),
        stddev: Some(variance.sqrt()),
    })
}

fn assign_u64_summary(
    stats: &mut Stats,
    metric: Metric,
    summary: U64Summary,
) -> anyhow::Result<()> {
    match metric {
        Metric::BinaryBytes => stats.binary_bytes = Some(summary),
        Metric::CpuMs => stats.cpu_ms = Some(summary),
        Metric::CtxSwitches => stats.ctx_switches = Some(summary),
        Metric::EnergyUj => stats.energy_uj = Some(summary),
        Metric::IoReadBytes => stats.io_read_bytes = Some(summary),
        Metric::IoWriteBytes => stats.io_write_bytes = Some(summary),
        Metric::MaxRssKb => stats.max_rss_kb = Some(summary),
        Metric::NetworkPackets => stats.network_packets = Some(summary),
        Metric::PageFaults => stats.page_faults = Some(summary),
        Metric::WallMs | Metric::ThroughputPerS => {
            bail!(
                "internal error: unsupported custom u64 metric {}",
                metric.as_str()
            )
        }
    }
    Ok(())
}

fn apply_u64_sample_values(
    samples: &mut [Sample],
    metric: Metric,
    values: &[f64],
) -> anyhow::Result<()> {
    if samples.len() != values.len() {
        bail!(
            "custom mapping for '{}' produced {} values but wall_ms has {}",
            metric.as_str(),
            values.len(),
            samples.len()
        );
    }

    for (sample, value) in samples.iter_mut().zip(values) {
        let value = f64_to_u64(*value, metric.as_str())?;
        match metric {
            Metric::BinaryBytes => sample.binary_bytes = Some(value),
            Metric::CpuMs => sample.cpu_ms = Some(value),
            Metric::CtxSwitches => sample.ctx_switches = Some(value),
            Metric::EnergyUj => sample.energy_uj = Some(value),
            Metric::IoReadBytes => sample.io_read_bytes = Some(value),
            Metric::IoWriteBytes => sample.io_write_bytes = Some(value),
            Metric::MaxRssKb => sample.max_rss_kb = Some(value),
            Metric::NetworkPackets => sample.network_packets = Some(value),
            Metric::PageFaults => sample.page_faults = Some(value),
            Metric::WallMs | Metric::ThroughputPerS => {}
        }
    }

    Ok(())
}

fn host_info(dataset: &CustomDataset, mapping: &CustomHostMapping) -> anyhow::Result<HostInfo> {
    let first_row = dataset.rows.first();
    Ok(HostInfo {
        os: host_string(dataset, first_row, mapping.os_field.as_deref())
            .unwrap_or_else(|| "unknown".to_string()),
        arch: host_string(dataset, first_row, mapping.arch_field.as_deref())
            .unwrap_or_else(|| "unknown".to_string()),
        cpu_count: host_u32(dataset, first_row, mapping.cpu_count_field.as_deref())?,
        memory_bytes: host_u64(dataset, first_row, mapping.memory_bytes_field.as_deref())?,
        hostname_hash: host_string(dataset, first_row, mapping.hostname_hash_field.as_deref()),
    })
}

fn host_string(
    dataset: &CustomDataset,
    row: Option<&CustomRow>,
    field: Option<&str>,
) -> Option<String> {
    let field = field?;
    let row = row?;
    field_value(dataset, row, field)
}

fn host_u32(
    dataset: &CustomDataset,
    row: Option<&CustomRow>,
    field: Option<&str>,
) -> anyhow::Result<Option<u32>> {
    let Some(field) = field else {
        return Ok(None);
    };
    let Some(value) = host_string(dataset, row, Some(field)) else {
        return Ok(None);
    };
    Ok(Some(value.parse::<u32>().with_context(|| {
        format!("custom host field '{field}' is not a u32")
    })?))
}

fn host_u64(
    dataset: &CustomDataset,
    row: Option<&CustomRow>,
    field: Option<&str>,
) -> anyhow::Result<Option<u64>> {
    let Some(field) = field else {
        return Ok(None);
    };
    let Some(value) = host_string(dataset, row, Some(field)) else {
        return Ok(None);
    };
    Ok(Some(value.parse::<u64>().with_context(|| {
        format!("custom host field '{field}' is not a u64")
    })?))
}

fn metadata_command(source_kind: CustomSourceKind, options: &CustomMappingOptions) -> Vec<String> {
    let mut command = vec![
        source_kind.command_label().to_string(),
        format!("source_kind={}", source_kind.label()),
    ];
    for mapping in &options.metrics {
        command.push(format!(
            "metric_mapping={}:{}:{}:{}",
            mapping.metric.as_str(),
            mapping.field,
            mapping.unit,
            direction_label(mapping.direction)
        ));
    }
    if let Some(sample_id_field) = &options.sample_id_field {
        command.push(format!("sample_identity_field={sample_id_field}"));
    } else {
        command.push("sample_identity=row_order".to_string());
    }
    command.push("sample_model=row_samples".to_string());
    command
}

fn field_value(dataset: &CustomDataset, row: &CustomRow, field: &str) -> Option<String> {
    row_field_value(row, field)
        .or_else(|| dataset.root_fields.get(field))
        .filter(|value| !value.trim().is_empty())
        .cloned()
}

fn row_field_value<'a>(row: &'a CustomRow, field: &str) -> Option<&'a String> {
    row.fields
        .get(field)
        .filter(|value| !value.trim().is_empty())
}

fn field_value_any(dataset: &CustomDataset, row: &CustomRow, fields: &[&str]) -> Option<String> {
    fields
        .iter()
        .find_map(|field| field_value(dataset, row, field))
}

fn split_key_value(raw: &str) -> Option<(&str, &str)> {
    raw.split_once('=')
        .map(|(key, value)| (key.trim(), value.trim()))
}

fn validate_unit(metric: Metric, unit: &str) -> anyhow::Result<()> {
    let normalized = normalize_label(unit);
    let valid = match metric {
        Metric::WallMs | Metric::CpuMs => matches!(
            normalized.as_str(),
            "ms" | "millisecond" | "milliseconds" | "s" | "sec" | "second" | "seconds"
        ),
        Metric::MaxRssKb => matches!(
            normalized.as_str(),
            "kb" | "kib" | "kilobyte" | "kilobytes" | "bytes" | "byte" | "b"
        ),
        Metric::IoReadBytes | Metric::IoWriteBytes | Metric::BinaryBytes => {
            matches!(normalized.as_str(), "bytes" | "byte" | "b")
        }
        Metric::PageFaults | Metric::CtxSwitches | Metric::NetworkPackets => matches!(
            normalized.as_str(),
            "count" | "counts" | "events" | "event" | "packets" | "packet"
        ),
        Metric::EnergyUj => matches!(
            normalized.as_str(),
            "uj" | "microjoule" | "microjoules" | "micro_joule" | "micro_joules"
        ),
        Metric::ThroughputPerS => matches!(
            normalized.as_str(),
            "per_s"
                | "per_sec"
                | "per_second"
                | "ops_s"
                | "ops_sec"
                | "ops_per_s"
                | "ops_per_sec"
                | "operations_s"
                | "operations_sec"
                | "operations_per_s"
                | "operations_per_sec"
                | "requests_s"
                | "requests_sec"
                | "requests_per_s"
                | "requests_per_sec"
                | "rps"
                | "items_s"
                | "items_sec"
                | "items_per_s"
                | "items_per_sec"
        ),
    };

    if valid {
        Ok(())
    } else {
        bail!(
            "custom metric mapping for '{}' has unsupported or ambiguous unit '{}'",
            metric.as_str(),
            unit
        )
    }
}

fn normalize_metric_value(metric: Metric, unit: &str, value: f64) -> anyhow::Result<f64> {
    if !value.is_finite() || value < 0.0 {
        bail!(
            "custom metric mapping for '{}' value must be finite and non-negative",
            metric.as_str()
        );
    }

    let normalized = normalize_label(unit);
    let value = match metric {
        Metric::WallMs | Metric::CpuMs => match normalized.as_str() {
            "s" | "sec" | "second" | "seconds" => value * 1000.0,
            _ => value,
        },
        Metric::MaxRssKb => match normalized.as_str() {
            "bytes" | "byte" | "b" => value / 1024.0,
            _ => value,
        },
        _ => value,
    };
    Ok(value)
}

fn f64_to_u64(value: f64, metric_name: &str) -> anyhow::Result<u64> {
    if !value.is_finite() || value < 0.0 {
        bail!("custom metric mapping for '{metric_name}' value must be finite and non-negative");
    }
    if value > u64::MAX as f64 {
        bail!("custom metric mapping for '{metric_name}' value is too large for perfgate.run.v1");
    }
    let rounded = value.round();
    if rounded == 0.0 && value > 0.0 {
        Ok(1)
    } else {
        Ok(rounded as u64)
    }
}

fn parse_direction(raw: &str) -> Option<Direction> {
    match normalize_label(raw).as_str() {
        "lower" | "lower_is_better" | "lower_better" => Some(Direction::Lower),
        "higher" | "higher_is_better" | "higher_better" => Some(Direction::Higher),
        _ => None,
    }
}

fn direction_label(direction: Direction) -> &'static str {
    match direction {
        Direction::Lower => "lower_is_better",
        Direction::Higher => "higher_is_better",
    }
}

fn normalize_label(raw: &str) -> String {
    raw.trim()
        .to_ascii_lowercase()
        .replace(['-', '/', ' '], "_")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mapping(raw: &str) -> CustomMetricMapping {
        parse_custom_metric_mapping_spec(raw).unwrap()
    }

    #[test]
    fn parses_custom_json_array_with_explicit_mappings() {
        let options = CustomMappingOptions {
            metrics: vec![
                mapping("wall_ms=duration_ms,unit=ms,direction=lower_is_better"),
                mapping("throughput_per_s=rps,unit=requests/s,direction=higher_is_better"),
            ],
            sample_id_field: Some("sample_id".to_string()),
            host: CustomHostMapping {
                os_field: Some("host.os".to_string()),
                arch_field: Some("host.arch".to_string()),
                cpu_count_field: Some("host.cpu_count".to_string()),
                memory_bytes_field: None,
                hostname_hash_field: None,
            },
        };
        let receipt = parse_custom_json(
            r#"{
              "benchmark": {"name": "api-smoke"},
              "host": {"os": "linux", "arch": "x86_64", "cpu_count": "8"},
              "samples": [
                {"sample_id": "a", "duration_ms": 101.0, "rps": 40.0},
                {"sample_id": "b", "duration_ms": 99.0, "rps": 42.0},
                {"sample_id": "c", "duration_ms": 105.0, "rps": 39.0}
              ]
            }"#,
            None,
            &options,
        )
        .unwrap();

        assert_eq!(receipt.schema, RUN_SCHEMA_V1);
        assert_eq!(receipt.bench.name, "api-smoke");
        assert_eq!(receipt.bench.repeat, 3);
        assert_eq!(receipt.stats.wall_ms.median, 101);
        assert_eq!(
            receipt.stats.throughput_per_s.as_ref().unwrap().median,
            40.0
        );
        assert_eq!(receipt.run.host.os, "linux");
        assert_eq!(receipt.run.host.arch, "x86_64");
        assert_eq!(receipt.run.host.cpu_count, Some(8));
        assert!(
            receipt
                .bench
                .command
                .contains(&"sample_identity_field=sample_id".to_string())
        );
    }

    #[test]
    fn parses_custom_csv_with_explicit_mappings() {
        let options = CustomMappingOptions {
            metrics: vec![
                mapping("wall_ms=duration_ms,unit=ms,direction=lower_is_better"),
                mapping("max_rss_kb=rss_bytes,unit=bytes,direction=lower_is_better"),
            ],
            sample_id_field: None,
            host: CustomHostMapping::default(),
        };
        let receipt = parse_custom_csv(
            "duration_ms,rss_bytes\n120,1048576\n118,2097152\n",
            Some("csv-smoke"),
            &options,
        )
        .unwrap();

        assert_eq!(receipt.bench.name, "csv-smoke");
        assert_eq!(receipt.samples.len(), 2);
        assert_eq!(receipt.stats.wall_ms.median, 119);
        assert_eq!(receipt.samples[0].max_rss_kb, Some(1024));
        assert_eq!(receipt.run.host.os, "unknown");
    }

    #[test]
    fn rejects_missing_wall_mapping() {
        let options = CustomMappingOptions {
            metrics: vec![mapping(
                "throughput_per_s=rps,unit=rps,direction=higher_is_better",
            )],
            ..Default::default()
        };
        let err = parse_custom_json(r#"[{"rps": 10.0}]"#, Some("bad"), &options).unwrap_err();

        assert!(
            err.to_string()
                .contains("requires a wall_ms metric mapping")
        );
    }

    #[test]
    fn rejects_ambiguous_unit() {
        let err = parse_custom_metric_mapping_spec(
            "wall_ms=duration,unit=duration,direction=lower_is_better",
        )
        .unwrap_err();

        assert!(err.to_string().contains("unsupported or ambiguous unit"));
    }

    #[test]
    fn rejects_direction_inversion() {
        let err = parse_custom_metric_mapping_spec(
            "throughput_per_s=rps,unit=rps,direction=lower_is_better",
        )
        .unwrap_err();

        assert!(err.to_string().contains("higher_is_better"));
    }

    #[test]
    fn rejects_csv_row_length_without_dumping_payload() {
        let options = CustomMappingOptions {
            metrics: vec![mapping(
                "wall_ms=duration_ms,unit=ms,direction=lower_is_better",
            )],
            ..Default::default()
        };
        let err =
            parse_custom_csv("duration_ms,rps\n100,10\n200\n", Some("bad"), &options).unwrap_err();

        assert!(err.to_string().contains("line 3"));
        assert!(!err.to_string().contains("duration_ms,rps"));
    }
}
