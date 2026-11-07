# AUDIT_LOCAL_DEV_WORKFLOWS — recurring local developer workflows

> Owner: gregModmanager maintainers.
> Status: audit/inventory of recurring local developer activities in the
> FULL-RUST workspace.
> Evidence: repository analysis of the workspace (`Cargo.toml`, `xtask`,
> `crates/*`, `scripts/`, `.forgejo/workflows/`).
> Review trigger: update whenever a new local dev workflow script, platform,
> or crate is added.

## 1. Purpose and scope

This audit inventories all recurring local developer activities that a
contributor normally repeats locally in the `gregModmanager` repository.

The project is FULL-RUST: a Cargo workspace (`crates/greg-core`,
`crates/greg-platform`, `crates/greg-steam`, `crates/greg-modstore`,
`crates/greg-loader`, `crates/greg-app`, `crates/greg-cli`, `xtask`) with Slint
UI, a headless CLI, and native Steam integration.

The audit maps each recurring activity to a canonical local script and to the
underlying Rust command. It complements the CI workflows
(`.forgejo/workflows/rust.yml`, `.forgejo/workflows/build-and-release.yml`) with
local, developer-facing equivalents.

Out of scope: CI-only promotion activities (quality gates, release promotion)
that have no local counterpart; those are documented in `TOOLS.md` and the CI
workflows.

## 2. Recurring local workflows

### 2.1 Quality gate (all crates)

- **Activity:** run formatting check, clippy (`-D warnings`) and the full test
  suite before opening a pull request.
- **Script:** `scripts/check.sh`
- **Command:** `cargo xtask check` — rustfmt `--all -- --check`, clippy
  `--workspace --all-targets -- -D warnings`, test `--workspace`.
- **Crate coverage:** all workspace members.

### 2.2 Dev build (all crates)

- **Activity:** debug build while developing.
- **Script:** `scripts/build.sh`
- **Command:** `cargo build --workspace`
- **Crate coverage:** all workspace members.

### 2.3 Desktop app: build from source and start

- **Activity:** recurring "App aus Code bauen und Starten" — copy the Steam
  native library and launch the Slint desktop UI in the debug profile.
- **Script:** `scripts/run-app.sh`
- **Command:** `cargo xtask setup` (copies the native Steam lib next to the dev
  binaries) -> `cargo run -p greg-app` (debug build + launch).
- **Crate coverage:** `greg-app` plus the domain crates it composes.

### 2.4 Headless CLI: build from source and run

- **Activity:** recurring local run of the headless CLI (`publish`, `status`,
  `list`).
- **Script:** `scripts/run-cli.sh`
- **Command:** `cargo run -p greg-cli -- "$@"`
- **Crate coverage:** `greg-cli` plus the domain crates it uses.

### 2.5 Release build (host platform) and packaging

- **Activity:** build the release binary and the portable package locally.
- **Script:** `scripts/build-release.sh`
- **Command:** `cargo xtask dist` with pass-through flags (`--target <triple>`,
  `--out <dir>`).
- **Crate coverage:** `greg-app`, `greg-cli`, native Steam lib staging,
  documentation, archive + SHA-256 sidecar.

### 2.6 Release builds per platform

- **Windows x64:** `scripts/build-windows.sh` -> `cargo xtask dist --target
  x86_64-pc-windows-gnu` (cross-build via mingw from Linux/Debian; or native via
  Git Bash/WSL).
- **Linux x64:** `scripts/build-linux.sh` -> `cargo xtask dist` (native Linux).
- **macOS x64 / arm64:** `scripts/build-macos.sh` -> `cargo xtask dist` (native
  macOS host; Apple SDK required, no cross-build from Linux).

## 3. Platform build-host matrix

| Host | Native build | Cross build | Run app |
| --- | --- | --- | --- |
| Linux (Ubuntu/Debian — build host) | `scripts/build-linux.sh`, `scripts/build-release.sh` | Windows via `scripts/build-windows.sh` (`x86_64-pc-windows-gnu`, mingw) | `scripts/run-app.sh` |
| macOS | `scripts/build-macos.sh`, `scripts/build-release.sh` | n/a (Apple SDK; not cross-buildable from Linux) | `scripts/run-app.sh` |
| Windows | `scripts/build-release.sh` (Git Bash/WSL) | n/a | `scripts/run-app.sh` (Git Bash/WSL) |

Important: only Ubuntu/Debian is available as the Linux build host. Windows
release artifacts are cross-built from that host through the mingw toolchain
(`gcc-mingw-w64-x86-64`) plus `rustup target add x86_64-pc-windows-gnu`. macOS
release builds require a macOS host with the Apple SDK and therefore cannot run
on the Linux build host.

## 4. Crate coverage

All crates created during the Rust restart are covered by the workspace-wide
commands (`--workspace`, `--all-targets`) and by the explicitly named binaries:

| Crate | Covered by |
| --- | --- |
| `greg-core` | `check.sh`, `build.sh`, workspace tests, upload/docs logic |
| `greg-platform` | `check.sh`, `build.sh`, platform shims |
| `greg-steam` | `check.sh`, `build.sh`, Steam backend |
| `greg-modstore` | `check.sh`, `build.sh`, REST/publish clients |
| `greg-loader` | `check.sh`, `build.sh`, game sync/launch |
| `greg-app` | `check.sh`, `build.sh`, `run-app.sh`, `build-release.sh`, per-platform release scripts |
| `greg-cli` | `check.sh`, `build.sh`, `run-cli.sh`, `build-release.sh`, per-platform release scripts |
| `xtask` | `check.sh`, `build.sh`, `run-app.sh` (`setup`), `build-release.sh` (`dist`) |

## 5. Contributor recurring checklist

1. Before opening a PR: run `./scripts/check.sh`.
2. During development: `./scripts/build.sh`; start the app or CLI via
   `./scripts/run-app.sh` / `./scripts/run-cli.sh`.
3. Before a release: `./scripts/build-release.sh` and, per platform,
   `./scripts/build-windows.sh` / `./scripts/build-linux.sh` / `./scripts/build-macos.sh`.

## 6. Notes and open questions

- macOS release builds require a macOS host (Apple SDK); no macOS cross-build
  from Linux exists.
- Windows cross-builds check for the mingw cross compiler and the Windows Rust
  target before invoking `cargo xtask dist`.
- Release promotion itself runs through CI (`promote-changelog.yml`); the local
  scripts mirror the build/packaging commands (`cargo xtask dist`).
