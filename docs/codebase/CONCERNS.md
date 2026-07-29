# Codebase Concerns

## Top Risks

| Severity | Concern | Evidence | Impact | Action |
|---|---|---|---|---|
| High | No general transaction/ownership store for deployment | `ModsFolderSyncService.cs` | mixed state or user-file deletion risk | implement P0 deployment/ownership |
| High | Most Webapp catalog/creator features lack a desktop API client | `Services/Auth/` vs Webapp `src/app/api/` | parity gap | add versioned catalog client |
| Medium | Adapter launch/profile capabilities are explicitly unsupported | `DataCenterGameAdapter.cs` | UI must not expose them | complete dependent roadmap tasks |
| Medium | .NET 9 runtime absent on analysis host | test output | local test friction | pin/install runtime in CI/docs |

## Technical Debt

| Item | Where | Risk | Fix |
|---|---|---|---|
| Direct Data-Center fallback paths | `SteamApiNativeLoader.cs`, `ModDependencyService.cs` | future adapters diverge | move remaining resolution behind adapter |
| Mutable JSON persistence | `JsonFilePreferences.cs`, `ModCollectionService.cs` | partial writes/schema drift | atomic writes and migrations |

## Security

| Risk | Evidence | Mitigation | Gap |
|---|---|---|---|
| Archive/path traversal | installer/archive services | safe ZIP extraction exists | common package validator and symlink policy |
| Token/log exposure | auth/logging services | redaction helpers | audit all diagnostic payloads |
| Recursive secret upload | Workshop/editor paths | partial preflight | sensitive-file review and confirmation |

## Performance

- Workshop sync copies complete trees in `ModsFolderSyncService`; large collections need cache/deduplication.
- Startup performs runtime checks and network-capable setup from `MainWindow`; offline status should become a non-blocking surface.

## Fragile Areas

- `MainWindow.axaml.cs`: startup, auth, protocol and runtime setup converge in one file; extract an orchestrator.
- `EditorPage.axaml.cs`: large UI/business integration surface; move upload planning to Core.

## `[ASK USER]`

1. [ASK USER] Soll die Desktop-App Forum, Tutorials und Adminseiten nativ nachbauen oder zunächst sicher im Browser öffnen?
2. [ASK USER] Soll der Legacy-GMod-Publisher aus `gregWeb.Modstore/app` in den Desktop-Client übernommen werden?

## Evidence

- `docs/codebase/.codebase-scan.txt`
- `src/GregModmanager.Core/Services/ModsFolderSyncService.cs`
- `src/GregModmanager.Core/Services/GameAdapters/DataCenterGameAdapter.cs`
