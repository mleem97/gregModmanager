# Contributor guide

**Owner:** gregModmanager maintainers. **Evidence:** workspace `Cargo.toml`, `crates/*`,
build scripts, and CI workflows. **Review trigger:** update after a dependency,
toolchain, command, release artifact, or repository-layout change.

## Local setup

Install the Rust stable toolchain via [rustup](https://rustup.rs) and the native
Linux X11 / font / GL development packages required by Slint (see the
[devcontainer Dockerfile](../.devcontainer/Dockerfile)), then restore, build,
and test from the repository root.

```bash
cargo build --workspace
cargo test --workspace
git diff --check
```

Run the desktop application during UI work:

```bash
cargo run -p greg-app
```

The Cargo workspace contains the domain crates (`greg-core`, `greg-steam`,
`greg-modstore`, `greg-loader`, `greg-platform`), the desktop shell
(`greg-app`), the headless CLI (`greg-cli`), and the build orchestration crate
(`xtask`).

## Repository map

| Path | Responsibility |
| --- | --- |
| `crates/greg-app/` | Desktop entry point, Slint UI, actions/state/worker |
| `crates/greg-core/` | Models, pure logic, docs/changelog, errors (no UI/Steam deps) |
| `crates/greg-steam/` | Steam Workshop backend, rate limiter, poller |
| `crates/greg-modstore/` | Modstore REST client, publish/install clients |
| `crates/greg-loader/` | Game discovery, installs, sync, launch |
| `crates/greg-platform/` | Filesystem/log/telemetry/workspace/process/platform shims |
| `crates/greg-cli/` | Headless CLI (`publish`, `status`, `list`) |
| `xtask/` | Build orchestration (`cargo xtask …`) |
| `scripts/` | Local developer workflow scripts |
| `build/scripts/` | Release scripts, installers, Linux packaging |
| `docker/` | Rust build/test containers |
| `docs/` | Canonical repository documentation |

Shared behaviour lives in the domain crates rather than in UI-specific service
logic.

## Build and package

Read [the build-script reference](../build/scripts/README.md) before creating a
release.

```bash
# Release build (host platform) + portable package
cargo xtask dist

# Cross-build the Windows x64 portable package from Linux (mingw)
cargo xtask dist --target x86_64-pc-windows-gnu

# Linux distro packages
./build/scripts/linux/build-rust-packages.sh <publish-dir> <version> <is_pre> <out-dir>
```

The Windows release build produces the portable ZIP via `cargo xtask dist`;
the Setup EXE is produced best-effort via Inno Setup under Wine (CI parity).
Linux distro packages are built via `build/scripts/linux/build-rust-packages.sh`.

macOS is an experimental release target. Release builds require a macOS host
with the Apple SDK; cross-publish can prepare an x64 artifact, but GUI validation
must occur on a real Mac.

## Continuous integration and release trigger

Quality gates run in `.forgejo/workflows/rust.yml` (fmt, clippy, test, audit,
cross-build) and release automation in `.forgejo/workflows/build-and-release.yml`.
Both Forgejo and Gitea copies must stay byte-identical (parity job). Release
artifacts are promoted from `dev` → `staging` via the release-train workflow and
from `staging` → `main` via the train workflow; see `TOOLS.md`.

A GitHub Release is created only for a tag whose name starts with `v`. Treat
artifacts from an untagged run as validation output, not as a promoted release.

## Signing Windows artifacts

The default signing mode creates a fresh, short-lived self-signed certificate
for that build. It makes the certificate subject traceable, but does not provide
public trust or suppress SmartScreen warnings. Use `-SigningMode pfx` with a
securely supplied `CODE_SIGN_PFX` or `CODE_SIGN_THUMBPRINT` for an existing
certificate. The exact commands and security constraints are in the
[code-signing guide](../build/installer/CODE_SIGNING.md).

## Tests and review

The current suite covers workspace logic, game-adapter behaviour, content checks,
and the headless CLI. It does not replace manual Steam, game, or desktop UI testing.

Before a pull request:

1. Run `./scripts/check.sh`.
2. Run `git diff --check`.
3. Do not commit generated packages, secrets, certificates, Steam credentials,
   or machine-specific game paths.
4. Update `CHANGELOG.md` for user-visible changes.
5. Update the document whose command, platform support, artifact, or user
   behaviour changed.

## Documentation maintenance

`docs/` is tracked directly in this repository. Do not edit a `wiki/` directory:
it does not exist in the checkout. Documentation is English by default; explain
prerequisites, expected results, limitations, and unsafe cases next to commands.

Use the current source and scripts as evidence. If a feature is intended but not
implemented, label it **planned** and link to the relevant issue or roadmap.
