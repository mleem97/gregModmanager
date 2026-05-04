# gregModmanager

Unified repository profile for the gregFramework workspace.

## Overview
`gregModmanager` is the desktop ModManager application for browsing, installing, and publishing mods.
It is a management client around the framework ecosystem.

## Architecture Layer
- Layer: MAUI ModManager
- Role: User-facing management and distribution tooling

## Status
- Lifecycle: Active
- Runtime/Core dependency: Integrates with Core ecosystem, does not replace Core SDK

## Workspace
- Local path: `gregFramework/gregModmanager`
- Main project: `GregModmanager.csproj`

## Quick Start

### Scripts und Automation
Alle Build-, Deployment- und Development-Aufgaben sind in **`scripts/`** organisiert:

```powershell
# Schneller Start zum Entwickeln
.\run.ps1

# Build + Installer
.\build.ps1

# Lokale Installation (schnell)
.\install-local.ps1

# Weitere Scripts und Helpers
cd scripts
Get-Content README.md  # Siehe scripts/README.md für vollständige Übersicht
```

**👉 Siehe [`scripts/README.md`](scripts/README.md) für vollständige Dokumentation aller verfügbaren Scripts.**

### Manuelle Kompilierung
```powershell
Set-Location "c:\Users\marvi\Desktop\gregModmanager"
dotnet build "GregModmanager.csproj" -c Debug
```

## CoreOnly Snapshot
Core gameplay/runtime hooks and services are provided by `gregCore`.
This repo provides management UX, packaging, and workflow integration.

## Related Repositories
- `gregCore`
- `gregStore`
- `gregBot`

## Maintainers
- teamGreg / mleem97

## ⚖️ Disclaimer & Credits

- **gregCore** uses code from [DataCenter-RustBridge](https://github.com/Joniii11/DataCenter-RustBridge) for the Rust implementation. **Joniii11** is an active part of **teamGreg**.
- **gregModmanager** utilizes modloading capabilities derived from [DataCenter-ModLoader](https://github.com/ASavageSwan/-DataCenter-ModLoader).
- Special thanks to all contributors and the creators of the libraries used in this project.

## 🚀 Join the Team!

To truly build the ultimate modding framework for *Data Center*, we still have a solid chunk of work ahead of us. That's why I am constantly on the lookout for fellow creators who want to join the team.

No matter what your strengths are, we are looking for people to:
- **Write Code** (C#, Rust, Lua, TS)
- **Create Models** (3D assets, new hardware... or more Steves!)
- **Write Docs** (Wiki, documentation, maintaining guides)
- **Or just play the game** (Testing, finding bugs, and giving feedback)

Everyone is absolutely welcome! If you want to be a part of the project, just shoot me a quick DM on **Discord** or visit [gregframework.eu](https://gregframework.eu).

---
*© 2026 teamGreg | [gregframework.eu](https://gregframework.eu)*



## Contributors
- @mleem97
- @Joniii11

