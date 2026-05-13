# perfgate 0.18.0 Plans

The 0.18.0 planning lane makes perfgate's product claims, architecture
decisions, policy ledgers, release proof, and Codex execution state traceable
through proposals, specs, ADRs, plans, and machine-readable active goals.

This README is the milestone index. The implementation plan and per-work-item
plans land after the source-of-truth scaffold.

## Initial Lane

```text
docs: define source-of-truth model
```

This first PR adds the taxonomy and templates only. It does not change product
behavior, Rust code, policy ledgers, schemas, workflows, or release state.

## Linked Homes

- Proposals: [`../../docs/proposals/`](../../docs/proposals/)
- Specs: [`../../docs/specs/`](../../docs/specs/)
- ADRs: [`../../docs/adr/`](../../docs/adr/)
- Status docs: [`../../docs/status/`](../../docs/status/)
- Handoffs: [`../../docs/handoffs/`](../../docs/handoffs/)
- Active goals: [`../../.codex/goals/`](../../.codex/goals/)
- Policy ledgers: [`../../policy/`](../../policy/)

## Guardrails For The Scaffold PR

No changes to:

```text
Cargo.toml
rust-toolchain.toml
.github/
crates/
xtask/
policy/
schemas/
```

Validation:

```bash
cargo +1.95.0 run -p xtask -- docs-check
cargo +1.95.0 run -p xtask -- doc-test
git diff --check
```
