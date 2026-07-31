# Contributor guide

## Local setup

Install the SDK selected by [global.json](../global.json), then restore, build,
and test from the repository root:

```bash
dotnet restore GregModmanager.sln
dotnet build GregModmanager.sln -c Release
dotnet test tests/GregModmanager.Tests/GregModmanager.Tests.csproj -c Release
git diff --check
```

Run the desktop application during UI work:

```bash
dotnet run --project src/GregModmanager.Avalonia/GregModmanager.Avalonia.csproj
```

The solution builds the desktop and core projects. `SubDirectoryFixer` is a
separate .NET 6 MelonLoader integration under
`src/GregModmanager.Melons/SubDirectoryFixer/`; the release script builds it and
copies its DLL into the desktop assets.

## Repository map

| Path | Responsibility |
| --- | --- |
| `src/GregModmanager.Avalonia/` | Avalonia entry point, views, resources, dependency injection |
| `src/GregModmanager.Core/` | models, services, persistence, Steam and game adapters |
| `src/GregModmanager.Melons/` | MelonLoader-specific projects |
| `tests/GregModmanager.Tests/` | xUnit tests |
| `build/scripts/` | non-interactive builds, local run/install helpers, Linux packaging |
| `build/installer/` | Inno Setup, WiX, and signing scripts |
| `docker/` | containerized Linux build/test and Windows-container guidance |
| `docs/` | canonical repository documentation |

Keep platform-specific implementation in the established Windows, Unix, and
macOS project boundaries when those projects are present. Keep shared behaviour
in Core rather than introducing UI-specific service logic.

## Build and package

Read [the build-script reference](../build/scripts/README.md) before creating a
release. From a Windows host, a full build creates a portable ZIP, Setup EXE,
and MSI; with WSL it can also create Linux packages. From Linux, run the Linux
packager directly for DEB, RPM, APK, and Arch output.

```powershell
# Windows, from the repository root
.\build\scripts\build.ps1
.\build\scripts\build.ps1 -SkipTest
.\build\scripts\build.ps1 -SigningMode pfx
```

```bash
# Linux package build, from the repository root
bash build/scripts/linux/build-avalonia-packages.sh
```

macOS is not a release target. Cross-publish can prepare an x64 artifact, but
GUI validation must occur on a real Mac. The Docker guide explains this boundary.

## Signing Windows artifacts

The default signing mode creates a fresh, short-lived self-signed certificate
for that build. It makes the certificate subject traceable, but does not provide
public trust or suppress SmartScreen warnings. Use `-SigningMode pfx` with a
securely supplied `CODE_SIGN_PFX` or `CODE_SIGN_THUMBPRINT` for an existing
certificate. The exact commands and security constraints are in the
[code-signing guide](../build/installer/CODE_SIGNING.md).

## Tests and review

The current suite covers project sanity, game-adapter behaviour, content checks,
and the SubDirectoryFixer installer. It does not replace manual Steam, game, or
desktop UI testing.

Before a pull request:

1. Run the build and relevant tests.
2. Run `git diff --check`.
3. Do not commit generated packages, secrets, certificates, Steam credentials,
   or machine-specific game paths.
4. Update `CHANGELOG.md` for user-visible changes.
5. Update the document whose command, platform support, artifact, or user
   behaviour changed.

## Documentation maintenance

`docs/` is tracked directly in this repository. Do not edit a `wiki/` directory:
it does not exist in the checkout. Documentation is English by default; explain
prerequisites, expected results, limitations, and unsafe cases next to commands.

Use the current source and scripts as evidence. If a feature is intended but not
implemented, label it **planned** and link to the relevant issue or roadmap.
