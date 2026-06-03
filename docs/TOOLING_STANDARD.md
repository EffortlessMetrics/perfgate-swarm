# Tooling Standard

perfgate standardizes on a small set of upstream engines, but the repository
contract is exposed through `xtask` and perfgate-owned policy files. Contributors
and agents should invoke repo-shaped commands first and treat upstream tools as
implementation details unless they are debugging a specific lane.

## Control-plane rule

```text
Make xtask the repo surface.
Make upstream tools the engine room.
```

This keeps policy encoded in Rust and checked from one place instead of
spreading release, security, testing, and source-policy decisions across ad hoc
YAML, shell scripts, or local aliases.

## Standard upstream substrate

| Plane | Standard upstream tools | perfgate role |
|-------|-------------------------|---------------|
| Syntax and codemods | `ast-grep`; rust-analyzer crates for Rust-specific authority | `ast-grep` finds syntactic candidates; Rust-aware tooling decides authoritative Rust identity. |
| Workspace graph | `cargo_metadata`; `guppy` when richer graph queries are needed | Cargo metadata inventories packages and targets; graph queries route changed-crate and risk-pack checks. |
| Test execution | `cargo-nextest`; `cargo test --doc` | Nextest is the default PR/runtime test runner; doctests remain a separate Cargo lane. |
| Coverage | `cargo-llvm-cov` | Coverage is execution-surface evidence and must not be described as proof of correctness. |
| Mutation | `ripr`; `cargo-mutants` | `ripr` shifts static mutation-exposure signal left; `cargo-mutants` remains the runtime backstop. |
| Unsafe and UB witnesses | `unsafe-review`; Miri | `unsafe-review` checks reviewability of unsafe seams; Miri supplies targeted concrete UB witnesses. |
| Source exceptions | `cargo-allow` | Source exceptions are receipted in ledgers, not buried in prose comments. |
| Dependency trust | `cargo-deny`; `cargo-vet`; RustSec/`cargo-audit`; `cargo-auditable` | Dependency policy, advisories, audits, and shipped-binary auditability stay separate but coordinated. |
| Public API and release | `cargo-semver-checks`; rustdoc JSON | Semver gates release compatibility; rustdoc JSON supports custom public-surface inventories. |
| Workflow policy | `actionlint`; `zizmor` | `actionlint` covers workflow correctness; `zizmor` covers workflow security posture. |
| Text and config hygiene | `taplo`; `typos`; Markdown link/style tooling | TOML, spelling, and Markdown checks should be wrapped before becoming blocking policy. |
| Workspace hygiene | `cargo-udeps`; `cargo-hakari` only when justified | Unused dependencies are scheduled/manual; hakari is for measured duplicate-build pain. |
| CI cache | `Swatinem/rust-cache`; `sccache` only when justified | Use simple Cargo cache defaults first; add compiler caching only when economics justify it. |

## PR/default lane doctrine

Default PR proof should optimize useful evidence per minute:

- Run repo wrappers such as `cargo run -p xtask -- pr` before raw tool commands.
- Use `ripr` on Rust behavior changes where static exposure evidence is useful.
- Route `cargo-mutants` only when risk warrants it on PRs; reserve broader
  mutation matrices for nightly and release lanes.
- Use `unsafe-review` for unsafe-contract review on PRs; reserve Miri for
  targeted, nightly, or release witness lanes.
- Keep heavyweight supply-chain, semver, coverage, and hygiene checks available
  behind stable `xtask` wrappers even if they are not all default PR gates.

## Stable wrapper names

As lanes mature, preserve these repo-facing names even if their upstream engines
or arguments change:

```bash
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
cargo run -p xtask -- mutation-targeted
cargo run -p xtask -- miri-targeted

cargo run -p xtask -- check-deps
cargo run -p xtask -- check-supply-chain
cargo run -p xtask -- semver-check
cargo run -p xtask -- check-workflows
cargo run -p xtask -- check-toml
cargo run -p xtask -- policy-report
```

Existing implemented lanes may still have historical names such as
`cargo run -p xtask -- pr`, `cargo run -p xtask -- ripr-pr`,
`cargo run -p xtask -- mutants`, and `cargo run -p xtask -- ci`. New work should
prefer adding compatibility aliases instead of making contributors memorize a new
upstream command surface.

## Non-goals

Do not make every upstream tool a mandatory default PR tax. In particular:

- Do not run full-workspace mutation testing on ordinary PRs.
- Do not run full Miri on ordinary PRs.
- Do not standardize Docker, Nix, Semgrep, `cargo-hakari`, or `sccache` across
  all repositories without a receipted product or CI-economics reason.
