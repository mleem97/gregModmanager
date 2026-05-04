# gregModmanager

Cross-platform desktop Mod Manager for the gregFramework ecosystem.

## Overview
`gregModmanager` provides workflows for browsing, installing, and publishing mods.
The repository currently contains:
- Existing MAUI application (`GregModmanager`)
- New Avalonia migration foundation (`GregModmanager.Avalonia`)

## Architecture Layer
- Layer: Desktop Mod Manager
- Role: User-facing management and distribution tooling

## Runtime Targets
- Windows desktop (primary)
- Linux desktop (Avalonia packaging path)

## Quick Start

### Build the full solution
```powershell
dotnet build .\GregModmanager.sln -c Release
```

### Run current MAUI desktop app
```powershell
.\scripts\start.ps1
```

### Run Avalonia migration app
```powershell
dotnet run --project .\GregModmanager.Avalonia\GregModmanager.Avalonia.csproj
```

## Scripts
All automation is centralized in `scripts/`.
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
Repository wiki source is maintained under `wiki/`.
Start at `wiki/Home.md`.

## Related Repositories
- `gregCore`
- `gregStore`
- `gregBot`

## Maintainers
- teamGreg / mleem97
