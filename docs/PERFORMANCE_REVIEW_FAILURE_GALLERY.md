# Performance Review Failure Gallery

This gallery describes common first-use performance review outcomes. Use it to
decide whether evidence should stay setup-only, advisory, gate-candidate, or
human-reviewed.

## Missing Baseline

Meaning:

```text
baseline status: missing
policy posture: smoke or advisory
```

Treat this as setup state, not a regression.

Next safe commands:

```bash
perfgate baseline doctor --config perfgate.toml --bench <bench>
perfgate baseline promote-plan --config perfgate.toml --bench <bench>
```

Do not promote blindly. Review workload intent, host context, sample model, and
noise support first.

## High Noise

Meaning:

```text
signal maturity: high_noise
recommendation: advisory or paired mode
```

The benchmark may be real, but the signal is not stable enough to enforce.

Next safe commands:

```bash
perfgate doctor signal --config perfgate.toml --bench <bench>
perfgate calibrate --config perfgate.toml --bench <bench> --emit-patch
```

Do not loosen thresholds to make the run pass. Use paired mode, more samples,
or a better workload.

## Host Mismatch

Meaning:

```text
host context: mismatch
proof freshness: not runner-compatible
```

The comparison may be measuring host drift instead of code movement.

Next safe command:

```bash
perfgate check --config perfgate.toml --bench <bench> --require-baseline
```

Run on the intended runner class before promoting baseline or policy.

## Summary-Only Evidence

Meaning:

```text
sample model: summary_only
noise support: limited_summary_only
```

Summary-only imports can be useful review context, but they have weaker noise
support than raw samples or paired evidence.

Next safe command:

```bash
perfgate review explain --config perfgate.toml --bench <bench>
```

Do not treat summary-only evidence as mature blocking proof.

## Bad Benchmark Fit

Signs:

- compile-heavy command used as a first-hour gate;
- network-heavy command without isolation;
- correctness tests and performance timing mixed together;
- tiny runtime without warmup; or
- workload does not answer a review question.

Next safe command:

```bash
perfgate adoption recommend
```

Keep poor-fit workloads advisory or retire them. Add a better benchmark before
making policy stricter.

## Stale Baseline

Meaning:

```text
baseline status: stale
proof freshness: stale
```

Stale proof can explain history, but it should not support new blocking policy.

Next safe commands:

```bash
perfgate baseline doctor --config perfgate.toml --bench <bench>
perfgate baseline promote-plan --config perfgate.toml --bench <bench>
```

Refresh evidence on the intended runner class before citing it.

## Regression

Meaning:

```text
gate verdict: warn or fail
meaningful movement exceeds policy
```

A regression needs reproduction and artifact inspection.

Next safe command:

```bash
perfgate review explain --config perfgate.toml --bench <bench>
```

Agents may inspect and optimize the changed code path. They must not update the
baseline or loosen thresholds to make CI green.

## Tradeoff Candidate

Meaning:

```text
one metric regressed
another meaningful metric improved
```

This is not a simple pass/fail story. It needs a structured decision when the
tradeoff matters.

Next safe command:

```bash
perfgate decision suggest --config perfgate.toml
```

Do not accept a bounded regression without decision evidence and reviewer
approval.

## Setup Timing Mistaken For Runtime Timing

Signs:

- compile/install/setup dominates the measured command;
- cache state changes the result more than code changes; or
- the command includes dependency download or environment setup.

Keep this advisory. Separate setup from runtime before creating a gate.

## Local k6 Mistaken For Production Capacity

Local HTTP smoke evidence can catch obvious regressions, but it is not
production capacity proof.

Use local k6 or HTTP smoke as advisory first-use evidence unless the repo has a
separate, reviewed load-test environment and policy.
