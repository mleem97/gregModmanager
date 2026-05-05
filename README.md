# gregModmanager

Current Version: `v1.5.0`

Cross-platform desktop Mod Manager for the gregFramework ecosystem.

## Overview
`gregModmanager` provides workflows for browsing, installing, and publishing mods.
Built with Avalonia UI 11.2 on .NET 9.

## Architecture Layer
- Layer: Desktop Mod Manager
- Role: User-facing management and distribution tooling

## Runtime Targets
- Windows desktop (primary)
- Linux desktop (Avalonia packaging path)

## Quick Start

### Interactive Builder (recommended)
```powershell
.\scripts\builder.ps1     # Windows
./scripts/builder.sh       # Linux / macOS
```

### Build the full solution
```powershell
dotnet build .\GregModmanager.sln -c Release
```

### Run Avalonia app
```powershell
dotnet run --project .\GregModmanager.Avalonia\GregModmanager.Avalonia.csproj
```

## Scripts
All automation is centralized in `scripts/`.

| Script | Purpose |
|--------|---------|
| `builder.ps1` / `builder.sh` | Interactive CLI menu for build, test, run, clean |
| `build.ps1` | Full CI-mirror build (Windows Setup + Portable, Linux tarball, packages) |
| `run.ps1` | Quick run in Release configuration |
| `install-local.ps1` | Local install to `%LOCALAPPDATA%` without Setup EXE |

See `scripts/README.md` for details.

## Linux Packaging
Use the Avalonia packaging scripts:
```powershell
.\scripts\linux\build-avalonia-packages.ps1
```

Artifacts include:
- `.deb` (Debian/Ubuntu)
- `.rpm` (Fedora/RHEL)
- `.pkg.tar.zst` (Arch)
- `.tar.gz` fallback

Optional Flatpak manifest is available in `scripts/linux/flatpak/`.

## Steam Safety
Upload flows enforce Steam-safe publish cooldown and retry feedback.
See `wiki/Steam-Limits-and-Cooldown.md`.

## Wiki
The project wiki is maintained as a **Git submodule** under `wiki/`.
```bash
git submodule update --init --recursive
```
Start at `wiki/Home.md`.

> **Note:** Do not commit wiki contents into the main repository.

## Related Repositories
- `gregCore`
- `gregStore`
- `gregBot`

## Maintainers
- teamGreg / mleem97
