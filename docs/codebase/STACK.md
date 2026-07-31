# Technology Stack

> Owner: gregModmanager maintainers. Evidence: project files and `global.json`.
> Update after a dependency, target framework, SDK, or build-tool change.

## Runtime Summary

| Area | Value | Evidence |
|---|---|---|
| Language | C# | `src/GregModmanager.Core/` |
| Runtime | .NET 9 (`net9.0`) | `src/GregModmanager.Core/GregModmanager.Core.csproj` |
| UI | Avalonia 12.1.1 | `src/GregModmanager.Avalonia/GregModmanager.Avalonia.csproj` |
| Build | .NET SDK/MSBuild | `GregModmanager.sln`, `global.json` |

## Production Dependencies

| Dependency | Version | Role | Evidence |
|---|---:|---|---|
| Avalonia | 12.1.1 | Cross-platform desktop UI | Avalonia project file |
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

`AppSettings.cs` and `JsonFilePreferences` provide local settings. Native Steam
libraries and `steam_appid.txt` are copied beside published output.
Trimming/single-file publishing is enabled in the Avalonia project.

### Environment variables

| Variable | Purpose | Secret? |
| --- | --- | --- |
| `DATA_CENTER_GAME_DIR` | override automatic game-root discovery | no |
| `IS_LOCAL_BUILD`, `IS_LOCAL_TEST_BUILD` | select local development endpoints | no |
| `MODSTORE_WEB_URL`, `MODSTORE_API_URL` | override web/API base URLs | no, but use trusted HTTPS endpoints |
| `MELONLOADER_RELEASES_URL`, `MELONLOADER_LATEST_API_URL` | override MelonLoader release lookup | no |
| `AUTH_LOGIN_URL_FORMAT`, `AUTH_CALLBACK_REDIRECT_URI` | override desktop authentication flow | no, but security-sensitive |
| `TELEMETRY_URL`, `GIT_SERVER_URL` | override telemetry/Git endpoints | telemetry credentials are secret; URLs are not |
| `GREGMODMANAGER_SOURCES`, `STEAM_LIBRARY_PATHS` | add plugin sources or Steam libraries | no |
| `WORKSHOP_UPLOADER_DEBUG_RUN` | enable uploader diagnostic behaviour | no; development only |

Never commit endpoint credentials, session tokens, certificate passwords, or
private keys. Validate any overridden endpoint before distributing a build.

## Evidence

- `src/GregModmanager.Avalonia/GregModmanager.Avalonia.csproj`
- `src/GregModmanager.Core/GregModmanager.Core.csproj`
- `.github/workflows/build-and-release.yml`
