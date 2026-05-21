# Spec style and source-of-truth stack

Perfgate separates durable product/architecture rails from tool execution state.

- Durable rails live in `.perfgate-spec/`.
- Human-facing guidance lives in `docs/`.
- Live policy ledgers stay in `policy/` and are referenced where needed.
- Existing `plans/` remains optional and only for established non-agent planning usage.

## External agent state

This repo may contain `.codex/`, `.claude/`, `.jules/`, or similar tool-specific directories.

Those directories are not the durable source of truth for this spec system.
Agents may read `.perfgate-spec/` to decide what to do, but this system does not manage agent scratch state.

## Spec Kit coexistence

If `.spec/` exists, it is reserved for Spec Kit / speckit workflows.

The repo-native long-term spec rails live in `.perfgate-spec/`.
This lane does not migrate, rewrite, validate, or depend on `.spec/`.
