//! Derived metadata for receipts created by ingest adapters.
//!
//! `perfgate.run.v1` intentionally stays stable. These helpers infer the
//! review limits that maturity and policy surfaces should show for imported
//! receipts without adding schema fields.

use perfgate_types::RunReceipt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ImportedEvidenceSummary {
    pub(crate) source_kind: String,
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

    Some(ImportedEvidenceSummary {
        source_kind,
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
        "(ingested)" => "ingested_run_v1".to_string(),
        _ => "unrecorded_ingest_source".to_string(),
    }
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
            ],
            0,
            "unknown",
            "unknown",
        );

        let summary = summarize_imported_receipt(&receipt).expect("imported summary");

        assert_eq!(summary.source_kind, "k6_summary_json");
        assert_eq!(summary.sample_model, "summary_only");
        assert_eq!(summary.host_context, "missing_or_partial");
        assert_eq!(summary.noise_support, "limited_summary_only");
        assert!(summary.is_summary_only());
        assert!(summary.has_missing_host_context());
    }

    #[test]
    fn returns_none_for_native_perfgate_receipts() {
        let mut receipt = imported_receipt(vec!["true".to_string()], 7, "linux", "x86_64");
        receipt.tool.name = "perfgate".to_string();

        assert!(summarize_imported_receipt(&receipt).is_none());
    }
}
