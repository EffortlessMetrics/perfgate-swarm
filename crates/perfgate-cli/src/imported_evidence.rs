//! Derived metadata for receipts created by ingest adapters.
//!
//! `perfgate.run.v1` intentionally stays stable. These helpers infer the
//! review limits that maturity and policy surfaces should show for imported
//! receipts without adding schema fields.

use perfgate_types::RunReceipt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ImportedEvidenceSummary {
    pub(crate) source_kind: String,
    pub(crate) source_path: Option<String>,
    pub(crate) metric_mappings: Vec<String>,
    pub(crate) sample_model: &'static str,
    pub(crate) host_context: &'static str,
    pub(crate) noise_support: &'static str,
    limitations: Vec<&'static str>,
}

impl ImportedEvidenceSummary {
    pub(crate) fn source_label(&self) -> String {
        format!("imported ({})", self.source_kind)
    }

    pub(crate) fn limitations(&self) -> &[&'static str] {
        &self.limitations
    }

    pub(crate) fn has_missing_host_context(&self) -> bool {
        self.host_context == "missing_or_partial"
    }

    pub(crate) fn is_summary_only(&self) -> bool {
        self.sample_model == "summary_only"
    }
}

pub(crate) fn summarize_imported_receipt(receipt: &RunReceipt) -> Option<ImportedEvidenceSummary> {
    if receipt.tool.name != "perfgate-ingest" {
        return None;
    }

    let source_kind = source_kind(receipt);
    let source_path = command_metadata(receipt, "source_path");
    let metric_mappings = metric_mappings(receipt, &source_kind);
    let sample_model = sample_model(receipt);
    let host_context = host_context(receipt);
    let noise_support = noise_support(receipt, sample_model);
    let mut limitations =
        vec!["imported evidence remains advisory until maturity and policy are reviewed"];

    if sample_model == "summary_only" {
        limitations.push("summary-only evidence has limited noise support");
    }
    if host_context == "missing_or_partial" {
        limitations.push("missing host context is not host-compatible proof");
    }
    if noise_support == "samples_without_cv" {
        limitations.push("raw samples need calibration before policy promotion");
    }
    if source_kind == "unrecorded_ingest_source" {
        limitations.push(
            "run.v1 receipt does not record adapter source kind; review the ingest command or source artifact",
        );
    }
    if source_path.is_none() {
        limitations.push("run.v1 receipt does not record adapter source path");
    }

    Some(ImportedEvidenceSummary {
        source_kind,
        source_path,
        metric_mappings,
        sample_model,
        host_context,
        noise_support,
        limitations,
    })
}

fn source_kind(receipt: &RunReceipt) -> String {
    for arg in &receipt.bench.command {
        if let Some(source_kind) = arg.strip_prefix("source_kind=") {
            return source_kind.to_string();
        }
    }

    let first = receipt
        .bench
        .command
        .first()
        .map(String::as_str)
        .unwrap_or_default();

    match first {
        "(ingested generic command JSON)" => "generic_command_json".to_string(),
        "(ingested Criterion benchmark)" => "criterion".to_string(),
        "(ingested pytest-benchmark JSON)" => "pytest_benchmark_json".to_string(),
        "(ingested k6 summary JSON)" => "k6_summary_json".to_string(),
        "(ingested custom JSON)" => "custom_json".to_string(),
        "(ingested custom CSV)" => "custom_csv".to_string(),
        "(ingested)" => "ingested_run_v1".to_string(),
        _ => "unrecorded_ingest_source".to_string(),
    }
}

fn command_metadata(receipt: &RunReceipt, key: &str) -> Option<String> {
    let prefix = format!("{key}=");
    receipt
        .bench
        .command
        .iter()
        .find_map(|arg| arg.strip_prefix(&prefix))
        .map(str::to_string)
}

fn command_metadata_all(receipt: &RunReceipt, key: &str) -> Vec<String> {
    let prefix = format!("{key}=");
    receipt
        .bench
        .command
        .iter()
        .filter_map(|arg| arg.strip_prefix(&prefix))
        .map(str::to_string)
        .collect()
}

fn metric_mappings(receipt: &RunReceipt, source_kind: &str) -> Vec<String> {
    let explicit = command_metadata_all(receipt, "metric_mapping");
    if !explicit.is_empty() {
        return explicit
            .into_iter()
            .map(|mapping| format_custom_mapping(&mapping))
            .collect();
    }

    match source_kind {
        "k6_summary_json" => k6_metric_mappings(receipt),
        "hyperfine_json" => hyperfine_metric_mappings(receipt),
        "criterion" => criterion_metric_mappings(receipt),
        "pytest_benchmark_json" => pytest_metric_mappings(receipt),
        "generic_command_json" => generic_metric_mappings(receipt),
        _ => fallback_metric_mappings(receipt),
    }
}

fn format_custom_mapping(mapping: &str) -> String {
    let parts: Vec<&str> = mapping.split(':').collect();
    if parts.len() == 4 {
        format!("{} <= {} ({}, {})", parts[0], parts[1], parts[2], parts[3])
    } else {
        mapping.to_string()
    }
}

fn k6_metric_mappings(receipt: &RunReceipt) -> Vec<String> {
    let mut mappings = Vec::new();
    let latency = command_metadata(receipt, "latency_metric")
        .unwrap_or_else(|| "http_req_duration".to_string());
    let time_unit = command_metadata(receipt, "summary_time_unit").unwrap_or_else(|| "ms".into());
    mappings.push(format!(
        "wall_ms <= {latency} ({time_unit}->ms, lower_is_better)"
    ));
    if let Some(throughput) = command_metadata(receipt, "throughput_metric") {
        mappings.push(format!(
            "throughput_per_s <= {throughput}.rate (requests/s, higher_is_better)"
        ));
    }
    mappings
}

fn hyperfine_metric_mappings(receipt: &RunReceipt) -> Vec<String> {
    let mut mappings =
        vec!["wall_ms <= hyperfine times (seconds->ms, lower_is_better)".to_string()];
    if receipt.stats.cpu_ms.is_some() {
        mappings.push("cpu_ms <= hyperfine user+system (seconds->ms, lower_is_better)".to_string());
    }
    mappings
}

fn criterion_metric_mappings(receipt: &RunReceipt) -> Vec<String> {
    let mut mappings =
        vec!["wall_ms <= Criterion wall-time (ns/us/ms/s->ms, lower_is_better)".to_string()];
    if receipt.bench.work_units.is_some() {
        mappings.push("bench.work_units <= Criterion throughput per_iteration".to_string());
    }
    mappings
}

fn pytest_metric_mappings(receipt: &RunReceipt) -> Vec<String> {
    let mut mappings =
        vec!["wall_ms <= pytest-benchmark seconds (seconds->ms, lower_is_better)".to_string()];
    if receipt.stats.throughput_per_s.is_some() {
        mappings
            .push("throughput_per_s <= pytest-benchmark ops (ops/s, higher_is_better)".to_string());
    }
    mappings
}

fn generic_metric_mappings(receipt: &RunReceipt) -> Vec<String> {
    fallback_metric_mappings(receipt)
        .into_iter()
        .map(|mapping| format!("{mapping}; source declared explicit unit and direction"))
        .collect()
}

fn fallback_metric_mappings(receipt: &RunReceipt) -> Vec<String> {
    let mut mappings = vec!["wall_ms <= source wall-time metric (ms, lower_is_better)".to_string()];
    if receipt.stats.cpu_ms.is_some() {
        mappings.push("cpu_ms <= source cpu metric (ms, lower_is_better)".to_string());
    }
    if receipt.stats.page_faults.is_some() {
        mappings
            .push("page_faults <= source page-fault metric (count, lower_is_better)".to_string());
    }
    if receipt.stats.ctx_switches.is_some() {
        mappings.push(
            "ctx_switches <= source context-switch metric (count, lower_is_better)".to_string(),
        );
    }
    if receipt.stats.max_rss_kb.is_some() {
        mappings.push("max_rss_kb <= source memory metric (KiB, lower_is_better)".to_string());
    }
    if receipt.stats.io_read_bytes.is_some() {
        mappings
            .push("io_read_bytes <= source I/O read metric (bytes, lower_is_better)".to_string());
    }
    if receipt.stats.io_write_bytes.is_some() {
        mappings
            .push("io_write_bytes <= source I/O write metric (bytes, lower_is_better)".to_string());
    }
    if receipt.stats.network_packets.is_some() {
        mappings.push(
            "network_packets <= source network packet metric (count, lower_is_better)".to_string(),
        );
    }
    if receipt.stats.energy_uj.is_some() {
        mappings.push("energy_uj <= source energy metric (uJ, lower_is_better)".to_string());
    }
    if receipt.stats.binary_bytes.is_some() {
        mappings
            .push("binary_bytes <= source binary-size metric (bytes, lower_is_better)".to_string());
    }
    if receipt.stats.throughput_per_s.is_some() {
        mappings.push(
            "throughput_per_s <= source throughput metric (per-second, higher_is_better)"
                .to_string(),
        );
    }
    mappings
}

fn sample_model(receipt: &RunReceipt) -> &'static str {
    if receipt
        .bench
        .command
        .iter()
        .any(|arg| arg == "sample_model=summary_only")
    {
        return "summary_only";
    }
    if receipt.samples.iter().any(|sample| !sample.warmup) {
        return "raw_samples";
    }
    "summary_only"
}

fn host_context(receipt: &RunReceipt) -> &'static str {
    let os = receipt.run.host.os.trim();
    let arch = receipt.run.host.arch.trim();
    if os.is_empty()
        || arch.is_empty()
        || os.eq_ignore_ascii_case("unknown")
        || arch.eq_ignore_ascii_case("unknown")
    {
        "missing_or_partial"
    } else {
        "present"
    }
}

fn noise_support(receipt: &RunReceipt, sample_model: &'static str) -> &'static str {
    if sample_model == "summary_only" {
        return "limited_summary_only";
    }
    if receipt.stats.wall_ms.cv().is_some() {
        "sample_cv_available"
    } else {
        "samples_without_cv"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use perfgate_types::{
        BenchMeta, HostInfo, RUN_SCHEMA_V1, RunMeta, RunReceipt, Stats, ToolInfo, U64Summary,
    };

    fn imported_receipt(command: Vec<String>, samples: usize, os: &str, arch: &str) -> RunReceipt {
        RunReceipt {
            schema: RUN_SCHEMA_V1.to_string(),
            tool: ToolInfo {
                name: "perfgate-ingest".to_string(),
                version: "0.21.0".to_string(),
            },
            run: RunMeta {
                id: "run".to_string(),
                started_at: "2026-05-20T00:00:00Z".to_string(),
                ended_at: "2026-05-20T00:00:01Z".to_string(),
                host: HostInfo {
                    os: os.to_string(),
                    arch: arch.to_string(),
                    cpu_count: None,
                    memory_bytes: None,
                    hostname_hash: None,
                },
            },
            bench: BenchMeta {
                name: "bench".to_string(),
                cwd: None,
                command,
                repeat: samples as u32,
                warmup: 0,
                work_units: None,
                timeout_ms: None,
            },
            samples: (0..samples)
                .map(|_| perfgate_types::Sample {
                    wall_ms: 100,
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
                .collect(),
            stats: Stats {
                wall_ms: U64Summary {
                    median: 100,
                    min: 100,
                    max: 100,
                    mean: Some(100.0),
                    stddev: Some(2.0),
                },
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
            },
        }
    }

    #[test]
    fn summarizes_known_ingest_source_and_sample_limits() {
        let receipt = imported_receipt(
            vec![
                "(ingested k6 summary JSON)".to_string(),
                "sample_model=summary_only".to_string(),
                "source_path=artifacts/k6-summary.json".to_string(),
            ],
            0,
            "unknown",
            "unknown",
        );

        let summary = summarize_imported_receipt(&receipt).expect("imported summary");

        assert_eq!(summary.source_kind, "k6_summary_json");
        assert_eq!(
            summary.source_path.as_deref(),
            Some("artifacts/k6-summary.json")
        );
        assert_eq!(summary.sample_model, "summary_only");
        assert_eq!(summary.host_context, "missing_or_partial");
        assert_eq!(summary.noise_support, "limited_summary_only");
        assert!(
            summary
                .metric_mappings
                .iter()
                .any(|mapping| mapping.contains("wall_ms <= http_req_duration"))
        );
        assert!(summary.is_summary_only());
        assert!(summary.has_missing_host_context());
    }

    #[test]
    fn formats_custom_metric_metadata() {
        let receipt = imported_receipt(
            vec![
                "(ingested custom JSON)".to_string(),
                "source_kind=custom_json".to_string(),
                "metric_mapping=wall_ms:duration_ms:ms:lower_is_better".to_string(),
                "metric_mapping=throughput_per_s:rps:rps:higher_is_better".to_string(),
            ],
            2,
            "linux",
            "x86_64",
        );

        let summary = summarize_imported_receipt(&receipt).expect("imported summary");

        assert_eq!(summary.source_kind, "custom_json");
        assert_eq!(
            summary.metric_mappings,
            vec![
                "wall_ms <= duration_ms (ms, lower_is_better)".to_string(),
                "throughput_per_s <= rps (rps, higher_is_better)".to_string(),
            ]
        );
    }

    #[test]
    fn returns_none_for_native_perfgate_receipts() {
        let mut receipt = imported_receipt(vec!["true".to_string()], 7, "linux", "x86_64");
        receipt.tool.name = "perfgate".to_string();

        assert!(summarize_imported_receipt(&receipt).is_none());
    }
}
