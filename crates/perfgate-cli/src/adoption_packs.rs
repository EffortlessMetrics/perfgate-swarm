//! Reviewable adoption packs for common repository shapes.

use clap::{Subcommand, ValueEnum};
use std::fmt::Write as _;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum AdoptionPackName {
    #[value(name = "rust-cli")]
    RustCli,
    #[value(name = "rust-workspace")]
    RustWorkspace,
    #[value(name = "python-service")]
    PythonService,
    #[value(name = "node-tool-action")]
    NodeToolAction,
    #[value(name = "http-local-smoke")]
    HttpLocalSmoke,
    #[value(name = "generic-command")]
    GenericCommand,
}

impl AdoptionPackName {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RustCli => "rust-cli",
            Self::RustWorkspace => "rust-workspace",
            Self::PythonService => "python-service",
            Self::NodeToolAction => "node-tool-action",
            Self::HttpLocalSmoke => "http-local-smoke",
            Self::GenericCommand => "generic-command",
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum AdoptionAction {
    /// List reviewable adoption packs without changing config.
    Packs {
        /// Show one adoption pack instead of the full catalog.
        #[arg(long)]
        pack: Option<AdoptionPackName>,
    },
}

#[derive(Debug)]
pub struct AdoptionPack {
    pub name: &'static str,
    pub repo_shape: &'static str,
    pub start_with: &'static str,
    pub benchmark_source: &'static str,
    pub evidence_intake: &'static str,
    pub expected_artifacts: &'static [&'static str],
    pub starting_policy: &'static str,
    pub action_posture: &'static str,
    pub local_reproduction: &'static [&'static str],
    pub promotion_path: &'static [&'static str],
    pub known_bad_fits: &'static [&'static str],
    pub do_not_infer: &'static [&'static str],
}

const ADOPTION_PACKS: &[AdoptionPack] = &[
    AdoptionPack {
        name: "rust-cli",
        repo_shape: "Small Rust CLI with one or two fast command workloads.",
        start_with: "perfgate init --ci github --profile standard --suggest-benches rust-cli-smoke",
        benchmark_source: "native perfgate check first; Criterion or hyperfine import when a dedicated benchmark already exists",
        evidence_intake: "perfgate check for first-party command receipts, or perfgate ingest --format criterion/hyperfine for existing benchmark artifacts",
        expected_artifacts: &[
            "artifacts/perfgate/<bench>/run.json",
            "artifacts/perfgate/<bench>/compare.json",
            "artifacts/perfgate/<bench>/report.json",
            "artifacts/perfgate/<bench>/comment.md",
            "artifacts/perfgate/<bench>/repair_context.json",
        ],
        starting_policy: "advisory, then gate_candidate for one mature fast command",
        action_posture: "GitHub Action may block only after baseline and signal maturity are reviewed",
        local_reproduction: &[
            "perfgate check --config perfgate.toml --bench <bench>",
            "perfgate policy review-packet --config perfgate.toml --bench <bench>",
        ],
        promotion_path: &[
            "prove repeated local and CI signal is stable",
            "review perfgate baseline doctor and perfgate doctor signal output",
            "emit a non-mutating policy patch before making the benchmark blocking",
        ],
        known_bad_fits: &[
            "using compile-heavy cargo run --release as the first required gate",
            "treating --help startup smoke as parser or throughput proof",
        ],
        do_not_infer: &[
            "one fast CLI check proves all CLI paths are mature",
            "Criterion confidence intervals replace perfgate maturity policy",
            "server ledger mode is required for correctness",
        ],
    },
    AdoptionPack {
        name: "rust-workspace",
        repo_shape: "Larger Rust workspace where compile, test, and integration setup can dominate.",
        start_with: "perfgate init --ci github --profile standard --suggest-benches rust-workspace-advisory",
        benchmark_source: "advisory workspace command plus smaller package or Criterion imports for gate candidates",
        evidence_intake: "perfgate check for scoped commands, perfgate ingest --format criterion for benchmark artifacts, or hyperfine for command timing",
        expected_artifacts: &[
            "artifacts/perfgate/<workspace-smoke>/run.json",
            "artifacts/perfgate/<package-bench>/compare.json",
            "artifacts/perfgate/<package-bench>/review-packet.md",
        ],
        starting_policy: "advisory until compile/setup noise is separated from workload movement",
        action_posture: "Action summary should explain advisory workspace signal separately from required package gates",
        local_reproduction: &[
            "perfgate check --config perfgate.toml --bench <package-bench>",
            "perfgate doctor signal --config perfgate.toml --bench <package-bench>",
            "perfgate policy doctor --config perfgate.toml --bench <package-bench>",
        ],
        promotion_path: &[
            "keep broad workspace timing advisory",
            "promote one scoped workload at a time",
            "consider paired mode when runner drift changes the verdict",
        ],
        known_bad_fits: &[
            "making cargo test --workspace a required performance gate before calibration",
            "using a broad command with no workload owner",
        ],
        do_not_infer: &[
            "a mature package benchmark proves the whole workspace",
            "compile-heavy command timing is runtime proof",
            "busy-runner fallback proves self-hosted runner performance",
        ],
    },
    AdoptionPack {
        name: "python-service",
        repo_shape: "Python service or library with pytest-benchmark or a dedicated benchmark script.",
        start_with: "perfgate init --ci github --profile standard --suggest-benches python-command",
        benchmark_source: "pytest-benchmark JSON when available; otherwise a stable python benchmark command",
        evidence_intake: "perfgate ingest --format pytest, or perfgate ingest --format custom-json/custom-csv with explicit metric mapping",
        expected_artifacts: &[
            "artifacts/pytest-benchmark.json",
            "artifacts/perfgate/<bench>/run.json",
            "artifacts/perfgate/<bench>/repair_context.json",
        ],
        starting_policy: "advisory until interpreter startup, dependency setup, and sample model are understood",
        action_posture: "Action summary should keep correctness-test failures separate from performance maturity",
        local_reproduction: &[
            "pytest --benchmark-json=artifacts/pytest-benchmark.json",
            "perfgate ingest --format pytest --input artifacts/pytest-benchmark.json --out artifacts/perfgate/<bench>/run.json",
            "perfgate policy review-packet --config perfgate.toml --bench <bench>",
        ],
        promotion_path: &[
            "prefer raw benchmark samples over summary-only evidence",
            "review Python/runtime limits in the imported evidence output",
            "promote only deterministic service workloads",
        ],
        known_bad_fits: &[
            "package installation or network startup inside the timed command",
            "treating passing pytest correctness tests as performance maturity",
        ],
        do_not_infer: &[
            "summary-only pytest evidence has full noise support",
            "unknown host context is compatible",
            "first imported result should become a baseline",
        ],
    },
    AdoptionPack {
        name: "node-tool-action",
        repo_shape: "Node CLI, tool, or GitHub Action with scriptable local benchmarks.",
        start_with: "perfgate init --ci github --profile standard --suggest-benches node-command",
        benchmark_source: "dedicated node benchmark script, hyperfine command timing, or custom JSON/CSV output",
        evidence_intake: "perfgate check for a stable script, hyperfine import for command timing, or custom mapping for tool-specific metrics",
        expected_artifacts: &[
            "artifacts/node-bench.json",
            "artifacts/perfgate/<bench>/run.json",
            "artifacts/perfgate/<bench>/comment.md",
        ],
        starting_policy: "advisory until JIT warmup, dependency cache, and runner variance are reviewed",
        action_posture: "Action summary should show advisory signal before any workflow starts blocking",
        local_reproduction: &[
            "node scripts/bench.js",
            "perfgate ingest --format custom-json --input artifacts/node-bench.json --metric wall_ms=duration_ms,unit=ms,direction=lower_is_better --out artifacts/perfgate/<bench>/run.json",
            "perfgate doctor signal --config perfgate.toml --bench <bench>",
        ],
        promotion_path: &[
            "separate install/build time from measured tool runtime",
            "calibrate after warmup behavior is visible",
            "promote only scripts with fixed local input",
        ],
        known_bad_fits: &[
            "npm install or package download inside the timed path",
            "benchmarks that call shared network services",
        ],
        do_not_infer: &[
            "a fast action smoke proves all workflow paths",
            "JIT-sensitive evidence is stable without repeated samples",
            "custom imports infer metric meaning without explicit mapping",
        ],
    },
    AdoptionPack {
        name: "http-local-smoke",
        repo_shape: "Local HTTP service smoke or load-test path with isolated dependencies.",
        start_with: "perfgate init --ci github --profile standard --suggest-benches http-smoke",
        benchmark_source: "local curl/script smoke first; k6 summary JSON when the team already has load-test output",
        evidence_intake: "perfgate check for local endpoint smoke, or perfgate ingest --format k6 for summary JSON",
        expected_artifacts: &[
            "artifacts/k6-summary.json",
            "artifacts/perfgate/<bench>/run.json",
            "artifacts/perfgate/<bench>/review-packet.md",
        ],
        starting_policy: "smoke or advisory; candidate policy only for isolated local endpoints",
        action_posture: "Action summary should say when HTTP evidence is not production capacity proof",
        local_reproduction: &[
            "k6 run --summary-export artifacts/k6-summary.json scripts/http-smoke.js",
            "perfgate ingest --format k6 --input artifacts/k6-summary.json --name <bench> --out artifacts/perfgate/<bench>/run.json",
            "perfgate policy review-packet --config perfgate.toml --bench <bench>",
        ],
        promotion_path: &[
            "keep shared or internet-backed endpoints advisory",
            "use paired mode or repeat evidence when runner networking dominates",
            "treat throughput/memory tradeoffs as structured decision candidates",
        ],
        known_bad_fits: &[
            "shared staging services as required PR gates",
            "treating local k6 output as production capacity evidence",
        ],
        do_not_infer: &[
            "successful HTTP smoke proves load capacity",
            "summary-only k6 output has raw per-request samples",
            "server ledger history is part of local correctness",
        ],
    },
    AdoptionPack {
        name: "generic-command",
        repo_shape: "Language-neutral repository with an existing benchmark command or artifact.",
        start_with: "perfgate init --ci github --profile standard --suggest-benches generic-command",
        benchmark_source: "stable local command, generic command JSON, or custom JSON/CSV with explicit metric mapping",
        evidence_intake: "perfgate check for direct commands, perfgate ingest --format generic-command-json, or custom-json/custom-csv for existing artifacts",
        expected_artifacts: &[
            "artifacts/source-evidence.json",
            "artifacts/perfgate/<bench>/run.json",
            "artifacts/perfgate/<bench>/repair_context.json",
        ],
        starting_policy: "advisory until unit, direction, host context, and sample model are explicit",
        action_posture: "Action summary should show source limits and the exact local reproduction command",
        local_reproduction: &[
            "./scripts/bench.sh > artifacts/source-evidence.json",
            "perfgate ingest --format generic-command-json --input artifacts/source-evidence.json --out artifacts/perfgate/<bench>/run.json",
            "perfgate baseline doctor --config perfgate.toml --bench <bench>",
        ],
        promotion_path: &[
            "fail closed on ambiguous units or metric direction",
            "prefer raw samples before required gates",
            "emit a reviewable policy patch instead of changing gates silently",
        ],
        known_bad_fits: &[
            "commands with mutable external data",
            "artifacts that omit metric unit or direction",
        ],
        do_not_infer: &[
            "generic import knows what a metric means without mapping",
            "missing host fields prove compatibility",
            "advisory evidence should block CI by default",
        ],
    },
];

pub fn adoption_packs() -> &'static [AdoptionPack] {
    ADOPTION_PACKS
}

pub fn adoption_pack(name: AdoptionPackName) -> &'static AdoptionPack {
    adoption_packs()
        .iter()
        .find(|pack| pack.name == name.as_str())
        .expect("all AdoptionPackName values have catalog entries")
}

pub fn render_adoption_packs(filter: Option<AdoptionPackName>) -> String {
    let mut out = String::new();
    out.push_str(
        "Adoption packs are reviewable starting points for existing benchmark ecosystems.\n",
    );
    out.push_str(
        "They do not detect benchmarks magically, promote baselines, make checks blocking, loosen thresholds, or require server ledger mode.\n\n",
    );

    let packs: Vec<&AdoptionPack> = match filter {
        Some(name) => vec![adoption_pack(name)],
        None => adoption_packs().iter().collect(),
    };

    for (idx, pack) in packs.iter().enumerate() {
        if idx > 0 {
            out.push('\n');
        }
        render_pack(&mut out, pack);
    }

    out
}

fn render_pack(out: &mut String, pack: &AdoptionPack) {
    let _ = writeln!(out, "Pack: {}", pack.name);
    let _ = writeln!(out, "Repo shape: {}", pack.repo_shape);
    let _ = writeln!(out, "Start with: {}", pack.start_with);
    let _ = writeln!(out, "Benchmark source: {}", pack.benchmark_source);
    let _ = writeln!(out, "Evidence intake: {}", pack.evidence_intake);
    render_list(out, "Expected artifacts", pack.expected_artifacts);
    let _ = writeln!(out, "Starting policy: {}", pack.starting_policy);
    let _ = writeln!(out, "Action posture: {}", pack.action_posture);
    render_list(out, "Local reproduction", pack.local_reproduction);
    render_list(out, "Promotion path", pack.promotion_path);
    render_list(out, "Known bad fits", pack.known_bad_fits);
    render_list(out, "Do not infer", pack.do_not_infer);
}

fn render_list(out: &mut String, label: &str, items: &[&str]) {
    let _ = writeln!(out, "{label}:");
    for item in items {
        let _ = writeln!(out, "  - {item}");
    }
}

pub fn execute_adoption_action(action: AdoptionAction) -> anyhow::Result<()> {
    match action {
        AdoptionAction::Packs { pack } => {
            print!("{}", render_adoption_packs(pack));
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_contains_planned_adoption_packs() {
        let names: Vec<_> = adoption_packs().iter().map(|pack| pack.name).collect();
        assert_eq!(
            names,
            vec![
                "rust-cli",
                "rust-workspace",
                "python-service",
                "node-tool-action",
                "http-local-smoke",
                "generic-command",
            ]
        );
    }

    #[test]
    fn rendered_catalog_preserves_review_boundaries() {
        let rendered = render_adoption_packs(None);
        assert!(rendered.contains("reviewable starting points"));
        assert!(rendered.contains("do not detect benchmarks magically"));
        assert!(rendered.contains("promote baselines"));
        assert!(rendered.contains("require server ledger mode"));
        assert!(rendered.contains("Pack: rust-cli"));
        assert!(rendered.contains("Pack: generic-command"));
        assert!(rendered.contains("Local reproduction:"));
        assert!(rendered.contains("Do not infer:"));
    }

    #[test]
    fn rendered_single_pack_excludes_other_packs() {
        let rendered = render_adoption_packs(Some(AdoptionPackName::HttpLocalSmoke));
        assert!(rendered.contains("Pack: http-local-smoke"));
        assert!(rendered.contains("k6 summary JSON"));
        assert!(!rendered.contains("Pack: rust-cli"));
    }
}
