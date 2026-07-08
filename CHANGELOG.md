# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- **CI Build Error**: Replaced MAUI `Preferences.Default` and Avalonia `SettingsPage` references in Core services with platform-agnostic `S.Preferences` and `AppSettings` APIs, restoring Windows/Linux build compatibility.

## [1.6.1] - 2026-06-29

### Fixed

- **CI Build Error**: Replaced MAUI `Preferences.Default` and Avalonia `SettingsPage` references in Core services with platform-agnostic `S.Preferences` and `AppSettings` APIs, restoring Windows/Linux build compatibility.

### Added

- **JSON Source Generation (AOT Support)**: Centralized registry in `AppJsonContext.cs` for all serialized models (`AuthResponse`, `DebugLogPayload`, `RalphTaskStatus`, `AssetModMetadata`, etc.) to ensure stability in trimmed builds.
- `global.json`: Pinned .NET SDK to 9.0.313 for build reproducibility and to bypass broken preview SDKs.
- `DESIGN.md` documenting the **Terminal Core** design system (colors, typography, layout grid, component specs, forbidden patterns).
- `EXTERNAL_DEPENDENCIES.md` fully updated to reflect Avalonia UI / .NET 9 stack and current repository layout.

### Changed

- **AOT Optimization**: Migrated `BetterAuthService`, `TelemetryService`, `RalphSyncService`, and `WorkspaceService` to use source-generated JSON serialization, eliminating `IL2026` trim warnings.
- **Compiler Warning Cleanup**: Refactored `SafeProcess`, `SessionManager`, and `SteamUgcPreviews` to resolve `CS1998` (async missing await).
- **Null Safety**: Updated `SettingsPage`, `NewProjectPage`, and `ProjectsPage` to resolve `CS8618` (uninitialized non-nullable fields) in Avalonia views.
- **Build artifacts naming convention** changed to `gregModmanager-{semVersion}{-pre}{-OS}.{ext}` across all platforms (Windows EXE/ZIP, Linux tarball/packages).
- `README.md` updated with **GitHub Actions status badges** (Build & Release, Linux Packages) in `for-the-badge` style.
- `README.md` now references official ecosystem hubs [gregframework.eu](https://gregframework.eu) and [datacentermods.com](https://datacentermods.com).
- `AGENTS.md` paths updated to match `src/` / `tests/` / `build/` directory layout.

### Fixed

- `build/builder.ps1` — renamed `Pause-AnyKey` to `Wait-KeyPress` to comply with PowerShell approved verbs (PSScriptAnalyzer).
- `build/installer/sign-authenticode.ps1` — `$PfxPassword` changed to `[Security.SecureString]`; automatic variable `$args` renamed to `$signArgs`.
- `build/scripts/linux/build-avalonia-packages.sh` — package names now follow consistent `gregModmanager-*-Linux.*` scheme.
- `.github/workflows/build-and-release.yml` — artifact upload/download globs aligned with new naming convention.
- `.github/workflows/linux-packages.yml` — artifact paths aligned with new naming convention.
- All build scripts, installer script, and CI workflows updated to support `-pre` suffix for prerelease/dev-branch builds.

## [1.5.1] - 2026-05-05

### Added

- Pre-release detection in build scripts: non-main branches and SemVer prerelease identifiers append `-pre` to artifact names.
- `build/scripts/linux/build-avalonia-packages.ps1` now forwards `-IsPre` flag to the underlying shell script via WSL.

### Changed

- Build output directory consolidated under `build/installer/Output/` (Windows) and `artifacts/` (Linux).
- `Get-ProjectVersion` helper in `build.ps1` now returns `IsPre`/`PreSuffix` properties for downstream naming.

### Fixed

- CI artifact attestations now point to correct artifact paths after rename.

## [1.5.0] - 2026-05-05

### Added

- **Avalonia UI 11.2 migration** — complete rewrite of the desktop UI layer replacing the legacy MAUI stack.
- **Linux packaging support** — DEB, RPM, Arch packages via `nfpm`; tar.gz portable builds for `linux-x64`.
- **Linux headless CLI support** — Steam Deck compatible mod management without GUI.
- **GitHub Actions CI/CD** — native Linux build job, artifact attestation, and optional GHCR container publishing.
- **Code signing pipeline** — Authenticode signing with ephemeral self-signed fallback for local builds; PFX support in CI.
- **Steamworks integration** — `Facepunch.Steamworks` for game root auto-detection and Workshop publishing.
- **Beta plugin source** — server API integration for beta plugin distribution.
- **SubDirectoryFixer** — MelonLoader `net6.0` plugin built and bundled during publish.
- **Localization** — multi-language resource files (en, de, es, it, ja, pl, ru, zh) with `GregModmanager.Localization.S` accessor.
- **Inno Setup installer** — Windows Setup EXE with automatic uninstall of older versions.
- **Interactive builder** — `build/builder.ps1` and `build/builder.sh` with Terminal Core styled CLI menu.
- **Unit tests** — xUnit project with `ContentStats` coverage and project sanity checks.

### Changed

- **Project layout restructured** to `src/` / `tests/` / `build/` for maintainability.
- Single source of truth for version: `src/GregModmanager.Avalonia/GregModmanager.Avalonia.csproj`.
- Size-optimized publish: `<PublishTrimmed>true</PublishTrimmed>`, `<TrimMode>full</TrimMode>`, `<PublishSingleFile>true</PublishSingleFile>`.

### Fixed

- **Security** — URL validation added to `Process.Start` to prevent insecure process launches.
- **Security** — secure process launch and URL opening hardened across all entry points.
- `steam_api64.dll` excluded from Authenticode signing loops (not a valid PE for signing).
- `nfpm` installation switched to GitHub releases due to broken apt repository.

## [1.0.3] - 2026-04-07

### Added

- Discord release notification workflow for Windows builds.
- Comprehensive GitHub issue templates (bug report, crash report, documentation, feature request, installer problem, performance, questions).
- Code-signing status documentation in README.
- Self-signed setup workflow for local development.

### Changed

- CI workflow enforces signed release gates before Discord notification.
- README updated with sponsorship information and badges.

### Fixed

- Windows build now signs app EXE before packaging into the installer.
- Test artifacts excluded from publish output.

## [1.0.2-pre.2] - 2026-04-07

### Fixed

- Pre-release packaging and workflow alignment fixes.

## [1.0.2-pre.1] - 2026-04-07

### Added

- Initial prerelease packaging pipeline.

## [1.0.0] - 2026-04-07

### Added

- Initial public release migrated from the GregTools monorepo to a standalone repository.
- ModManagerPage UI with store, installed, favorites, and diagnostics views.
- Project creator with real source code templates (`.csproj`, `.cs`) for ModStore and Workshop.
- Complete template coverage: UXML, Models, Textures, Audio.
- Authentication and install intent handling.
- Multi-language support infrastructure.
- WebView2 user data folder configuration.
- Logging and crash reporting across multiple pages.
- Directory writable checks and improved error handling.

---

[Unreleased]: https://github.com/mleem97/gregModmanager/compare/v1.5.1...HEAD
[1.5.1]: https://github.com/mleem97/gregModmanager/compare/v1.5.0...v1.5.1
[1.5.0]: https://github.com/mleem97/gregModmanager/compare/v1.0.3...v1.5.0
[1.0.3]: https://github.com/mleem97/gregModmanager/compare/v1.0.0...v1.0.3
[1.0.0]: https://github.com/mleem97/gregModmanager/releases/tag/v1.0.0
