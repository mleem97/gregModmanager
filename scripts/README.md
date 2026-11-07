# scripts — gregModmanager

Build, test and helper scripts for local developer workflows.

## Local developer workflows

| Activity | Script | Command |
| --- | --- | --- |
| Quality gate (rustfmt + clippy `-D warnings` + tests, all crates) | [`check.sh`](check.sh) | `cargo xtask check` |
| Dev/debug build (all crates) | [`build.sh`](build.sh) | `cargo build --workspace` |
| Build app from source and start (debug) | [`run-app.sh`](run-app.sh) | `cargo xtask setup` + `cargo run -p greg-app` |
| Run headless CLI from source | [`run-cli.sh`](run-cli.sh) | `cargo run -p greg-cli -- "$@"` |
| Release build (host platform) | [`build-release.sh`](build-release.sh) | `cargo xtask dist` (+ `--target`, `--out`) |
| Release build Windows x64 | [`build-windows.sh`](build-windows.sh) | `cargo xtask dist --target x86_64-pc-windows-gnu` |
| Release build Linux x64 | [`build-linux.sh`](build-linux.sh) | `cargo xtask dist` (Linux host) |
| Release build macOS x64/arm64 | [`build-macos.sh`](build-macos.sh) | `cargo xtask dist` (macOS host) |

Zurück: [README.md](../README.md) · Docs: [docs/INDEX.md](../docs/INDEX.md).
