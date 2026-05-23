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
6. Keep artifacts in the directory that matches their registered kind.
7. Keep ID-bearing artifact filenames prefixed by their registered artifact ID; support and policy registries are exempt.
8. Keep registered lane trackers at `.rails/lanes/<lane-id>/tracker.toml`.
9. Keep registered lane `id`, `name`, `status`, and `owner` values
   synchronized with the lane tracker.
10. When a lane becomes `implemented`, register an implemented closeout artifact for that lane in `.rails/index.toml`.
11. Keep `.rails/index.toml` header fields stable: schema version `1.0`, repo
   `perfgate`, framework `rails`, root `.rails`, the registered prefix
   conventions, and the external namespace map.
12. Keep artifact links typed: `linked_proposal` points to a proposal,
   `linked_specs` point to specs, and `linked_adrs` point to ADRs.
13. Keep support claim IDs and policy ledger IDs unique inside each registered
    support or policy artifact.
14. Keep support claim proof command entries non-empty.
15. Keep lane work item IDs unique and non-empty inside each lane tracker.
16. Keep lane work item statuses in the accepted lane-work vocabulary:
    `planned`, `ready`, `active`, `blocked`, `implemented`, or `superseded`.
17. Keep lane work item `proposal` and `spec` references populated and linked
    to registered proposal and spec artifacts.
18. Keep lane work item `adr` references empty when not used, or linked to a
    registered ADR artifact when used.
19. Keep lane work item `implementation_plan` paths resolvable.
20. Keep lane work item `blocks` and `blocked_by` entries non-empty, scoped to
    work item IDs in the same lane tracker, and non-self-referential.
21. Keep lane work item proof command entries non-empty.

## Validation

Run this before opening or merging Rails artifact changes:

```bash
cargo run -p xtask -- rails check
```

The check validates `.rails/index.toml` schema/project/convention/namespace
fields, registered artifact and lane paths, status values, ID prefixes, artifact
kind directories, filename identity, registry links, link target kinds,
unregistered owned artifacts, support claim references, policy ledger paths,
duplicate support claim or policy ledger IDs, lane tracker path/schema and
identity/status/owner consistency, lane work item IDs and statuses, non-empty
support and lane work item proof commands, lane work item source links and
implementation plans, lane work item dependencies, closeouts for implemented
lanes, the required human docs, and the absence of the legacy `.perfgate-spec/`
namespace.
