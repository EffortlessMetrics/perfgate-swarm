//! Shared benchmark passport rendering for review surfaces.

use serde::Serialize;

use crate::baseline_doctor::BaselineDoctorRow;
use crate::doctor::SignalDoctorRow;
use crate::imported_evidence::ImportedEvidenceSummary;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BenchmarkPassport {
    pub(crate) bench: String,
    pub(crate) source_kind: String,
    pub(crate) source_artifact: Option<String>,
    pub(crate) sample_model: String,
    pub(crate) host_context: String,
    pub(crate) noise_support: String,
    pub(crate) metric_mappings: Vec<String>,
    pub(crate) baseline_status: String,
    pub(crate) signal_maturity: String,
    pub(crate) policy_posture: String,
    pub(crate) proof_freshness: String,
    pub(crate) known_non_inferences: Vec<String>,
    pub(crate) next_safe_action: String,
}

impl BenchmarkPassport {
    pub(crate) fn from_rows(
        baseline: &BaselineDoctorRow,
        signal: &SignalDoctorRow,
        policy_posture: impl Into<String>,
        proof_freshness: impl Into<String>,
        known_non_inferences: Vec<String>,
        next_safe_action: impl Into<String>,
    ) -> Self {
        let source = BenchmarkPassportSource::from_rows(baseline, signal);
        Self {
            bench: baseline.bench.clone(),
            source_kind: source.kind,
            source_artifact: source.artifact,
            sample_model: source.sample_model,
            host_context: source.host_context,
            noise_support: source.noise_support,
            metric_mappings: source.metric_mappings,
            baseline_status: baseline.maturity.as_str().to_string(),
            signal_maturity: signal.recommendation.as_str().to_string(),
            policy_posture: policy_posture.into(),
            proof_freshness: proof_freshness.into(),
            known_non_inferences,
            next_safe_action: next_safe_action.into(),
        }
    }

    pub(crate) fn render_terminal(&self, out: &mut String) {
        out.push_str("\nBenchmark passport:\n");
        out.push_str(&format!("  bench: {}\n", self.bench));
        out.push_str(&format!("  source kind: {}\n", self.source_kind));
        out.push_str(&format!(
            "  source artifact: {}\n",
            self.source_artifact.as_deref().unwrap_or("unrecorded")
        ));
        out.push_str(&format!("  sample model: {}\n", self.sample_model));
        out.push_str(&format!("  host context: {}\n", self.host_context));
        out.push_str(&format!("  noise support: {}\n", self.noise_support));
        out.push_str(&format!("  baseline status: {}\n", self.baseline_status));
        out.push_str(&format!("  signal maturity: {}\n", self.signal_maturity));
        out.push_str(&format!("  policy posture: {}\n", self.policy_posture));
        out.push_str(&format!("  proof freshness: {}\n", self.proof_freshness));
        out.push_str(&format!("  next safe action: {}\n", self.next_safe_action));
        if !self.metric_mappings.is_empty() {
            out.push_str("  metric mappings:\n");
            for mapping in &self.metric_mappings {
                out.push_str(&format!("    - {mapping}\n"));
            }
        }
        if !self.known_non_inferences.is_empty() {
            out.push_str("  known non-inferences:\n");
            for item in &self.known_non_inferences {
                out.push_str(&format!("    - {item}\n"));
            }
        }
    }

    pub(crate) fn render_markdown(&self, out: &mut String) {
        out.push_str("\n## Benchmark Passport\n\n");
        out.push_str(&format!("- Bench: `{}`\n", self.bench));
        out.push_str(&format!("- Source kind: `{}`\n", self.source_kind));
        out.push_str(&format!(
            "- Source artifact: `{}`\n",
            self.source_artifact.as_deref().unwrap_or("unrecorded")
        ));
        out.push_str(&format!("- Sample model: `{}`\n", self.sample_model));
        out.push_str(&format!("- Host context: `{}`\n", self.host_context));
        out.push_str(&format!("- Noise support: `{}`\n", self.noise_support));
        out.push_str(&format!("- Baseline status: `{}`\n", self.baseline_status));
        out.push_str(&format!("- Signal maturity: `{}`\n", self.signal_maturity));
        out.push_str(&format!("- Policy posture: `{}`\n", self.policy_posture));
        out.push_str(&format!("- Proof freshness: {}\n", self.proof_freshness));
        out.push_str(&format!(
            "- Next safe action: `{}`\n",
            self.next_safe_action
        ));
        if !self.metric_mappings.is_empty() {
            out.push_str("- Metric mappings:\n");
            for mapping in &self.metric_mappings {
                out.push_str(&format!("  - `{mapping}`\n"));
            }
        }
        if !self.known_non_inferences.is_empty() {
            out.push_str("- Known non-inferences:\n");
            for item in &self.known_non_inferences {
                out.push_str(&format!("  - {item}\n"));
            }
        }
    }
}

struct BenchmarkPassportSource {
    kind: String,
    artifact: Option<String>,
    sample_model: String,
    host_context: String,
    noise_support: String,
    metric_mappings: Vec<String>,
}

impl BenchmarkPassportSource {
    fn from_rows(baseline: &BaselineDoctorRow, signal: &SignalDoctorRow) -> Self {
        if let Some(imported) = imported_evidence(baseline, signal) {
            return Self::from_imported(imported);
        }

        Self {
            kind: "native perfgate run".to_string(),
            artifact: Some(signal.run_path.display().to_string()),
            sample_model: "native_receipts".to_string(),
            host_context: format!("perfgate_host_receipt ({})", signal.host_stability),
            noise_support: "native_samples_and_cv_when_present".to_string(),
            metric_mappings: vec!["perfgate run.v1 metrics".to_string()],
        }
    }

    fn from_imported(imported: &ImportedEvidenceSummary) -> Self {
        Self {
            kind: imported.source_label(),
            artifact: imported.source_path.clone(),
            sample_model: imported.sample_model.to_string(),
            host_context: imported.host_context.to_string(),
            noise_support: imported.noise_support.to_string(),
            metric_mappings: imported.metric_mappings.clone(),
        }
    }
}

fn imported_evidence<'a>(
    baseline: &'a BaselineDoctorRow,
    signal: &'a SignalDoctorRow,
) -> Option<&'a ImportedEvidenceSummary> {
    signal
        .imported_evidence
        .as_ref()
        .or(baseline.imported_evidence.as_ref())
}
