# gregModmanager

> Cross-platform desktop Mod Manager for the gregFramework ecosystem — install, manage, and publish mods for MelonLoader-based games.

[![Discord](https://img.shields.io/badge/Discord-Join-5865F2?style=for-the-badge&logo=discord&logoColor=white)](https://discord.gg/greg)
[![gregFramework](https://img.shields.io/badge/gregFramework-Website-blue?style=for-the-badge)](https://gregframework.eu)
[![License](https://img.shields.io/badge/License-MIT-green?style=for-the-badge)](./LICENSE)
[![Version](https://img.shields.io/badge/Version-1.6.0-orange?style=for-the-badge)]()
[![.NET](https://img.shields.io/badge/.NET-9.0-512BD4?style=for-the-badge&logo=dotnet&logoColor=white)]()
[![Avalonia](https://img.shields.io/badge/Avalonia-11.2-8B44AC?style=for-the-badge)]()

![Build & Release](https://img.shields.io/github/actions/workflow/status/mleem97/gregModmanager/build-and-release.yml?style=for-the-badge&label=Build%20%26%20Release)
![Linux Packages](https://img.shields.io/github/actions/workflow/status/mleem97/gregModmanager/linux-packages.yml?style=for-the-badge&label=Linux%20Packages)

## Links

- **Repository:** [github.com/mleem97/gregModmanager](https://github.com/mleem97/gregModmanager)
- **Discord / Support:** [discord.gg/greg](https://discord.gg/greg)
- **Website:** [gregframework.eu](https://gregframework.eu) · [datacentermods.com](https://datacentermods.com)

## Overview

**gregModmanager** is a modern, user-friendly desktop application that simplifies installing, managing, and publishing mods for MelonLoader-based games. Built with Avalonia UI for true cross-platform support, it integrates with Steam and provides an intuitive interface for mod management.

> [!NOTE]
> *gregModmanager* is an independent community project developed by the author. The project name and any related domains (e.g. `gregmodmanager.eu`) are owned by the author. Official hubs of the ecosystem are [gregframework.eu](https://gregframework.eu) and [datacentermods.com](https://datacentermods.com).

## Compatibility

| Platform | Status |
|----------|--------|
| Windows x64 | Supported |
| Linux x64 | Supported |
| macOS | Planned (unconfirmed) |

## Features

- Browse and install mods from [datacentermods.com](https://datacentermods.com) or local files
- Manage load order and resolve dependencies automatically
- Handle profiles for different gameplay configurations
- Update mods with one click
- Publish your own mods and plugins via Steam Workshop
- Steam integration — auto-detect library, launch via Steam, Workshop support
- Smart Installer — automatically downloads and installs required VC++ runtimes
- Health monitoring — built-in crash reporting and performance telemetry
- Localization — English, German, Spanish (community translations welcome)

## Installation

### For Users

**Windows (Recommended):** Download the **Smart Online Installer** from [Releases](https://github.com/mleem97/gregModmanager/releases). It automatically installs required VC++ Runtimes.

**Windows (Portable):** Download the `.zip` archive. Ensure VC++ 2015-2022 Redist is installed.

**Linux:**
```bash
wget https://github.com/mleem97/gregModmanager/releases/download/v1.6.0/gregModmanager-v1.6.0-Linux.tar.gz
tar -xzf *.tar.gz
./GregModmanager
```

## Dependencies

### Runtime

- **.NET 9.0** runtime (self-contained builds available)
- **Visual C++ Redistributable** (Windows, auto-installed by Smart Installer)

### NuGet packages

- Avalonia UI 11.2, Microsoft.Extensions.DependencyInjection, Facepunch.Steamworks

### Build only

- .NET 9 SDK
- PowerShell 5.1+ (Windows) or bash (Linux)
- Inno Setup 6 (Windows installer, optional)
- nfpm (Linux packages, optional)

## Build from Source

Requirements:

- .NET 9 SDK
- Git

```bash
git clone https://github.com/mleem97/gregModmanager.git
cd gregModmanager
dotnet restore GregModmanager.sln
dotnet build GregModmanager.sln -c Release
```

Run locally:

```bash
dotnet run --project src/GregModmanager.Avalonia
```

Or use the convenience scripts:

```bash
# Windows
.\build\builder.ps1

# Linux
./build/builder.sh
```

Full build (installer + packages):

```bash
# Windows
.\build\scripts\build.ps1

# Linux packages
./build/scripts/linux/build-avalonia-packages.sh
```

## Repository Layout

```
gregModmanager/
├── src/
│   ├── GregModmanager.Avalonia/    # Avalonia UI executable, Views, DI setup
│   ├── GregModmanager.Core/        # Shared services, models, Steam integration
│   └── SubDirectoryFixer/          # MelonLoader plugin (net6.0)
├── tests/
│   └── GregModmanager.Tests/       # Unit tests (xUnit)
├── build/
│   ├── scripts/                    # Build and deployment scripts
│   ├── installer/                  # Inno Setup script and signing tools
│   ├── builder.ps1                 # Interactive CLI builder (Windows)
│   └── builder.sh                  # Interactive CLI builder (Linux)
├── docs/                           # User and developer documentation
├── wiki/                           # Documentation wiki (Git submodule)
├── .github/
│   ├── workflows/                  # CI/CD pipelines
│   └── ISSUE_TEMPLATE/             # Issue templates
├── GregModmanager.sln              # Solution file
├── AGENTS.md                       # Agent-oriented instructions
├── CHANGELOG.md                    # Keep a Changelog format
├── DESIGN.md                       # Terminal Core design system
├── EXTERNAL_DEPENDENCIES.md        # License and dependency transparency
└── SPONSORS.md                     # Sponsorship tiers
```

## Documentation

- **End-User Guide:** [docs/01_END_USER_GUIDE.md](docs/01_END_USER_GUIDE.md)
- **Mod Creator Guide:** [docs/02_MOD_CREATOR_GUIDE.md](docs/02_MOD_CREATOR_GUIDE.md)
- **Contributor Guide:** [docs/03_CONTRIBUTOR_GUIDE.md](docs/03_CONTRIBUTOR_GUIDE.md)
- **Full index:** [docs/INDEX.md](docs/INDEX.md)
- **Design System:** [DESIGN.md](DESIGN.md)

## Architecture

- **Framework:** Avalonia UI 11.2 (cross-platform)
- **Runtime:** .NET 9.0 (self-contained, trimmed)
- **Pattern:** Layered architecture with clean Core/UI separation
- **Integration:** Steam API, gregCore/gregFramework, datacentermods.com

## Contributing

See [Contributor Guide](docs/03_CONTRIBUTOR_GUIDE.md) for guidelines.

**Code of Conduct:** Be respectful. All contributors are valued.

## Privacy and Telemetry

gregModmanager includes an optional telemetry system to help improve the application and fix startup issues.

- **What we collect:** Anonymized crash reports, startup success/failure, and sync performance metrics.
- **What we DON'T collect:** Your name, mod files, or any personal data.
- **Opt-out:** Disable telemetry at any time in the Settings menu.
- **Transparency:** All telemetry requests use a secure connection to our self-hosted Loki instance.

## Credits

| Role | Contributor |
|------|-------------|
| **Codebase** | [mleem97](https://github.com/mleem97) ([teamGreg Modding](https://github.com/teamGregModding)) |

## License

This project is licensed under the **MIT License**. See [`LICENSE`](./LICENSE).

## Support

- **Issues:** [GitHub Issues](https://github.com/mleem97/gregModmanager/issues)
- **Discussions:** [GitHub Discussions](https://github.com/mleem97/gregModmanager/discussions)
- **Discord:** [discord.gg/greg](https://discord.gg/greg)

## Related Repositories

- **[gregCore](https://github.com/mleem97/gregCore)** — Game framework
- **[gregStore](https://github.com/mleem97/gregStore)** — Asset distribution
- **[gregBot](https://github.com/mleem97/gregBot)** — Discord bot integration
- **[Wiki](https://github.com/mleem97/gregModmanager.wiki)** — Documentation (submodule)

## Join the gregFramework Team

Building the ultimate modding framework for Data Center is a massive undertaking. gregFramework is currently maintained by a passionate core team, and we are looking for fellow creators to help us scale this mission.

**Your place in the team:** We won't throw you into the deep end. Depending on your individual strengths and skills, we will match you with the right areas of the project so you can contribute exactly where you have the most fun.

**Language Requirement:** A solid grasp of written English is required. Being comfortable speaking English in voice chats is a huge plus, but we completely respect those who prefer to stick to text.

**We are looking for motivated volunteers across several roles:**

- Code Wizards (C#, Rust, Lua, TS, GO) — Build and expand the core framework and mod packages
- Asset Creators (3D Models, hardware assets) — Bring the framework to life visually
- Technical Writers — Craft wiki entries, maintain documentation, and write user guides
- Alpha Testers — Hunt down bugs, stress-test the framework, and provide critical feedback
- System Guardians — Maintain our Linux servers, Docker containers, and infrastructure
- Community Managers — Foster our Discord community, gather feedback, and keep the energy high

Interested? Send us an email at **apply@gregframework.eu**, shoot a quick DM, or drop a message on [Discord](https://discord.gg/greg).

---

**gregFramework — powered by the community.**
