# Contributing: repo-native spec rails

When adding durable artifacts, place them under `.perfgate-spec/` and register them in `.perfgate-spec/index.toml`.

## Required chain

Proposal -> Spec -> ADR (if needed) -> Lane tracker -> Implementation plan -> PR proof -> Closeout

## Constraints

- Do not place durable artifacts in `.codex/`, `.spec/`, `.claude/`, or `.jules/`.
- Keep policy enforcement data in `policy/*.toml`; reference those ledgers from `.perfgate-spec/policy/ledgers.toml`.
