# PERFGATE-ADR-0002: Receipts-first performance decisions

Status: accepted
Date: 2026-05-13
Owner: perfgate maintainers
Linked proposal: docs/proposals/PERFGATE-PROP-0001-spec-driven-governance.md
Linked specs: docs/specs/PERFGATE-SPEC-0003-performance-decision-contract.md

## Decision

perfgate decisions are receipts-first. Local artifacts are the primary product
contract. The server is an optional team-scale ledger, not a prerequisite for
correctness.

`perfgate decision evaluate` must be able to produce a review-ready decision
from local compare, probe, scenario, and tradeoff receipts. `perfgate decision
bundle` must be able to package indexed local evidence for release, audit,
issue, or agent handoff without server access.

## Context

Performance review is useful only when reviewers can inspect the evidence that
led to the judgment. Benchmark logs and CI output are not a stable product
contract by themselves; they are execution traces. perfgate's stable contract is
the versioned receipt and the rendered decision derived from those receipts.

The decision workflow now covers:

- command measurement and comparison receipts;
- probe evidence and probe comparisons;
- scenario weighting;
- tradeoff policy;
- decision markdown;
- decision artifact indexes;
- portable decision bundles; and
- optional server-side decision history and debt summaries.

Those features should share one evidence model. The server should persist and
query decisions; it should not define a separate correctness path.

## Consequences

- `decision evaluate` remains local-first.
- `decision bundle` remains portable and based on `decision.index.json`.
- Server upload consumes receipt evidence rather than inventing a different
  decision model.
- Agents should reason from receipts, indexes, bundles, and decision markdown,
  not raw benchmark logs.
- GitHub Action output should surface local reproduction commands and artifact
  paths for decision-enabled gates.
- Schema compatibility remains part of decision release proof.
- Server ledger availability must not decide whether local evidence is valid.

## Alternatives considered

### Server-first decision authority

Rejected. It would make correctness depend on service availability and would
weaken local CI, release, and audit workflows.

### Markdown-only decisions

Rejected. Markdown is the review surface, but indexed machine-readable receipts
are needed for bundles, audits, agents, and schema compatibility.

### Benchmark-log-driven review

Rejected. Logs are useful diagnostics, but they are not stable enough to be the
review contract. Receipts provide versioned structure and deterministic links
between evidence and decision output.

### Separate server decision model

Rejected. The server ledger should store, query, export, prune, and summarize
decisions derived from the same receipt model used locally.

## Follow-up specs / plans

- `docs/specs/PERFGATE-SPEC-0003-performance-decision-contract.md`
- `plans/0.18.0/performance-decision-contract.md`
- `docs/status/PRODUCT_CLAIMS.md`
- `docs/PERFORMANCE_DECISIONS.md`
- `docs/RELEASE_READINESS.md`

Follow-on implementation should strengthen the receipt contract without making
the server mandatory.
