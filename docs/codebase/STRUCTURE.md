# Codebase Structure

> Owner: gregModmanager maintainers. Evidence: workspace `Cargo.toml`, crate
> manifests, repository tree. Update after any crate, entry-point, or
> directory-layout change.

## Top-Level Map

| Path | Purpose | Evidence |
| --- | --- | --- |
| `crates/` | Rust workspace: domain, platform, steam, modstore, loader, app, cli | `Cargo.toml` |
| `xtask/` | Build orchestration (`cargo xtask …`) | `xtask/src/main.rs` |
| `scripts/` | Local developer workflow scripts | `scripts/` |
| `build/` | Packaging scripts, installers, Linux packaging | `build/` |
| `docker/` | Rust build/test containers | `docker/` |
| `docs/` | Canonical repository documentation | `docs/` |
| `crates/greg-app/ui/` | Slint UI sources | `crates/greg-app/ui/` |

## Entry Points

- Desktop runtime: `crates/greg-app/src/main.rs`.
- Headless path: `crates/greg-cli/src/main.rs`.
- Native plugin source: `.reference/modmanager-old/src/GregModmanager.Melons/`.

## Module Boundaries

| Boundary | Owns | Must not own |
| --- | --- | --- |
| Domain crates | Steam, Modstore, game integration, pure logic | UI layout |
| `greg-platform` | Filesystem/log/telemetry/workspace/platform shims | Domain rules |
| `greg-app` | UI composition root + presentation | Generic deployment policy |
| `greg-cli` | Headless commands | UI |

## Organization

Rust module layout per crate; no barrel files. Platform-specific code lives in
`crates/greg-platform` behind `cfg(target_os = …)`; game adapters live in
`crates/greg-loader`.

## Evidence

- `Cargo.toml`
- `crates/greg-app/src/main.rs`
- `crates/greg-cli/src/main.rs`
- `.reference/modmanager-old/GregModmanager.sln`
