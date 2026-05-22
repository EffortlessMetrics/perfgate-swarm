# First Useful Performance Review

This guide shows the shortest path from an existing repository to a reviewable
performance answer. The goal is not to make the first benchmark blocking. The
goal is to make the first evidence understandable, reproducible, and safe to
improve.

The first useful loop is:

```text
recommend setup
-> emit dry-run files
-> run or ingest evidence
-> explain posture
-> inspect the benchmark passport
-> decide what not to infer
-> plan baseline or policy promotion only after proof
```

## 1. Recommend a Pack

Start with a reviewable recommendation:

```bash
perfgate adoption recommend
```

Use JSON when another tool or agent needs to inspect the result:

```bash
perfgate adoption recommend --json
```

The recommendation names the pack, confidence, inspected inputs, non-inspected
inputs, bad fits, and next command. It is not automatic benchmark selection.

## 2. Emit Dry-Run Setup

Generate setup files without writing into the repository:

```bash
perfgate adoption apply --pack rust-cli --ci github --dry-run
```

The dry run writes reviewable artifacts under `target/perfgate-adoption/`:

- `perfgate.toml.patch`
- `github-workflow.yml`
- `local-commands.md`
- `non-inferences.md`

Review those files before copying anything into the repo. Dry-run setup does
not promote a baseline, loosen a threshold, or make a benchmark blocking.

## 3. Run Native Evidence

After applying a reviewed config, run the benchmark:

```bash
perfgate check --config perfgate.toml --bench cli-help
```

For first evidence, missing baseline is setup state. It is not a regression.

## 4. Ingest Existing Evidence

If the repo already uses a measurement tool, keep it. Import its output:

```bash
hyperfine --warmup 3 --runs 10 --export-json artifacts/hyperfine.json "cargo run -q -- --help"

perfgate ingest \
  --format hyperfine \
  --input artifacts/hyperfine.json \
  --name cli-help \
  --out artifacts/perfgate/cli-help/run.json
```

Imported evidence keeps source metadata, sample model, host context, and known
limits visible in perfgate receipts and review surfaces. Summary-only evidence
stays weaker than raw samples or paired runs.

## 5. Explain the Review

Use one command to compose baseline health, signal maturity, policy posture,
evidence source, non-inferences, artifacts, next commands, and agent guardrails:

```bash
perfgate review explain --config perfgate.toml --bench cli-help
```

Use JSON for automation:

```bash
perfgate review explain --config perfgate.toml --bench cli-help --json
```

The output is advisory. It does not change config, baselines, thresholds,
policy, or server settings.

## 6. Inspect the Benchmark Passport

The benchmark passport summarizes the review identity:

```text
source kind
source artifact
sample model
host context
baseline status
signal maturity
policy posture
proof freshness
known non-inferences
next safe action
```

Use the passport to decide whether the evidence is smoke, advisory, a
gate-candidate, or still blocked by missing maturity.

## 7. Read the Review Packet

For a compact Markdown packet:

```bash
perfgate policy review-packet --config perfgate.toml --bench cli-help
```

Review packets are for humans and agents. Receipts remain the source of truth.

## 8. Promote a Baseline Only After Review

Plan baseline promotion before running a mutating command:

```bash
perfgate baseline promote-plan --config perfgate.toml --bench cli-help
```

The plan reports candidate source, host context, sample model, noise support,
age, safety, and the exact promote command only when reasonable. It does not
write a baseline.

## 9. Promote Policy Only After Proof

Plan policy promotion separately:

```bash
perfgate policy promote-plan --config perfgate.toml --bench cli-help --to gate_candidate
perfgate policy promote-plan --config perfgate.toml --bench cli-help --to required_gate
```

`gate_candidate` means evidence is reviewable. It is not blocking policy.
`required_gate` is a human policy decision and needs explicit approval.

Use a copy-ready patch only after reviewing the plan:

```bash
perfgate policy emit-patch --config perfgate.toml --bench cli-help --to gate_candidate
```

## Agent Guardrails

Agents may:

- inspect receipts and review packets;
- rerun local reproduction commands;
- summarize posture, maturity, signal, and proof freshness;
- recommend paired mode or more samples; and
- propose non-mutating patches.

Agents must not, without human review:

- promote baselines;
- loosen thresholds;
- make a benchmark `required_gate`;
- accept tradeoffs; or
- require server ledger mode for local correctness.

## What Not To Infer

- First-run evidence is not a mature baseline.
- Summary-only imported evidence is not raw-sample proof.
- Missing host context cannot prove host compatibility.
- A benchmark passport does not replace receipts.
- Review packets do not change exit-code behavior.
- Server ledger history is optional team history, not local correctness.
