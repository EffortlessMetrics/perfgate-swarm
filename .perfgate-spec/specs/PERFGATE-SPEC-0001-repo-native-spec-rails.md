# PERFGATE-SPEC-0001: Repo-native spec rails

Status: accepted
Owner: repo-architecture
Created: 2026-05-21
Linked proposal: PERFGATE-PROP-0001
Linked ADRs: PERFGATE-ADR-0001
Linked lane: spec-system
Linked issues: n/a
Linked PRs: n/a
Support-tier impact: none
Policy impact: references only

## Problem

The repository needs a durable and tool-neutral location for product and architecture artifacts.

## Behavior

- Durable rails are owned under `.perfgate-spec/`.
- Human-facing method documentation is under `docs/`.
- Live enforcement ledgers stay in `policy/*.toml` and may be referenced by `.perfgate-spec/policy/ledgers.toml`.
- `.codex/`, `.spec/`, `.claude/`, and `.jules/` are external/tool-owned state and are not durable artifact homes.

## Non-goals

- Rewriting or migrating `.spec/` workflows.
- Managing agent scratch state.

## Required evidence

- `git diff --check`
- artifact linkage through `.perfgate-spec/index.toml`

## Acceptance examples

- Proposal path resolves to `.perfgate-spec/proposals/...`
- Spec path resolves to `.perfgate-spec/specs/...`
- Artifact paths do not point under `.codex/`, `.spec/`, `.claude/`, `.jules/`

## Test mapping

- Future: `cargo xtask check-repo-spec`

## Implementation mapping

- `.perfgate-spec/*`
- `docs/spec-style.md`
- `docs/contributing/spec-rails.md`

## CI proof

- `git diff --check`

## Metrics / promotion rule

Stable when all new durable rails enter through `.perfgate-spec/index.toml`.

## Failure modes

- Artifact added only in tool-state directories.
- Unlinked durable artifact files.
