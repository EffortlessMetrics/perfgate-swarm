# Handoffs

Handoffs own closeout and operator context. They capture what changed, what
was validated, what remains, and which source-of-truth files the next person or
agent should read.

Use handoffs when a lane, release proof, queue drain, or policy pass needs a
durable closeout record.

## Naming

```text
YYYY-MM-DD-short-lane-name.md
```

Example:

```text
2026-05-13-spec-driven-governance-scaffold.md
```

## Suggested Shape

```md
# Handoff: Title

Status:
Date:
Owner:
Linked proposal:
Linked specs:
Linked ADRs:
Linked plan:
Linked policy:
Support/status impact:
Proof commands:

## Summary

What changed?

## Evidence

What commands ran, and what passed or failed?

## Remaining work

What is intentionally deferred?

## Next operator notes

What should the next person or agent read first?
```

## Boundaries

- Do not redefine behavior here; link to specs.
- Do not redefine policy exceptions here; link to policy ledgers.
- Do not use handoffs as the only source of release readiness.
