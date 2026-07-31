# Build scripts

Run the following commands from the repository root unless a command says
otherwise. The SDK version is pinned in `global.json`.

## Output matrix

| Host and command | Output |
| --- | --- |
| Windows: `build/scripts/build.ps1` | Windows ZIP, Setup EXE, MSI; Linux ZIP/tarball |
| Windows + WSL: same command | also DEB, RPM, APK, and Arch packages |
| Linux: `bash build/scripts/linux/build-avalonia-packages.sh` | DEB, RPM, APK, Arch package, and tarball |
| Docker Linux target | repeatable restore/build/test only; see `docker/README.md` |

`build.ps1` only calls the distro-package adapter through `wsl.exe`. On a Linux
host, invoke the Linux shell script directly instead of expecting PowerShell to
create packages.

## Release build

```powershell
# Windows host
.\build\scripts\build.ps1
.\build\scripts\build.ps1 -SkipTest
.\build\scripts\build.ps1 -SkipWindows
.\build\scripts\build.ps1 -SkipLinux
```

A complete Windows release requires Inno Setup 6 and WiX CLI 5. The script builds
the separate `src/GregModmanager.Melons/SubDirectoryFixer` project before each
publish and fails when that required project is missing.

## Signing

```powershell
# fresh short-lived self-signed certificate for this build
.\build\scripts\build.ps1 -SigningMode self-signed

# existing PFX or certificate-store thumbprint supplied through the environment
.\build\scripts\build.ps1 -SigningMode pfx

# sign an already-created Setup EXE without rebuilding
.\build\scripts\build.ps1 -SignOnly -SetupPath 'D:\dist\gregModmanager-1.6.1-Windows.exe'
```

`auto` and `self-signed` deliberately ignore existing signing environment values
and generate a new certificate. Use `pfx` for `CODE_SIGN_PFX` or
`CODE_SIGN_THUMBPRINT`. `-SignOnly` runs on Windows and signs exactly one
existing Setup EXE. See [the detailed signing guide](../installer/CODE_SIGNING.md).

## Development helpers

| Script | Purpose |
| --- | --- |
| `build/scripts/run.ps1` | run the application from source on Windows/PowerShell |
| `build/scripts/install-local.ps1` | local non-installer deployment |
| `build/scripts/start.ps1` | start an existing local desktop build |
| `build/builder.ps1`, `build/builder.sh` | interactive build menus |

These scripts have no root-level wrapper counterparts.
