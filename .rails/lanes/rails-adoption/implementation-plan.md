# Rails adoption: implementation plan

## Objective

Install the portable `.rails/` framework root and establish the initial linked artifact graph.

## PR sequence

1. Add `.rails/` footprint and human docs. Done.
2. Add templates and foundational artifacts. Done.
3. Add validators and graph enforcement. Done.

## Proof strategy

- `git diff --check`
- `cargo +1.95.0 run -p xtask -- rails check`
