# External Dependencies — gregModmanager

> **Note:** This document reflects the current repository structure (Avalonia UI, .NET 9, `src/`/`tests/`/`build/` layout). For additional context, see `README.md` and the project Wiki.

This document is an **open-source transparency** note: what this application ships or relies on, and how that relates to licenses. It is **not legal advice**.

---

## Open source at a glance

| | |
|--|--|
| **This application's source code** | Licensed under the same terms as the containing repository (see root `LICENSE` if present). |
| **Open-source components we ship or depend on** | **.NET**, **Avalonia UI**, **Microsoft.Extensions.DependencyInjection**, **Facepunch.Steamworks** — typically **MIT** or permissive open-source licenses; see tables below and upstream notices. |
| **Not open source (binary redistributables)** | **steam_api64.dll** from Valve's **Steamworks SDK** — distributed under Valve's terms, not under a public source license. You must comply with [Steamworks](https://partner.steamgames.com/) agreements when building or redistributing the app. |
| **Runtime on the user's PC** | **Visual C++ Redistributable** and **.NET** runtime components may be installed separately; those are governed by their respective Microsoft (or other) licenses. |

For release outputs (portable ZIP, Setup EXE, Linux tarball): the publish output includes managed assemblies from NuGet (see `src/GregModmanager.Avalonia/GregModmanager.Avalonia.csproj` and `src/GregModmanager.Core/GregModmanager.Core.csproj`) plus native **steam_api64.dll** and **steam_appid.txt** as described below.

---

## Runtime and framework (open source)

| Component | Use | License / terms |
|-----------|-----|-------------------|
| **.NET 9** | Runtime and SDK | [MIT](https://github.com/dotnet/runtime/blob/main/LICENSE.TXT) |
| **Avalonia UI 11.2** | Cross-platform UI framework | [MIT](https://github.com/AvaloniaUI/Avalonia/blob/master/licence.md) |
| **Microsoft.Extensions.DependencyInjection** | Service registration / DI container | [MIT](https://www.nuget.org/packages/Microsoft.Extensions.DependencyInjection) |

## NuGet packages (open source)

| Package | Use | License |
|---------|-----|---------|
| **Avalonia** | Core Avalonia framework | MIT |
| **Avalonia.Desktop** | Desktop backend (Windows/Linux/macOS) | MIT |
| **Avalonia.Themes.Fluent** | Built-in Fluent theme base | MIT |
| **Avalonia.Fonts.Inter** | Inter font integration | MIT |
| **Microsoft.Extensions.DependencyInjection** | DI container used in `Program.cs` | MIT |
| **Facepunch.Steamworks** | Steamworks API wrapper (managed) | **MIT** ([repository](https://github.com/Facepunch/Facepunch.Steamworks)) |
| **xunit** / **xunit.runner.visualstudio** / **coverlet.collector** | Unit testing (tests project only) | MIT / Apache-2.0 |
| **Microsoft.NET.Test.Sdk** | Test runner SDK (tests project only) | MIT |

## Binary redistributables (not "open source")

| File | Source | Notes |
|------|--------|-------|
| **steam_api64.dll** | [Steamworks SDK](https://partner.steamgames.com/doc/sdk) | Valve **redistributable** for Steam-enabled applications. You do not have source code; distribution is governed by **Steamworks SDK** and Steam partner terms. Do not imply Valve endorses this project. |
| **steam_appid.txt** | Project / game configuration | Text file containing the Steam AppID (`4170200`); not a library. |

Using Steamworks implies acceptance of Valve's applicable agreements for developers and players.

## Fonts (vendored)

The following fonts are embedded as `AvaloniaResource` or `EmbeddedResource` and are used under their respective open-source licenses:

| Font | License | Usage |
|------|---------|-------|
| **Inter** | [OFL 1.1](https://github.com/rsms/inter/blob/master/LICENSE.txt) | Primary UI font (body, labels, buttons) |
| **Inter Tight** | [OFL 1.1](https://github.com/rsms/inter/blob/master/LICENSE.txt) | Condensed UI variants |
| **JetBrains Mono** | [OFL 1.1](https://github.com/JetBrains/JetBrainsMono/blob/master/OFL.txt) | Monospace metadata, status values, paths, IDs |
| **Space Grotesk** | [OFL 1.1](https://github.com/floriankarsten/space-grotesk/blob/master/OFL.txt) | Display and headline typography |
| **Nexa** | Proprietary / see vendor license | Branding elements (if included) |

## Windows components

| Component | Notes |
|-----------|-------|
| **Visual C++ runtime** | May be required on some PCs; [Microsoft VC++ Redistributable](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist) (see Microsoft license terms). |
| **Inno Setup** (build-time) | Used to create the Windows Setup EXE. [Inno Setup License](https://jrsoftware.org/isfaq.php?license) (free for commercial and non-commercial use). |

## Build tooling

| Tool | Purpose | License |
|------|---------|---------|
| **PowerShell 5.1+** | Build orchestration (`build/scripts/build.ps1`) | MIT |
| **Inno Setup 6** | Windows installer creation (`build/installer/gregModmanager.iss`) | Inno Setup License |
| **nfpm** | Linux package building (DEB/RPM/Arch) | [Apache-2.0](https://github.com/goreleaser/nfpm/blob/main/LICENSE) |
| **tar** | Linux tarball creation | GPL (typically provided by the host OS) |
| **WSL** (optional) | Linux package builds from Windows hosts | Microsoft Terms |
| **Docker** (optional) | GHCR container image builds | Apache-2.0 |

## CI / CD (GitHub Actions)

The workflow files live under `.github/workflows/`:

- **`build-and-release.yml`** — Main CI workflow: test, build Windows (Setup + Portable ZIP), build Linux (tarball), build Linux packages (DEB/RPM/Arch), create GitHub Release.
- **`linux-packages.yml`** — Standalone Linux package workflow triggered on `workflow_dispatch` or pushes affecting Linux packaging files.

Version resolution uses `src/GregModmanager.Avalonia/GregModmanager.Avalonia.csproj` as the single source of truth.

## GitHub Releases and distribution formats

### Windows

- **Portable ZIP** (`win64-v{version}-portable.zip`) — Self-contained `win-x64` publish output. Best compression (`Compress-Archive -CompressionLevel Optimal`).
- **Setup EXE** (`gregModmanager-{version}-Setup.exe`) — Inno Setup installer with auto-uninstall of previous versions.

### Linux

- **Tarball** (`gregmodmanager-{version}-linux-x64.tar.gz`) — Self-contained `linux-x64` single-file publish.
- **Packages** (optional, via `nfpm`):
  - `.deb` (Debian/Ubuntu)
  - `.rpm` (Fedora/openSUSE)
  - `.pkg.tar.zst` (Arch Linux)

### Container (optional)

- **GHCR image** (`ghcr.io/{owner}/gregmodmanager:{version}`) — Built from `mcr.microsoft.com/dotnet/runtime-deps:9.0` base image.

## SubDirectoryFixer (MelonLoader plugin)

The `src/SubDirectoryFixer/` project is a **.NET 6.0** MelonLoader plugin distributed alongside the main application. It is built separately and its output (`SubDirectoryFixer.dll`) is copied to `src/GregModmanager.Avalonia/Assets/SubDirectoryFixer/` during the build process.

- **Runtime target**: `net6.0` (Unity IL2CPP + MelonLoader compatibility requirement).
- **External reference**: `MelonLoader.dll` (from the game's `MelonLoader` folder, marked `<Private>false</Private>`).

## Trademarks

**Steam** is a trademark of Valve Corporation. **Data Center** and related assets are the property of their respective rights holders. This tool is a community modding utility and is not affiliated with or endorsed by Valve or the game publisher unless explicitly stated elsewhere.

**Avalonia UI** is a trademark of the Avalonia UI project. **.NET** is a trademark of Microsoft Corporation.

---

*Last updated: 2026-05-05*
