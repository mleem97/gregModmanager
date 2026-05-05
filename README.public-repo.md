# gregModmanager (public README variant)

**gregModmanager (Workshop Uploader)** — Windows **.NET MAUI** app for **Data Center** Steam Workshop: Mod Store, local upload projects, `metadata.json` / `content/config.json`, and Steam publish (Facepunch.Steamworks).

[![Sponsor mleem97](https://img.shields.io/badge/Sponsor-mleem97-EA4AAA?style=for-the-badge&logo=GitHub-Sponsors&logoColor=white)](https://github.com/sponsors/mleem97)
[![Build & Sign](https://img.shields.io/github/actions/workflow/status/mleem97/gregModmanager/build-and-sign.yml?branch=main&style=for-the-badge&label=Build%20%26%20Sign)](https://github.com/mleem97/gregModmanager/actions/workflows/build-and-sign.yml)
[![Discord Notify](https://img.shields.io/github/actions/workflow/status/mleem97/gregModmanager/discord-release-notify.yml?branch=main&style=for-the-badge&label=Discord%20Notify)](https://github.com/mleem97/gregModmanager/actions/workflows/discord-release-notify.yml)
[![Daily Security Scan](https://img.shields.io/github/actions/workflow/status/mleem97/gregModmanager/daily-malicious-code-scan.yml?branch=main&style=for-the-badge&label=Daily%20Security%20Scan)](https://github.com/mleem97/gregModmanager/actions/workflows/daily-malicious-code-scan.yml)

---

## Part of gregFramework

This directory is part of the **[gregFramework](https://github.com/mleem97/gregFramework)** workspace. Clone sibling repositories side by side so each project lives at `gregFramework/<RepoName>/`. See the workspace [README](https://github.com/mleem97/gregFramework/blob/master/README.md) for the full layout and migration notes.

For the full guide (installers, Linux bundles, troubleshooting), use **`README.md`** in this directory.

---

## Build

```powershell
dotnet build GregModmanager.sln -c Release
```

```powershell
dotnet publish GregModmanager.csproj -c Release -p:SelfContained=true -p:RuntimeIdentifier=win10-x64
```

See `README.md` in this directory for troubleshooting, Steam layout, and headless CLI.

---

## Releases (GitHub Actions)

In the **monorepo**, releases are driven by `.github/workflows/gregtools-modmanager-release.yml`. Push a tag:

`gregtools-modmanager-v1.5.0`

After this folder is its **own repository**, edit that workflow (or copy it to `.github/workflows/release.yml`) and switch the tag pattern to `v*` if you prefer.

**First releases** ship a **ZIP** of the self-contained `win10-x64` publish folder at **best compression**. Optional **MSIX / App Installer** packaging is documented in [EXTERNAL_DEPENDENCIES.md](./EXTERNAL_DEPENDENCIES.md) (CI job is off until signing is configured).

**VirusTotal:** [file relations](https://www.virustotal.com/gui/file/c0ea7929eee9d754e81363e9ec81c601e763e65f7db1eb0d971edf2c2036f0af/relations) (SHA-256 `c0ea7929eee9d754e81363e9ec81c601e763e65f7db1eb0d971edf2c2036f0af`) — see `README.md` for context.

---

## Code signing and SmartScreen status

- Official OV/EV code signing is currently **not enabled** because I cannot afford a commercial certificate at the moment.
- For testing/community builds, CI uses **self-signed** signatures (rotated on a 7-day cadence or renewed when close to expiry).
- SmartScreen and trust prompts can still appear for users, even when binaries are signed.
- Runtime note: this is a Windows .NET MAUI app; startup issues are often resolved by installing/repairing the Visual C++ Redistributable and Windows App SDK runtime.
- If you want to support the project, GitHub Sponsors is enabled: **[github.com/sponsors/mleem97](https://github.com/sponsors/mleem97)**.

---

## Sponsorship

- **Sponsor page:** [https://github.com/sponsors/mleem97](https://github.com/sponsors/mleem97)

---

## Crash dumps (WER LocalDumps)

If the app hard-crashes (native exception / process abort), Windows can write `.dmp` files automatically:

```powershell
.\installer\configure-localdumps.ps1
```

Machine-wide (requires elevated PowerShell):

```powershell
.\installer\configure-localdumps.ps1 -Scope Machine
```

Disable again:

```powershell
.\installer\configure-localdumps.ps1 -Disable
```

Default dump folders:

- CurrentUser: `%LOCALAPPDATA%\gregModmanager\dumps`
- Machine: `C:\ProgramData\gregModmanager\dumps`

---

## Open-source and external dependencies

See **[EXTERNAL_DEPENDENCIES.md](./EXTERNAL_DEPENDENCIES.md)** for licenses, **Steamworks** / **steam_api64.dll**, and distribution notes.

---

## License

Use the same license as the parent [DataCenterExporter / gregFramework](https://github.com/mleem97/gregFramework) project unless you add a dedicated `LICENSE` to this repository.

