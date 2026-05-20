# Evidence Intake

perfgate can sit above existing benchmark tools. The benchmark tool still
measures; perfgate imports the result into reviewable receipts, maturity, policy,
and Action surfaces.

The foundational intake format is reviewable generic command JSON. It is
intended for teams that already run a script and can emit a small JSON artifact.

## Generic Command JSON

Create a source artifact that names the benchmark, command, metrics, units, and
directions explicitly:

```json
{
  "source_kind": "generic_command_json",
  "benchmark": {
    "name": "parser-smoke",
    "command": ["node", "bench.js"],
    "work_units": 5000
  },
  "host": {
    "os": "linux",
    "arch": "x86_64"
  },
  "metrics": {
    "wall_ms": {
      "unit": "ms",
      "direction": "lower_is_better",
      "samples": [120.0, 118.0, 123.0]
    },
    "throughput_per_s": {
      "unit": "ops/s",
      "direction": "higher_is_better",
      "summary": {
        "median": 41000.0,
        "min": 39000.0,
        "max": 42500.0,
        "mean": 40800.0,
        "stddev": 1300.0
      }
    }
  }
}
```

Import it into a normal run receipt:

```bash
perfgate ingest --format generic-command-json --input artifacts/parser-source.json --out artifacts/perfgate/run.json
```

Then use existing perfgate surfaces:

```bash
perfgate baseline doctor --config perfgate.toml
perfgate doctor signal --config perfgate.toml
perfgate policy doctor --config perfgate.toml --bench parser-smoke
perfgate policy review-packet --config perfgate.toml --bench parser-smoke --out artifacts/perfgate/review-packet.md
```

## Contract

Generic command JSON must include a `wall_ms` metric so perfgate can emit the
existing `perfgate.run.v1` receipt without schema churn. The metric must include
either raw `samples` or a `summary`.

Each metric must include:

```text
unit
direction
samples or summary
```

perfgate accepts known perfgate metrics such as `wall_ms`, `max_rss_kb`,
`page_faults`, and `throughput_per_s`. Ambiguous units or directions fail
closed. Throughput is higher-is-better; wall time, memory, and fault/count
metrics are lower-is-better.

If host context is missing, the imported receipt uses `unknown` host fields and
the CLI reminds reviewers not to infer host compatibility.

## Non-inferences

Imported evidence remains advisory until maturity and policy surfaces support a
stronger posture.

Do not infer:

- a successful import means the benchmark is mature;
- the first imported result should become a baseline;
- missing host context is host-compatible;
- summary-only evidence has the same noise support as raw samples;
- imported evidence should block CI by default; or
- server ledger mode is required for correctness.

Use `perfgate policy emit-patch` only after reviewing maturity and promotion
guidance.

## hyperfine JSON

hyperfine remains the measurement tool. perfgate imports its JSON as command
timing evidence:

```bash
hyperfine --warmup 3 --runs 10 --export-json artifacts/hyperfine.json "cargo run -q -- --help"
perfgate ingest --format hyperfine --input artifacts/hyperfine.json --name cli-help --out artifacts/perfgate/run.json
```

Then run the same maturity and policy surfaces:

```bash
perfgate baseline doctor --config perfgate.toml
perfgate doctor signal --config perfgate.toml
perfgate policy doctor --config perfgate.toml --bench cli-help
perfgate policy review-packet --config perfgate.toml --bench cli-help --out artifacts/perfgate/review-packet.md
```

Mapping:

```text
hyperfine source kind  -> hyperfine_json
times[]                -> raw wall_ms samples
mean/median/stddev/min/max -> wall_ms summary
exit_codes[]           -> sample exit_code values
user + system          -> cpu_ms summary when present
command                -> bench.command as a single shell command string
host                   -> unknown
```

hyperfine timings are lower-is-better command timings. They can be useful as
smoke, advisory, or gate-candidate evidence, but compile-heavy or setup-heavy
commands should stay advisory until baseline and signal maturity prove they are
stable enough to promote.

Do not infer:

- hyperfine command timing excludes shell, setup, cache, or compile overhead;
- missing host context proves host compatibility;
- user and system time remain separate after import;
- the first imported result should become a baseline; or
- successful import means the benchmark should block CI.

## Criterion

Criterion remains the Rust measurement tool. perfgate imports clear wall-time
evidence from Criterion-compatible outputs and keeps the result advisory until
normal maturity and policy surfaces support promotion.

Preferred input is cargo-criterion's JSON message stream:

```bash
cargo criterion --message-format=json > artifacts/criterion.jsonl
perfgate ingest --format criterion --input artifacts/criterion.jsonl --name parser-large --out artifacts/perfgate/run.json
```

perfgate also accepts Criterion `raw.csv` files for a single benchmark:

```bash
perfgate ingest --format criterion --input target/criterion/parser/new/raw.csv --name parser-large --out artifacts/perfgate/run.json
```

Then run the same maturity and policy surfaces:

```bash
perfgate baseline doctor --config perfgate.toml
perfgate doctor signal --config perfgate.toml
perfgate policy doctor --config perfgate.toml --bench parser-large
perfgate policy review-packet --config perfgate.toml --bench parser-large --out artifacts/perfgate/review-packet.md
```

Mapping:

```text
Criterion source kind        -> criterion
cargo-criterion measured_values / iteration_count -> raw wall_ms samples
Criterion raw.csv sample_measured_value / iteration_count -> raw wall_ms samples
mean/median estimates       -> wall_ms summary fields where present
throughput per_iteration    -> bench.work_units where present
host                         -> unknown
```

Criterion `new/estimates.json` can be imported as a summary-only fallback, but
Criterion documents those JSON files as private implementation details in its
[CSV output guide](https://bheisler.github.io/criterion.rs/book/user_guide/csv_output.html).
Prefer cargo-criterion JSON messages or `raw.csv` when reviewable sample
evidence is needed.

Do not infer:

- Criterion confidence intervals are the same as perfgate maturity policy;
- summary-only `estimates.json` has raw sample/noise support;
- missing host context proves host compatibility;
- Criterion throughput units are fully preserved in `perfgate.run.v1`;
- the first imported result should become a baseline; or
- successful import means the benchmark should block CI.

## pytest-benchmark JSON

pytest-benchmark remains the Python measurement tool. perfgate imports its JSON
as Python benchmark evidence while keeping correctness test success separate
from performance maturity.

Generate JSON from an existing pytest benchmark run:

```bash
pytest --benchmark-json=artifacts/pytest-benchmark.json
perfgate ingest --format pytest-benchmark --input artifacts/pytest-benchmark.json --name parser-py --out artifacts/perfgate/run.json
```

Then run the same maturity and policy surfaces:

```bash
perfgate baseline doctor --config perfgate.toml
perfgate doctor signal --config perfgate.toml
perfgate policy doctor --config perfgate.toml --bench parser-py
perfgate policy review-packet --config perfgate.toml --bench parser-py --out artifacts/perfgate/review-packet.md
```

Mapping:

```text
pytest-benchmark source kind -> pytest_benchmark_json
stats.data[]                 -> raw wall_ms samples when present
stats min/median/mean/max/stddev -> wall_ms summary fields
stats.ops                    -> throughput_per_s point summary when present
machine_info.system          -> run.host.os when present
machine_info.machine/processor -> run.host.arch when present
machine_info.cpu.count       -> run.host.cpu_count when present
machine_info python fields   -> bench.command metadata when present
host                         -> unknown for missing fields
```

If `stats.data` is missing, perfgate imports the summary fields without creating
synthetic samples. The receipt remains useful for baseline comparison, but noise
and maturity guidance is weaker until raw samples are available.

Use pytest-benchmark's documented JSON output flag
[`--benchmark-json`](https://pytest-benchmark.readthedocs.io/en/latest/usage.html)
to create the source artifact.

Do not infer:

- passing pytest tests are performance maturity evidence;
- pytest-benchmark `ops` is a full sample distribution after import;
- summary-only JSON has raw sample/noise support;
- Python interpreter/runtime details have dedicated structured policy semantics
  in `perfgate.run.v1`;
- missing host fields prove host compatibility;
- the first imported result should become a baseline; or
- successful import means the benchmark should block CI.

## k6 Summary JSON

k6 remains the HTTP/load-test runner. perfgate imports k6 end-of-test summary
JSON as summary-only evidence so the same maturity and policy surfaces can
review it:

```bash
k6 run --summary-export artifacts/k6-summary.json script.js
perfgate ingest --format k6 --input artifacts/k6-summary.json --name http-smoke --out artifacts/perfgate/run.json
```

Then run the same maturity and policy surfaces:

```bash
perfgate baseline doctor --config perfgate.toml
perfgate doctor signal --config perfgate.toml
perfgate policy doctor --config perfgate.toml --bench http-smoke
perfgate policy review-packet --config perfgate.toml --bench http-smoke --out artifacts/perfgate/review-packet.md
```

Mapping:

```text
k6 source kind             -> k6_summary_json
http_req_duration values   -> wall_ms summary fields
http_reqs.rate             -> throughput_per_s point summary when present
http_reqs.count            -> bench.repeat and bench.work_units when present
http_req_failed rate/passes/fails -> bench.command metadata
scenario tags              -> bench.command metadata where available
host                       -> unknown
samples                    -> none; summary-only source
```

If `http_req_duration` is missing, perfgate falls back to
`iteration_duration`. k6 trend values are treated as milliseconds by default;
when the summary declares `summaryTimeUnit` as `s` or `us`, perfgate converts
those timing values to `wall_ms`.

k6 is useful for HTTP smoke, advisory load evidence, and candidate policy review
when the environment is controlled enough to trust. A local or shared-runner k6
summary is not production capacity proof.

Do not infer:

- k6 summary JSON contains raw per-request samples;
- summary-only load evidence has full noise/maturity support;
- a local or shared-runner run proves production capacity;
- missing host context proves host compatibility;
- error-rate metadata has a dedicated `perfgate.run.v1` metric field;
- the first imported result should become a baseline; or
- successful import means the benchmark should block CI.
