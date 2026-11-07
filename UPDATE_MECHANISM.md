# Update-Mechanismus – GitHub Releases

> Konzept-/Planungsstand für den in-App-Update-Mechanismus basierend auf den GitHub Releases des gregModmanager-Repositories.

## 1. Ziel

Die Desktop-Anwendung (`greg-app`) soll Nutzer:innen ermöglichen, über GitHub Releases auf neue Versionen aktualisiert zu werden – manuell und optional automatisch.

## 2. Status

**Stand: 2026-10-29 · Version 1.7.7**

Ein In-App-Update-Mechanismus ist noch nicht implementiert. Die folgenden Punkte sind als Konzept, Planung und TODO-Basis zu verstehen.

## 3. Abhängigkeiten und Voraussetzungen

- **Release-Pipeline:** Veröffentlichung von GitHub Releases via `v*` Tags (GitHub Releases des Repositories `mleem97/gregModmanager`); Changelog-Versionierung über `VERSION` bzw. Workspace-`Cargo.toml` als Fallback.
- **Intern:** Bereits vorhandene Bausteine, die für den Update-Flow wiederverwendet werden können:
  - `reqwest` (rustls) – bereits für Modstore-/API-Kommunikation im Workspace vorhanden → für GitHub-REST-API-Nutzung wiederverwendbar.
  - `zip` – für Artifact-Extraction.
  - `tracing`, `anyhow`/`thiserror` – Logging/Fehlerbehandlung.
- **Neu erforderlich (extern):** Checksumm-Verarbeitung (SHA-256), v. a. über einen bisher nicht im Workspace genutzten Checksumm-Crate (z. B. `sha2`); hieraus folgt eine Änderung an `Cargo.toml`/`Cargo.lock` und die Anpassung von `EXTERNAL_DEPENDENCIES.md` sowie `RUST_GRAPH.md` (Repo-Regel).
- **Umgebung:** Netzwerkzugriff auf die GitHub-REST-API (`/repos/{owner}/{repo}/releases/latest`); ggf. Token für Rate-Limits/private Ziele.

## 4. Umfang

### 4.1 In Scope

- Versionsabgleich lokale Version ↔ GitHub Release.
- Asset-Auswahl pro Zielplattform/OS.
- Download + Checksummen-Validierung.
- Installationsstrategie (Portable-Update bzw. Backup + Restore).
- UI-Flow: manuelles Update (z. B. Settings/Top-Bar) und automatischer Check (opt-in bzw. Kanal).
- Post-Update: Versionssynchronisation, Logging, optional Restart.

### 4.2 Out of Scope

- Beta-/Pre-Release-Kanäle (Planung: getrennt, aber mit).
- Paketmanager-basierte Updates (DEB/RPM/APK/Arch) – hier übernimmt der jeweilige Paketmanager; nur portable Update-Pfade in Scope.
- macOS Signing/Notarization bzw. nicht-portable macOS-Update-Pfade.
- Silent-Auto-Update-Default (Entscheidung offen, v. a. Opt-in).

## 5. Implementierungsplan

### Phase 1 – Versions- und Release-Abgleich

- [ ] Lokale Versionsquelle (`VERSION`) mit Release-Tag (`v<Version>`) und Workspace-Fallback vereinheitlichen.
- [ ] Abgleich gegen `releases/latest` bzw. konfigurierten Release-Kanal.

### Phase 2 – GitHub-Release-Client

- [ ] Release-Client (GitHub API) mit Asset-Metadaten (Name, Größen, Browser-Download-URL) und Fehlerbehandlung (Rate Limit, nicht gefunden, kein Asset).
- [ ] Plattform-/OS-Filterung der Assets.

### Phase 3 – Validierung und Download

- [ ] Download des Ziel-Assets.
- [ ] SHA-256-Validierung vor Installation; bei Fehlschlag: Abbruch + lokaler Fallback/Informationsmeldung.

### Phase 4 – Installationsstrategie

- [ ] Portable-Pfad: Backup bestehender Version, Extraction, Restore bei Misserfolg.
- [ ] Cleanup/Rollback.

### Phase 5 – UI-Flow

- [ ] Manuelles Update (Trigger, Fortschritt, Status, Fehler).
- [ ] Automatischer Check (Einstellung, Kanal, Häufigkeit, Opt-in).

### Phase 6 – Post-Update

- [ ] Version in UI/Status synchronisieren.
- [ ] Logging via `tracing`.
- [ ] Optionaler Restart.

## 6. Risiken und offene Entscheidungen

- GitHub-API-Rate-Limits und Authentifizierung (Token) für private/instanzielle Ziele.
- Versionsdrift zwischen `VERSION`, Changelog und Release-Tag (`v…`) – Release-Prozess muss gepflegt werden.
- Netzwerk-/Offline-Fall: Update-Check und Update darf offline sauber scheitern.
- Berechtigungen bei Portable-Update (Schreibrechte Installationsverzeichnis; Linux-Pfade via `dirs`/validated user paths).
- Fehlerbehandlung: Teil-Download, Checksum-Fail, Asset-Mismatch je Plattform.

## 7. Verifikation

- [ ] Unit-/Integrationstests für Release-Client, Asset-Auswahl, Validierung.
- [ ] Manueller Update-Test über GitHub Release (z. B. `vX.Y.Z` → `vX.Y.(Z+1)`).
- [ ] Offline-/Fehlerfall-Szenarien.

## Quellen

- `CONCEPT.md` (Abschnitt 12), `TODO.MD` (Abschnitt „Update-Mechanismus (GitHub Releases)"), `CHANGELOG.md`, `README.md`, `EXTERNAL_DEPENDENCIES.md`, `RUST_GRAPH.md`, `desktop-gap-matrix.md`, `docs/CI-CD.md`.
