# External dependencies (Workshop Uploader / gregModmanager)

> **Note:** This document is considered Outdated as it reflects the Old Application Structure. For the latest information on external dependencies, please refer to the updated documentation in the `README.md` and `EXTERNAL_DEPENDENCIES.md` files in the Wiki (gregframework.eu)

This document is an **open-source transparency** note: what this app ships or relies on, and how that relates to licenses. It is **not legal advice**.

## Open source at a glance

| | |
|--|--|
| **This application’s source code** | Licensed under the same terms as the containing repository (see root `LICENSE` if present). |
| **Open-source components we ship or depend on** | **.NET**, **.NET MAUI**, **Windows App SDK** (when bundled), **Microsoft.Maui.Controls**, **Facepunch.Steamworks** — typically **MIT** or Microsoft’s open-source licenses; see tables below and upstream notices. |
| **Not open source (binary redistributables)** | **steam_api64.dll** from Valve’s **Steamworks SDK** — distributed under Valve’s terms, not under a public source license. You must comply with [Steamworks](https://partner.steamgames.com/) agreements when building or redistributing the app. |
| **Runtime on the user’s PC** | **WebView2**, **Visual C++ Redistributable**, and optional **Windows App SDK** / **.NET** components may be installed separately; those are governed by their respective Microsoft (or other) licenses. |

For release **ZIP** contents: the publish output includes managed assemblies from NuGet (see `GregModmanager.csproj` and lock files if you use them) plus native **steam_api64.dll** and **steam_appid.txt** as described below.

## Runtime and framework (open source)

| Component | Use | License / terms |
|-----------|-----|-------------------|
| **.NET** | Runtime and SDK | [MIT](https://github.com/dotnet/runtime/blob/main/LICENSE.TXT) |
| **.NET MAUI** | UI framework | [MIT](https://github.com/dotnet/maui/blob/main/LICENSE) |
| **Microsoft.Windows.SDK.NET.Ref** (via TFM) | Windows API projections | MIT |
| **Windows App SDK** (bundled when `WindowsAppSDKSelfContained` is enabled) | WinUI / unpackaged Windows support | [MIT / see Windows App SDK license](https://www.nuget.org/packages/Microsoft.WindowsAppSDK) |

## NuGet packages (open source)

| Package | Use | License |
|---------|-----|---------|
| **Microsoft.Maui.Controls** | MAUI controls | MIT |
| **Facepunch.Steamworks** | Steamworks API from managed code | **MIT** ([repository](https://github.com/Facepunch/Facepunch.Steamworks)) |

## Binary redistributables (not “open source”)

| File | Source | Notes |
|------|--------|--------|
| **steam_api64.dll** | [Steamworks SDK](https://partner.steamgames.com/doc/sdk) | Valve **redistributable** for Steam-enabled applications. You may not have source code; distribution is governed by **Steamworks SDK** and Steam partner terms. Do not imply Valve endorses this project. |
| **steam_appid.txt** | Project / game configuration | Text file with the app ID; not a library. |

Using Steamworks implies acceptance of Valve’s applicable agreements for developers and players.

## Windows and Edge components

| Component | Notes |
|-----------|--------|
| **WebView2** | Used by MAUI/Edge WebView2 on Windows. [BSD-style license](https://github.com/MicrosoftEdge/WebView2Feedback/blob/main/LICENSE); runtime may be installed separately on the machine. |
| **Visual C++ runtime** | May be required on some PCs; [Microsoft VC++ Redistributable](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist) (see Microsoft license terms). |

## GitHub Actions (monorepo vs standalone repo)

The workflow file lives at **`.github/workflows/gregtools-modmanager-release.yml`** in the parent repository.

- **Inside the monorepo**, use tags matching **`gregtools-modmanager-v*`** (e.g. `gregtools-modmanager-v1.0.0`) so this workflow does not collide with other projects that use generic `v*` tags.
- **Standalone `workshopuploader` repo** (this app at the repository root): copy the same workflow into your new repo’s `.github/workflows/`, then change the `on.push.tags` entry to **`v*`** (or your preferred prefix) and tag releases accordingly.

Path detection in the workflow supports, in order: **`GregModmanager.csproj`** at repo root, **`workshopuploader/GregModmanager.csproj`**, or **`GregModmanager/GregModmanager.csproj`**.

## GitHub Releases and distribution formats

### Current default: ZIP (self-contained folder)

Releases are published as a **ZIP** at **best compression** (`Compress-Archive -CompressionLevel Optimal` in CI). The archive contains the **self-contained** `win10-x64` publish output (including bundled Windows App SDK where enabled in the project).

This avoids code signing and store requirements for early versions.

### App Installer / MSIX (optional, future)

**Windows App Installer** (`.appinstaller`) and **MSIX** packages are the standard way to ship auto-updateable Windows apps. This repository’s **optional** CI job for MSIX is **disabled by default** (`if: false` in `.github/workflows/gregtools-modmanager-release.yml`) because:

- MSIX sideloading typically requires **code signing** (certificate / Azure Trusted Signing / store pipeline).
- The MAUI project currently uses **unpackaged** defaults (`WindowsPackageType=None`) for broad compatibility.

When you are ready:

1. Configure packaging (e.g. MSIX) and signing locally.
2. Store signing secrets in GitHub Actions (e.g. certificate thumbprint / key vault).
3. Enable the `package-app-installer` job and align MSBuild properties with your signing method.

## Trademarks

**Steam** is a trademark of Valve Corporation. **Data Center** and related assets are the property of their respective rights holders. This tool is a community modding utility and is not affiliated with or endorsed by Valve or the game publisher unless explicitly stated elsewhere.

