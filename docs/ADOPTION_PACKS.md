# Adoption Packs

Adoption packs help teams keep their existing benchmark tools while adding
perfgate receipts, maturity guidance, policy posture, review packets, and CI
summaries around the evidence.

They are reviewable starting points. They do not detect benchmarks magically,
promote baselines, make checks blocking, loosen thresholds, or require server
ledger mode.

List the catalog:

```bash
perfgate adoption packs
```

Recommend a pack from the current repository shape:

```bash
perfgate adoption recommend
perfgate adoption recommend --json
```

The recommendation is review guidance. It reports the chosen pack, confidence,
why it matched, which markers were inspected, what was not inspected, known bad
fits, and the next command. It does not inspect runtime behavior, baseline
maturity, signal noise, host compatibility, or write any setup files.

Show one pack:

```bash
perfgate adoption packs --pack rust-cli
perfgate adoption packs --pack rust-workspace
perfgate adoption packs --pack python-service
perfgate adoption packs --pack node-tool-action
perfgate adoption packs --pack http-local-smoke
perfgate adoption packs --pack generic-command
```

Use adoption packs after choosing the measurement source. Use
[`BENCHMARK_RECIPES.md`](BENCHMARK_RECIPES.md) when the team still needs to
choose a workload, and use [`EVIDENCE_INTAKE.md`](EVIDENCE_INTAKE.md) when the
team already has output from hyperfine, Criterion, pytest-benchmark, k6, custom
JSON, or custom CSV.

## Common Flow

Use the same review path for every pack:

```bash
perfgate adoption packs --pack <pack>
perfgate init --ci github --profile standard --suggest-benches
perfgate check --config perfgate.toml --bench <bench>
perfgate baseline doctor --config perfgate.toml --bench <bench>
perfgate doctor signal --config perfgate.toml --bench <bench>
perfgate policy doctor --config perfgate.toml --bench <bench>
perfgate policy review-packet --config perfgate.toml --bench <bench> --out artifacts/perfgate/review-packet.md
```

For imported evidence, run the source benchmark and import the artifact before
using maturity and policy surfaces:

```bash
hyperfine --warmup 3 --runs 10 --export-json artifacts/hyperfine.json "cargo run -q -- --help"
perfgate ingest --format hyperfine --input artifacts/hyperfine.json --name cli-help --out artifacts/perfgate/cli-help/run.json
perfgate baseline doctor --config perfgate.toml --bench cli-help
perfgate doctor signal --config perfgate.toml --bench cli-help
perfgate policy review-packet --config perfgate.toml --bench cli-help --out artifacts/perfgate/cli-help/review-packet.md
```

The first result is evidence to review, not a baseline to trust by default.
Promote only after the workload, host, samples, and signal look representative:

```bash
perfgate baseline promote --config perfgate.toml --bench <bench>
perfgate check --config perfgate.toml --bench <bench> --require-baseline
```

## GitHub Actions Shape

The Action should preserve the local reproduction path. Direct perfgate command
checks can use the generated workflow shape:

```yaml
- uses: EffortlessMetrics/perfgate@v0.18
  with:
    config: perfgate.toml
    all: "true"
    require_baseline: "false"
    upload_artifact: "true"
```

Use `require_baseline: "true"` only after reviewed baselines exist and the
policy is intentionally blocking.

Imported-evidence workflows need an explicit install plus source/import step
before the Action summary can review the imported artifact. Pin the install and
Action ref to a release or commit that contains the adapter being used; do not
cite this doc as public install proof for unreleased adapters.

```yaml
- name: Install perfgate
  run: cargo install perfgate-cli --locked --version <released-version-with-adapter>

- name: Produce benchmark evidence
  run: hyperfine --warmup 3 --runs 10 --export-json artifacts/hyperfine.json "cargo run -q -- --help"

- name: Import benchmark evidence
  run: perfgate ingest --format hyperfine --input artifacts/hyperfine.json --name cli-help --out artifacts/perfgate/cli-help/run.json

- uses: EffortlessMetrics/perfgate@<released-ref-with-adapter>
  with:
    config: perfgate.toml
    all: "true"
    require_baseline: "false"
    upload_artifact: "true"
```

For canaries against current source, replace the release placeholders with a
specific tested commit. Do not make server ledger upload part of local
correctness.

## Pack Selection

Choose by repository shape and by the benchmark output the team already owns.
If the team still needs to choose a workload, start with
[`BENCHMARK_RECIPES.md`](BENCHMARK_RECIPES.md). If the workload already emits
Criterion, hyperfine, pytest-benchmark, k6, custom JSON, or custom CSV output,
use [`EVIDENCE_INTAKE.md`](EVIDENCE_INTAKE.md) for adapter mechanics after
selecting a pack.

| Pack | Start with | Good fit | Bad fit |
|------|------------|----------|---------|
| `rust-cli` | native command check, Criterion, or hyperfine | fast CLI startup or one scoped command workload | compile-heavy first required gates or treating `--help` as throughput proof |
| `rust-workspace` | advisory workspace command plus smaller package benches | broad health signal and one scoped gate candidate | `cargo test --workspace` as an uncalibrated blocking performance gate |
| `python-service` | pytest-benchmark JSON or a stable Python bench script | deterministic benchmark functions with raw samples | dependency installation, network setup, or correctness tests as performance proof |
| `node-tool-action` | dedicated Node script, hyperfine, or custom JSON/CSV | fixed local input and separated build/install steps | package download, network calls, or JIT-sensitive evidence with no repeats |
| `http-local-smoke` | local endpoint smoke or k6 summary JSON | isolated local service smoke and advisory load evidence | shared staging services or local k6 output described as production capacity |
| `generic-command` | generic command JSON or explicit custom mapping | language-neutral artifacts with units, directions, and host context | missing units, missing metric direction, or mutable external data |

Choose the smallest pack that matches the repo. A larger pack is not stronger
evidence; it usually just carries more assumptions to review.

## Local Reproduction By Pack

### rust-cli

```bash
perfgate init --ci github --profile standard --suggest-benches rust-cli-smoke
perfgate check --config perfgate.toml --bench <bench>
perfgate policy review-packet --config perfgate.toml --bench <bench>
```

Use Criterion or hyperfine import only when the repo already owns that benchmark
source:

```bash
cargo criterion --message-format=json > artifacts/criterion.jsonl
perfgate ingest --format criterion --input artifacts/criterion.jsonl --name <bench> --out artifacts/perfgate/<bench>/run.json
```

### rust-workspace

```bash
perfgate init --ci github --profile standard --suggest-benches rust-workspace-advisory
perfgate check --config perfgate.toml --bench <package-bench>
perfgate doctor signal --config perfgate.toml --bench <package-bench>
perfgate policy doctor --config perfgate.toml --bench <package-bench>
```

Keep broad compile or workspace timing advisory until setup noise is separated
from workload movement.

### python-service

```bash
pytest --benchmark-json=artifacts/pytest-benchmark.json
perfgate ingest --format pytest-benchmark --input artifacts/pytest-benchmark.json --name <bench> --out artifacts/perfgate/<bench>/run.json
perfgate policy review-packet --config perfgate.toml --bench <bench>
```

Prefer raw benchmark samples. Summary-only pytest evidence is useful for review
but has weaker noise support.

### node-tool-action

```bash
node scripts/bench.js
perfgate ingest --format custom-json --input artifacts/node-bench.json --metric wall_ms=duration_ms,unit=ms,direction=lower_is_better --out artifacts/perfgate/<bench>/run.json
perfgate doctor signal --config perfgate.toml --bench <bench>
```

Keep install/build time outside the measured path unless that setup time is the
intentional workload.

### http-local-smoke

```bash
k6 run --summary-export artifacts/k6-summary.json scripts/http-smoke.js
perfgate ingest --format k6 --input artifacts/k6-summary.json --name <bench> --out artifacts/perfgate/<bench>/run.json
perfgate policy review-packet --config perfgate.toml --bench <bench>
```

Use this for local smoke and advisory load evidence. Do not call it production
capacity proof.

### generic-command

```bash
./scripts/bench.sh > artifacts/source-evidence.json
perfgate ingest --format generic-command-json --input artifacts/source-evidence.json --out artifacts/perfgate/<bench>/run.json
perfgate baseline doctor --config perfgate.toml --bench <bench>
```

If the artifact is custom JSON or CSV, map fields explicitly:

```bash
perfgate ingest --format custom-csv --input artifacts/bench.csv --name <bench> --metric wall_ms=duration_ms,unit=ms,direction=lower_is_better --out artifacts/perfgate/<bench>/run.json
```

## Promotion Path

Promotion is deliberate:

```text
smoke or advisory evidence
-> baseline and signal maturity reviewed
-> policy doctor says promotion may be reasonable
-> non-mutating policy patch is reviewed
-> only then make the gate blocking
```

Use:

```bash
perfgate baseline doctor --config perfgate.toml --bench <bench>
perfgate doctor signal --config perfgate.toml --bench <bench>
perfgate calibrate --config perfgate.toml --bench <bench> --emit-patch
perfgate policy doctor --config perfgate.toml --bench <bench>
perfgate policy emit-patch --config perfgate.toml --bench <bench> --to gate_candidate
```

Do not promote baselines or loosen thresholds to fix setup failures, missing
baselines, noisy signal, or ambiguous imported metrics.

## Non-Inferences

Adoption packs do not prove:

- the selected benchmark is mature;
- the first run should become a baseline;
- imported summary-only evidence has raw-sample noise support;
- unknown host context is compatible;
- local HTTP or k6 output is production capacity proof;
- a broad workspace command is an isolated runtime benchmark;
- policy should become blocking by default;
- server ledger history is required for correctness; or
- unreleased adapter examples are public release proof.

The review contract remains local receipts, artifacts, reproduction commands,
maturity output, and explicit policy review.
