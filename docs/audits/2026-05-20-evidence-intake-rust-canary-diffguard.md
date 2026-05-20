# Evidence Intake Rust Canary: diffguard

Date: 2026-05-20
Repo shape: external Rust CLI workspace
Canary repo: `D:\Code\Rust\diffguard`
Canary repo commit: `804b4ce`
perfgate source: `D:\Code\Rust\perfgate-swarm`
perfgate source commit: `f7ad829`
perfgate binary: source-built `target/debug/perfgate.exe`
Support/status impact: supports the 0.21 Rust evidence-intake canary row in
`docs/status/CANARY_MATRIX.md`. This is current-source proof, not public release
artifact proof.

## Purpose

Prove that a real Rust CLI repository can keep an existing command workload,
express it as explicit generic command JSON, import it into perfgate receipts,
and then review the result through maturity, policy, and review-packet
surfaces.

The canary used `diffguard.exe --help` as a small command workload. This is a
smoke workload, not throughput or parser proof.

## Setup

The canary wrote temporary files under:

```text
target/canaries/diffguard-rust-intake/
```

No files were written to the `diffguard` worktree.

The generated canary config was:

```toml
[defaults]
repeat = 5
warmup = 1
threshold = 1.00
warn_factor = 0.50
noise_threshold = 1.00
noise_policy = "warn"
out_dir = "D:/Code/Rust/perfgate-swarm/target/canaries/diffguard-rust-intake/artifacts/perfgate"
baseline_dir = "D:/Code/Rust/perfgate-swarm/target/canaries/diffguard-rust-intake/baselines"

[[bench]]
name = "diffguard-help"
command = ["D:/Code/Rust/diffguard/target/debug/diffguard.exe", "--help"]
required = false
```

The source evidence was generic command JSON with explicit metric unit and
direction:

```text
source_kind: generic_command_json
bench: diffguard-help
metric: wall_ms
unit: ms
direction: lower_is_better
samples: 5 measured command samples
host: windows-x86_64
```

## Commands

```bash
perfgate ingest --format generic-command-json --input target/canaries/diffguard-rust-intake/source-baseline.json --out target/canaries/diffguard-rust-intake/artifacts/perfgate/diffguard-help/run.json --pretty
perfgate baseline doctor --config target/canaries/diffguard-rust-intake/perfgate.toml --bench diffguard-help
perfgate baseline promote --config target/canaries/diffguard-rust-intake/perfgate.toml --bench diffguard-help --current target/canaries/diffguard-rust-intake/artifacts/perfgate/diffguard-help/run.json --to target/canaries/diffguard-rust-intake/baselines/diffguard-help.json --pretty --force
perfgate ingest --format generic-command-json --input target/canaries/diffguard-rust-intake/source-current.json --out target/canaries/diffguard-rust-intake/artifacts/perfgate/diffguard-help/run.json --pretty
perfgate compare --baseline target/canaries/diffguard-rust-intake/baselines/diffguard-help.json --current target/canaries/diffguard-rust-intake/artifacts/perfgate/diffguard-help/run.json --threshold 1.00 --noise-threshold 1.00 --noise-policy warn --out target/canaries/diffguard-rust-intake/artifacts/perfgate/diffguard-help/compare.json --pretty
perfgate report --compare target/canaries/diffguard-rust-intake/artifacts/perfgate/diffguard-help/compare.json --out target/canaries/diffguard-rust-intake/artifacts/perfgate/diffguard-help/report.json --md target/canaries/diffguard-rust-intake/artifacts/perfgate/diffguard-help/comment.md --pretty
perfgate baseline doctor --config target/canaries/diffguard-rust-intake/perfgate.toml --bench diffguard-help
perfgate doctor signal --config target/canaries/diffguard-rust-intake/perfgate.toml --bench diffguard-help
perfgate policy doctor --config target/canaries/diffguard-rust-intake/perfgate.toml --bench diffguard-help
perfgate policy review-packet --config target/canaries/diffguard-rust-intake/perfgate.toml --bench diffguard-help --out-dir target/canaries/diffguard-rust-intake/artifacts/perfgate --out target/canaries/diffguard-rust-intake/artifacts/perfgate/diffguard-help/review-packet.md
```

## Result

The canary passed the import-to-review path:

```text
ingest baseline: pass
baseline doctor before promotion: missing baseline guidance
baseline promote: pass
ingest current: pass
compare: pass
report/comment: pass
baseline doctor after promotion: high_noise
signal doctor: use_paired_mode
policy doctor: advisory
policy review packet: generated
```

The compare receipt produced a passing gate verdict under the intentionally
wide canary threshold:

```text
metric: wall_ms
baseline median: 175 ms
current median: 121 ms
delta: -30.86%
budget: 100.0% lower-is-better
status: pass
```

The important product result is not the apparent improvement. The important
result is that maturity and policy output refused to turn noisy imported smoke
evidence into blocking policy.

## Maturity And Policy Output

`baseline doctor` classified the promoted baseline as:

```text
status: high_noise
samples: 5 measured samples
cv: 19.7%
host: windows-x86_64
source: imported (generic_command_json)
sample model: raw_samples
host context: present
noise support: sample_cv_available
recommendation: keep advisory; calibrate or use paired mode before blocking PRs
```

`doctor signal` recommended:

```text
recommendation: use_paired_mode
meaning: ordinary runs are noisy; compare baseline/current under paired conditions
```

`policy doctor` kept the benchmark advisory:

```text
current posture: advisory
recommended posture: advisory
baseline maturity: high_noise
signal confidence: use_paired_mode
evidence source: imported (generic_command_json)
host context: present
missing:
  - paired-mode or calibration review
  - paired-mode evidence
```

The review packet included imported-evidence source metadata, metric mapping,
local artifacts, reviewer commands, agent guardrails, and do-not guidance.

## What This Proves

- A real Rust CLI repo can feed command evidence into `perfgate ingest
  --format generic-command-json`.
- The imported run receipt can be promoted into a baseline and compared against
  a later imported run.
- `baseline doctor`, `doctor signal`, `policy doctor`, and `policy
  review-packet` surface imported-evidence metadata and maturity limits.
- No policy, threshold, baseline, or server setting is changed by maturity or
  policy doctor output.
- Noisy imported smoke evidence stays advisory and points reviewers toward
  calibration or paired mode.

## What This Does Not Prove

- This does not prove public release artifacts for 0.21 adapters.
- This does not prove hosted GitHub Action import workflows.
- This does not prove Criterion or hyperfine adoption in an external Rust repo.
- This does not prove the workload is a good PR gate.
- This does not prove parser throughput, steady-state behavior, or production
  performance.
- This does not prove non-Rust intake paths.
- This does not make server ledger mode part of local correctness.

## Follow-Up

The remaining 0.21 canary gap is a non-Rust command, Python, Node, HTTP, JSON,
CSV, or k6 repo that proves the same import-to-review path outside a Rust CLI
workspace.
