# GregModmanager Scripts

This directory contains build, run, install, packaging, and development helper scripts.

## Core Scripts

### `build.ps1`

Builds and packages the application. A full Windows build creates a signed
portable ZIP, an Inno Setup EXE, and a WiX MSI. Full Linux builds create a
portable archive plus DEB/RPM/APK/Arch packages through the Linux packaging
adapter.

```powershell
.\build.ps1
.\build.ps1 -SkipPublish
.\build.ps1 -Sign
.\build.ps1 -SigningMode self-signed
.\build.ps1 -SigningMode pfx
.\build.ps1 -SigningMode none
```

### `run.ps1`

Runs the app from source.

```powershell
.\run.ps1
.\run.ps1 -- -h
```

### `install-local.ps1`

Performs local non-installer deployment.

```powershell
.\install-local.ps1
.\install-local.ps1 -SkipPublish
.\install-local.ps1 -Uninstall
```

### `start.ps1`

Starts the built desktop binary directly.

```powershell
.\start.ps1
```

## Development Helpers

### `dev-helpers.ps1`

XAML/UI maintenance helper commands.

```powershell
. .\dev-helpers.ps1
Fix-AppShellResources
Fix-UiPageResourcesAll
```

### `fix-csharp-strings.csx`

One-off C# string interpolation fixer.

```powershell
dotnet script fix-csharp-strings.csx
```

## Linux Packaging

### Legacy bundle packager

- `linux/build-linux-packages.ps1`
- `linux/build-linux-packages.sh`

### Avalonia Linux package pipeline

- `linux/build-avalonia-packages.ps1`
- `linux/build-avalonia-packages.sh`
- `linux/flatpak/com.gregframework.gregModmanager.yml`

Build distro packages:

```powershell
.\linux\build-avalonia-packages.ps1
```

Outputs:

- `.deb`
- `.rpm`
- `.apk`
- `.pkg.tar.zst`
- `.tar.gz`

Linux package installation automatically stops a running gregModmanager,
removes known legacy installation paths and stale desktop launchers, and
refreshes the desktop database. The canonical paths are `/opt/gregmodmanager`,
`/usr/bin/gregmodmanager`, and `gregmodmanager.desktop`.

## Root Wrappers

Root-level wrappers (`build.ps1`, `run.ps1`, `install-local.ps1`) forward into this folder for convenience.
