# First Useful Performance Review Closeout

Status: implemented
Owner: perfgate maintainers
Created: 2026-05-22
Milestone: 0.22.0
Linked proposal: [`PERFGATE-PROP-0002-first-useful-performance-review`](../../.rails/proposals/PERFGATE-PROP-0002-first-useful-performance-review.md)
Linked specs: [`PERFGATE-SPEC-0002-first-useful-performance-review`](../../.rails/specs/PERFGATE-SPEC-0002-first-useful-performance-review.md)
Linked plan: [`implementation-plan.md`](../../.rails/lanes/first-useful-performance-review/implementation-plan.md)
Support/status impact: [`PRODUCT_CLAIMS.md`](../status/PRODUCT_CLAIMS.md), [`CANARY_MATRIX.md`](../status/CANARY_MATRIX.md), and [`PROOF_FRESHNESS.md`](../status/PROOF_FRESHNESS.md)
Proof commands: docs-check, doc-test, docs-source-check, product-claims-check, git diff --check

## Summary

This lane moved perfgate from evidence intake and policy surfaces into one
coherent first-use review loop. A team can now start from an existing repo,
choose a reviewable setup, run or import evidence, inspect the posture, hand a
bounded packet to a reviewer or agent, and plan baseline or policy graduation
without mutating policy by default.

The implemented loop is:

```text
recommend adoption pack
-> emit dry-run setup artifacts
-> run native evidence or ingest existing benchmark output
-> explain review posture
-> surface benchmark passport
-> emit agent-safe repair context and guardrails
-> plan baseline and policy promotion without writing
```

The lane preserved the intended boundary: perfgate remains a performance
evidence and review control plane. It did not become a dashboard, scheduler,
benchmark engine, automatic baseline promoter, automatic threshold loosener,
mandatory server path, or release/publish surface.

## What Changed

Implemented user path:

- `perfgate adoption recommend` and `--json` report a reviewable pack
  recommendation, confidence, inspected inputs, non-inspected inputs, bad fits,
  and next command;
- `perfgate adoption apply --pack <pack> --ci github --dry-run` emits
  `target/perfgate-adoption/perfgate.toml.patch`,
  `target/perfgate-adoption/github-workflow.yml`,
  `target/perfgate-adoption/local-commands.md`, and
  `target/perfgate-adoption/non-inferences.md` without writing repo policy;
- `perfgate review explain --config perfgate.toml --bench <bench>` composes
  baseline health, signal maturity, policy posture, evidence source, artifacts,
  non-inferences, next commands, and agent guardrails;
- `perfgate review explain --json` emits the same posture for tools and
  agents;
- benchmark passports now surface source kind, source artifact, sample model,
  host context, baseline status, signal maturity, policy posture, proof
  freshness, known non-inferences, and next safe action;
- review packets and Action summaries surface the benchmark passport without
  changing configured exit-code behavior;
- repair context and review explain output preserve agent-safe allowed,
  review-required, and forbidden-by-default guidance;
- `perfgate baseline promote-plan` reports candidate source, host context,
  sample model, noise support, age, safety, and exact promote command only when
  reasonable; and
- `perfgate policy promote-plan --to gate_candidate|required_gate` reports
  missing evidence, risk explanation, review checklist, next commands, and a
  reviewable config fragment without writing policy.

Implemented docs:

- [`FIRST_USEFUL_PERFORMANCE_REVIEW.md`](../FIRST_USEFUL_PERFORMANCE_REVIEW.md)
  gives the paved first-use review path; and
- [`PERFORMANCE_REVIEW_FAILURE_GALLERY.md`](../PERFORMANCE_REVIEW_FAILURE_GALLERY.md)
  explains missing baseline, high noise, host mismatch, summary-only evidence,
  bad benchmark fit, stale baseline, regression, tradeoff candidate, setup
  timing, and local k6 non-inferences.

## Product Claims

Product claims now map the 0.22 surface conservatively:

- PG-CLAIM-0034 covers first-use setup recommendation and dry-run artifacts.
- PG-CLAIM-0035 covers review explain and benchmark passport output.
- PG-CLAIM-0036 covers agent-safe repair context and guardrails.
- PG-CLAIM-0037 covers non-mutating baseline and policy promotion plans.

These claims cite current source-built and fixture-backed proof. They do not
claim public release behavior for the next shipped version or hosted external
Action proof for the full first-useful-review loop.

## Canary State

Current proof for this lane is source-built and in-repo:

- adoption recommendation and dry-run fixtures;
- review explain fixtures;
- policy review packet and benchmark passport fixtures;
- repair context and agent guardrail fixtures;
- baseline promote-plan fixtures;
- policy promote-plan fixtures; and
- Action summary fixture checks for passport/posture behavior.

The canary matrix records two explicit gaps:

- hosted first-useful-review Action canary: `unproven`;
- public release first-useful-review canary: `unproven`.

Those gaps are intentional non-inferences. They prevent source-built proof from
being cited as hosted or public-artifact proof.

## Proof Records

Durable lane artifacts:

- [`PERFGATE-PROP-0002-first-useful-performance-review`](../../.rails/proposals/PERFGATE-PROP-0002-first-useful-performance-review.md)
- [`PERFGATE-SPEC-0002-first-useful-performance-review`](../../.rails/specs/PERFGATE-SPEC-0002-first-useful-performance-review.md)
- [`implementation-plan.md`](../../.rails/lanes/first-useful-performance-review/implementation-plan.md)
- [`FIRST_USEFUL_PERFORMANCE_REVIEW.md`](../FIRST_USEFUL_PERFORMANCE_REVIEW.md)
- [`PERFORMANCE_REVIEW_FAILURE_GALLERY.md`](../PERFORMANCE_REVIEW_FAILURE_GALLERY.md)
- [`PRODUCT_CLAIMS.md`](../status/PRODUCT_CLAIMS.md)
- [`CANARY_MATRIX.md`](../status/CANARY_MATRIX.md)

Representative behavior proof from the lane included:

```bash
cargo +1.95.0 test -p perfgate-cli --all-features adoption
cargo +1.95.0 test -p perfgate-cli --all-features review
cargo +1.95.0 test -p perfgate-cli --all-features repair
cargo +1.95.0 test -p perfgate-cli --all-features baseline
cargo +1.95.0 test -p perfgate-cli --all-features policy
cargo +1.95.0 run -p xtask -- action-check
```

Closeout proof:

```bash
cargo +1.95.0 run -p xtask -- rails check
cargo +1.95.0 run -p xtask -- docs-check
cargo +1.95.0 run -p xtask -- doc-test
cargo +1.95.0 run -p xtask -- docs-source-check
cargo +1.95.0 run -p xtask -- product-claims-check
git diff --check
```

## What Not To Infer

- This lane did not add a dashboard.
- This lane did not add a benchmark scheduler.
- This lane did not add a benchmark engine.
- This lane did not expand public crates.
- This lane did not change receipt schemas.
- This lane did not change release aliases.
- This lane did not move release, publish, signing, tags, package metadata, or
  release secrets into `perfgate-swarm`.
- This lane did not make server ledger mode required.
- This lane did not auto-promote baselines.
- This lane did not auto-loosen thresholds.
- This lane did not make mature evidence blocking by default.
- `gate_candidate` remains reviewable evidence, not blocking policy.
- `required_gate` remains a human policy decision.
- Source-built proof does not prove public release artifacts.
- In-repo fixtures do not prove every external repo or hosted runner.
- A successful recommendation does not prove the recommended workload is a
  good blocking gate.

## Remaining Work

Good follow-up work should start from real adoption pressure:

- run a hosted external first-useful-review Action canary;
- run a public-release first-useful-review canary after the next release;
- prove a mature external `gate_candidate` promotion without making it
  `required_gate`;
- add ecosystem canaries for Criterion, hyperfine, pytest-benchmark, or k6 only
  when a real repo already uses those tools;
- deepen shell portability proof for non-Rust command wrappers;
- keep product claims tied to proof freshness; and
- keep SRP/refactor PRs separate from product claims and canary claims.

## Active Goal Handling

This closeout archives `.codex/goals/active.toml` as
`.codex/goals/archive/perfgate-first-useful-performance-review.toml` with
status `completed`.
