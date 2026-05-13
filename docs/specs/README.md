# Specs

Specs own what must be true. A spec defines behavior, proof, acceptance
examples, CI evidence, product surface impact, and implementation ownership.

Specs are product controls. They are not roadmaps, release notes, or diaries.
When a spec needs concrete exception data, it links to the relevant policy
ledger instead of copying it.

## Naming

```text
PERFGATE-SPEC-000N-short-title.md
```

Example:

```text
PERFGATE-SPEC-0001-source-of-truth-stack.md
```

## Required Header

```md
Status: accepted
Owner:
Created:
Milestone:
Behavior version:
Product surface:
CI surface:
Schema impact:
Action impact:
Server impact:
Linked proposal:
Linked ADRs:
Linked plan:
Linked policy:
Support/status impact:
Proof commands:
```

## Template

```md
# PERFGATE-SPEC-000N: Title

Status: accepted
Owner:
Created:
Milestone:
Behavior version:
Product surface:
CI surface:
Schema impact:
Action impact:
Server impact:
Linked proposal:
Linked ADRs:
Linked plan:
Linked policy:
Support/status impact:
Proof commands:

## Problem

What behavior or contract gap exists?

## Behavior

What must be true?

## Non-goals

What is out of scope?

## Required evidence

What proof is required?

## Acceptance examples

Concrete pass/fail examples.

## Test mapping

Which tests, fixtures, or policy checks cover this?

## Implementation mapping

Which crates, modules, docs, or policy files own this?

## CI proof

Which commands and lanes prove it?

## Promotion rule

What moves this from proposed to accepted to implemented?
```

## Truth Ownership

| Truth | Source |
|-------|--------|
| Product claim support | `docs/status/SUPPORT_TIERS.md` and `docs/status/PRODUCT_CLAIMS.md` |
| Public crate surface | `policy/public_crates.txt` |
| Absorbed/private crate disposition | `policy/absorbed_crates.txt` |
| No-panic state | `policy/no-panic-*.toml` |
| Non-Rust file surface | `policy/*-allowlist.toml` |
| Active Codex state | `.codex/goals/active.toml` |
| PR sequencing | `plans/<milestone>/implementation-plan.md` |
| Why a lane exists | `docs/proposals/` |
| Behavior contract | `docs/specs/` |
| Architecture decision | `docs/adr/` and the historical `docs/adrs/` archive |

## Acceptance Rules

- A plan may link to policy files but must not copy policy entries.
- A spec may define package boundary behavior but must not list every package
  unless the list itself is the behavior under test.
- A proposal may describe alternatives but must not become the PR checklist.
- A goal TOML may reference a spec and plan but must not define new behavior.
