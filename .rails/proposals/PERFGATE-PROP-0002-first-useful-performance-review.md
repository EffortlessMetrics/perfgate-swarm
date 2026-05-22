# PERFGATE-PROP-0002: First useful performance review

Status: implemented
Owner: product-platform
Created: 2026-05-21
Target milestone: 0.22
Linked specs: PERFGATE-SPEC-0002
Linked ADRs: none
Linked lanes: first-useful-performance-review

## Problem

perfgate can ingest native and imported performance evidence, report maturity, surface policy posture, and emit review artifacts. The remaining product gap is that a new team still has to know which command to run first and how to connect those surfaces into one useful PR review.

The desired user question is not "did a benchmark run?" It is:

```text
What performance evidence exists?
Can I trust it?
What changed?
What can I not infer?
What should I run locally?
What may an agent inspect or fix?
What requires human review?
Is this ready to graduate from advisory to blocking?
```

## Users and surfaces

Primary users:

- maintainers adding perfgate to an existing repo
- reviewers reading performance evidence on a PR
- agents repairing code after a performance signal
- release operators checking whether evidence and claims are fresh enough to cite

Product surfaces likely affected:

- `perfgate adoption recommend`
- `perfgate adoption apply --dry-run`
- `perfgate review explain`
- benchmark review packets
- Action summaries
- repair context JSON
- baseline and policy promote-plan commands
- docs, canaries, and product claims

## Success criteria

The lane succeeds when a user can start from an existing repo and get one coherent performance review without learning perfgate internals first.

At the end, perfgate should answer:

```text
what evidence exists
where it came from
what metric moved
what the baseline status is
what the signal maturity is
what the host context says
what the policy posture is
what must not be inferred
what command to run next
what agents may do
what requires human review
```

## Proposed shape

Build a review loop on top of the existing evidence-intake and maturity substrate:

1. Recommend a reviewable adoption pack from repository shape.
2. Emit dry-run setup patches and local commands without mutating policy by default.
3. Compose baseline doctor, signal doctor, policy doctor, evidence source, and next-command guidance into `review explain`.
4. Add a benchmark passport to review packets and Action summaries.
5. Emit agent-safe repair context and an optional copyable repair prompt.
6. Add non-mutating baseline and policy promote plans.
7. Prove the loop through hosted/public canaries before strengthening product claims.

## Alternatives considered

- Build a dashboard first. Rejected because the next useful surface is a good PR review, not another place to look.
- Add another benchmark engine. Rejected because perfgate should sit above Criterion, hyperfine, pytest-benchmark, k6, custom scripts, and project-specific tools.
- Make mature evidence blocking by default. Rejected because graduation from advisory to blocking must stay explicit and reviewable.
- Put server ledger in the core path. Rejected because local receipts remain the correctness contract and server history stays optional.

## Specs to create or update

- PERFGATE-SPEC-0002: First useful performance review contract

## Architecture decisions needed

- none expected initially

## Implementation campaign shape

1. Define the first-use review contract.
2. Add a lane plan and active execution state.
3. Make adoption packs operational with recommend and dry-run apply.
4. Add `review explain` and benchmark passport output.
5. Extend repair context with agent-safe review guidance.
6. Add non-mutating baseline and policy promotion plans.
7. Record external canaries and update product claims only where proof exists.

## Evidence plan

Expected proof commands:

```bash
cargo +1.95.0 run -p xtask -- rails check
cargo +1.95.0 run -p xtask -- docs-source-check
cargo +1.95.0 run -p xtask -- docs-check
cargo +1.95.0 run -p xtask -- doc-test
cargo +1.95.0 run -p xtask -- product-claims-check
git diff --check
```

Behavior PRs should add focused CLI tests and run the narrowest relevant crate checks before broader CI.

## Risks

- The lane could become another source-of-truth framework instead of a product path.
- Adoption recommendations could look like magic benchmark selection if confidence, inspected inputs, and bad fits are not explicit.
- Review output could overstate first-run evidence as mature enough to block.
- Agent guidance could be mistaken for permission to promote baselines, loosen thresholds, or accept tradeoffs.

## Non-goals

- dashboard
- scheduler
- new benchmark engine
- automatic baseline promotion
- automatic threshold loosening
- default blocking gates
- mandatory server ledger
- public crate expansion
- receipt schema changes without an explicit spec
- release, publish, signing, tag, or alias changes

## Exit criteria

This proposal is done when the lane has a linked spec, plan, proof-backed implementation closeout, and product-claim updates that distinguish implemented behavior from remaining canary or public-release gaps.
