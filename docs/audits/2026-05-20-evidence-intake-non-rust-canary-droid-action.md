# Evidence Intake Non-Rust Canary: droid-action

Date: 2026-05-20
Repo shape: external TypeScript GitHub Action repository
Canary repo: `H:\Code\Typescript\droid-action`
Canary repo branch: `sz/99-fork-smoke-harness`
Canary repo commit: `3f325c1`
perfgate source: `D:\Code\Rust\perfgate-swarm`
perfgate source commit: `511a1d2`
perfgate binary: source-built `target/debug/perfgate.exe`
Support/status impact: supports the 0.21 non-Rust evidence-intake canary row
in `docs/status/CANARY_MATRIX.md`. This is current-source proof, not public
release artifact proof.

## Purpose

Prove that a real non-Rust repository can keep an existing command workflow,
express command timing as explicit generic command JSON, import it into
perfgate receipts, and review the result through baseline maturity, signal
maturity, policy posture, review packet, and local check repair surfaces.

The canary used `bun run typecheck` as the workload. The broader
`bun test` suite was not used as the performance source because it currently
has unrelated failing tests in the external repository. This canary proves a
passing TypeScript command smoke path, not overall `droid-action` correctness.

## Setup

The canary wrote temporary files under:

```text
target/canaries/droid-action-non-rust-intake/
```

No files were written to the `droid-action` worktree. The worktree was clean
before and after the canary.

The generated canary config used an explicit working-directory wrapper so
review reproduction does not depend on the caller's current directory:

```toml
[defaults]
repeat = 5
warmup = 1
threshold = 1.00
warn_factor = 0.50
noise_threshold = 1.00
noise_policy = "warn"
out_dir = "D:/Code/Rust/perfgate-swarm/target/canaries/droid-action-non-rust-intake/artifacts/perfgate"
baseline_dir = "D:/Code/Rust/perfgate-swarm/target/canaries/droid-action-non-rust-intake/baselines"

[[bench]]
name = "droid-action-typecheck"
command = ["powershell", "-NoProfile", "-Command", "Set-Location 'H:/Code/Typescript/droid-action'; bun run typecheck"]
required = false
```

The source evidence was generic command JSON with explicit metric unit and
direction:

```text
source_kind: generic_command_json
bench: droid-action-typecheck
metric: wall_ms
unit: ms
direction: lower_is_better
samples: 5 measured command samples
host: windows-x86_64
```

## Commands

```bash
perfgate ingest --format generic-command-json --input target/canaries/droid-action-non-rust-intake/source-baseline.json --out target/canaries/droid-action-non-rust-intake/artifacts/perfgate/droid-action-typecheck/run.json --pretty
perfgate baseline doctor --config target/canaries/droid-action-non-rust-intake/perfgate.toml --bench droid-action-typecheck
perfgate baseline promote --config target/canaries/droid-action-non-rust-intake/perfgate.toml --bench droid-action-typecheck --current target/canaries/droid-action-non-rust-intake/artifacts/perfgate/droid-action-typecheck/run.json --to target/canaries/droid-action-non-rust-intake/baselines/droid-action-typecheck.json --pretty --force
perfgate ingest --format generic-command-json --input target/canaries/droid-action-non-rust-intake/source-current.json --out target/canaries/droid-action-non-rust-intake/artifacts/perfgate/droid-action-typecheck/run.json --pretty
perfgate compare --baseline target/canaries/droid-action-non-rust-intake/baselines/droid-action-typecheck.json --current target/canaries/droid-action-non-rust-intake/artifacts/perfgate/droid-action-typecheck/run.json --threshold 1.00 --noise-threshold 1.00 --noise-policy warn --out target/canaries/droid-action-non-rust-intake/artifacts/perfgate/droid-action-typecheck/compare.json --pretty
perfgate report --compare target/canaries/droid-action-non-rust-intake/artifacts/perfgate/droid-action-typecheck/compare.json --out target/canaries/droid-action-non-rust-intake/artifacts/perfgate/droid-action-typecheck/report.json --md target/canaries/droid-action-non-rust-intake/artifacts/perfgate/droid-action-typecheck/comment.md --pretty
perfgate baseline doctor --config target/canaries/droid-action-non-rust-intake/perfgate.toml --bench droid-action-typecheck
perfgate doctor signal --config target/canaries/droid-action-non-rust-intake/perfgate.toml --bench droid-action-typecheck
perfgate policy doctor --config target/canaries/droid-action-non-rust-intake/perfgate.toml --bench droid-action-typecheck
perfgate policy review-packet --config target/canaries/droid-action-non-rust-intake/perfgate.toml --bench droid-action-typecheck --out-dir target/canaries/droid-action-non-rust-intake/artifacts/perfgate --out target/canaries/droid-action-non-rust-intake/artifacts/perfgate/droid-action-typecheck/review-packet.md
perfgate check --config target/canaries/droid-action-non-rust-intake/perfgate-check.toml --bench droid-action-typecheck --require-baseline
```

The final check used a separate `perfgate-check.toml` and artifact directory so
it could prove reviewer reproduction and repair-context generation without
overwriting the imported-evidence review packet artifacts.

## Result

The import-to-review path passed:

```text
typecheck command: pass
ingest baseline: pass
baseline doctor before promotion: missing baseline guidance
baseline promote: pass
ingest current: pass
compare imported evidence: pass
report/comment: pass
baseline doctor after promotion: high_noise
signal doctor: increase_samples
policy doctor: advisory
policy review packet: generated
separate check reproduction: exit 0, warning/regression plus high_noise guidance
repair_context.json from check reproduction: generated
```

The imported compare receipt produced a passing verdict under the intentionally
wide canary threshold:

```text
metric: wall_ms
baseline median: 2175 ms
current median: 1905 ms
delta: -12.41%
budget: 100.0% lower-is-better
status: pass
```

The separate `check --require-baseline` run compared a fresh native command run
against the imported baseline and produced advisory warning output because the
benchmark is configured as `required = false`:

```text
status: warn
wall_ms regression: 96.83%
cv: 36.59%
repair_context.json: generated
```

That warning is useful canary evidence: perfgate reproduced the non-Rust
command locally, generated Action-consumable artifacts, and told reviewers not
to treat noisy command evidence as release proof.

## Maturity And Policy Output

`baseline doctor` classified the promoted imported baseline as:

```text
status: high_noise
samples: 5 measured samples
cv: 12.6%
host: windows-x86_64
source: imported (generic_command_json)
sample model: raw_samples
host context: present
noise support: sample_cv_available
recommendation: keep advisory; calibrate or use paired mode before blocking PRs
```

`doctor signal` recommended:

```text
recommendation: increase_samples
meaning: collect more measured samples before tightening or blocking
```

`policy doctor` kept the benchmark advisory:

```text
current posture: advisory
recommended posture: advisory
baseline maturity: high_noise
signal confidence: increase_samples
evidence source: imported (generic_command_json)
missing:
  - paired-mode or calibration review
  - signal sample count
```

The review packet included imported-evidence source metadata, metric mapping,
local artifacts, reviewer commands, agent guardrails, and do-not guidance.

## What This Proves

- A real non-Rust TypeScript repository can feed command evidence into
  `perfgate ingest --format generic-command-json`.
- The imported run receipt can be promoted into a baseline and compared against
  a later imported run.
- `baseline doctor`, `doctor signal`, `policy doctor`, and `policy
  review-packet` surface imported-evidence metadata and maturity limits.
- A separate local `perfgate check --require-baseline` can reproduce the
  non-Rust command, generate report/comment/repair-context artifacts, and keep
  noisy advisory evidence non-blocking.
- No policy, threshold, baseline, or server setting is changed by maturity or
  policy doctor output.

## What This Does Not Prove

- This does not prove public release artifacts for 0.21 adapters.
- This does not prove hosted GitHub Action import workflows.
- This does not prove k6, pytest-benchmark, hyperfine, or Criterion adoption in
  an external non-Rust repo.
- This does not prove HTTP/load-test evidence or production capacity.
- This does not prove shell portability beyond this Windows PowerShell command
  wrapper.
- This does not prove the `bun run typecheck` workload is a good PR gate.
- This does not prove the external repo's full test suite.
- This does not make server ledger mode part of local correctness.

## Follow-Up

The remaining 0.21 proof gap is hosted Action intake from current-source or a
future public release. HTTP/k6 and tool-specific non-Rust adapter canaries can
follow when a suitable external repo is available.
