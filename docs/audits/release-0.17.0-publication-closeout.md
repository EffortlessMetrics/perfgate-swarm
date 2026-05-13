# v0.17.0 Publication Closeout

Date: 2026-05-13

Purpose: reconcile the public v0.17.0 release state after the separate
readiness proof, publish, tag, and GitHub release steps completed.

This closeout records current public state. It does not publish crates, move
tags, create releases, or modify release assets.

## Public State

| Surface | Verified state |
| --- | --- |
| Latest GitHub release | `v0.17.0` |
| GitHub release URL | `https://github.com/EffortlessMetrics/perfgate/releases/tag/v0.17.0` |
| GitHub release published at | `2026-05-12T13:07:05Z` |
| Release commit | `71bdc33117d515d95885deb2d9350d9d67905265` |
| Exact tag | `v0.17.0` points to `71bdc33117d515d95885deb2d9350d9d67905265` |
| Action alias tag | `v0.17` points to `71bdc33117d515d95885deb2d9350d9d67905265` |
| Major action alias tag | `v0` points to `71bdc33117d515d95885deb2d9350d9d67905265` |
| Release assets | Six platform archives plus `sha256sums.txt` uploaded to the GitHub release |
| crates.io package set | `perfgate-types`, `perfgate`, `perfgate-client`, `perfgate-server`, `perfgate-cli` |

## crates.io Verification

The crates.io sparse index contains `0.17.0` entries for every allowed public
crate:

```bash
curl https://index.crates.io/pe/rf/perfgate-types
curl https://index.crates.io/pe/rf/perfgate
curl https://index.crates.io/pe/rf/perfgate-client
curl https://index.crates.io/pe/rf/perfgate-server
curl https://index.crates.io/pe/rf/perfgate-cli
```

Each response included a line with `"vers":"0.17.0"`.

`cargo +1.95.0 search perfgate-cli --limit 5` also reported:

```text
perfgate-cli = "0.17.0"
perfgate-server = "0.17.0"
```

## GitHub Release Verification

GitHub release metadata was checked with:

```bash
gh release view v0.17.0 --json tagName,name,isPrerelease,isDraft,publishedAt,targetCommitish,url,assets,body
gh api repos/EffortlessMetrics/perfgate/git/refs/tags/v0.17.0
gh api repos/EffortlessMetrics/perfgate/git/refs/tags/v0.17
gh api repos/EffortlessMetrics/perfgate/git/refs/tags/v0
```

Observed state:

- `v0.17.0` is not a draft and not a prerelease.
- `v0.17.0` was published at `2026-05-12T13:07:05Z`.
- Uploaded release assets include:
  - `perfgate-aarch64-apple-darwin.tar.gz`
  - `perfgate-aarch64-unknown-linux-gnu.tar.gz`
  - `perfgate-x86_64-apple-darwin.tar.gz`
  - `perfgate-x86_64-pc-windows-msvc.zip`
  - `perfgate-x86_64-unknown-linux-gnu.tar.gz`
  - `perfgate-x86_64-unknown-linux-musl.tar.gz`
  - `sha256sums.txt`
- `v0.17.0`, `v0.17`, and `v0` resolve to release commit
  `71bdc33117d515d95885deb2d9350d9d67905265`.

## Public Install Smoke

The public registry install path was tested from an isolated root:

```bash
cargo +1.95.0 install perfgate-cli --version 0.17.0 --locked --root C:/perfgate-smoke/release-reconcile-0170 --force
C:/perfgate-smoke/release-reconcile-0170/bin/perfgate.exe --version
C:/perfgate-smoke/release-reconcile-0170/bin/perfgate.exe doctor --help
```

Results:

- `cargo install` installed `perfgate-cli v0.17.0` from crates.io.
- `perfgate --version` printed `perfgate 0.17.0`.
- `perfgate doctor --help` printed the doctor command help.

## Relationship To Readiness Proof

`docs/audits/release-0.17.0-publish-readiness.md` remains the pre-publish
readiness record. It intentionally says that the readiness PR did not publish
crates, create tags, or create a GitHub release.

This closeout records the later public state after those separate release steps
completed.

## Remaining Work

- Keep future release readiness docs split between pre-publish proof and
  post-publish closeout evidence.
- Do not infer future publish, tag, GitHub release, or moving-alias updates from
  dry-run success alone.
