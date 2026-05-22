# Rails adoption: implementation plan

## Objective

Install the portable `.rails/` framework root and establish the initial linked artifact graph.

## PR sequence

1. Add `.rails/` footprint and human docs.
2. Add templates and foundational artifacts.
3. Add validators and graph enforcement.

## Proof strategy

- `git diff --check`
- Future: `cargo run -p xtask -- rails check`
