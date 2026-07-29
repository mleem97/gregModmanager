# Architecture

## Architectural Style

Layered cross-platform desktop client with dependency-injected services and event-driven Workshop/session updates. `Program.BuildServices` composes Core services; Avalonia pages consume them; Steam callbacks and pollers publish status events.

Primary constraints are Steam initialization order, platform-specific native libraries, and local game-file safety.

## System Flow

```text
Program/App -> MainWindow -> injected Core service -> Steam/files/Web API -> page/log/dialog
```

Startup preloads Steam and builds DI. `MainWindow` initializes the session and workspace. Pages invoke Workshop/auth/workspace services. Services call native Steamworks, HTTPS or local filesystem APIs. Results return as records/events and are rendered in Avalonia.

## Responsibilities

| Module | Owns | Must not own | Evidence |
|---|---|---|---|
| `Program`/DI | startup, native preload, registrations | domain rules | `Program.cs` |
| Core services | Workshop, auth, workspace, downloads | UI layout | `src/GregModmanager.Core/Services/` |
| Game adapters | Data Center detection, paths, capabilities and plans | arbitrary script execution | `Services/GameAdapters/` |
| Avalonia views | navigation and user actions | generic deployment policy | `Views/` |

## Patterns

| Pattern | Location | Reason |
|---|---|---|
| Dependency injection | `Program.BuildServices` | shared lifetimes and testability |
| Adapter registry | `GameAdapterRegistry` | future games without UI rewrites |
| Event callbacks | `WorkshopSyncOrchestrator`, `SessionManager` | asynchronous status propagation |

## Risks

- General transactional deployment/ownership is not implemented yet.
- Legacy Steam loader and some path fallbacks still contain Data-Center-specific strings; adapter migration is incremental.

## Evidence

- `src/GregModmanager.Avalonia/Program.cs`
- `src/GregModmanager.Avalonia/MainWindow.axaml.cs`
- `src/GregModmanager.Core/Services/GameAdapters/DataCenterGameAdapter.cs`
- `src/GregModmanager.Core/Services/SteamWorkshopService.cs`
