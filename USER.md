# USER.md — Project Context

## Repository

- Application: gregModmanager.
- Purpose: cross-platform desktop mod manager for the gregFramework ecosystem.
- UI framework: Avalonia UI 12.1.0.
- Desktop target: .NET 9.
- Runtime-facing helper compatibility: .NET 6 unless explicitly requested and validated.
- Primary solution: `GregModmanager.sln`.

## Important paths

- Desktop app: `src/GregModmanager.Avalonia/`.
- Core services and models: `src/GregModmanager.Core/`.
- SubDirectoryFixer helper: `src/GregModmanager.Melons/SubDirectoryFixer/`.
- Tests: `tests/GregModmanager.Tests/`.
- Build scripts: `build/scripts/`.
- Installer assets: `build/installer/`.
- Canonical documentation: `docs/` (tracked directly in this repository).

## Architecture

- Core remains platform-agnostic and should not reference Avalonia.
- Avalonia references Core.
- Service registration belongs in the Avalonia `Program.cs` composition root.
- Platform-specific behavior should be behind runtime checks or compile-time guards.

## Release context

- Version source of truth: `src/GregModmanager.Avalonia/GregModmanager.Avalonia.csproj`.
- Changelog source of truth: `CHANGELOG.md`.
- Release promotion normally runs through `.github/workflows/promote-changelog.yml`.
