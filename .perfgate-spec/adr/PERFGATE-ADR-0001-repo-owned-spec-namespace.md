# PERFGATE-ADR-0001: Repo-owned spec namespace

Status: accepted
Date: 2026-05-21
Owner: repo-architecture
Linked proposal: PERFGATE-PROP-0001
Linked specs: PERFGATE-SPEC-0001

## Decision

Use `.perfgate-spec/` as the long-term, repo-owned namespace for proposals, specs, ADRs, lane trackers, implementation plans, support references, policy references, and closeouts.

## Context

Tool-specific directories are valuable execution context but are not stable ownership boundaries for durable artifacts.

## Consequences

Improves auditability and enables deterministic linkage via `.perfgate-spec/index.toml`.

## Alternatives considered

- Tool-owned directories as canonical rails (rejected).
- Flat docs-only approach without index linkage (rejected).

## Follow-up specs / plans

- Add validator commands in `xtask` for index and lane tracker integrity.
