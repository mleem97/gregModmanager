# Documentation index

## Choose a task

| I want to… | Read |
| --- | --- |
| install a supported release | [End-user guide](01_END_USER_GUIDE.md) |
| understand the available desktop screens | [End-user guide: current scope](01_END_USER_GUIDE.md#current-scope) |
| create or publish a Workshop project | [Creator guide](02_MOD_CREATOR_GUIDE.md) |
| set up a development checkout | [Contributor guide: local setup](03_CONTRIBUTOR_GUIDE.md#local-setup) |
| build Windows, Linux, or packages | [Build-script reference](../build/scripts/README.md) |
| test in a Linux Docker container or prepare a real-Mac test | [Docker guide](../docker/README.md) |
| sign a Windows release | [Code-signing guide](../build/installer/CODE_SIGNING.md) |
| inspect design decisions and limitations | [Codebase reference](codebase/ARCHITECTURE.md) |

## Source of truth

| Topic | Source |
| --- | --- |
| application version and runtime | `src/GregModmanager.Avalonia/GregModmanager.Avalonia.csproj` |
| pinned SDK | `global.json` |
| CI release outputs | `.github/workflows/build-and-release.yml` |
| local release orchestration | `build/scripts/build.ps1` |
| Linux package formats | `build/scripts/linux/build-avalonia-packages.sh` |
| Windows installers | `build/installer/` |

## Limits and terminology

Windows x64 and Linux x64 are release platforms. macOS x64 may be
cross-published and tested manually on a physical Mac, but it is not a signed,
notarized, or published release target. “Package” means a DEB, RPM, APK, or Arch
package; “portable archive” means ZIP or tarball.

The `docs/` directory is tracked directly by this repository. Do not follow old
instructions that refer to `wiki/` or `RELEASENOTE.md`; the changelog is
[`CHANGELOG.md`](../CHANGELOG.md).
