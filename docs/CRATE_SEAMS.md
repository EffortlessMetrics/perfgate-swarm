# Perfgate Crate Seams and Architecture

This document defines the target public crate surface and the absorption plan for internal implementation crates.

## Governing Rule

> **Crates are public contracts. Folders are architectural boundaries.**

## Target Public Crates (13)

### Primary Product / Contract Crates

| Crate | Purpose |
|-------|---------|
| `perfgate` | Umbrella library for local performance-gating workflows |
| `perfgate-cli` | Installed binary package, command-line UX |
| `perfgate-server` | Centralized baseline management server |
| `perfgate-client` | Client SDK for baseline service |
| `perfgate-api` | Shared baseline-service API contract |
| `perfgate-types` | Versioned receipt/config/domain DTO contract |
| `perfgate-config` | Config loading, parsing, merging, normalization |
| `perfgate-domain` | Pure decision logic: stats, budgets, significance, paired/scaling analysis, host mismatch policy |

### Published Support Crates

| Crate | Purpose |
|-------|---------|
| `perfgate-error` | Shared error surface across the public crate family |
| `perfgate-render` | Markdown, GitHub annotation text, report rendering |
| `perfgate-export` | CSV, JSONL, HTML, Prometheus, JUnit export formats |
| `perfgate-ingest` | Importing Criterion, hyperfine, and external benchmark formats |
| `perfgate-github` | GitHub API / PR-comment integration |

## Absorbed Crates

The following crates will be moved into module folders under their owning crates:

| Current Crate | New Location | Owner Crate |
|---------------|--------------|-------------|
| `perfgate-validation` | `perfgate-types::validation` | perfgate-types |
| `perfgate-auth` | `perfgate-api::auth` | perfgate-api |
| `perfgate-summary` | `perfgate-render::summary` | perfgate-render |
| `perfgate-stats` | `perfgate-domain::stats` | perfgate-domain |
| `perfgate-significance` | `perfgate-domain::significance` | perfgate-domain |
| `perfgate-budget` | `perfgate-domain::budget` | perfgate-domain |
| `perfgate-paired` | `perfgate-domain::paired` | perfgate-domain |
| `perfgate-host-detect` | `perfgate-domain::host_mismatch` | perfgate-domain |
| `perfgate-scaling` | `perfgate-domain::scaling` | perfgate-domain |
| `perfgate-app` | `perfgate::workflow` | perfgate |
| `perfgate-adapters` | `perfgate::ops` | perfgate |
| `perfgate-profile` | `perfgate::diagnostics::profile` | perfgate |
| `perfgate-sha256` | `perfgate::fingerprint` | perfgate |
| `perfgate-sensor` | `perfgate::integrations::cockpit` | perfgate |
| `perfgate-fake` | `tests/support` or `perfgate-testkit` | internal |
| `perfgate-selfbench` | `benches/` or internal | internal |

## Dependency Direction Rules

### Good Dependencies

```
perfgate-cli -> perfgate -> perfgate-domain -> perfgate-types
perfgate-server -> perfgate-api -> perfgate-domain -> perfgate-types
perfgate-client -> perfgate-api -> perfgate-types
perfgate-render/export/ingest -> perfgate-types
perfgate-config -> perfgate-types
```

### Bad Dependencies (Forbidden)

```
perfgate-types -> perfgate-domain
perfgate-config -> perfgate-client
perfgate-domain -> perfgate-render
perfgate-domain -> perfgate-config
perfgate-api -> perfgate-server
perfgate-client -> perfgate-server
perfgate-cli -> perfgate-app/adapters/profile/sensor/scaling
```

## Module-Layer Rules

1. **One owner crate per concept**
2. **One folder per absorbed former crate**
3. **Curated public facades**
4. **`pub(crate)` by default**
5. **No deep lateral imports**
6. **Module-local tests and docs**

## Publication Rules

- Only the 13 public crates should have `publish = true`
- All internal/dev-only packages must have `publish = false`
- Absorbed crates must be removed from `[workspace].members`
- Absorbed crates must be removed from `[workspace.dependencies]`
- No absorbed folder should contain its old `Cargo.toml`

## Migration Order

1. Phase 0: Create this documentation and policy files
2. Phase 1: Add workspace defaults
3. Phase 2: Absorb contract-adjacent crates (validation, auth, summary)
4. Phase 3: Collapse domain shards (stats, significance, budget, paired, host-detect, scaling)
5. Phase 4: Collapse runtime/orchestration (adapters, profile, sensor, app, sha256)
6. Phase 5: Clean up config/client dependency direction
7. Phase 6: Simplify CLI dependencies
8. Phase 7: Move test-only utilities (fake, selfbench)
9. Phase 8: Final manifest shrink and validation

## Validation Commands

Run after each phase:
```bash
cargo check --workspace --all-targets
cargo test --workspace
cargo run -p xtask -- ci
```

Run before publishing:
```bash
cargo package --list -p perfgate
cargo publish --dry-run -p perfgate
```

Check for absorbed names:
```bash
rg "perfgate-(stats|significance|budget|paired|host-detect|scaling|app|adapters|profile|sha256|sensor|summary|validation|auth)"
```
