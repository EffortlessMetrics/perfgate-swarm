# xtask

Developer automation crate for the perfgate workspace.

## What It Does

- Generates and checks JSON schemas (`schema`, `schema-check`, `schema-compat`).
- Runs the standard CI command bundle (`ci`).
- Validates crates.io packaging metadata before release (`publish-check`), with
  opt-in package-list and publish dry-run proof for release prep.
- Validates GitHub Action install and release asset wiring (`action-check`).
- Validates public crate dispositions and compatibility-wrapper isolation (`public-surface`).
- Validates source-of-truth rails, source docs, product-claim proof maps, and
  generated-file policy (`rails check`, `docs-source-check`,
  `product-claims-check`, `check-file-policy`).
- Enforces workspace architecture dependency rules (`arch`).
- Checks generated documentation drift (`docs-check`).
- Validates documentation CLI examples plus TOML, JSON, and YAML snippets (`doc-test`).
- Validates fixtures against vendored contracts (`conform`).
- Runs the fast local PR gate bundle (`pr`, `check-pr`).
- Wraps standard upstream engines for tests, coverage, dependencies, semver,
  workflow checks, TOML checks, and targeted mutation (`test-pr`, `test-docs`,
  `coverage`, `check-deps`, `check-supply-chain`, `semver-check`,
  `check-workflows`, `check-toml`, `mutation-targeted`, `miri-targeted`).
- Syncs golden fixtures into `contracts/fixtures` (`sync-fixtures`).
- Runs mutation testing helpers (`mutants`).

## Why It Exists

`xtask` keeps project maintenance flows in typed Rust code instead of shell scripts, so local dev and CI use the same logic. The repository tooling standard keeps upstream tools in the engine room while `xtask` remains the public control plane; see [`docs/TOOLING_STANDARD.md`](../docs/TOOLING_STANDARD.md).

## Usage

```bash
cargo run -p xtask -- schema
cargo run -p xtask -- schema-check
cargo run -p xtask -- schema-compat
cargo run -p xtask -- ci
cargo run -p xtask -- publish-check
cargo run -p xtask -- publish-check --package-list
cargo run -p xtask -- publish-check --dry-run --package perfgate-types
cargo run -p xtask -- action-check
cargo run -p xtask -- public-surface
cargo run -p xtask -- rails check
cargo run -p xtask -- arch
cargo run -p xtask -- docs-check
cargo run -p xtask -- doc-test
cargo run -p xtask -- docs-source-check
cargo run -p xtask -- product-claims-check
cargo run -p xtask -- check-file-policy
cargo run -p xtask -- conform
cargo run -p xtask -- pr
cargo run -p xtask -- check-pr
cargo run -p xtask -- fix-pr
cargo run -p xtask -- pr-summary
cargo run -p xtask -- allow-check
cargo run -p xtask -- allow-diff
cargo run -p xtask -- ripr-pr
cargo run -p xtask -- unsafe-review-pr
cargo run -p xtask -- test-pr
cargo run -p xtask -- test-docs
cargo run -p xtask -- coverage
cargo run -p xtask -- check-deps
cargo run -p xtask -- check-supply-chain
cargo run -p xtask -- semver-check
cargo run -p xtask -- check-workflows
cargo run -p xtask -- check-toml
cargo run -p xtask -- policy-report
cargo run -p xtask -- miri-targeted
cargo run -p xtask -- mutants --crate perfgate-domain --summary  # logical alias for perfgate::domain
cargo run -p xtask -- mutation-targeted --crate perfgate-domain
```

## License

Licensed under either Apache-2.0 or MIT.
