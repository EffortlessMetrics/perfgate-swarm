# Evidence Intake and Adoption Packs Closeout

Status: implemented
Owner: perfgate maintainers
Created: 2026-05-20
Milestone: 0.21.0
Linked proposal: [`PERFGATE-PROP-0008-evidence-intake-adoption-packs`](../proposals/PERFGATE-PROP-0008-evidence-intake-adoption-packs.md)
Linked specs: [`PERFGATE-SPEC-0013-evidence-source-contract`](../specs/PERFGATE-SPEC-0013-evidence-source-contract.md)
Linked ADRs: [`PERFGATE-ADR-0002-receipts-first-performance-decisions`](../adr/PERFGATE-ADR-0002-receipts-first-performance-decisions.md)
Linked plan: [`evidence-intake-adoption-packs.md`](../../plans/0.21.0/evidence-intake-adoption-packs.md)
Support/status impact: [`PRODUCT_CLAIMS.md`](../status/PRODUCT_CLAIMS.md), [`CANARY_MATRIX.md`](../status/CANARY_MATRIX.md), and [`PROOF_FRESHNESS.md`](../status/PROOF_FRESHNESS.md)
Proof commands: docs-check, doc-test, docs-source-check, product-claims-check, git diff --check

## Summary

This lane moved perfgate from policy-aware evidence review into existing
benchmark ecosystem intake. Teams can now keep their existing measurement
tools and still bring the result into perfgate receipts, maturity guidance,
policy posture, review packets, and Action-facing summaries.

The implemented path is:

```text
existing benchmark output
  -> explicit adapter mapping
  -> perfgate.run.v1 receipt
  -> baseline/signal maturity
  -> policy posture
  -> review packet and Action posture
```

The lane preserved the intended boundary: perfgate remains the evidence and
policy layer above measurement tools. It did not become another benchmark
engine, scheduler, dashboard, or mandatory server path.

## What Changed

Implemented intake adapters:

- `generic-command-json`, requiring explicit metric unit, direction, samples or
  summary, and host context where available;
- `hyperfine`, preserving command timing evidence with command/setup
  non-inferences;
- `criterion`, importing stable wall-time fields from cargo-criterion JSON,
  `raw.csv`, and summary fallback shapes while keeping Criterion statistics
  distinct from perfgate maturity policy;
- `pytest-benchmark`, preserving Python/runtime context where available while
  separating correctness test success from performance maturity;
- `k6`, importing summary-only HTTP/load-test evidence without describing local
  or shared-runner output as production capacity proof; and
- `custom-json` and `custom-csv`, requiring explicit field, unit, and direction
  mappings and failing closed for ambiguous inputs.

Implemented review surfaces:

- imported evidence metadata reaches baseline doctor, signal doctor,
  calibration, policy doctor, policy patches, review packets, and Action
  posture output where receipts expose source metadata;
- imported summary-only evidence is called out as weaker noise support;
- missing host, source path, mapping, or raw samples are review limits rather
  than native-proof assumptions;
- repair context remains agent-safe and does not authorize baseline promotion
  or threshold loosening; and
- Action checks preserve existing verdict and reproduction behavior.

Implemented adoption packs:

- `rust-cli`
- `rust-workspace`
- `python-service`
- `node-tool-action`
- `http-local-smoke`
- `generic-command`

Each pack names source, expected artifacts, local reproduction, Action posture,
promotion path, bad fits, and non-inferences. Packs are reviewable starting
points, not automatic benchmark selection or policy promotion.

## External Canaries

Two source-built external canaries are current for this lane:

- [`2026-05-20-evidence-intake-rust-canary-diffguard.md`](../audits/2026-05-20-evidence-intake-rust-canary-diffguard.md)
- [`2026-05-20-evidence-intake-non-rust-canary-droid-action.md`](../audits/2026-05-20-evidence-intake-non-rust-canary-droid-action.md)

The Rust canary proved that a real Rust CLI repo can import generic command
JSON into receipts, promote a baseline, compare a later imported run, and
review imported evidence through baseline doctor, signal doctor, policy doctor,
and review packet output.

The non-Rust canary proved the same source-built local path in a TypeScript
GitHub Action repository. It also ran a separate local
`perfgate check --require-baseline` against the imported baseline and generated
report/comment/repair-context artifacts while keeping noisy command evidence
advisory.

Both canaries deliberately cite current-source proof only. They are not public
release artifact proof.

## Product Claims

Product claims now map the 0.21 support surface conservatively:

- PG-CLAIM-0031 covers implemented evidence intake adapters.
- PG-CLAIM-0032 covers imported evidence in maturity, policy, review packet,
  and Action posture surfaces.
- PG-CLAIM-0033 covers reviewable adoption packs for common repo shapes.

Known limits remain visible in the claims and canary matrix. In particular,
source-built Rust and non-Rust canaries do not prove hosted Action intake,
public release artifacts, every upstream tool JSON variant, every shell, or
that smoke workloads are good PR gates.

## Proof Records

Durable lane artifacts:

- [`PERFGATE-PROP-0008-evidence-intake-adoption-packs`](../proposals/PERFGATE-PROP-0008-evidence-intake-adoption-packs.md)
- [`PERFGATE-SPEC-0013-evidence-source-contract`](../specs/PERFGATE-SPEC-0013-evidence-source-contract.md)
- [`evidence-intake-adoption-packs.md`](../../plans/0.21.0/evidence-intake-adoption-packs.md)
- [`EVIDENCE_INTAKE.md`](../EVIDENCE_INTAKE.md)
- [`ADOPTION_PACKS.md`](../ADOPTION_PACKS.md)
- [`PRODUCT_CLAIMS.md`](../status/PRODUCT_CLAIMS.md)
- [`CANARY_MATRIX.md`](../status/CANARY_MATRIX.md)
- [`2026-05-20-evidence-intake-rust-canary-diffguard.md`](../audits/2026-05-20-evidence-intake-rust-canary-diffguard.md)
- [`2026-05-20-evidence-intake-non-rust-canary-droid-action.md`](../audits/2026-05-20-evidence-intake-non-rust-canary-droid-action.md)

Representative proof from the lane included targeted adapter fixture tests,
baseline doctor tests, signal doctor tests, policy tests, review packet tests,
`action-check`, schema compatibility, docs-source validation, product-claims
validation, adoption pack tests, and the two external canaries.

Closeout proof:

```bash
cargo +1.95.0 run -p xtask -- docs-check
cargo +1.95.0 run -p xtask -- doc-test
cargo +1.95.0 run -p xtask -- docs-source-check
cargo +1.95.0 run -p xtask -- product-claims-check
git diff --check
```

## What Not To Infer

- This lane did not add a dashboard.
- This lane did not add a benchmark scheduler.
- This lane did not replace Criterion, hyperfine, pytest-benchmark, k6, or
  project-specific scripts.
- This lane did not expand public crates.
- This lane did not change receipt schemas.
- This lane did not require server ledger mode.
- This lane did not auto-promote baselines.
- This lane did not auto-loosen thresholds.
- This lane did not make imported evidence blocking by default.
- Source-built canaries do not prove public release artifacts.
- Local canaries do not prove hosted Action intake workflows.
- The Rust canary did not prove Criterion or hyperfine adoption in an external
  Rust repo.
- The non-Rust canary did not prove HTTP/k6, pytest-benchmark, every shell, or
  every hosted runner.
- A successful import does not prove benchmark maturity, host compatibility, or
  baseline quality.

## Remaining Work

Good follow-up work should start from adoption pressure:

- run a hosted external Action intake canary when the 0.21 Action path needs
  fresh proof;
- rerun a public-release intake canary after the next public release;
- add external Criterion, hyperfine, pytest-benchmark, or k6 canaries only when
  a real repo has those tools already;
- deepen shell portability proof for non-Windows command wrappers;
- keep adapter support claims tied to fixture and canary freshness; and
- avoid adding a dashboard or scheduler until repeated team usage proves the
  need.

## Active Goal Handling

This closeout archives `.codex/goals/active.toml` as
`.codex/goals/archive/perfgate-evidence-intake-adoption-packs.toml` with status
`completed`.
