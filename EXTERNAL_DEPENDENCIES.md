# External dependencies

This inventory is a technical transparency aid, not legal advice. Project files
and the third-party licence text are the authoritative sources when distributing
a release.

## Application dependencies

| Component | Current use | Licence/source |
| --- | --- | --- |
| Rust `slint` / `slint-build` 1.18.1 + `lucide-slint` 1.49.1 | cross-platform desktop UI (retained mode, single binary) | [MIT OR Apache-2.0](https://github.com/slint-ui/slint) + [ISC](https://github.com/lucide-icons/lucide/blob/main/LICENSE) for the icon artwork |
| Rust `steamworks` 0.13.1 + `steamworks-sys` 0.13.0 (`greg-steam`) | Workshop API; `-sys` used directly only for gallery `AddItemPreviewFile`/ManualDispatch polling (no safe wrappers) | [MIT](https://github.com/NoButDrHope/steamworks-rs) |
| Rust `tokio`, `reqwest` (rustls), `serde`, `serde_json`, `thiserror`, `anyhow`, `semver`, `dirs`, `clap`, `rfd`, `notify`, `zip`, `tracing` | async runtime, Modstore REST + OAuth, models/errors, validation, paths, dialogs, packaging, logging | crate-specific licences |
| Rust `zip` 9 (locked 9.1.3) / `rustls` 0.23.46 | ZIP packaging and extraction / TLS | [MIT](https://github.com/zip-rs/zip2) / [Apache-2.0 OR ISC OR MIT](https://github.com/rustls/rustls) |
| Rust `rusqlite` 0.41.1 + bundled `libsqlite3-sys` 0.39.1 | embedded SQLite settings database, no database server | [MIT](https://github.com/rusqlite/rusqlite) / SQLite public domain |
| Rust `image` 0.25.10 (PNG/JPEG/GIF/WebP) | bounded preview decoding and resizing | MIT OR Apache-2.0 |
| Rust toolchain via rustup (stable) | build, test, release | [MIT OR Apache-2.0](https://github.com/rust-lang/rust) |

## Redistributables and platform assets

The status bar bundles `crates/greg-app/ui/assets/steam-icon.svg`, supplied by
the maintainer from `/mnt/sdb/Downloads/steam-icon.svg`. Its embedded provenance
identifies SVG Repo; no licence text was included in that supplied file.
`crates/greg-app/ui/assets/melonloader.svg` is adapted from the
[MelonWiki logo](https://raw.githubusercontent.com/LavaGang/MelonWiki/9e52f65e18f321494cec8e91277722df2c97f33e/docs/_media/logo.svg)
at pinned commit `9e52f65e18f321494cec8e91277722df2c97f33e`: tighter viewBox,
colored/white fills removed to retain recognizable monochrome contours.
The gregAPI status uses `ui/assets/greg-status.svg`, a tightly framed derivative
of the existing root `G.svg`. All status logos are tinted by the UI.

| Asset | Platforms | Distribution note |
| --- | --- | --- |
| Steam native library (Windows `steam_api64.dll`) | Windows x64 publish output | governed by Valve Steamworks terms; do not present it as open source |
| Steam native library (Linux `libsteam_api.so` / Windows `steam_api64.dll` / macOS `libsteam_api.dylib`, Steamworks SDK interface set `SteamApps_v008`/`SteamFriends_v017`) | Publish output staged next to the binaries by `cargo xtask dist`; sourced from the `steamworks-sys` build output of the Rust `steamworks` crate | governed by Valve Steamworks terms; do not present it as open source. Newer SDK builds (e.g. 1.62.0 with `SteamApps_v009`) are **not** compatible and break the Rust Workshop bindings on Linux |
| Steam native libraries via `steamworks-sys` (Rust restart): Windows `steam_api64.dll`, Linux `libsteam_api.so`, macOS `libsteam_api.dylib` | staged next to the binaries by `cargo xtask dist` (rpath/`$ORIGIN` on Linux/macOS, DLL search on Windows) | governed by Valve Steamworks terms; do not present them as open source. `steam_appid.txt` (`4170200`) ships alongside |
| `steam_appid.txt` | platform publish output where configured | project configuration, not a library |
| bundled fonts | desktop resources | see the licence files beside the vendored fonts |
| Visual C++ Redistributable | Windows when required by the installed runtime/components | governed by Microsoft terms |

macOS is an experimental source/build target. The Rust implementation now
handles the macOS Steam library location, Unity `.app` game bundles, and
macOS process detection. A supported release still requires validation on a
real Mac with Steam, the game, and MelonLoader, plus Apple signing,
notarization, Gatekeeper testing, and a native dependency review.

## Optional mod build tooling

Editor build tests invoke an installed .NET SDK 8+ or Rust SDK explicitly.
NuGet/Cargo may restore a project's declared dependencies. Build tasks execute
project code; source builds are never run merely by downloading Workshop content.
The separate gregCore/Companion runtime remains .NET 6 and requires the maintained
MelonLoader startup hook described in `docs/integration/managed-runtime.md`.

## Build and release tooling

| Tool | Purpose |
| --- | --- |
| Rust toolchain via rustup (stable) | restore, build, test, release (`cargo`, `cargo xtask`) |
| Inno Setup 6 | Windows Setup EXE on the native Windows build agent |
| WiX CLI 5 | Windows MSI |
| nfpm | DEB, RPM, APK, and Arch packages; Alpine additionally needs gcompat and runtime validation |
| Docker/Buildx | repeatable Linux build/test targets; not a Windows or macOS GUI emulator |
| Jenkins (self-hosted, multi-agent; currently inactive) | CI check/build matrix and guarded GitHub release configuration (`Jenkinsfile`) |
| GitHub Actions | Linux and Windows test-package builds and guarded dev-to-staging promotion |
| cargo-tarpaulin | optional local workspace LCOV coverage |
| Codacy Coverage Reporter | local helper can upload LCOV; not currently part of the Jenkins matrix |
| Former Forgejo/Gitea Actions workflows | removed during Jenkins migration; Discord and coverage jobs still require replacements |
| GitHub Releases and GitHub CLI | release artifacts and update checks; Jenkins publication is blocked pending package validation |

## Release formats

| Platform | Formats |
| --- | --- |
| Windows x64 | portable ZIP, Setup EXE, MSI |
| Linux x64 | portable ZIP/tarball, DEB, RPM, APK, Arch package |
| macOS x64/arm64 | experimental `.tar.gz` cross-publish; no signed/notarized release |

The Windows self-signed certificate used by default is newly generated for each
build and is not a redistribution dependency. See
[Code signing](build/installer/CODE_SIGNING.md) for its trust limitations.

## MelonLoader integration

`.reference/modmanager-old/src/GregModmanager.Melons/SubDirectoryFixer/` is a
`net6.0` MelonLoader plugin (frozen, not part of the Rust workspace). Its local
`MelonLoader.dll` reference is a game-environment dependency, not a Rust crate.
Configure the reference on the build machine without committing machine-specific
game paths or copied game binaries.
