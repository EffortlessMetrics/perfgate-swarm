# perfgate Plans

Plans own how work lands. They translate accepted proposals and specs into
PR-sized work items with scoped file changes, proof commands, blockers,
rollback, and deferred work.

Plans are operational. They should be easy for Codex and humans to execute
without searching old chats.

## Layout

```text
plans/
  <milestone>/
    README.md
    implementation-plan.md
    <work-item>.md
```

Example:

```text
plans/0.18.0/implementation-plan.md
```

## Required Header

```md
Status:
Owner:
Created:
Milestone:
Current PR:
Linked proposal:
Linked specs:
Linked ADRs:
Linked policy:
Support/status impact:
Proof commands:
Blocks:
Blocked by:
Rollback:
```

## Work Item Template

````md
## Work item: short-id

Status: ready
Linked proposal:
Linked spec:
Linked ADR:
Blocks:
Blocked by:

### Goal

What will this PR-sized item accomplish?

### Production delta

What files or behavior will change?

### Non-goals

What is intentionally out of scope?

### Acceptance

What must be true?

### Proof commands

```bash
cargo +1.95.0 run -p xtask -- docs-check
cargo +1.95.0 run -p xtask -- doc-test
git diff --check
```

### Rollback

How to revert safely.
````

## Boundaries

- Link to proposals for why.
- Link to specs for behavior.
- Link to ADRs for durable architecture decisions.
- Link to policy ledgers for governed surfaces and exceptions.
- Keep PR sequence and file scope here, not in specs.
