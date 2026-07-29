# Coding Conventions

## Naming

| Item | Rule | Example | Evidence |
|---|---|---|---|
| Files | PascalCase; views pair `.axaml.cs` | `SteamWorkshopService.cs` | `Services/` |
| Methods | PascalCase; async methods use `Async` | `EnsureCurrentAsync` | installer services |
| Types | PascalCase; interfaces use `I` | `IGameAdapter` | `GameAdapters/IGameAdapter.cs` |
| Environment keys | uppercase snake case | `DATA_CENTER_GAME_DIR` | `AppSettings.cs` |

## Formatting and Quality

No repository-wide formatter was found. Codacy configuration exists under `.codacy/`; nullable analysis and compiler build are active. Run `dotnet build`, `dotnet test` and `git diff --check`.

## Imports

C# namespace imports are explicit where needed; implicit usings are enabled. No barrel files or application-wide alias convention was found.

## Errors and Logging

Expected service failures commonly return result records; boundary exceptions are logged through `AppFileLog`/`AppLogService`; UI displays user-facing text. Diagnostic payloads must not contain tokens. Correlation/operation context is not yet consistent.

## Testing

Tests are in `tests/GregModmanager.Tests/` and named `*Tests.cs`. Existing tests use temporary filesystem fixtures and no mocking framework. No coverage threshold is configured.

## Evidence

- `.codacy/codacy.yaml`
- `src/GregModmanager.Core/Services/AppFileLog.cs`
- `tests/GregModmanager.Tests/GameAdapterTests.cs`
