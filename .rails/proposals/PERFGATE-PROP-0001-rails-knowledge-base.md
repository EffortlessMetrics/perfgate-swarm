# PERFGATE-PROP-0001: Rails knowledge base for durable repo truth

Status: accepted
Owner: docs-platform
Created: 2026-05-21
Target milestone: 0.19
Linked specs: PERFGATE-SPEC-0001
Linked ADRs: PERFGATE-ADR-0001
Linked lanes: rails-adoption

## Problem

Durable source-of-truth artifacts are currently distributed across docs surfaces without a single framework root, making adoption and tooling composition harder.

## Users and surfaces

Maintainers, release owners, and contributors across docs, CI policy, and implementation planning surfaces.

## Success criteria

A portable `.rails/` directory exists with proposal/spec/ADR/lane/closeout/support/policy anchors and a canonical index.

## Proposed shape

Adopt `.rails/` as the durable framework footprint and keep external agent namespaces awareness-only.

## Alternatives considered

- Keep only `docs/` for durable artifacts (rejected: weak typing of artifact roles).
- Store durable artifacts in agent namespaces (rejected: ownership boundary violation).

## Specs to create or update

- PERFGATE-SPEC-0001

## Architecture decisions needed

- PERFGATE-ADR-0001

## Implementation campaign shape

1. Introduce `.rails/` and docs.
2. Add templates and first artifact graph.
3. Add validators and downstream surfaces.

## Evidence plan

- `git diff --check`

## Risks

Potential duplication with existing docs unless clear ownership boundaries are maintained.

## Non-goals

Migrating `.codex/`, `.spec/`, `.claude/`, or `.jules/` content.

## Exit criteria

`.rails/` footprint and index are present with linked foundational artifacts and contributor guidance.
