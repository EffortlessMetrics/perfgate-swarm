# PERFGATE-PROP-0001: Spec-driven governance

Status: proposed
Owner: perfgate maintainers
Created: 2026-05-13
Target milestone: 0.18.0
Linked specs: PERFGATE-SPEC-0001-source-of-truth-stack, PERFGATE-SPEC-0002-package-surface-boundary, PERFGATE-SPEC-0003-performance-decision-contract, PERFGATE-SPEC-0004-user-devex-paved-road, PERFGATE-SPEC-0005-release-proof-contract, PERFGATE-SPEC-0006-policy-ledger-contracts
Linked ADRs: PERFGATE-ADR-0001-public-crates-are-contracts, PERFGATE-ADR-0002-receipts-first-performance-decisions, PERFGATE-ADR-0003-local-receipts-first-server-ledger-optional
Linked plan: plans/0.18.0/implementation-plan.md
Support/status impact: docs/status/SUPPORT_TIERS.md and docs/status/PRODUCT_CLAIMS.md
Policy impact: policy ledgers remain source of truth; specs link to ledgers instead of copying entries

## Problem

perfgate now has enough product, release, and governance surface that intent can
drift away from implementation unless the repo has a durable source-of-truth
model.

The 0.17.0 governance work made this visible. PR
[#349](https://github.com/EffortlessMetrics/perfgate/pull/349) landed targeted
Rust 1.95 API cleanup, PR
[#350](https://github.com/EffortlessMetrics/perfgate/pull/350) prepared the
0.17.0 release state, and PR
[#351](https://github.com/EffortlessMetrics/perfgate/pull/351) validated
publish readiness while preserving the explicit boundary that crates, tags, and
GitHub release assets were outside that PR.

Those changes left perfgate with real governed surfaces:

- performance decision receipts
- probe, scenario, and tradeoff policy
- optional server ledger history and debt reporting
- Rust 1.95 MSRV governance
- Clippy, no-panic, and file-policy ledgers
- five-crate public surface enforcement
- release-order publish proof
- routed CI evidence lanes

Without a linked stack, future contributors and agents have to reconstruct the
architecture from README prose, release notes, policy files, old handoffs, and
chat history. That makes scope harder to review and makes silent drift more
likely.

## Users and surfaces

- CLI users need product claims that match tested behavior.
- GitHub Action users need clear local reproduction commands and artifact
  expectations.
- Server users need to know which decision-ledger behavior is optional team
  infrastructure versus local correctness.
- Maintainers need public surface, policy, release, and CI claims to point at
  durable proof.
- Codex needs a narrow active-goal manifest that identifies scope, allowed
  files, forbidden files, linked specs, and proof commands.
- Release operators need release-readiness claims to link to proof records
  rather than duplicate release matrices across docs.

## Success criteria

- The repo defines artifact ownership for proposals, specs, ADRs, plans, goal
  TOML, policy ledgers, status docs, and handoffs.
- Every active product or governance lane links to a proposal or an existing
  accepted proposal.
- Every behavior contract has a spec with proof commands and explicit
  non-goals.
- Durable architecture decisions have ADRs instead of being buried in plans or
  release notes.
- PR-sized work lives in plans and does not redefine behavior already owned by
  specs.
- `.codex/goals/active.toml` identifies the current campaign, current work
  item, linked proposal/spec/plan, allowed files, forbidden files, proof
  commands, and completion criteria.
- Policy TOMLs remain the reviewed source of truth for concrete exceptions and
  governed surfaces.
- README and product claims link to status proof maps instead of free-floating
  prose.
- Release claims link to release-readiness and audit proof records instead of
  restating every gate.
- A future maintainer or agent can answer from repo artifacts alone: why the
  lane exists, what behavior must be true, what architecture decision
  constrains it, what policy ledger owns exceptions, what product claim is
  affected, what PR comes next, and what proof shows the work is real.

## Proposed shape

The governance stack separates artifact responsibilities:

| Artifact | Owns |
|----------|------|
| Proposal | Why a lane exists, who benefits, alternatives, and success criteria |
| Spec | What behavior or proof contract must be true |
| ADR | Durable architecture decisions |
| Plan | PR-sized sequencing, file scope, proof commands, rollback, and blockers |
| Goal TOML | Current Codex execution state |
| Policy ledger | Machine-readable exceptions, gates, and governed surfaces |
| Status docs | Product claim support tiers and proof map |
| Handoff | Closeout notes, remaining work, and next-operator context |

This proposal starts the 0.18.0 spec-governance lane. The lane should land in
small PRs: scaffold first, one semantic proposal/spec/ADR/status artifact per
PR, then narrow checkers once the vocabulary exists.

## Alternatives considered

### Keep adding prose to existing topic docs

Rejected. Topic docs such as `docs/RELEASE_READINESS.md`,
`docs/PERFORMANCE_DECISIONS.md`, and `docs/CRATE_SEAMS.md` are useful, but
they should not also own lane motivation, behavior contracts, active agent
state, and PR sequencing.

### Put every detail in specs

Rejected. Specs should define behavior and proof contracts. They should link to
policy ledgers, release proof, and status maps instead of copying their rows.

### Start with strict enforcement

Rejected. Enforcing a graph before the repo has the taxonomy and initial
artifacts would make the checker encode guesses. The first checker should only
verify headers, IDs, links, known status values, plan links, and TOML parsing.

### Store agent state under `.perfgate/`

Rejected. `.perfgate/` is already a product/user-facing artifact namespace from
`perfgate init`. Agent execution state belongs under `.codex/goals/`.

### Treat unpublished production crates as a durable category

Rejected. perfgate's governance direction is that public crates are contracts,
folders and modules are architecture boundaries, and every workspace package
must be classified through the public-surface policy.

## Specs to create or update

- `PERFGATE-SPEC-0001-source-of-truth-stack`
- `PERFGATE-SPEC-0002-package-surface-boundary`
- `PERFGATE-SPEC-0003-performance-decision-contract`
- `PERFGATE-SPEC-0004-user-devex-paved-road`
- `PERFGATE-SPEC-0005-release-proof-contract`
- `PERFGATE-SPEC-0006-policy-ledger-contracts`

## Architecture decisions needed

- `PERFGATE-ADR-0001-public-crates-are-contracts`
- `PERFGATE-ADR-0002-receipts-first-performance-decisions`
- `PERFGATE-ADR-0003-local-receipts-first-server-ledger-optional`

## Policy impact

This lane should not duplicate policy rows in specs or proposals. The current
policy source files remain authoritative:

- [`policy/public_crates.txt`](../../policy/public_crates.txt)
- [`policy/absorbed_crates.txt`](../../policy/absorbed_crates.txt)
- [`policy/clippy-lints.toml`](../../policy/clippy-lints.toml)
- [`policy/clippy-debt.toml`](../../policy/clippy-debt.toml)
- [`policy/clippy-exceptions.toml`](../../policy/clippy-exceptions.toml)
- [`policy/no-panic-allowlist.toml`](../../policy/no-panic-allowlist.toml)
- [`policy/no-panic-baseline.toml`](../../policy/no-panic-baseline.toml)
- [`policy/non-rust-allowlist.toml`](../../policy/non-rust-allowlist.toml)
- [`policy/generated-allowlist.toml`](../../policy/generated-allowlist.toml)
- [`policy/executable-allowlist.toml`](../../policy/executable-allowlist.toml)
- [`policy/workflow-allowlist.toml`](../../policy/workflow-allowlist.toml)
- [`policy/dependency-surface-allowlist.toml`](../../policy/dependency-surface-allowlist.toml)

Specs should describe the behavior contract and link to these files when the
concrete reviewed surface matters.

## Evidence plan

Documentation-only PRs in this lane should run:

```bash
cargo +1.95.0 run -p xtask -- docs-check
cargo +1.95.0 run -p xtask -- doc-test
git diff --check
```

Package and release specs should point at existing proof gates instead of
copying their full tables:

```bash
cargo +1.95.0 run -p xtask -- public-surface --strict
cargo +1.95.0 run -p xtask -- arch
cargo +1.95.0 run -p xtask -- action-check
cargo +1.95.0 run -p xtask -- schema-compat
cargo +1.95.0 run -p xtask -- publish-check --package-list
```

Later enforcement PRs should add narrow checkers:

```text
cargo +1.95.0 run -p xtask -- docs-source-check
cargo +1.95.0 run -p xtask -- product-claims-check
```

## Risks

- The stack could become decorative if docs do not link to proof commands.
- Specs could become another source of drift if they copy policy ledgers.
- Plans could become behavior specs if work items do not link back to specs.
- Enforcement could become brittle if full graph completeness is required too
  early.
- Active goal manifests could become stale if they are not archived or updated
  at lane boundaries.

## Non-goals

- No product behavior changes are introduced by this proposal.
- No crates are added, removed, published, or reclassified by this proposal.
- No release tag, GitHub release, or crates.io publish step is implied.
- No policy ledger row is changed by this proposal.
- No full documentation graph checker is required before the initial artifacts
  exist.
- No existing historical ADR is rewritten by this proposal.

## Exit criteria

This proposal is complete when:

- the source-of-truth scaffold exists;
- the source-of-truth stack spec is accepted;
- package-surface, performance-decision, user-devex, release-proof, and
  policy-ledger contracts have specs or explicitly deferred follow-up issues;
- public-crate and receipts-first architecture decisions are recorded in ADRs;
- product claims have support tiers and proof mapping;
- the 0.18.0 implementation plan identifies PR-sized work and proof commands;
- `.codex/goals/active.toml` describes the current campaign state;
- initial source-of-truth and product-claim checkers exist or are explicitly
  deferred in the plan; and
- docs gates pass on the final lane closeout.
