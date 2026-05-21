# perfgate repo-native spec rails

`.perfgate-spec/` is the durable, repo-owned source of truth for proposal/spec/ADR/lane/closeout artifacts.

## Scope

This namespace owns long-lived product and architecture memory:

- `roadmap/` milestone direction
- `proposals/` why and success criteria
- `specs/` behavior contracts and required evidence
- `adr/` durable architecture decisions
- `lanes/` focused implementation trackers
- `support/` support claim references
- `policy/` references to live policy ledgers in `policy/*.toml`
- `closeouts/` campaign outcomes and proof

## External tool namespaces

Tool/session directories are awareness-only for this system and are not owned by `.perfgate-spec/`:

- `.codex/`
- `.spec/`
- `.claude/`
- `.jules/`
- other agent/tool-specific state directories
