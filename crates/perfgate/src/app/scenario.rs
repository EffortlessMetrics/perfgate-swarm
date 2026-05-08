use crate::domain::budget::{aggregate_verdict, calculate_regression, determine_status};
use perfgate_types::{
    CompareReceipt, CompareRef, ConfigFile, Delta, HostInfo, Metric, MetricStatus, RunMeta,
    SCENARIO_SCHEMA_V1, ScenarioComponent, ScenarioConfigFile, ScenarioMeta, ScenarioReceipt,
    ToolInfo,
};
use std::collections::BTreeMap;
use time::OffsetDateTime;
use uuid::Uuid;

const DEFAULT_SCENARIO_NAME: &str = "configured_workload";

#[derive(Debug)]
pub struct ScenarioEvaluateRequest {
    pub config: ConfigFile,
    pub inputs: Vec<ScenarioEvaluateInput>,
    pub workload_name: Option<String>,
    pub tool: ToolInfo,
}

#[derive(Debug)]
pub struct ScenarioEvaluateInput {
    pub config: ScenarioConfigFile,
    pub compare_ref: CompareRef,
    pub compare: CompareReceipt,
}

#[derive(Debug)]
pub struct ScenarioEvaluateOutcome {
    pub receipt: ScenarioReceipt,
}

pub struct ScenarioUseCase;

impl ScenarioUseCase {
    pub fn evaluate(req: ScenarioEvaluateRequest) -> anyhow::Result<ScenarioEvaluateOutcome> {
        if req.inputs.is_empty() {
            anyhow::bail!("no scenario inputs provided");
        }

        for input in &req.inputs {
            if input.compare.bench.name != input.config.bench {
                anyhow::bail!(
                    "scenario '{}' expected compare receipt for benchmark '{}', got '{}'",
                    input.config.name,
                    input.config.bench,
                    input.compare.bench.name
                );
            }
        }

        let components = build_components(&req.inputs);
        let weighted_deltas = build_weighted_deltas(&req.inputs, &req.config)?;
        let statuses: Vec<MetricStatus> =
            weighted_deltas.values().map(|delta| delta.status).collect();
        let mut verdict = aggregate_verdict(&statuses);
        let warnings = build_warnings(&req.inputs, &weighted_deltas);
        verdict.reasons = weighted_deltas
            .iter()
            .filter(|(_, delta)| !matches!(delta.status, MetricStatus::Pass))
            .map(|(metric, delta)| {
                format!("scenario_{}_{}", metric.as_str(), delta.status.as_str())
            })
            .collect();

        let receipt = ScenarioReceipt {
            schema: SCENARIO_SCHEMA_V1.to_string(),
            tool: req.tool,
            run: make_run_meta(),
            scenario: scenario_meta(&req.inputs, req.workload_name.as_deref()),
            baseline_ref: None,
            current_ref: None,
            components,
            weighted_deltas,
            verdict,
            warnings,
        };

        Ok(ScenarioEvaluateOutcome { receipt })
    }
}

fn build_components(inputs: &[ScenarioEvaluateInput]) -> Vec<ScenarioComponent> {
    inputs
        .iter()
        .map(|input| ScenarioComponent {
            name: input.config.name.clone(),
            weight: input.config.weight,
            benchmark: Some(input.config.bench.clone()),
            compare_ref: Some(input.compare_ref.clone()),
            deltas: stringify_deltas(&input.compare.deltas),
            probes: Vec::new(),
            status: metric_status_from_verdict(input.compare.verdict.status),
            reasons: input.compare.verdict.reasons.clone(),
        })
        .collect()
}

fn stringify_deltas(deltas: &BTreeMap<Metric, Delta>) -> BTreeMap<String, Delta> {
    deltas
        .iter()
        .map(|(metric, delta)| (metric.as_str().to_string(), delta.clone()))
        .collect()
}

fn build_weighted_deltas(
    inputs: &[ScenarioEvaluateInput],
    config: &ConfigFile,
) -> anyhow::Result<BTreeMap<String, Delta>> {
    let mut totals: BTreeMap<Metric, WeightedMetricTotals> = BTreeMap::new();

    for input in inputs {
        for (metric, delta) in &input.compare.deltas {
            let entry = totals.entry(*metric).or_default();
            entry.weight += input.config.weight;
            entry.baseline += delta.baseline * input.config.weight;
            entry.current += delta.current * input.config.weight;
            entry.statistic = Some(delta.statistic);
        }
    }

    let threshold = config.defaults.threshold.unwrap_or(0.20);
    let warn_threshold = threshold * config.defaults.warn_factor.unwrap_or(0.90);

    let mut weighted = BTreeMap::new();
    for (metric, total) in totals {
        if total.weight <= 0.0 {
            continue;
        }
        let baseline = total.baseline / total.weight;
        let current = total.current / total.weight;
        if baseline <= 0.0 {
            anyhow::bail!(
                "cannot evaluate weighted scenario metric '{}' with non-positive baseline {}",
                metric.as_str(),
                baseline
            );
        }

        let ratio = current / baseline;
        let pct = (current - baseline) / baseline;
        let regression = calculate_regression(baseline, current, metric.default_direction());
        let status = determine_status(regression, threshold, warn_threshold);
        weighted.insert(
            metric.as_str().to_string(),
            Delta {
                baseline,
                current,
                ratio,
                pct,
                regression,
                cv: None,
                noise_threshold: config.defaults.noise_threshold,
                statistic: total.statistic.unwrap_or_default(),
                significance: None,
                status,
            },
        );
    }

    Ok(weighted)
}

#[derive(Default)]
struct WeightedMetricTotals {
    weight: f64,
    baseline: f64,
    current: f64,
    statistic: Option<perfgate_types::MetricStatistic>,
}

fn build_warnings(
    inputs: &[ScenarioEvaluateInput],
    weighted_deltas: &BTreeMap<String, Delta>,
) -> Vec<String> {
    let mut warnings = Vec::new();
    for metric in weighted_deltas.keys() {
        let parsed_metric = Metric::parse_key(metric);
        let missing: Vec<_> = inputs
            .iter()
            .filter(|input| {
                parsed_metric
                    .map(|metric| !input.compare.deltas.contains_key(&metric))
                    .unwrap_or(false)
            })
            .map(|input| input.config.name.as_str())
            .collect();
        if !missing.is_empty() {
            warnings.push(format!(
                "metric '{}' was missing from scenario component(s): {}",
                metric,
                missing.join(", ")
            ));
        }
    }
    warnings
}

fn scenario_meta(inputs: &[ScenarioEvaluateInput], workload_name: Option<&str>) -> ScenarioMeta {
    if inputs.len() == 1 {
        let input = &inputs[0];
        return ScenarioMeta {
            name: input.config.name.clone(),
            weight: input.config.weight,
            description: input.config.description.clone(),
            command: Some(input.compare.bench.command.clone()),
        };
    }

    ScenarioMeta {
        name: workload_name.unwrap_or(DEFAULT_SCENARIO_NAME).to_string(),
        weight: 1.0,
        description: Some("Weighted workload from configured scenarios".to_string()),
        command: None,
    }
}

fn metric_status_from_verdict(status: perfgate_types::VerdictStatus) -> MetricStatus {
    match status {
        perfgate_types::VerdictStatus::Pass => MetricStatus::Pass,
        perfgate_types::VerdictStatus::Warn => MetricStatus::Warn,
        perfgate_types::VerdictStatus::Fail => MetricStatus::Fail,
        perfgate_types::VerdictStatus::Skip => MetricStatus::Skip,
    }
}

fn make_run_meta() -> RunMeta {
    let now = OffsetDateTime::now_utc();
    let timestamp = now
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string());

    RunMeta {
        id: Uuid::new_v4().to_string(),
        started_at: timestamp.clone(),
        ended_at: timestamp,
        host: HostInfo {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            cpu_count: None,
            memory_bytes: None,
            hostname_hash: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use perfgate_types::{
        BenchMeta, COMPARE_SCHEMA_V1, DefaultsConfig, MetricStatistic, Verdict, VerdictCounts,
        VerdictStatus,
    };

    fn compare_receipt(
        bench: &str,
        baseline: f64,
        current: f64,
        status: MetricStatus,
    ) -> CompareReceipt {
        CompareReceipt {
            schema: COMPARE_SCHEMA_V1.to_string(),
            tool: ToolInfo {
                name: "perfgate".to_string(),
                version: "0.1.0".to_string(),
            },
            bench: BenchMeta {
                name: bench.to_string(),
                cwd: None,
                command: vec!["echo".to_string(), bench.to_string()],
                repeat: 1,
                warmup: 0,
                work_units: None,
                timeout_ms: None,
            },
            baseline_ref: CompareRef {
                path: Some(format!("baselines/{bench}.json")),
                run_id: Some(format!("{bench}-base")),
            },
            current_ref: CompareRef {
                path: Some(format!("artifacts/{bench}/run.json")),
                run_id: Some(format!("{bench}-current")),
            },
            budgets: BTreeMap::new(),
            deltas: BTreeMap::from([(
                Metric::WallMs,
                Delta {
                    baseline,
                    current,
                    ratio: current / baseline,
                    pct: (current - baseline) / baseline,
                    regression: calculate_regression(
                        baseline,
                        current,
                        Metric::WallMs.default_direction(),
                    ),
                    cv: None,
                    noise_threshold: None,
                    statistic: MetricStatistic::Median,
                    significance: None,
                    status,
                },
            )]),
            verdict: Verdict {
                status: match status {
                    MetricStatus::Pass => VerdictStatus::Pass,
                    MetricStatus::Warn => VerdictStatus::Warn,
                    MetricStatus::Fail => VerdictStatus::Fail,
                    MetricStatus::Skip => VerdictStatus::Skip,
                },
                counts: VerdictCounts {
                    pass: u32::from(matches!(status, MetricStatus::Pass)),
                    warn: u32::from(matches!(status, MetricStatus::Warn)),
                    fail: u32::from(matches!(status, MetricStatus::Fail)),
                    skip: u32::from(matches!(status, MetricStatus::Skip)),
                },
                reasons: Vec::new(),
            },
        }
    }

    fn scenario_input(
        name: &str,
        weight: f64,
        bench: &str,
        compare: CompareReceipt,
    ) -> ScenarioEvaluateInput {
        ScenarioEvaluateInput {
            config: ScenarioConfigFile {
                name: name.to_string(),
                weight,
                bench: bench.to_string(),
                description: None,
                compare: None,
            },
            compare_ref: CompareRef {
                path: Some(format!("artifacts/{bench}/compare.json")),
                run_id: compare.current_ref.run_id.clone(),
            },
            compare,
        }
    }

    #[test]
    fn scenario_evaluate_computes_weighted_deltas() {
        let outcome = ScenarioUseCase::evaluate(ScenarioEvaluateRequest {
            config: ConfigFile {
                defaults: DefaultsConfig {
                    threshold: Some(0.20),
                    warn_factor: Some(0.50),
                    ..Default::default()
                },
                ..Default::default()
            },
            inputs: vec![
                scenario_input(
                    "large_file_parse",
                    0.75,
                    "large-file",
                    compare_receipt("large-file", 100.0, 90.0, MetricStatus::Pass),
                ),
                scenario_input(
                    "small_edit",
                    0.25,
                    "small-edit",
                    compare_receipt("small-edit", 100.0, 110.0, MetricStatus::Fail),
                ),
            ],
            workload_name: None,
            tool: ToolInfo {
                name: "perfgate".to_string(),
                version: "0.1.0".to_string(),
            },
        })
        .expect("evaluate scenario");

        let delta = &outcome.receipt.weighted_deltas["wall_ms"];
        assert_eq!(outcome.receipt.schema, SCENARIO_SCHEMA_V1);
        assert_eq!(outcome.receipt.components.len(), 2);
        assert!((delta.current - 95.0).abs() < f64::EPSILON);
        assert_eq!(delta.status, MetricStatus::Pass);
        assert_eq!(outcome.receipt.verdict.status, VerdictStatus::Pass);
    }

    #[test]
    fn scenario_evaluate_rejects_mismatched_compare_benchmark() {
        let err = ScenarioUseCase::evaluate(ScenarioEvaluateRequest {
            config: ConfigFile::default(),
            inputs: vec![scenario_input(
                "large_file_parse",
                1.0,
                "large-file",
                compare_receipt("other-bench", 100.0, 90.0, MetricStatus::Pass),
            )],
            workload_name: None,
            tool: ToolInfo {
                name: "perfgate".to_string(),
                version: "0.1.0".to_string(),
            },
        })
        .expect_err("mismatched benchmark should fail");

        assert!(err.to_string().contains("expected compare receipt"));
    }
}
