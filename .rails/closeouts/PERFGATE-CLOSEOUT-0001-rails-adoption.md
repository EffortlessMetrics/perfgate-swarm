# Rails adoption: closeout

Date: 2026-05-22
Owner: docs-platform
Linked proposal: PERFGATE-PROP-0001
Linked specs: PERFGATE-SPEC-0001
Linked ADRs: PERFGATE-ADR-0001

## What Landed

- `.rails/` is the durable source-of-truth framework root.
- `.rails/index.toml` is the single registry for Rails artifacts and lanes.
- Human guidance exists in `docs/rails.md` and `docs/contributing/rails.md`.
- Foundational proposal, spec, and ADR artifacts are registered and linked.
- `cargo xtask rails check` validates registry parseability, artifact IDs, file paths, statuses, links, and the absence of `.perfgate-spec/`.

## Proof

- `cargo +1.95.0 run -p xtask -- rails check`
- `git diff --check`

## Follow-Up Work

- Product lanes should use `.rails/` for durable proposals, specs, trackers, support maps, policy references, and closeouts.
- `.codex/` remains agent execution state only and is not a durable Rails artifact root.
- Future validator slices may add deeper lane tracker schema checks when the lane vocabulary stabilizes.
