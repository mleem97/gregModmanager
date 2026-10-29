# gregModmanager

> Cross-platform desktop mod manager for the **_gregFramework ecosystem_** — local Workshop projects, Steam Workshop publishing, Mod Store, and one-click game launch for **_Data Center_** _(by Waseku)_.

[![Built With Rust](https://forthebadge.com/images/badges/built-with-rust.svg)](https://www.rust-lang.org)
[![Made With Slint](https://img.shields.io/badge/MADE_WITH-SLINT-%238C7FF0?style=for-the-badge&logo=slate&logoColor=white)](https://slang.dev)
[![Discord Join](https://img.shields.io/badge/DISCORD-JOIN_GREGFRAMEWORK-%235865F2?style=for-the-badge&logo=discord&logoColor=white)](https://discord.gg/greg)
[![Modmanager Website](https://img.shields.io/badge/MODMANAGER-WEBSITE-%23007ACC?style=for-the-badge)](https://modmanager.gregframework.eu)
[![License Apache](https://forthebadge.com/images/badges/license-apache-2.0.svg)](LICENSE)
[![Release Version](https://raw.githubusercontent.com/mleem97/gregModmanager/main/VERSION)](https://github.com/mleem97/gregModmanager/releases)
[![Release Version Staging](https://raw.githubusercontent.com/mleem97/gregModmanager/staging/VERSION)](https://github.com/mleem97/gregModmanager/tree/staging)
[![Release Version Dev](https://raw.githubusercontent.com/mleem97/gregModmanager/dev/VERSION)](https://github.com/mleem97/gregModmanager/tree/dev)
[![GitHub Sponsors](https://img.shields.io/badge/GITHUB-SPONSORS-%23EA4AAA?style=for-the-badge&logo=githubsponsors&logoColor=white)](https://github.com/sponsors/mleem97)
[![Rust Version](https://img.shields.io/badge/RUST-STABLE-%23CE422B?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org)
[![Slint Version](https://img.shields.io/badge/SLINT-V1.18.1-%238C7FF0?style=for-the-badge&logo=slate&logoColor=white)](https://slang.dev)
[![Supported OS Linux & Windows](https://img.shields.io/badge/SUPPORTED_OS-LINUX_%7C_WINDOWS-%230078D4?style=for-the-badge&logo=windows&logoColor=white)](https://github.com/mleem97/gregModmanager/releases)

<!-- Alternative mit macOS (Ausgeklammert für später): -->
<!-- [![Supported OS All](https://img.shields.io/badge/SUPPORTED_OS-LINUX_%7C_WINDOWS_%7C_MACOS(SOON)-%2389B4FA?style=for-the-badge&logo=apple&logoColor=white)](https://github.com/mleem97/gregModmanager/releases) -->

[![Powered by MelonLoader](https://img.shields.io/badge/POWERED_BY-MELONLOADER-%23303030?style=for-the-badge)](https://melonwiki.xyz)
[![Powered by Datacentermods](https://img.shields.io/badge/POWERED_BY-DATACENTERMODS.COM-%231E1E2E?style=for-the-badge)](https://datacentermods.com)

---

## Links

- **Repository:** [github.com/mleem97/gregModmanager](https://github.com/mleem97/gregModmanager) (mirror — primary instance is Forgejo)
- **Discord / Support:** [discord.gg/greg](https://discord.gg/greg)
- **Website:** [gregframework.eu](https://gregframework.eu)
- **Modmanager Website:** [modmanager.gregframework.eu](https://modmanager.gregframework.eu)
- **Releases:** [github.com/mleem97/gregModmanager/releases](https://github.com/mleem97/gregModmanager/releases)

## Overview

**gregModmanager** is a cross-platform desktop client for the gregFramework modding ecosystem. It manages local Workshop projects, publishes them to the Steam Workshop via native Steamworks calls, browses and subscribes Mod Store items, and launches Data Center with or without mods — on Windows and Linux.

## Compatibility

| Platform        | Build status | Distribution status                                              |
| --------------- | ------------ | ---------------------------------------------------------------- |
| Windows x64     | Supported    | Portable ZIP, Inno Setup EXE, WiX MSI                            |
| Linux x64       | Supported    | Portable ZIP/tarball, DEB, RPM, APK, Arch package                |
| macOS x64/arm64 | Experimental | Real-Mac validation required; no signed or notarized release yet |

| Requirement    | Version                                |
| -------------- | -------------------------------------- |
| Rust toolchain | stable via [rustup](https://rustup.rs) |
| Steam client   | running (for Workshop features)        |
| Data Center    | Steam AppID `4170200`                  |

## Features

- **Projects** — local projects tab (search, refresh, edit) + Workshop uploads tab (import, update, open on Steam)
- **Editor** — title, description, tags, visibility, preview image, screenshots, changelog, upload readiness check
- **Steam Workshop publishing** — native `ISteamUGC` calls (create + update), change notes, live progress
- **Mod Store** — browse and subscribe Workshop items (Full / Decide-later modes)
- **Start Game** — launch Data Center without local mods
- **With Mods** — launch Data Center with the installed local mods
- **Headless mode** — scripted/CI publishing without UI
- **Settings** — workspace folder, game root, language, mod-store mode
- **Localization** — English + German (minimum), more via the `greg-core` l10n modules

The desktop UI exposes **Projects**, **New Project**, and **Settings**. In Full and Decide-later modes, it also exposes a Steam Workshop **Mod Store**. Profile management, manual load-order editing, automatic conflict resolution, and an in-app updater are not currently released features.

## Installation

### Windows

1. Download `gregmodmanager-<version>-win-x64.zip` (or the Setup EXE) from the [releases page](https://github.com/mleem97/gregModmanager/releases)
2. Extract the ZIP anywhere (or run the installer) and start `gregmodmanager.exe`
3. Make sure **Steam is running** before starting the app

### Linux

1. Download `gregmodmanager-<version>-linux-x64.tar.gz` (or a DEB/RPM/APK/Arch package) from the [releases page](https://github.com/mleem97/gregModmanager/releases)
2. Extract the tarball anywhere and start `./gregmodmanager` (or install the package)
3. Make sure **Steam is running** before starting the app — `libsteam_api64.so` / `libsteam_api.so` ship next to the binary

## Dependencies

### Runtime

- **Steam client** (running — required for Workshop features only)
- **Data Center** on Steam (AppID `4170200`, for game launch + Workshop target)

### Rust crates (workspace dependencies)

- `slint` / `slint-build` 1.18.1 + `lucide-slint` 1.49.1 — desktop UI (retained mode, single binary)
- `steamworks` 0.13.1 + `steamworks-sys` — Steam Workshop integration (native library staging)
- `tokio`, `reqwest` (rustls) — async runtime / Modstore REST + OAuth
- `serde` / `serde_json`, `thiserror`, `anyhow`, `semver`, `dirs`, `clap`, `rfd`, `notify`, `zip`, `tracing`

### Build only

- Rust toolchain via rustup (stable)
- `cargo xtask dist` — release build + portable package incl. native Steam library
- `nfpm` (Linux distro packages only)

## Build from Source

Requirements:

- Rust stable toolchain via [rustup](https://rustup.rs)
- Linux X11 / font / GL development packages for Slint (`libxkbcommon`, `fontconfig`, `freetype6`, `libgl1`) — see [the devcontainer Dockerfile](.devcontainer/Dockerfile) for the full set
- A running Steam client (only needed for Workshop features, not for build/test)

```bash
git clone <forgejo-url>/gregModmanager.git
cd gregModmanager
cargo build --workspace
cargo test --workspace
cargo run -p greg-app

```

For release artifacts, start with `cargo xtask dist` (portable package incl. native
Steam library). See [the build-script reference](https://www.google.com/search?q=build/scripts/README.md) and
[Docker build and test containers](https://www.google.com/search?q=docker/README.md).

The Windows release build produces the portable ZIP via `cargo xtask dist`; the
Setup EXE is produced best-effort via Inno Setup under Wine (CI parity). Linux
distro packages are built via `build/scripts/linux/build-rust-packages.sh`.

## Repository Layout

```text
crates/
  greg-core/                     domain models, docs/changelog logic, errors — NO UI deps
  greg-platform/                  cfg(target_os) shims (replaces Unix/Windows/MacOs projects)
  greg-steam/                     Steam Workshop via the `steamworks` crate
  greg-modstore/                  Modstore REST client (catalog, updates, atomic installs, publish)
  greg-loader/                    game/MelonLoader discovery, installs, adapters, launch
  greg-app/                       desktop UI binary (Slint)
  greg-cli/                       headless publish/status/list binary
xtask/                           build orchestration (`cargo xtask …`)
scripts/                         local developer workflow scripts (check, build, run, release)
build/                           release scripts, installers, Linux packaging
docker/                          Rust build/test containers
docs/                            repository documentation
.forgejo/workflows/ + .gitea/workflows/   CI pipelines, byte-identical (Forgejo + Gitea Actions; GitHub is a read-only archive mirror)
VERSION                          single source of truth for the version (workspace Cargo.toml is the fallback)
LICENSE                          Apache 2.0

```

`SubDirectoryFixer` targets .NET 6 because MelonLoader requires that runtime. It
stays frozen in `.reference/modmanager-old/` and is built separately from the
Rust workspace.

## Documentation

- [QuickStart guide](QUICKSTART.md)
- [Documentation index](https://www.google.com/search?q=docs/INDEX.md)
- [User guide](https://www.google.com/search?q=docs/01_END_USER_GUIDE.md)
- [Creator guide](https://www.google.com/search?q=docs/02_MOD_CREATOR_GUIDE.md)
- [Contributor guide](https://www.google.com/search?q=docs/03_CONTRIBUTOR_GUIDE.md)
- [Build scripts](https://www.google.com/search?q=build/scripts/README.md)
- [Docker build and test containers](https://www.google.com/search?q=docker/README.md)
- [Code-signing behaviour](https://www.google.com/search?q=build/installer/CODE_SIGNING.md)
- [Dependency and licence inventory](https://www.google.com/search?q=EXTERNAL_DEPENDENCIES.md)
- [Changelog](CHANGELOG.md)

## Credits

| Role         | Contributor                                                                                    |
| ------------ | ---------------------------------------------------------------------------------------------- |
| **Codebase** | [mleem97](https://github.com/mleem97) ([TeamGreg Modding](https://github.com/teamGregModding)) |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is licensed under the **Apache License 2.0**. See [`LICENSE`](https://www.google.com/search?q=LICENSE).

## 🚀 Join the gregFramework Team

Building the ultimate modding framework for Data Center is a massive undertaking. gregFramework is currently maintained by a passionate core team of three, and we are looking for fellow creators to help us scale this mission!

**Your place in the team:** We won't throw you into the deep end. Depending on your individual strengths and skills, we will match you with the right areas of the project so you can contribute exactly where you have the most fun.

**🌍 Language Requirement:** A solid grasp of written English is required (without relying on machine translation). Being comfortable speaking English in voice chats is a huge plus, but we completely respect those who prefer to stick to text!

**We are looking for motivated volunteers to join our crew across several roles:**

- 💻 **Code Wizards** (Rust, Lua, TS, GO) — Build and expand the core framework and mod packages
- 🎨 **Asset Creators** (3D Models, hardware assets) — Bring the framework to life visually
- 📚 **Technical Writers** — Craft wiki entries, maintain documentation, and write user guides
- 🎮 **Alpha Testers** — Hunt down bugs, stress-test the framework, and provide critical feedback
- ⚙️ **System Guardians** — Maintain our Linux servers, Docker containers, and infrastructure
- 🤝 **Community Managers** — Foster our Discord community, gather feedback, and keep the energy high

Interested in joining the project? Everyone is absolutely welcome! Send us an email at **[apply@gregframework.eu](https://www.google.com/search?q=mailto%3Aapply%40gregframework.eu)**, shoot a quick DM, or drop a message on [Discord](https://discord.gg/greg).

---

**gregFramework — powered by the community.**

