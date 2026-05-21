# Contributing to Rails artifacts

Use `.rails/` for durable repository knowledge, and keep tool/agent state in external namespaces.

## Source-of-truth split

- Proposal: why work exists and what outcomes matter.
- Spec: behavior contracts and required evidence.
- ADR: durable architecture decisions.
- Lane tracker: focused implementation sequence.
- Support map: what users may claim and what proves it.
- Policy reference: where live ledgers are governed.
- Closeout: what landed, what proved it, and what remains.

## Rules

1. Add or update owned artifacts under `.rails/` only.
2. Link every owned artifact through `.rails/index.toml`.
3. Keep lane trackers focused by lane; do not create one giant shared active queue.
4. Do not migrate or rewrite `.codex/`, `.spec/`, `.claude/`, or `.jules/` as part of Rails changes.
5. Keep artifact IDs repo-scoped (`PERFGATE-PROP-*`, `PERFGATE-SPEC-*`, `PERFGATE-ADR-*`).
