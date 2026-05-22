# PERFGATE-SPEC-0002: First useful performance review contract

Status: implemented
Owner: product-platform
Created: 2026-05-21
Linked proposal: PERFGATE-PROP-0002
Linked ADRs: none
Linked lane: first-useful-performance-review
Linked issues:
Linked PRs:
Support-tier impact: planned product claims only after proof
Policy impact: advisory; no automatic promotion or blocking

## Problem

perfgate has mature evidence surfaces, but the first useful review still requires users to compose adoption guidance, ingestion, baseline health, signal maturity, policy posture, review packets, and repair context by hand.

The product contract for this lane is that perfgate should produce one reviewable answer for a PR without hiding uncertainty or promoting advisory evidence into blocking policy.

## Behavior

A first-use performance review must answer:

- what evidence exists
- where the evidence came from
- what metric moved
- what source artifact backs the evidence
- what the sample model supports
- what the baseline status is
- what the signal maturity is
- what the host context says
- what the policy posture is
- what must not be inferred
- what local command to run next
- what agents may safely inspect or fix
- what actions require human review

The review contract uses this outcome vocabulary:

- `setup_missing`
- `baseline_missing`
- `summary_only`
- `host_missing`
- `host_mismatch`
- `high_noise`
- `advisory_ok`
- `gate_candidate_ready`
- `required_gate_not_ready`
- `regression_review_required`
- `tradeoff_review_required`

Review output must preserve these boundaries:

- advisory evidence does not become blocking from rendering alone
- first-run evidence is not a mature baseline
- summary-only imported evidence is not raw-sample proof
- missing host context cannot prove host compatibility
- server ledger history is optional and not required for local correctness
- agents may inspect, rerun, summarize, and propose patches
- agents must not promote baselines, loosen thresholds, make gates blocking, accept tradeoffs, or require server ledger history without human review

## Non-goals

- dashboard
- scheduler
- new benchmark engine
- default blocking gates
- automatic baseline promotion
- automatic threshold loosening
- mandatory server ledger
- public crate expansion
- release, publish, signing, tag, or alias changes

## Required evidence

Each implementation slice must provide focused proof for the surface it changes.

The lane-level proof floor is:

```bash
cargo +1.95.0 run -p xtask -- rails check
cargo +1.95.0 run -p xtask -- docs-source-check
cargo +1.95.0 run -p xtask -- docs-check
cargo +1.95.0 run -p xtask -- doc-test
cargo +1.95.0 run -p xtask -- product-claims-check
git diff --check
```

Behavior slices should add CLI tests, fixtures, snapshots, or action-check proof before product claims move.

## Acceptance examples

For native evidence:

```text
perfgate adoption recommend
perfgate adoption apply --pack rust-cli --ci github --dry-run
perfgate check --config perfgate.toml --bench cli-help
perfgate review explain --config perfgate.toml --bench cli-help
```

For imported evidence:

```text
hyperfine --warmup 3 --runs 10 --export-json artifacts/hyperfine.json "cargo run -q -- --help"
perfgate ingest --format hyperfine --input artifacts/hyperfine.json --name cli-help --out artifacts/perfgate/cli-help/run.json
perfgate review explain --config perfgate.toml --bench cli-help
```

Expected first-run posture:

```text
Performance review: advisory

Benchmark passport:
  source: hyperfine
  sample model: raw_samples
  host context: present
  baseline: missing
  signal maturity: insufficient history
  policy posture: advisory

Do not infer:
  The first run is a trustworthy baseline.
  This host proves production compatibility.
  This benchmark should be required_gate.
```

## Test mapping

Planned test coverage:

- adoption recommendation fixtures for common repo shapes
- dry-run apply output snapshots
- review explain snapshots for native and imported evidence
- benchmark passport markdown and JSON fixtures
- repair context fixtures for missing baseline, noisy signal, regression, tradeoff candidate, and host mismatch
- baseline and policy promote-plan fixtures
- Action summary checks once the passport is surfaced there

## Implementation mapping

Likely owners:

- `crates/perfgate-cli/src/adoption*`
- `crates/perfgate-cli/src/ingest*`
- `crates/perfgate-cli/src/check*`
- `crates/perfgate-cli/src/repair_context*`
- `crates/perfgate-cli/src/policy*`
- `crates/perfgate-cli/src/baseline*`
- `action.yml` and action summary fixtures
- `.rails/` proposal/spec/plan/closeout artifacts
- `docs/status/PRODUCT_CLAIMS.md`

## CI proof

Expected recurring proof:

```bash
cargo +1.95.0 fmt --all -- --check
cargo +1.95.0 clippy -p perfgate-cli --all-targets --all-features -- -D warnings
cargo +1.95.0 test -p perfgate-cli --all-features adoption
cargo +1.95.0 test -p perfgate-cli --all-features review
cargo +1.95.0 test -p perfgate-cli --all-features policy
cargo +1.95.0 run -p xtask -- action-check
cargo +1.95.0 run -p xtask -- rails check
cargo +1.95.0 run -p xtask -- docs-source-check
cargo +1.95.0 run -p xtask -- product-claims-check
git diff --check
```

## Metrics / promotion rule

This contract can move from proposed to accepted when:

- the lane plan exists
- at least one implementation slice proves the review loop end to end in fixtures
- product claims remain conservative and proof-backed
- advisory/blocking boundaries remain explicit in CLI output, docs, and Action summaries

It can move from accepted to implemented only after closeout records:

- implemented commands
- review packet/passport surfaces
- agent guardrail coverage
- canary state
- known non-inferences
- remaining unproven surfaces

## Failure modes

These must not silently pass:

- an imported summary is treated as raw-sample proof
- a first run is represented as a mature baseline
- host-missing evidence is represented as host-compatible
- noisy evidence is represented as gate-ready
- advisory evidence changes exit-code behavior through summary rendering
- agents are told to promote baselines, loosen thresholds, make gates blocking, or accept tradeoffs without human review
- product claims cite stale, superseded, or unproven evidence as current support
