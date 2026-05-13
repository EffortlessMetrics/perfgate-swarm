# Architecture Decision Records

ADRs own durable architectural decisions. Use them for choices that should
survive individual releases and continue constraining future implementation.

New spec-driven governance ADRs use the `PERFGATE-ADR-000N-*` naming scheme in
this directory. The existing [`../adrs/`](../adrs/) directory remains the
historical numbered ADR archive and should be linked when it already records the
decision context.

## Naming

```text
PERFGATE-ADR-000N-short-title.md
```

Example:

```text
PERFGATE-ADR-0001-public-crates-are-contracts.md
```

## Required Header

```md
Status: accepted
Date:
Owner:
Linked proposal:
Linked specs:
```

## Template

```md
# PERFGATE-ADR-000N: Title

Status: accepted
Date:
Owner:
Linked proposal:
Linked specs:

## Decision

State the durable architecture decision.

## Context

Why this decision exists.

## Consequences

What this enables and constrains.

## Alternatives considered

What was rejected.

## Follow-up specs / plans

What must be implemented or checked.
```

## Boundaries

- Use proposals for product strategy and motivation.
- Use specs for behavior and proof contracts.
- Use plans for PR order, file lists, rollback, and blockers.
- Do not use ADRs for temporary implementation notes.
