# TOOLS.md — Build, CI, and Operations

## Build

- Quality gate: `./scripts/check.sh` (rustfmt, Clippy with warnings denied, tests).
- Development build: `./scripts/build.sh`; app/CLI helpers are in `scripts/`.
- Jenkins (`Jenkinsfile`, currently inactive) defines Rust gates and a workspace build in native
  Linux, Windows and macOS matrix agents. Linux installs the compiler, pkg-config
  and Slint native headers; Windows/macOS agents need their native build tools.
- GitHub Actions runs test builds while Jenkins is inactive. GitHub Releases is
  the download and update source; test builds do not publish a release.
- `cargo xtask dist` builds the portable package. Jenkins is configured to build
  native installers after the documented validation when it is reactivated;
  see `docs/JENKINS.md`. Signing, Discord and coverage uploads are not automated.

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

## Code coverage (Codacy)

The local coverage helper `build/scripts/linux/coverage-codacy.sh` can generate
LCOV through cargo-tarpaulin and upload with `CODACY_PROJECT_TOKEN`. The current
Jenkins check/build matrix does not run coverage or upload reports. Optional
self-hosted reporter API/version conventions require reporter-specific support;
the wrapper currently documents them but does not explicitly forward them.
Do not assume the removed Actions coverage job still runs.

## Troubleshooting

- Duplicate window chrome: check custom-chrome settings in `crates/greg-app/ui/`.
- Steam upload blocked: inspect rate-limit state and cooldown copy.
- Linux package build failures: verify external packaging tools listed in `EXTERNAL_DEPENDENCIES.md`.
- Missing Codacy coverage: run the local coverage helper with the project token;
  Jenkins currently does not publish coverage automatically.
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
