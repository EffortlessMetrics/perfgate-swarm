# perfgate Swarm Promotion Contract

Status: accepted
Owner: perfgate maintainers
Created: 2026-05-20

This document defines the permanent repository roles and promotion rules for
`EffortlessMetrics/perfgate`, `EffortlessMetrics/perfgate-swarm`, and the future
`EffortlessMetrics/perfgate-dev` intake repo.

## Repository Roles

| Repository | Role | Release/publish/signing authority |
|------------|------|------------------------------------|
| `EffortlessMetrics/perfgate` | Canonical publishing and release repo | Yes |
| `EffortlessMetrics/perfgate-swarm` | Internal swarm development repo | Never |
| `EffortlessMetrics/perfgate-dev` | Future external PR intake repo | Never |

`EffortlessMetrics/perfgate` owns crates.io package metadata, release tags,
release branches, publish workflows, signing workflows, GitHub Releases,
canonical repository/homepage URLs, and final release provenance.

`EffortlessMetrics/perfgate-swarm` owns trusted internal feature branches, swarm
PRs, routed self-hosted development CI, and fast iteration. It does not receive
release secrets, publish tokens, signing keys, release tags, or canonical
package metadata.

`EffortlessMetrics/perfgate-dev` is reserved for external contribution intake.
When created, it should use fork-safe hosted CI and no release secrets. Accepted
external patches should be maintainer-vetted and ported into `perfgate-swarm`.

## Merge Policy

Use squash merges for normal PRs into `perfgate-swarm`.

Do not squash promotion from `perfgate-swarm` into `perfgate`. Promotion PRs into
the publishing repo must use a normal merge commit so `perfgate` preserves the
sequence of delivered swarm commits.

The intended graph is:

```text
perfgate/main:
A -- B -- C ---------------------- M
             \                    /
perfgate-swarm/main:
              S1 -- S2 -- S3 ----
```

Where:

```text
S1/S2/S3 = squash-merged swarm PRs
M        = merge commit into perfgate/main
```

After a promotion merge lands in `perfgate`, advance `perfgate-swarm/main` to
include the publishing repo merge commit before starting the next promotion
batch.

## Workflow Ownership

Workflow files may exist in both repos, but their authority is repository-gated.

- `.github/workflows/ci.yml` is the canonical hosted CI workflow for
  `EffortlessMetrics/perfgate`. Its jobs are guarded to run only in the
  publishing repo.
- `.github/workflows/release.yml` is the canonical release workflow for
  `EffortlessMetrics/perfgate`. Its jobs are guarded to run only in the
  publishing repo.
- `.github/workflows/em-swarm-ci.yml` is the routed swarm development workflow
  for `EffortlessMetrics/perfgate-swarm`. Its jobs are guarded to run only in
  the swarm repo.

This lets `perfgate-swarm` merge back to `perfgate` without deleting publishing
automation or replacing publishing CI with routed self-hosted development CI.

## Normal Development

```text
branch from perfgate-swarm/main
open PR to perfgate-swarm/main
run routed swarm CI
squash merge
delete branch
```

`perfgate-swarm` branch protection should require the normalized final check:

```text
Perfgate Swarm CI Result
```

Do not require conditional implementation jobs such as:

```text
Route Perfgate Rust Small
Perfgate Rust Small on CX43
Perfgate Rust Small on CX53
Perfgate Rust Small on GitHub Hosted
```

Those jobs are intentionally conditional.

## Branch Hygiene

Delete normal swarm branches after their PRs squash-merge. If a branch survives
because automation or a manual merge did not delete it, it may be removed when
all of these are true:

- the branch belongs to `EffortlessMetrics/perfgate-swarm`;
- the branch is not `main`, a release ref, or an explicit backup ref;
- there is no open PR for the branch; and
- the branch head is either a merged PR head, a closed/superseded PR head, or
  generated churn that has been replaced by a newer run.

For closed but unmerged branches, preserve the closed PR as the durable review
record and delete only the stale branch ref. Do not delete publishing-repo
branches from swarm cleanup. Do not delete `backup/*` refs unless the repair or
backup they protect has been explicitly retired.

Generated badge and baseline branches follow the same rule: merge them only when
they are current, generated-only, and proven by the required check; otherwise
close or supersede the PR and delete the stale generated branch.

## Promotion to Publishing

Create promotion PRs in `EffortlessMetrics/perfgate` from `perfgate-swarm/main`
when a coherent batch is ready, immediately before release prep, or when an
urgent fix must reach the publishing repo.

Promotion PRs must:

- target `EffortlessMetrics/perfgate:main`;
- use a normal merge commit;
- not use squash merge or rebase merge;
- preserve `EffortlessMetrics/perfgate` as the only publishing authority;
- preserve canonical package metadata pointing at
  `https://github.com/EffortlessMetrics/perfgate`;
- run publishing-repo validation before publication; and
- avoid importing release secrets or tags into `perfgate-swarm`.

After the promotion PR merges, update `perfgate-swarm/main` from
`perfgate/main` so future comparisons return to a small ahead count.

## Drift Rules

During normal development, `perfgate-swarm` should compare against `perfgate` as:

```text
behind_by: 0
ahead_by: N
```

`N` should be only swarm development commits that have not yet been promoted.

Routine generated churn should not accumulate independently in both repos. If a
bot-generated change belongs to normal development, route it through
`perfgate-swarm`. If it is release-proof or publishing-only material, land it in
`perfgate` and then bring that commit back into `perfgate-swarm`.

## Non-Goals

- Do not move release/publish/signing authority to `perfgate-swarm`.
- Do not use `perfgate-swarm` as the external PR intake repo.
- Do not give `perfgate-swarm` release secrets or signing keys.
- Do not mint release tags in `perfgate-swarm`.
- Do not change package `repository` or `homepage` metadata to point at
  `perfgate-swarm`.
- Do not promote swarm work into `perfgate` with a squash merge.
