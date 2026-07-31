# Desktop/Webapp gap matrix — planning snapshot

> **Scope:** This matrix compares this desktop repository with a separate web
> application snapshot from 2026-07-29. Its entries are prioritisation input,
> not a release contract. The desktop source, current user guide, and tested API
> contracts take precedence when they conflict.
>
> **Owner:** product and desktop maintainers. **Revalidation trigger:** before
> reprioritising desktop parity or documenting any listed feature as released.

Stand: 2026-07-29. Status basiert auf Quellcode, nicht allein auf Roadmap-Checkboxen.

| Bereich | Desktop-Evidenz | Webapp-Evidenz | Status | Priorität / nächste Aufgabe |
|---|---|---|---|---|
| Game-Erkennung | `AppSettings`, `GameAdapters/DataCenterGameAdapter` | nicht webverantwortlich | TEILWEISE_IMPLEMENTIERT | GAD-001 Ausführung/Contract abschließen |
| Steam Workshop | `SteamWorkshopService`, `WorkshopDownloadService`, `EditorPage` | `/api/steam/items`, Steam-Sync | TEILWEISE_IMPLEMENTIERT | native Pfade behalten, Web-Metadatenclient ergänzen |
| Auth | `AuthApiClient`, `SessionManager`, `ProtocolSingleInstance` | `/auth/login`, `/auth/token`, `/auth/logout`, Better Auth | TEILWEISE_IMPLEMENTIERT | Profil-/Sessionverlust-/Notification-Flows |
| Modstore-Katalog | Steam-Browse statt Webapp-Katalog | `/api/mods`, `/api/assets`, Suche/Detailseiten | FEHLT | P0-Katalog-Read-Client |
| Projektbearbeitung | `WorkspaceService`, `EditorPage`, Steam-Metadaten | Dashboard/Mod-APIs | TEILWEISE_IMPLEMENTIERT | Webapp-Projekte/Versionen laden und aktualisieren |
| Upload | Steam-UGC-Upload | `/api/mods/submit`, `/api/upload-url`, Moderation | TEILWEISE_IMPLEMENTIERT | Webapp-kompatibler Upload-Assistent |
| lokale Installation | `ModsFolderSyncService` direkte Kopie | bewusst nicht Webapp | FEHLERHAFT | transaktionales Deployment + Besitzjournal |
| Dependencies | `ModDependencyService`, einfache Upload-Prüfung | Mod-/Asset-Relations | TEILWEISE_IMPLEMENTIERT | Resolver mit Versionsbereichen |
| Profile | Collections, keine isolierten Modprofile | Collections/Library | FEHLT | Profile/Lockfile |
| Versionen | Steam PublishedFileId/Change Note | Releases/Changelogs/Status | TEILWEISE_IMPLEMENTIERT | Releasehistorie und kompatible Updates |
| Collections | lokale `ModCollectionService`/Steam-Sync | Collections-CRUD/Share | TEILWEISE_IMPLEMENTIERT | versionierte Web-Revisionen |
| Community | keine native Webapp-Parität | Forum, Tutorials, Docs | FEHLT | zunächst sichere Browseransicht oder native Read-Client |
| Admin/Moderation | keine Desktop-Adminoberfläche | rollenabhängige Admin-APIs | NICHT_FÜR_DESKTOP_RELEVANT | erst nach Nutzer-/Creator-Parität |

## Widersprüche

1. `current_features.md` bezeichnet Webapp-Code als „fertig“, während `functions.md` dieselben Funktionen als Desktop-Gaps führt. Tatsächlicher Desktop-Code entscheidet: Webapp-Funktion ist nicht automatisch Desktop-implementiert.
2. `api-compatibility.md` dokumentiert Auth-Routen, aber der Desktop besitzt noch keinen allgemeinen Client für die übrigen dokumentierten Modstore-/Creator-Routen.
3. Die Roadmap akzeptiert beim Game-Adapter `Deploy`, `Uninstall` und `Launch`; der aktuelle Stand liefert dafür nur validierte Pläne. Daher bleibt GAD-001 `IN_ARBEIT`.
