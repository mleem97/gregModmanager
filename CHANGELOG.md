# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Modstore publish: `gregcli publish --to modstore` (API token via
  `--token`/`GREG_MODSTORE_TOKEN`, `--mod-id`, `--category`, `--license`,
  `--api-url`, `--file`) and an editor "Modstore Upload" button sharing
  one flow (`greg-modstore::publish`: zip `content/`, presigned URL, PUT,
  submit). The Modstore id persists in `metadata.json` (`modstore_id`)
  for updates; outcome names mod, release and scan status.
- Supervised game launch: Start Game / With Mods / No Mods start the game
  exe directly as a tracked child (PID, play time, exit code in log and
  status bar) with a header Stop button. The Steamworks session releases
  before the start and reconnects after exit, so Steam no longer sees the
  game as running through the manager. Stale game processes block a new
  start and can be killed via Stop (also from the Problems inbox).
- Real vanilla mode: No Mods parks `Mods/` to `Mods.disabled-by-manager`
  and restores it on game exit; leftovers from killed sessions recover at
  the next start. Mod changes (toggle, remove, pack apply) are refused
  while the game runs.

### Fixed

- Layout: all list pages (My Mods/Plugins/Libs, Problems, Modpacks/
  Collections, Projects, Workshop Browse, Modstore Catalog/Updates) fill
  the viewport and scroll their list regions internally (`ScrollView` +
  stretch) instead of growing past the window; verified with 30–40-row
  headless screenshots. The content wrapper no longer uses
  `alignment: start` (it ignored stretch and collapsed scroll boxes).

### Changed

- Header buttons (Start Game, With Mods, No Mods, Login) are now flat
  custom `TopBarButton`s: transparent at rest, slightly lighter
  (`container-high`) on hover.

## [1.7.1] - 2026-09-28

### Fixed

- Modstore E2E (live `datacentermods.com` verified 2026-09-28): catalog
  (`GET /api/v1/mods`), updates (`POST /api/v1/mods/updates/check`),
  collections (`GET /api/collections`), upload (`POST /api/upload-url` +
  `POST /api/mods/submit`, 401 without Bearer) and the OAuth browser flow
  (`GET /auth/login` → 307, `POST /auth/token`, `POST /auth/logout`)
  match the desktop client. Store is live but empty
  (`{"schemaVersion":1,"mods":[]}`).
- OAuth session restore no longer accepts a `200 null` get-session body:
  `BetterAuthClient::verify_session` is fail-closed and accepts both the
  flat `{id, email}` shape and BetterAuth's `{user, session}` wrapper.
  Expired/invalid tokens now force a fresh browser login instead of a
  stale session.

## [1.7.0] - 2026-09-28

### Added

- Modmanager core loop: My Mods/Plugins/Libs (enable/disable/remove),
  local Modpacks (create, apply, export/import) and shareable Steam-like
  Collections, Problems inbox with jump actions, Report-a-Bug tracker link.
  Browse unifies Browse/Subscribed/Favorited with auto-load (My Uploads
  removed — own uploads live in Projects).
- `cargo xtask dist`: portable release packages with native Steam libraries
  staged next to the binaries — Windows `steam_api64.dll` (`.zip`),
  Linux `libsteam_api.so` (`.tar.gz`), macOS `libsteam_api.dylib` (`.tar.gz`),
  plus `steam_appid.txt`, docs, and SHA-256 sidecars. Cross-builds via
  `--target` (win-x64 verified with mingw).
- Rust workspace restart (`feat/rust-workspace`): `greg-core` (models, Keep a
  Changelog/SemVer, Markdown→Steam-BBCode, upload checks, l10n en/de),
  `greg-platform` (workspace, prefs, logs, protocol, repro bundles, telemetry),
  `greg-steam` (`steamworks` crate backend with publish/browse/subscribe/
  download), `greg-modstore` (catalog, updates, atomic installs, intents,
  auth), `greg-loader` (game adapters, health checks, sync, channels,
  installers, templates), `greg-cli` headless publish/status/list, `greg-app`
  Slint main window (icon bar, conditional navigation, Steam/GregApi status
  bar with Modstore gating), and `xtask`. C# sources moved to
  `.reference/modmanager-old/`.

- Dual CI: `.gitea/workflows/` added as byte-identical copy of
  `.forgejo/workflows/` (Gitea Actions alongside Forgejo Actions, guarded by a
  `parity` job). GitHub has no workflows directory and runs no automation —
  it is a read-only archive mirror only.
- File-based project docs: `README.md` (Markdown) is the description source and
  `CHANGELOG.md` (Keep a Changelog + SemVer) is the changelog source. Steam
  uploads get converted BBCode/plain text automatically — the Modstore keeps
  native Markdown. The changelog section matching `metadata.json`'s version is
  auto-read, so updates need no manual input in the app or `--changelog` flag.
  New template projects ship both files; upload checks validate SemVer and the
  Keep a Changelog format.
- Workshop gallery upload: editor screenshots (`additional_previews`) are
  attached in a second update after publish (≤ 1 MiB each, larger/missing
  files logged and skipped). Steam appends gallery files (no replace API).
- Workshop gallery download: import fetches an item's gallery into
  `screenshots/` and records it in `additional_previews`.
- Modstore API calls use the OAuth session: catalog and update checks send
  the access token as Bearer when logged in (release downloads stay
  unsigned CDN URLs).
- List thumbnails: My Mods shows sibling previews (`Foo.png`/`.jpg` next
  to the file), Projects shows the project preview file, the editor shows
  a large preview box, and Browse/Modstore download remote previews into
  a cache in the background (placeholder box until loaded).

### Changed

### Deprecated

### Removed

### Fixed

- Main-window page layout: content pages are compact and top-anchored again
  (no void above lists, no bottom-anchored rows). Root causes: the content
  wrapper packed stretch-less pages at the end, `vertical-stretch` on
  fixed-content list containers only propagated tallness upward while bare
  status texts absorbed the free space mid-page, and `TabWidget` sized its
  `Flickable` tab content to zero height. The Modstore now uses custom tab
  buttons with conditional content panels instead of `TabWidget`.
- My Mods auto-scan: reopening/refreshing a local page re-scans the content
  directories on change (signature-based change detection).
- Upload freeze fixed: project sync-state hashing (SHA-256 over every
  `content/` byte) ran synchronously on the UI thread on every Projects
  refresh/save/Upload click, freezing the app for large mods. Hashing now
  runs in a background job with a sync-state cache — lists paint instantly
  ("?" while unknown) and flip to SYNCED/UPDATED! when the job drains.
  Covered by the `project_sync_hashes_off_ui_thread` regression test.
- Empty changelog no longer blocks the first publish (warning only), and
  a blocked Save & Upload now names the failing checks in the status line
  instead of doing nothing. Worker panics surface as failed outcomes.
- Health check looks for `Mods/gregCore.dll` (renamed assembly).
- OAuth session flow: `greg://auth/callback` (plus legacy `greg://v1/…`)
  handled at startup and forwarded from second instances (single-instance
  guard + handoff file), request-id validation, session restore on boot,
  profile card + session menu (My Mods, Upload, Settings, Logout).
- Rust CI (`rust.yml`, byte-identical in `.forgejo/` + `.gitea/`): fmt,
  clippy, tests, Linux dist build, Windows cross-build.

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
