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
| Rust toolchain via rustup (stable) | build, test, release | [MIT OR Apache-2.0](https://github.com/rust-lang/rust) |

## Redistributables and platform assets

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

## Build and release tooling

| Tool | Purpose |
| --- | --- |
| Rust toolchain via rustup (stable) | restore, build, test, release (`cargo`, `cargo xtask`) |
| Inno Setup 6 (+ Wine) | Windows Setup EXE (best-effort; CI cross-builds the portable ZIP) |
| WiX CLI 5 | Windows MSI |
| nfpm and native package tools | DEB, RPM, APK, and Arch packages |
| Docker/Buildx | repeatable Linux build/test targets; not a Windows or macOS GUI emulator |
| Jenkins (self-hosted, multi-agent) | CI check/build matrix (`Jenkinsfile`): fmt, clippy, test, build on Linux, Windows and macOS agents |
<<<<<<< HEAD
| cargo-tarpaulin | Rust code coverage (LCOV) generated per workspace, uploaded to Codacy |
| Codacy Coverage Reporter | uploads LCOV coverage reports to Codacy (CI job `coverage`) |
| Forgejo Actions (`.forgejo/workflows/`) + Gitea Actions (`.gitea/workflows/`, byte-identical) | CI on Linux-only runners; Windows artifacts are cross-built, releases go through the instance API, one Discord notification per run. GitHub is a read-only archive mirror with no workflows. |
=======
| Forgejo Actions (`.forgejo/workflows/`) + Gitea Actions (`.gitea/workflows/`, byte-identical) | Linux-only CI and release path (Rust: fmt, clippy, test, audit, cross-build, packages, releases). |
| GitHub | read-only archive mirror — no CI, no release automation. |
>>>>>>> 4994627e2ee675af0363c19e5ab81256bacfeb0a

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
