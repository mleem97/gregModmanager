# External dependencies

This inventory is a technical transparency aid, not legal advice. Project files
and the third-party licence text are the authoritative sources when distributing
a release.

## Application dependencies

| Component | Current use | Licence/source |
| --- | --- | --- |
| .NET 9 | application runtime and SDK | [MIT](https://github.com/dotnet/runtime/blob/main/LICENSE.TXT) |
| Avalonia 12.1.1 | cross-platform desktop UI | [MIT](https://github.com/AvaloniaUI/Avalonia/blob/master/licence.md) |
| Microsoft.Extensions.DependencyInjection 10.0.10 | service composition | [MIT](https://www.nuget.org/packages/Microsoft.Extensions.DependencyInjection) |
| Cherry.Facepunch.Steamworks 2.5.0 | managed Steamworks integration | [MIT](https://github.com/Facepunch/Facepunch.Steamworks) |
| xUnit, Microsoft.NET.Test.Sdk, Coverlet | tests only | package-specific licences |

## Redistributables and platform assets

| Asset | Platforms | Distribution note |
| --- | --- | --- |
| Steam native library (Windows `steam_api64.dll`) | Windows x64 publish output | governed by Valve Steamworks terms; do not present it as open source |
| Steam native library (Linux `libsteam_api.so` / `libsteam_api64.so`, Steamworks SDK 1.61 interface set: `SteamApps_v008`/`SteamFriends_v017`) | Linux x64 publish output, sourced from the `Facepunch.Steamworks.Dll 1.61.0` NuGet redistributable (`runtimes/linux-x64/native/libsteam_api.so`) to match `Cherry.Facepunch.Steamworks 2.5.0` | governed by Valve Steamworks terms; do not present it as open source. Newer SDK builds (e.g. 1.62.0 with `SteamApps_v009`) are **not** compatible with the managed wrapper and break `SteamClient.Init` on Linux |
| `steam_appid.txt` | platform publish output where configured | project configuration, not a library |
| bundled fonts | desktop resources | see the licence files beside the vendored fonts |
| Visual C++ Redistributable | Windows when required by the installed runtime/components | governed by Microsoft terms |

macOS is not a distributed release target. Do not republish a cross-published
macOS directory as a supported macOS product without native dependency review,
signing, and notarization.

## Build and release tooling

| Tool | Purpose |
| --- | --- |
| .NET SDK (pinned by `global.json`) | restore, build, test, publish |
| Inno Setup 6 | Windows Setup EXE |
| WiX CLI 5 | Windows MSI |
| nfpm and native package tools | DEB, RPM, APK, and Arch packages |
| WSL | optional Windows-host Linux package adapter |
| Docker/Buildx | repeatable Linux build/test targets; not a Windows or macOS GUI emulator |
| Forgejo Actions (`.forgejo/workflows/`) | CI on Linux-only runners; Windows artifacts are cross-built, releases go through the Forgejo API, one Discord notification per run. GitHub is a read-only mirror with no workflows. |

## Release formats

| Platform | Formats |
| --- | --- |
| Windows x64 | portable ZIP, Setup EXE, MSI |
| Linux x64 | portable ZIP/tarball, DEB, RPM, APK, Arch package |
| macOS x64 | no published format |

The Windows self-signed certificate used by default is newly generated for each
build and is not a redistribution dependency. See
[Code signing](build/installer/CODE_SIGNING.md) for its trust limitations.

## MelonLoader integration

`src/GregModmanager.Melons/SubDirectoryFixer/` is a `net6.0` MelonLoader plugin.
Its local `MelonLoader.dll` reference is a game-environment dependency, not a
NuGet dependency. Configure the reference on the build machine without
committing machine-specific game paths or copied game binaries.
