# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [1.6.1] - 2026-09-25

### Added

- Unified open-source documentation layout (gregCore template): `LICENSE`
  (Apache 2.0 — replaces the undocumented MIT claim), `VERSION` file as the
  single source of truth (`.csproj` stays in sync as fallback, CI reads
  `VERSION` first), `CONTRIBUTING.md`, `QUICKSTART.md`, and a rewritten
  `README.md` with for-the-badge badges, links, compatibility, features,
  installation, dependencies, credits, and team sections.
- Forgejo Actions CI in `.forgejo/workflows/` (build, test, Linux packages,
  Forgejo releases, one Discord notification per run). Windows artifacts are
  cross-built on Linux runners; the Inno Setup EXE is best-effort under Wine.
- Headless publish accepts `--changelog <text>` (alias `--change-note`) so
  Steam Workshop updates carry a change note for version control.

- Shared Terminal Core components for secondary, compact, danger, and active-tab
  buttons as well as reusable card and content-surface containers.
- Docker Buildx targets for deterministic test runs, Windows cross-publish,
  and macOS x64/arm64 cross-publish; a Windows-container Dockerfile is
  available for compatible Windows Docker hosts.
- SSH-based workflow to build, test, and prepare a manual GUI test on a real
  macOS host from a Linux development machine.
- Source-backed user, creator, contributor, build, and codebase documentation
  now identifies its scope, evidence, maintainer ownership, and review triggers.
- Headless Workshop publishing, the mode-dependent Steam Workshop Mod Store,
  local session storage, telemetry defaults, and reproduction-bundle privacy
  boundaries are documented.

### Fixed

- **Linux Steam publish**: replaced the broken Facepunch `Editor` publish path
  with direct native `ISteamUGC` calls via reflection (`NativePublishAsync`).
  `SetItemContent`/`SetItemPreview` through the `Editor` wrapper returned
  `FileNotFound` on Linux even though every file existed and was readable.
  The preview is staged to a temp copy before the upload (locked originals —
  file manager, image viewer, or our own UI — no longer block Steam), live
  progress and elapsed time are reported during the upload, and the wait loop
  exits early on `CommittingChanges` instead of hanging. `TrimmerRoots.xml`
  is now referenced from the Avalonia project so trimmed release builds keep
  the `ISteamUGC` update methods.
- **Linux Steam connection**: replaced the bundled Linux `libsteam_api.so` /
  `libsteam_api64.so` (Steamworks SDK 1.62 interface set, `SteamApps_v009`) with
  the SDK 1.61 build (`SteamApps_v008`/`SteamFriends_v017`) matching
  `Cherry.Facepunch.Steamworks 2.5.0`. `SteamClient.Init` failed with
  `EntryPointNotFoundException` before, so the app permanently showed
  "Steam Disconnected" on Linux; Workshop browse, subscribe, and publish work
  again against a running Steam client.
- **Project editing**: each project card now has an explicit Edit/Bearbeiten
  button (reliable `Click` handling alongside tap-to-open). Opening a project
  can no longer fail silently: navigation and load errors are logged and shown
  as a dialog instead of leaving the Projects page without feedback.
  Navigation resolves the main window via the DI singleton with a visual-tree
  fallback.
- **Merged Projects/My Uploads page**: the separate My Uploads page is gone.
  The Projects page now has two tabs — local projects (search, refresh, edit)
  and Workshop uploads (refresh, import selected, edit/add-update/download/view
  on Steam, paging). Importing an item refreshes the local list and opens the
  editor directly. The sidebar entry and profile-menu link point here.
- **Steam native-load diagnostics**: `SteamApiNativeLoader` now records every
  searched candidate path, so the "not found" hint actually lists where the
  loader looked instead of an empty path list.
- **Modstore authentication routing**: desktop login, token exchange, logout,
  and Better Auth now use `datacentermods.com` as the primary service and
  fail over to the trusted `datacentermods.home` endpoint only when the public
  endpoint is unreachable or unavailable.
- **Steam fallback**: invalid Steam API initializations no longer cause project
  opening or the My Uploads list to crash; local projects remain editable when
  Workshop metadata cannot be refreshed.
- **Workshop editing**: the editor and Mod Store now use consistent tab and
  action states, while item actions remain usable on narrow desktop windows.
- **Linux Steam detection**: native Steam API discovery now also checks the
  Flatpak Steam installation and accepts whitespace-formatted Steam library
  VDF files.
- **Linux release artifacts**: portable CI builds exclude the Windows Steam DLL
  and include both supported Linux Steam API library names.
- **Linux protocol registration**: desktop entries correctly quote application
  paths and refresh the actual applications directory.
- **CI Build Error**: Replaced MAUI `Preferences.Default` and Avalonia `SettingsPage` references in Core services with platform-agnostic `S.Preferences` and `AppSettings` APIs, restoring Windows/Linux build compatibility.
- **Build consistency**: aligned GitHub Actions with the SDK pinned in
  `global.json`, updated non-MelonLoader package/action dependencies, and
  replaced the stale root Linux build pipeline with a compatibility wrapper.
- **Build usability**: corrected the interactive Bash builder's release-script
  paths and host-specific commands.

### Removed

- GitHub Actions workflows (`.github/workflows/`): GitHub is now a read-only
  mirror, all automation lives in `.forgejo/workflows/`. The standalone
  Discord release workflow is gone — the main pipeline sends exactly one
  Discord message per completed run.

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
