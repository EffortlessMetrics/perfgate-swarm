# PERFGATE-PROP-0001: Repo-native spec knowledge base

Status: accepted
Owner: repo-architecture
Created: 2026-05-21
Target milestone: unversioned infra
Linked specs: PERFGATE-SPEC-0001
Linked ADRs: PERFGATE-ADR-0001
Linked lanes: spec-system

## Problem

Durable planning and decision artifacts are currently distributed across mixed locations, including tool-specific state directories, which weakens long-term maintainability.

## Users and surfaces

Maintainers, contributors, and automation agents that need consistent proposal/spec/ADR/lane/closeout linkage.

## Success criteria

A repo-owned namespace exists and is recognized as the durable source of truth, while tool directories remain awareness-only.

## Proposed shape

Adopt `.perfgate-spec/` as the control plane for roadmap, proposals, specs, ADRs, lane trackers, implementation plans, support references, policy references, and closeouts.

## Alternatives considered

- Continue using mixed docs and agent state directories (rejected: unclear ownership).
- Use `.spec/` as canonical home (rejected: external tool ownership).

## Specs to create or update

- PERFGATE-SPEC-0001

## Architecture decisions needed

- PERFGATE-ADR-0001

## Implementation campaign shape

1. Add namespace doctrine and index.
2. Add templates.
3. Seed first proposal/spec/ADR/lane artifacts.

## Evidence plan

- `git diff --check`

## Risks

Contributor confusion during transition.

## Non-goals

- Migrating or modifying external tool state directories.

## Exit criteria

Core namespace, templates, and seed artifacts are committed and linked from index.
