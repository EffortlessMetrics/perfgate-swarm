# First useful performance review: implementation plan

## Objective

Turn the evidence-intake and maturity substrate into one coherent first-use review loop.

The lane should make this flow boring:

```text
existing repo
  -> recommend a reviewable adoption pack
  -> emit dry-run setup patches
  -> run or ingest evidence
  -> explain performance posture
  -> show a benchmark passport
  -> emit agent-safe repair context
  -> produce baseline and policy promotion plans
  -> prove the path with canaries before claims strengthen
```

## PR sequence

1. `plans(0.22): add first useful performance review plan`
   - Add this lane tracker and active execution pointer.
   - Promote PERFGATE-PROP-0002 and PERFGATE-SPEC-0002 to accepted.
   - Keep behavior and product claims unchanged.

2. `adoption: recommend packs from repository shape`
   - Add `perfgate adoption recommend` and `--json`.
   - Report recommended pack, confidence, why, inspected inputs, non-inspected inputs, bad fits, and next command.
   - Do not auto-select benchmarks or mutate config.

3. `adoption: emit dry-run setup patches`
   - Add `perfgate adoption apply --pack <pack> --ci github --dry-run`.
   - Emit reviewable config/workflow/local-command/non-inference artifacts.
   - Keep default behavior non-mutating; no baseline promotion, threshold loosening, or required-gate changes.

4. `test(adoption): add fixture-backed pack recommendation`
   - Add fixtures for Rust CLI, Rust workspace, Python service, Node tool/action, HTTP local smoke, and generic command.
   - Prove recommendation and dry-run output against stable fixtures.

5. `review: explain first-use performance posture`
   - Add `perfgate review explain --config perfgate.toml --bench <bench>` and `--json`.
   - Compose baseline doctor, signal doctor, policy doctor, evidence-source summary, next commands, and non-inferences.

6. `review: add benchmark passport to review packets`
   - Surface source kind, source artifact, sample model, host context, baseline status, signal maturity, policy posture, proof freshness, non-inferences, and next safe action.

7. `action: surface benchmark passport in summaries`
   - Include verdict, evidence source, baseline/signal/policy posture, non-inferences, local reproduction, and next safe command.
   - Preserve configured exit-code behavior.

8. `repair: emit agent-safe performance review context`
   - Extend repair context with classification, metric, movement, source kind, artifact paths, safe commands, forbidden changes, human-review requirements, and proof commands.

9. `review: emit copyable agent repair prompt`
   - Add optional agent prompt output from review explain.
   - Keep it advisory and bounded by repair-context guardrails.

10. `baseline: add non-mutating promote-plan`
    - Add `perfgate baseline promote-plan`.
    - Explain candidate source, host context, sample model, noise support, age, safety, and exact promote command only when reasonable.

11. `policy: add non-mutating promote-plan`
    - Add `perfgate policy promote-plan --to gate_candidate|required_gate`.
    - Emit missing evidence, risk explanation, review checklist, and reviewable patch.

12. `dogfood: record hosted Action intake canary`
    - Prove the first-use review path on an external hosted Action path.
    - Record what it proves and what it does not prove.

13. `docs: add first useful performance review guide`
    - Document the user path from recommendation to dry-run setup, evidence, review explain, passport, non-inferences, and promotion plans.

14. `docs: add performance review failure gallery`
    - Cover missing baseline, high noise, host mismatch, summary-only evidence, bad benchmark fit, stale baseline, regression, tradeoff candidate, setup timing, and local k6 non-inferences.

15. `docs(status): map first useful review claims to proof`
    - Add or update product claims only after behavior and proof exist.
    - Keep hosted/public canary status explicit.

16. `docs(handoff): close first useful performance review lane`
    - Record implemented commands, claims, canaries, advisory surfaces, non-goals, and remaining unproven surfaces.

## Proof strategy

Docs/source-of-truth PRs:

```bash
cargo +1.95.0 run -p xtask -- rails check
cargo +1.95.0 run -p xtask -- docs-source-check
cargo +1.95.0 run -p xtask -- docs-check
cargo +1.95.0 run -p xtask -- doc-test
cargo +1.95.0 run -p xtask -- product-claims-check
git diff --check
```

Behavior PRs add focused proof, usually one or more of:

```bash
cargo +1.95.0 fmt --all -- --check
cargo +1.95.0 clippy -p perfgate-cli --all-targets --all-features -- -D warnings
cargo +1.95.0 test -p perfgate-cli --all-features adoption
cargo +1.95.0 test -p perfgate-cli --all-features review
cargo +1.95.0 test -p perfgate-cli --all-features policy
cargo +1.95.0 run -p xtask -- action-check
cargo +1.95.0 run -p xtask -- schema-compat
git diff --check
```

Closeout requires:

- `cargo +1.95.0 run -p xtask -- rails check`
- `cargo +1.95.0 run -p xtask -- docs-check`
- `cargo +1.95.0 run -p xtask -- doc-test`
- `cargo +1.95.0 run -p xtask -- docs-source-check`
- `cargo +1.95.0 run -p xtask -- product-claims-check`
- targeted behavior proofs from the final product slices
- explicit canary freshness and non-inferences
