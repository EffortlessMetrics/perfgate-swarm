# Proposals

Proposals own why a lane exists. They describe the problem, the users and
surfaces affected, rejected alternatives, success criteria, and which specs,
ADRs, plans, policy ledgers, or status docs the lane should create.

Proposals should be stable enough for future readers to understand the reason
for the work, but they are not PR checklists.

## Naming

```text
PERFGATE-PROP-000N-short-title.md
```

Example:

```text
PERFGATE-PROP-0001-spec-driven-governance.md
```

## Required Header

```md
Status: proposed
Owner:
Created:
Target milestone:
Linked specs:
Linked ADRs:
Linked plan:
Support/status impact:
Policy impact:
```

## Template

```md
# PERFGATE-PROP-000N: Title

Status: proposed
Owner:
Created:
Target milestone:
Linked specs:
Linked ADRs:
Linked plan:
Support/status impact:
Policy impact:

## Problem

What risk, user pain, or repo gap exists?

## Users and surfaces

Who benefits? CLI users, action users, server users, maintainers, agents?

## Success criteria

What must be true when complete?

## Proposed shape

What are we doing?

## Alternatives considered

What did we reject and why?

## Specs to create or update

- PERFGATE-SPEC-...

## Architecture decisions needed

- PERFGATE-ADR-...

## Evidence plan

Proof commands, fixtures, policy gates, and status docs.

## Risks

What can go wrong?

## Non-goals

What is explicitly out of scope?

## Exit criteria

When is this proposal done?
```

## Boundaries

- Link to plans for PR sequencing.
- Link to specs for behavior contracts.
- Link to policy files for governed surfaces and exceptions.
- Link to status docs for product claim support.
- Do not copy release-readiness matrices or policy ledger entries here.
