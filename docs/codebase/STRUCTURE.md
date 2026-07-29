# Codebase Structure

## Top-Level Map

| Path | Purpose | Evidence |
|---|---|---|
| `src/GregModmanager.Core/` | Models, services, Steam and persistence | repository scan |
| `src/GregModmanager.Avalonia/` | Desktop entry point, window and pages | `Program.cs`, `MainWindow.axaml.cs` |
| `tests/GregModmanager.Tests/` | xUnit tests | test project |
| `build/` | Packaging scripts | repository tree |
| `docs/` | User/developer documentation | repository tree |

## Entry Points

- Main runtime: `src/GregModmanager.Avalonia/Program.cs`.
- Headless path: `src/GregModmanager.Core/Services/HeadlessRunner.cs`.
- Native plugin project: `src/SubDirectoryFixer/`.
- Avalonia selects desktop lifetime; `--headless` is handled before UI startup.

## Module Boundaries

| Boundary | Owns | Must not own |
|---|---|---|
| Core services | Steam, auth, workspace, downloads and local state | Avalonia controls |
| Game adapters | game detection, paths, capabilities and plans | generic deployment execution |
| Avalonia views | user interaction, navigation and presentation | generic file ownership rules |
| Tests | isolated behavior verification | production side effects |

## Organization

Files use PascalCase C# names and `.axaml`/`.axaml.cs` view pairs. The source is layer-oriented (`Core`, `Avalonia`, `Models`, `Services`, `Steam`); there are no barrel files.

## Evidence

- `GregModmanager.sln`
- `src/GregModmanager.Avalonia/Program.cs`
- `src/GregModmanager.Core/Services/GameAdapters/`
