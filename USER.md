# USER.md — Project Context

## Repository

- Application: gregModmanager.
- Purpose: cross-platform desktop mod manager for the gregFramework ecosystem.
- UI framework: Slint (`crates/greg-app/ui/`), icons via `lucide-slint`.
- Desktop targets: Rust workspace crates under `crates/` (see `RUST_GRAPH.md`).
- Primary solution: Cargo workspace (`Cargo.toml` at the repository root).

## Important paths

- Desktop app: `crates/greg-app/`.
- Domain services and models: `crates/greg-core/`, `crates/greg-steam/`,
  `crates/greg-modstore/`, `crates/greg-loader/`.
- Platform shims: `crates/greg-platform/`.
- Headless CLI: `crates/greg-cli/`.
- Build orchestration: `xtask/`.
- Local developer workflow scripts: `scripts/`.
- Installer assets: `build/installer/`.
- Canonical documentation: `docs/` (tracked directly in this repository).

## Architecture

- Domain crates must not depend on UI crates (`slint`).
- `greg-app` / `greg-cli` are thin shells: composition root + presentation.
- Platform-specific behavior lives in `crates/greg-platform` behind
  `cfg(target_os)`.

## Release context

- Version source of truth: `VERSION` file (the workspace `Cargo.toml` version is
  the fallback).
- Changelog source of truth: `CHANGELOG.md`.
- Release promotion normally runs through `.forgejo/workflows/promote-changelog.yml`.
