# TOOLS.md — Build, CI, and Operations

## Build

- Quality gate (rustfmt + clippy `-D warnings` + tests, all crates): `./scripts/check.sh`
- Local dev build (all crates): `./scripts/build.sh`
- Interactive builder: none — the Rust workspace exposes `cargo build`, `cargo clippy`,
  `cargo test`, and `cargo xtask check|setup|dist` directly (see `scripts/check.sh`,
  `scripts/build.sh`, `scripts/run-app.sh`, `scripts/run-cli.sh`).
- CI workflows: `.forgejo/workflows/` (Forgejo Actions) and `.gitea/workflows/`
  (Gitea Actions) — both directories are byte-identical by convention (enforced
  by the `parity` job), Linux-only runners; Windows artifacts are cross-built.
  GitHub is a read-only archive mirror — `.github/` contains no workflows.
- Release orchestration: `xtask` (`cargo xtask dist`, `cargo xtask check`, `cargo xtask setup`).
- Keep script and workflow changes aligned unless the PR explicitly documents a staged migration.

## Steam Workshop

- Steam AppID for Data Center: `4170200`.
- `SteamPublishRateLimiter.Shared` enforces cooldown behavior for publish attempts.
- UI cooldown copy should show seconds when rate-limit state is visible.

## Signing

- Windows signing uses `build/installer/sign-authenticode.ps1`.
- `auto`/`self-signed` creates a fresh build certificate; use `-SigningMode pfx`
  with `CODE_SIGN_THUMBPRINT`, or `CODE_SIGN_PFX` plus `CODE_SIGN_PFX_PASSWORD`,
  to reuse an existing certificate.
- Do not include `steam_api64.dll` in signing loops unless the maintainer confirms the vendor binary is safe to sign.

## Telemetry

- Telemetry implementation: `crates/greg-platform/src/telemetry.rs`.
- Telemetry must respect the enabled flag from `crates/greg-platform/src/prefs.rs`.
- Installation tracking uses an anonymized machine identifier.

## Troubleshooting

- Duplicate window chrome: check custom-chrome settings in `crates/greg-app/ui/`.
- Steam upload blocked: inspect rate-limit state and cooldown copy.
- Release build fails on Linux UI deps: install the Slint system dependencies
  (`libxkbcommon-dev`, `libfontconfig1-dev`, …) — see `scripts/check.sh` and the CI jobs.

## Commands

| Task | Command |
| --- | --- |
| Quality gate (rustfmt + clippy `-D warnings` + tests, all crates) | `./scripts/check.sh` |
| Dev/debug build (all crates) | `./scripts/build.sh` |
| Build app from source and start (debug) | `./scripts/run-app.sh` |
| Run headless CLI from source | `./scripts/run-cli.sh` |
| Release build (host platform) | `./scripts/build-release.sh` |
| Release build Windows x64 | `./scripts/build-windows.sh` |
| Release build Linux x64 | `./scripts/build-linux.sh` |
| Release build macOS x64/arm64 | `./scripts/build-macos.sh` |
