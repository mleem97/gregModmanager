# Technology Stack

## Runtime Summary

| Area | Value | Evidence |
|---|---|---|
| Language | C# | `src/GregModmanager.Core/` |
| Runtime | .NET 9 (`net9.0`) | `src/GregModmanager.Core/GregModmanager.Core.csproj` |
| UI | Avalonia 12.1.0 | `src/GregModmanager.Avalonia/GregModmanager.Avalonia.csproj` |
| Build | .NET SDK/MSBuild | `GregModmanager.sln`, `global.json` |

## Production Dependencies

| Dependency | Version | Role | Evidence |
|---|---:|---|---|
| Avalonia | 12.1.0 | Cross-platform desktop UI | Avalonia project file |
| Cherry.Facepunch.Steamworks | 2.5.0 | Steam/Workshop integration | Core project file |
| Microsoft.Extensions.DependencyInjection | 10.0.10 | Service composition | Avalonia project file |

## Development Toolchain

- xUnit 2.9.3 and Microsoft.NET.Test.Sdk 18.8.1: `tests/...csproj`.
- Coverlet collector 10.0.1: `tests/...csproj`.
- Codacy configuration: `.codacy/`.

## Commands

```bash
dotnet restore GregModmanager.sln
dotnet build src/GregModmanager.Avalonia/GregModmanager.Avalonia.csproj -c Release
DOTNET_ROLL_FORWARD=Major dotnet test tests/GregModmanager.Tests/GregModmanager.Tests.csproj -c Release
git diff --check
```

## Configuration

`AppSettings.cs` and `JsonFilePreferences` provide local settings; optional environment overrides include `DATA_CENTER_GAME_DIR`, `MODSTORE_*`, auth and telemetry URLs. Native Steam libraries and `steam_appid.txt` are copied beside published output. Trimming/single-file publishing is enabled in the Avalonia project.

## Evidence

- `src/GregModmanager.Avalonia/GregModmanager.Avalonia.csproj`
- `src/GregModmanager.Core/GregModmanager.Core.csproj`
- `.github/workflows/build-and-release.yml`
