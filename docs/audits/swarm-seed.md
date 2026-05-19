# perfgate-swarm seed audit

Date: 2026-05-19

## Source

- Publishing repo: `EffortlessMetrics/perfgate`
- Swarm development repo: `EffortlessMetrics/perfgate-swarm`
- Seed source commit: `f35e3c6365c89a2136a12b14ffd1f5907a7c94a6`
- Seed source ref: `EffortlessMetrics/perfgate@main`

## Permanent repo boundaries

`EffortlessMetrics/perfgate` remains the canonical publishing repo forever. It owns crates.io package metadata, release tags, release branches, publish workflows, signing workflows, GitHub Releases, canonical repository/homepage URLs, and final release provenance.

`EffortlessMetrics/perfgate-swarm` is the internal swarm development repo. It owns trusted same-repo feature branches, swarm PRs, routed development CI, fast iteration, and development branch protection.

`EffortlessMetrics/perfgate-dev` is reserved for later external PR intake. It will own fork-safe contribution review when created.

## Seed choices

- Release tags were not pushed to `perfgate-swarm`.
- Cargo package `repository` and `homepage` metadata remain pointed at `https://github.com/EffortlessMetrics/perfgate`.
- Release, publish, signing, tag, package metadata, and release secrets do not move to `perfgate-swarm`.
- `.github/workflows/release.yml` was intentionally omitted from the swarm seed.
- `perfgate-swarm` is not the external PR intake repo.

## Development flow

New internal development goes to `EffortlessMetrics/perfgate-swarm`.

Release promotion remains a controlled operation into `EffortlessMetrics/perfgate`.

External PR intake will be handled later through `EffortlessMetrics/perfgate-dev`.

## CI routing target

The initial swarm CI target is:

```text
Linux xtask CI: CX43 -> CX53 -> GitHub-hosted
Windows: GitHub-hosted
Postgres integration: GitHub-hosted
cargo-deny: GitHub-hosted
release/publish/signing: absent
```

Branch protection should require only the normalized final result check after routed CI proof.
