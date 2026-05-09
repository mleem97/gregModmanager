# gregModmanager

![Version](https://img.shields.io/badge/version-1.6.0-blue?style=for-the-badge)
![.NET](https://img.shields.io/badge/.NET-9.0-512BD4?style=for-the-badge&logo=dotnet)
![Avalonia](https://img.shields.io/badge/Avalonia-11.2-8B44AC?style=for-the-badge)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux-lightgrey?style=for-the-badge)
![License](https://img.shields.io/badge/license-MIT-green?style=for-the-badge)
![Steam](https://img.shields.io/badge/Steam-Workshop-1b2838?style=for-the-badge&logo=steam)
<br>
![Build & Release](https://img.shields.io/github/actions/workflow/status/mleem97/gregModmanager/build-and-release.yml?style=for-the-badge&label=Build%20%26%20Release)
![Linux Packages](https://img.shields.io/github/actions/workflow/status/mleem97/gregModmanager/linux-packages.yml?style=for-the-badge&label=Linux%20Packages)

**Current Version:** `v1.6.0`

Cross-platform desktop Mod Manager for the gregFramework ecosystem.

> **Note:** *gregModmanager* is an independent community project developed by the author. The project name and any related domains (e.g. `gregmodmanager.eu`) are **not** owned by the author. Official hubs of the ecosystem are [gregframework.eu](https://gregframework.eu) and [datacentermods.com](https://datacentermods.com).

## What is gregModmanager?

**gregModmanager** is a modern, user-friendly application that simplifies installing, managing, and publishing mods for MelonLoader-based games. It supports Windows and Linux (macOS coming soon), integrates with Steam, and provides an intuitive interface for:

- **Browse & Install** mods from [datacentermods.com](https://datacentermods.com) or local files
- **Manage Load Order** and resolve dependencies automatically
- **Handle Profiles** for different gameplay configurations
- **Update Mods** with one click
- **Publish Your Own** mods and plugins

## 📚 Documentation

👉 **New users?** Start with the [End-User Guide](docs/01_END_USER_GUIDE.md)  
📦 **Creating mods?** See the [Mod Creator Guide](docs/02_MOD_CREATOR_GUIDE.md)  
🔧 **Want to contribute?** Read the [Contributor Guide](docs/03_CONTRIBUTOR_GUIDE.md)  
📖 **Full index:** [docs/INDEX.md](docs/INDEX.md)

## 🚀 Quick Start

### For Users

**Download and run:**
- **Windows (Recommended):** Download the **Smart Online Installer** from [Releases](https://github.com/mleem97/gregModmanager/releases). It automatically installs required VC++ Runtimes.
- **Windows (Portable):** Download the `.zip` archive. Ensure VC++ 2015-2022 Redist is installed.
- **Linux:** `wget https://github.com/mleem97/gregModmanager/releases/download/v1.6.0/gregModmanager-v1.6.0-Linux.tar.gz && tar -xzf *.tar.gz && ./GregModmanager`

### For Developers

**Build locally:**

```powershell
# Clone repository
git clone https://github.com/mleem97/gregModmanager.git
cd gregModmanager

# Install dependencies
dotnet restore GregModmanager.sln

# Build
dotnet build GregModmanager.sln -c Release

# Run
dotnet run --project GregModmanager.Avalonia
```

**Or use the convenience script:**
```powershell
.\scripts\run.ps1
```

## 📋 System Requirements

| Platform | Minimum | Recommended |
|----------|---------|-------------|
| **Windows** | Windows 10 v1909, .NET 9 | Windows 11, 4+ GB RAM |
| **Linux** | Debian 11+, Ubuntu 20.04 LTS+ | Fedora 36+, 4+ GB RAM |
| **macOS** | Planned Q3 2026 | — |

See [System Requirements](docs/01_END_USER_GUIDE.md#system-requirements) for details.

## 🏗️ Project Structure

```
gregModmanager.sln
├── GregModmanager.Core/              # Core business logic
├── GregModmanager.Avalonia/          # Avalonia UI (Windows, Linux)
├── SubDirectoryFixer/                # Helper utility
├── GregModmanager.Tests/             # Unit tests
├── scripts/                          # Build and deployment scripts
├── installer/                        # Inno Setup installer script
├── docs/                             # User and developer documentation
└── wiki/                             # Documentation wiki (Git submodule)
```

## ✨ Features

✅ **Mod Management** — Install, enable, disable, update, manage profiles  
✅ **Dependency Resolution** — Automatic conflict detection and installation  
✅ **Steam Integration** — Auto-detect library, launch via Steam, Workshop support  
✅ **Smart Installer** — Automatically downloads and installs required VC++ runtimes  
✅ **Health Monitoring** — Built-in crash reporting and performance telemetry  
✅ **Cross-Platform** — Windows, Linux, macOS (coming soon)  
✅ **Localization** — English, German, Spanish (community translations welcome)

## 🏗️ Architecture

- **Framework:** Avalonia UI 11.2 (cross-platform)
- **Runtime:** .NET 9.0
- **Pattern:** Layered architecture with clean Core/UI separation
- **Integration:** Steam API, gregCore/gregFramework, datacentermods.com

## 🔨 Build & Release

### Full Build (Windows + Linux)

```powershell
.\scripts\build.ps1
```

**Produces:**
- Windows installer (`.exe`)
- Windows portable (`.zip`)
- Linux AppImage, `.deb`, `.rpm`, `.tar.gz`

### Quick Build

```powershell
.\scripts\run.ps1              # Build & run locally
.\scripts\builder.ps1          # Interactive CLI menu
```

## 📦 External Integrations

- **[datacentermods.com](https://datacentermods.com)** — Central mod repository and modding community
- **[gregframework.eu](https://gregframework.eu)** — gregCore / gregFramework ecosystem documentation
- **[melonwiki.xyz](https://melonwiki.xyz)** — MelonLoader documentation
- **Steam API** — Game library detection and workshop

## 🤝 Contributing

We welcome contributions! See [CONTRIBUTING.md](docs/03_CONTRIBUTOR_GUIDE.md) for:

- Development environment setup
- Coding standards and conventions
- Testing requirements
- Pull request workflow
- Localization guidelines

**Code of Conduct:** Be respectful. All contributors are valued.

## 🔒 Privacy & Telemetry

gregModmanager includes an optional telemetry system to help us improve the application and fix startup issues.

- **What we collect:** Anonymized crash reports, startup success/failure, and sync performance metrics.
- **What we DON'T collect:** Your name, mod files, or any personal data.
- **Opt-out:** You can disable telemetry at any time in the **Settings** menu.
- **Transparency:** All telemetry requests use a secure connection to our self-hosted Loki instance at `telemetry.datacentermods.com`.

## 📄 License

MIT License — See [LICENSE](LICENSE)

## 🎯 Support

- **Documentation:** [docs/](docs/) and [wiki/](wiki/)
- **Issues:** [GitHub Issues](https://github.com/mleem97/gregModmanager/issues)
- **Discussions:** [GitHub Discussions](https://github.com/mleem97/gregModmanager/discussions)
- **Website:** [gregframework.eu](https://gregframework.eu) · [datacentermods.com](https://datacentermods.com)

## Related Repositories

- **[gregCore](https://github.com/mleem97/gregcore)** — Game framework
- **[gregStore](https://github.com/mleem97/gregstore)** — Asset distribution
- **[gregBot](https://github.com/mleem97/gregbot)** — Discord bot integration
- **[Wiki](https://github.com/mleem97/gregModmanager.wiki)** — Documentation (submodule)

## Maintainers
- teamGreg / mleem97
