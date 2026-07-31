# gregModmanager

Desktop client for the gregFramework modding ecosystem. The current application
can create and manage local Workshop projects, work with Steam Workshop services,
and provide account and settings screens for the Data Center integration.

## Status at a glance

| Platform | Build status | Distribution status |
| --- | --- | --- |
| Windows x64 | Supported | Portable ZIP, Inno Setup EXE, and WiX MSI |
| Linux x64 | Supported | Portable ZIP/tarball, DEB, RPM, APK, and Arch package |
| macOS x64 | Experimental cross-publish | Manual testing on a real Mac only; no signed or notarized release |

The desktop UI currently exposes **Projects**, **New Project**, **My Uploads**,
and **Settings**. Profile management, load-order editing, automatic conflict
resolution, an in-app updater, and a public mod-store browser are not currently
released features. The Data Center adapter explicitly reports that profiles are
unsupported.

## Quick start

Prerequisite: install the SDK pinned in [global.json](global.json).

```bash
dotnet restore GregModmanager.sln
dotnet build GregModmanager.sln -c Release
dotnet test tests/GregModmanager.Tests/GregModmanager.Tests.csproj -c Release
dotnet run --project src/GregModmanager.Avalonia/GregModmanager.Avalonia.csproj
```

For release artifacts, start with [the build-script reference](build/scripts/README.md).
The normal Windows release build requires Inno Setup 6 and WiX CLI 5. Linux
packages are built directly by the Linux packaging script or from Windows through
WSL; a non-Windows invocation of `build.ps1` does not create distro packages.

## Repository layout

```text
src/
  GregModmanager.Avalonia/       Avalonia desktop application
  GregModmanager.Core/           models, services, Steam and local-state code
  GregModmanager.Melons/         MelonLoader integrations, including SubDirectoryFixer
tests/GregModmanager.Tests/      xUnit test project
build/                           interactive builders, release scripts, installers
docker/                          Linux build/test container and Windows-container notes
docs/                            repository documentation (the tracked wiki content)
```

`SubDirectoryFixer` targets .NET 6 because MelonLoader requires that runtime. It
is built separately from the main solution and copied to the Avalonia assets by
`build/scripts/build.ps1`.

## Documentation

- [Documentation index](docs/INDEX.md)
- [User guide](docs/01_END_USER_GUIDE.md)
- [Creator guide](docs/02_MOD_CREATOR_GUIDE.md)
- [Contributor guide](docs/03_CONTRIBUTOR_GUIDE.md)
- [Build scripts](build/scripts/README.md)
- [Docker build and test containers](docker/README.md)
- [Code-signing behaviour](build/installer/CODE_SIGNING.md)
- [Dependency and licence inventory](EXTERNAL_DEPENDENCIES.md)

## Technology

- .NET 9 (`net9.0`) and Avalonia 12.1.0
- Cherry.Facepunch.Steamworks 2.5.0
- Microsoft.Extensions.DependencyInjection 10.0.10
- xUnit for automated tests

## Licence and support

The project declares the MIT licence in its repository metadata. Report
reproducible defects in the project's GitHub issue tracker; include the
platform, application version, and relevant non-secret logs.
