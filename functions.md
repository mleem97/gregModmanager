# gregWeb.Modstore – historical function inventory

> **Scope:** This document inventories a separate `gregWeb.Modstore` checkout
> as of 2026-07-29. It is a planning reference for desktop parity, not
> documentation of features available in gregModmanager or a public web API
> contract.
>
> **Owner:** web-app maintainers. **Revalidation trigger:** before using an
> inventory item as a desktop implementation requirement.

Stand: 2026-07-29  
Quelle: `/home/marvin/Dokumente/GregFramework/WebRepositories/gregWeb.Modstore/`

Dieses Dokument ist ein Review-Inventar für die geplante Parität des Desktop-Modmanagers mit der Webapp. Es beschreibt Funktionen, die im Quellcode oder in den API-Routen nachweisbar sind. Es ist keine Aussage darüber, dass jede Funktion aktuell fehlerfrei produktiv funktioniert.

Legende:

- **Bestätigt:** Seite, Komponente oder Route ist im Quellcode vorhanden.
- **Rollenabhängig:** Zugriff wird durch Login, Benutzerrolle, Eigentümerschaft oder interne Authentifizierung begrenzt.
- **Desktop-Gap:** Im aktuellen `gregApp.Modmanager` ist keine entsprechende Funktion erkennbar.
- **[TODO]:** Verhalten oder Backend-Vertrag muss für die Desktop-Implementierung noch verifiziert werden.

## 1. Produktbereiche

Die Webapp besteht funktional aus diesen Bereichen:

| Bereich | Inhalt | Desktop-Relevanz |
| --- | --- | --- |
| Öffentlicher Modstore | Mods, Plugins, Assets, Suche, Detailseiten, Bewertungen, Kommentare, Downloads | Muss im Modstore-Bereich der App verfügbar sein |
| Benutzerkonto | Login, Registrierung, OAuth, Passkeys, 2FA, Profil, Avatar, E-Mail, Benachrichtigungen | Muss als vollständiger Logged-in-State verfügbar sein |
| Creator-/Modmanager-Bereich | Dashboard, eigene Mods, Upload-Assistent, Releases, Dokumentation, Import aus Git | Muss im Modmanager-Bereich verfügbar sein |
| Community | Collections, Forum, Tutorials, öffentliche Profile | Je nach Produktumfang in der App verfügbar machen |
| Admin/Moderation | Freigaben, Security-Triage, Nutzer-/Rollenverwaltung, Discord, Steam, Settings, Storage | Nur für berechtigte Rollen; nicht blind in die normale Nutzeroberfläche übernehmen |
| Integrations- und System-APIs | Steam, Discord, Security Pipeline, MinIO, Deployment, Bot, Scanner | Desktop nutzt nur die für den Benutzerworkflow erforderlichen APIs |
| Legacy-GMod-Publisher | Separate Svelte/Tauri-Oberfläche unter `app/` und `src-tauri/` | [TODO] Prüfen, ob dieser Teil noch Bestandteil der gewünschten Parität ist |

## 2. Öffentliche Funktionen ohne Login

### Startseite

Quelle: `src/app/[locale]/page.tsx`

- Sucheingabe mit Weiterleitung zur Suche.
- Einstieg in Modsuche, Upload, Dokumentation und Community.
- Kategorie-/Bereichskarten.
- Hervorgehobene beziehungsweise aktuelle Mods.
- Links zu Asset-Upload und Entwickler-/Dokumentationsbereichen.
- Einstieg in Forum und Tutorials.
- Öffentliche Status-/Informationsbereiche.

### Suche und Browse

Quelle: `src/app/[locale]/search/page.tsx`, `src/app/[locale]/search/SearchFilters.tsx`

- Volltext-/Titel-Suche über Query `q`.
- Suche nach Kategorien und Tags.
- Filter für Mods und Assets.
- Sortierung/Paginierung beziehungsweise Ergebnisbegrenzung. [TODO: exakte UI- und Defaultwerte prüfen]
- Öffnen einer Mod-Detailseite.
- Öffnen einer Asset-Detailseite.
- Öffnen von Plugin-/Asset-Bereichen.

### Mod-Detailseite

Quelle: `src/app/[locale]/mod/[id]/page.tsx`, `src/components/ModCard.tsx`, `src/components/ReportButton.tsx`

- Titel, Beschreibung, Autor, Tags, Kategorie und Metadaten anzeigen.
- Download-Link beziehungsweise Download über Mod-ID.
- `greg://install?modId=<id>` für Installation über den Desktop-Modmanager.
- GitHub-/Homepage-/Dokumentationslinks.
- Autorprofil öffnen.
- Tags als Suchfilter verwenden.
- Favorisieren/Entfavorisieren, wenn eingeloggt.
- Bewerten und Reviewtext verwalten, wenn eingeloggt.
- Kommentare anzeigen und schreiben, wenn eingeloggt.
- Mod melden.
- Watchlist/Update-Benachrichtigungen aktivieren oder deaktivieren.
- Release-/Versionshistorie anzeigen.
- Abhängigkeiten und Installationsinformationen anzeigen.
- Downloadzähler erhöhen.
- Steam-Workshop-Verknüpfung/Steam-Informationen anzeigen, sofern vorhanden.

### Plugins

Quelle: `src/app/[locale]/plugins/page.tsx`

- Plugin-Übersicht öffnen.
- Plugin-Suche/Browse über die allgemeine Suche.
- Navigation zu Asset- und Mod-Bereichen.

### Assets

Quellen: `src/app/[locale]/assets/page.tsx`, `src/app/[locale]/assets/[slug]/page.tsx`, `src/app/[locale]/assets/models/page.tsx`, `src/app/[locale]/assets/upload/page.tsx`

- Asset-Kategorien anzeigen.
- Asset-Typen, insbesondere Modelle, anzeigen.
- Asset-Suche und Asset-Filter.
- Asset-Detailseite öffnen.
- Vorschaubilder und Asset-Metadaten anzeigen.
- Asset herunterladen.
- Assets favorisieren.
- Assets bewerten und kommentieren, sofern die Detailseite dies anbietet.
- Neues Asset hochladen, mit Login und Berechtigungsprüfung.
- Asset-Metadaten beziehungsweise Versionen verwalten. [TODO: genaue UI-Felder je Asset-Typ prüfen]

### Dokumentation und Tutorials

Quellen: `src/app/[locale]/docs/page.tsx`, `src/app/[locale]/docs/[slug]/page.tsx`, `src/app/[locale]/tutorials/page.tsx`

- Dokumentationsübersicht.
- Dokumentationsseite per Slug.
- Tutorialübersicht.
- Tutorial anzeigen.
- Tutorial erstellen, bearbeiten und löschen, wenn der eingeloggte Benutzer Autor ist.
- Öffentliche Dokumentation zu Mods/Plugins.

### Community und Forum

Quellen: `src/app/[locale]/forum/page.tsx`, `src/app/[locale]/forum/new/page.tsx`, `src/app/[locale]/forum/[id]/page.tsx`, `src/app/[locale]/collections/page.tsx`

- Forumkategorien anzeigen.
- Threads nach Kategorie anzeigen.
- Thread-Detailseite öffnen.
- Neuen Thread erstellen, wenn eingeloggt.
- Beiträge lesen und erstellen, wenn eingeloggt.
- Collections anzeigen.
- Collection-Detailseite anzeigen.
- Öffentliche Share-Links für Collections verwenden.
- Collection erstellen und verwalten, wenn eingeloggt.

## 3. Authentifizierung und Konto

Quellen: `src/lib/auth.ts`, `src/components/UserMenu.tsx`, `src/app/[locale]/login/page.tsx`, `src/app/[locale]/onboarding/page.tsx`, `src/app/[locale]/setup/page.tsx`

- E-Mail-/Passwort-Registrierung.
- Login und Logout.
- E-Mail-Verifikation.
- Passwort vergessen.
- Passwort zurücksetzen.
- GitHub OAuth.
- Discord OAuth.
- Account-Linking zwischen vertrauenswürdigen Providern.
- Passkey/WebAuthn registrieren.
- Vorhandene Passkeys auflisten.
- Passkey löschen.
- Zwei-Faktor-Authentifizierung über better-auth-Plugin.
- Benutzer-Onboarding.
- Erstinstallation/Setup des ersten Admin-Benutzers.
- Login-/Registrierungs-Sperre über Site-Konfiguration.
- Rollen im Session-/User-Kontext.
- Benutzeravatar anzeigen.
- Benutzeravatar hochladen und aktualisieren.
- Profil anzeigen und bearbeiten.
- E-Mail-Adresse ändern.
- Benachrichtigungen anzeigen, als gelesen markieren und löschen.
- Session-Cookie ohne veralteten User-Cache; Rollenänderungen werden neu gelesen.

### Account-Menü

Quelle: `src/components/UserMenu.tsx`

Für eingeloggte Benutzer:

- Dashboard öffnen.
- Profil bearbeiten.
- Eigene Collections öffnen.
- Mod hochladen.
- Admin-Dashboard öffnen, wenn Rolle `admin`, `moderator` oder `superadmin`.
- Abmelden.

Für Gäste:

- Anmelden.
- Konto erstellen.

## 4. Eingeloggter Benutzerbereich / Creator

### Dashboard

Quelle: `src/app/[locale]/dashboard/page.tsx`

- Zusammenfassung der eigenen Mod-Aktivität.
- Eigene Mods öffnen.
- Benachrichtigungen öffnen.
- Upload starten.
- Aktuelle/zuletzt bearbeitete Mods öffnen.
- Creator-Bereich öffnen.
- Bibliothek öffnen.
- Community/Collections öffnen.
- API-Key-Bereich öffnen.
- Account-/App-Einstellungen öffnen.

### Eigene Mods

Quelle: `src/app/[locale]/dashboard/mods/page.tsx`

- Eigene Mods auflisten.
- Modstatus und Veröffentlichungsstatus anzeigen.
- Mod-Detailseite öffnen.
- Mod-Dokumentation öffnen.
- Neue Version beziehungsweise neuer Upload starten.
- Mod verwalten. Die konkrete Bearbeitungsaktion ist teilweise über das Dashboard/API verteilt; [TODO] vollständigen Edit-Flow gegen die Webapp testen.

### Creator Dashboard

Quelle: `src/app/[locale]/dashboard/creator/page.tsx`

- Creator-Übersicht.
- Eigene veröffentlichte und ausstehende Mods anzeigen.
- Upload starten.
- Zu eigenen Mods, Bibliothek und Community navigieren.
- Creator-Metriken/Status anzeigen, soweit vorhanden.

### Bibliothek

Quelle: `src/app/[locale]/dashboard/library/page.tsx`

- Eigene beziehungsweise gespeicherte Mods anzeigen.
- Collections anzeigen.
- Collection erstellen.
- Collection-Detail öffnen.
- Modsuche öffnen.
- Upload starten.

### Community-Dashboard

Quelle: `src/app/[locale]/dashboard/community/page.tsx`

- Community-Übersicht.
- Collections öffnen/erstellen.
- Forum öffnen.
- Thread erstellen.
- Community-Aktivitäten [TODO: genaue Datenquelle und Aktionen prüfen].

## 5. Upload- und Veröffentlichungsworkflow

Quellen: `src/components/upload/UploadWizard.tsx`, `src/app/[locale]/upload/page.tsx`, `src/app/api/mods/submit/route.ts`, `src/app/api/upload-url/route.ts`

Der Upload-Assistent hat fünf Schritte:

1. **Methode auswählen**
2. **Metadaten erfassen**
3. **Bilder hochladen**
4. **Review**
5. **Entscheidung/Ergebnis**

### Uploadmethoden

- Git/GitHub-Import.
- ZIP-Datei.
- DLL.
- Lua-Datei.
- Python-Datei.
- Go-Datei.
- 3D-Modelle: OBJ, FBX, GLTF, GLB oder ZIP.
- Texturen: PNG, JPG, WebP, TGA, DDS oder ZIP.
- Animationen: FBX, ANIM, GLTF, GLB oder ZIP.

### Upload-Metadaten

- Titel.
- Version.
- Kategorie.
- Beschreibung.
- Lizenztext und Lizenztyp.
- Abhängigkeiten.
- Kompatibilitätshinweise.
- Sicherheitshinweise.
- GitHub-URL.
- Homepage.
- Kontakt-E-Mail.
- Spenden-URL.
- Minimale Spielversion.
- Empfohlene Spielversion.
- Tags.
- Installationsanleitung.
- Support-URL.
- Cover-/Galeriebilder.

### Uploadausführung

- Authentifizierung prüfen.
- Rate-Limit prüfen.
- Upload-URL über MinIO/S3-Presign anfordern.
- Datei direkt in Object Storage hochladen.
- Bilder einzeln hochladen.
- GitHub-Repository beziehungsweise Release importieren.
- Mod, Release, Bilder und Moderationsqueue erzeugen.
- Security-Scan beziehungsweise Security-Pipeline einplanen.
- Scanstatus anzeigen: `passed`, `rejected`, `needs_review`, `manual_required`, `auto_published`.
- Bei erfolgreicher Verarbeitung Mod-ID anzeigen.
- Discord-/Moderationsbenachrichtigungen auslösen.
- Bei Fehlern verständliche Uploadmeldung anzeigen.

### Git-Import

- GitHub-Repository prüfen.
- Release-Tag oder Release-Asset ermitteln.
- Unterstützten Repository-Host prüfen.
- Release-Datei laden.
- Release als Modversion übernehmen.
- Optionales Zugriffstoken für private/restricted Releases verwenden.
- Webhook-basierte spätere Release-Updates unterstützen.

## 6. Mod-Interaktion und Versionsverwaltung

Ermittelte API-Funktionen:

- Modstatus anzeigen.
- Mod genehmigen oder ablehnen, rollenabhängig.
- Kommentare laden, erstellen und löschen.
- Favoriten setzen und entfernen.
- Bewertungen laden, erstellen, aktualisieren beziehungsweise löschen.
- Watchlist setzen und entfernen.
- Update-Benachrichtigungen für beobachtete Mods verwalten.
- Releases auflisten.
- Neue Releaseversion anlegen.
- Mod-Dokumentation hochladen.
- Mod aus Git importieren.
- Mod melden.
- Download starten und Downloadzähler erhöhen.
- Steam-Workshop-ID anzeigen beziehungsweise Mod mit Steam-Workshop-Daten verbinden.

## 7. Collections

Quellen: `src/app/[locale]/collections/*`, `src/app/api/collections/*`, `src/lib/collections.ts`

- Collections auflisten.
- Collection erstellen.
- Collection anzeigen.
- Collection bearbeiten.
- Collection löschen.
- Mods zu Collection hinzufügen.
- Mods aus Collection entfernen.
- Collection per Share-Link lesen.
- Share-Link erzeugen beziehungsweise verwalten.
- Collectiontitel, Beschreibung, Sichtbarkeit und Slug verwalten. [TODO: genaue Felder prüfen]

## 8. Benachrichtigungen

Quellen: `src/app/[locale]/dashboard/notifications/page.tsx`, `src/app/api/notifications/route.ts`, `src/app/api/notifications/[id]/route.ts`

- Benachrichtigungen laden.
- Alle Benachrichtigungen als gelesen markieren.
- Einzelne Benachrichtigung aktualisieren.
- Einzelne Benachrichtigung löschen.
- Update-/Moderations-/Community-Ereignisse anzeigen.

## 9. Benutzerprofil

Quellen: `src/app/[locale]/profile/[username]/page.tsx`, `src/app/api/user/profile/route.ts`, `src/app/api/user/avatar/route.ts`, `src/app/api/user/update-email/route.ts`

- Öffentliches Profil per Username anzeigen.
- Benutzername/Anzeigename anzeigen.
- Avatar anzeigen.
- Rollen-/Profilinformationen anzeigen, abhängig von Sichtbarkeit.
- Eigene Mods/Beiträge/Collections anzeigen, abhängig von UI.
- Profil bearbeiten.
- Bio beziehungsweise Profildaten aktualisieren.
- Avatar hochladen.
- E-Mail-Adresse ändern.
- Rolle aus Session beziehungsweise Benutzerprofil laden.

## 10. Admin- und Moderationsfunktionen

Quelle: `src/app/[locale]/admin/*`, `src/lib/admin.ts`

### Admin-Dashboard

- Systemübersicht.
- Kennzahlen und Metriken.
- Moderationsstatus.
- Security-/GregScan-Status.
- App-Status.
- Links zu Nutzer-, Mod-, Discord-, Steam-, Storage- und Settings-Verwaltung.

### Modverwaltung

- Mods nach Status/Tabs filtern: pending, published, rejected, disabled beziehungsweise weitere im Code vorhandene Zustände.
- Mod-Details öffnen.
- Öffentliche Modseite öffnen.
- Releasehistorie prüfen.
- Mod genehmigen.
- Mod ablehnen.
- Mod deaktivieren beziehungsweise Kill-Switch setzen.
- Security-Gate-Zustand prüfen.
- Moderationsnotizen und Queue-Status verwalten.

### Moderationsqueue

- Queue laden.
- Nach Suchtext, Typ, Status und Risiko filtern.
- Queue-Fall auf `reviewing`, `resolved` oder `dismissed` setzen.
- Mod genehmigen.
- Mod ablehnen.
- Link zur öffentlichen Seite öffnen.
- Link zur Admin-Review öffnen.
- Risikostufe und Security-Details prüfen.

### GregScan / Security-Triage

- Scanberichte laden.
- Nach Status und Threat/Risk filtern.
- Findings zusammenfassen.
- Scanbericht als approved oder rejected auflösen.
- Mod-Review beziehungsweise öffentliche Modseite öffnen.

### Nutzer- und Rollenverwaltung

- Nutzerliste anzeigen.
- Nutzerrolle ändern.
- Moderationsaktion auf Nutzer anwenden.
- Discord-Rollen-Mappings verwalten.
- Höchste/effektive Rolle des Benutzers für UI und Berechtigungen verwenden.

### Admin-Settings und Feature Flags

- Settings-Gruppen anzeigen.
- Settings-Gruppe öffnen.
- Site-/App-Konfiguration ändern.
- Feature Flags laden.
- Feature Flag aktivieren/deaktivieren.
- App-Status auf `maintenance`, `public`, `devmode` oder `testing` setzen.
- Maintenance-Bypass temporär setzen und entfernen.

### API-Keys

- API-Keys auflisten.
- API-Key erstellen.
- Rohschlüssel nur einmal anzeigen.
- API-Key löschen.

### Audit und Observability

- Audit-Logs anzeigen.
- Metriken anzeigen.
- Admin-Health prüfen.
- Storage-Metriken anzeigen.
- Deploymentstatus anzeigen.
- Discord-Botstatus, Logs und Live-/Webhookzustände anzeigen.

### Discord-Verwaltung

- Bot-Konfiguration lesen und ändern.
- Botstatus prüfen.
- Botlogs laden.
- Guild-Rollen laden.
- Guild-Channels laden.
- Discord-Rollen-Mappings erstellen, ändern und löschen.
- Invite-Link öffnen.
- Discord-Test-/Announcement-Funktionen auslösen.
- Telegram-Admins verwalten.
- Telegram-Testnachricht senden.

### Steam-Verwaltung

- Steam-Workshop-Sync starten.
- Einzelnen Steam-Workshop-Item importieren.
- Steam-Workshop-Items und Syncstatus prüfen.
- Steam-Daten in lokale Moddatensätze überführen.

### Dokumentations- und Storage-Administration

- Hosted Docs auflisten.
- Dokumentation öffnen.
- Storage-Buckets/Metriken anzeigen.
- Scannerstatus und letzte Scans anzeigen.

## 11. API-Inventar – öffentliche und Benutzer-APIs

Alle folgenden Routen wurden unter `src/app/api` gefunden. HTTP-Methoden entsprechen den exportierten Route-Handlern.

| Route | Methoden | Funktion |
| --- | --- | --- |
| `/api/app-auth` | POST, DELETE | Desktop-/App-Authentifizierung beziehungsweise Session beenden |
| `/api/client-session` | POST | Client-Session erzeugen/validieren |
| `/api/assets` | GET, POST | Assets suchen/listen und erstellen |
| `/api/assets/categories` | GET, POST | Asset-Kategorien lesen und verwalten |
| `/api/assets/[id]` | GET, PATCH, DELETE | Asset laden, bearbeiten, löschen |
| `/api/assets/[id]/dependencies` | GET, POST, DELETE | Asset-Abhängigkeiten verwalten |
| `/api/assets/[id]/download` | GET, POST | Assetdownload und Downloadverarbeitung |
| `/api/assets/[id]/favorite` | POST, DELETE | Assetfavorit setzen/entfernen |
| `/api/assets/[id]/ratings` | GET, POST, DELETE | Assetbewertungen verwalten |
| `/api/assets/[id]/versions` | GET, POST | Assetversionen laden/anlegen |
| `/api/auth/[...better-auth]` | Framework-Handler | better-auth: Login, Registrierung, Sessions, OAuth, Passwort-/E-Mail-Flows |
| `/api/auth/passkey/list` | GET | Passkeys auflisten |
| `/api/auth/passkey/register` | POST | Passkey-Registrierung starten |
| `/api/auth/passkey/register/verify` | POST | Passkey-Registrierung bestätigen |
| `/api/auth/passkey/[id]` | DELETE | Passkey löschen |
| `/api/auth/steam` | GET | Steam-Login starten |
| `/api/auth/steam/callback` | GET | Steam-Login-Callback verarbeiten |
| `/api/collections` | GET, POST | Collections auflisten/erstellen |
| `/api/collections/[id]` | GET, PATCH, DELETE | Collection laden/bearbeiten/löschen |
| `/api/collections/[id]/mods` | POST, DELETE | Mod zur Collection hinzufügen/entfernen |
| `/api/collections/share` | GET, POST | Collection-Share-Link lesen/erstellen |
| `/api/docs` | GET, POST | Hosted Docs laden/erstellen |
| `/api/forum/categories` | GET, POST | Forumkategorien lesen/verwalten |
| `/api/forum/threads` | GET, POST | Threads laden/erstellen |
| `/api/forum/threads/[id]` | GET | Thread laden |
| `/api/forum/threads/[id]/posts` | GET, POST | Beiträge laden/erstellen |
| `/api/mods/submit` | POST | Mod-Upload und Moderationssubmission |
| `/api/mods/[id]/status` | GET | Eigentümerbezogenen Modstatus laden |
| `/api/mods/[id]/comments` | GET, POST, DELETE | Modkommentare verwalten |
| `/api/mods/[id]/ratings` | GET, POST, DELETE | Modbewertungen verwalten |
| `/api/mods/[id]/favorite` | POST, DELETE | Modfavorit verwalten |
| `/api/mods/[id]/watch` | POST, DELETE | Mod-Watchlist verwalten |
| `/api/mods/[id]/docs/upload` | POST | Moddokumentation hochladen |
| `/api/mods/[id]/import-from-git` | POST | Release/Mod aus Git importieren |
| `/api/mods/[id]/approve` | POST | Mod genehmigen, rollenabhängig |
| `/api/mods/[id]/reject` | POST | Mod ablehnen, rollenabhängig |
| `/api/notifications` | GET, POST | Benachrichtigungen laden/als gelesen markieren |
| `/api/notifications/[id]` | PATCH, DELETE | Benachrichtigung bearbeiten/löschen |
| `/api/report` | POST | Inhalt/Mod melden |
| `/api/steam/items` | GET, POST | Steam-Workshop-Items laden/verknüpfen |
| `/api/tutorials` | GET, POST | Tutorials laden/erstellen |
| `/api/tutorials/[id]` | GET, PATCH, DELETE | Tutorial laden/bearbeiten/löschen |
| `/api/upload-url` | POST | Presigned Upload-URL für Mod/Bild erzeugen |
| `/api/user/avatar` | POST | Avatar hochladen |
| `/api/user/profile` | GET, POST | eigenes Profil laden/speichern |
| `/api/user/update-email` | POST | E-Mail-Adresse ändern |
| `/api/keys` | GET, POST | Benutzer-API-Keys laden/erstellen |
| `/api/keys/[id]` | DELETE | Benutzer-API-Key löschen |
| `/api/status` | GET | öffentlicher Appstatus |
| `/api/version` | GET | Release-/Versionsstatus |

## 12. API-Inventar – Admin-APIs

| Route | Methoden | Funktion / Berechtigung |
| --- | --- | --- |
| `/api/admin/api-keys` | GET, POST | Admin-API-Keys auflisten/erstellen; strict admin |
| `/api/admin/api-keys/[id]` | DELETE | Admin-Key löschen |
| `/api/admin/app-status` | GET, POST | Appstatus lesen/ändern |
| `/api/admin/audit-logs` | GET | Auditlogs lesen |
| `/api/admin/health` | GET | Health-/DB-/Systemprüfung |
| `/api/admin/metrics` | GET | Adminmetriken |
| `/api/admin/storage-metrics` | GET | Storage-Metriken |
| `/api/admin/maintenance-bypass` | POST, DELETE | Maintenance-Bypass setzen/entfernen |
| `/api/admin/feature-flags` | GET | Feature Flags laden |
| `/api/admin/feature-flags/[key]` | PATCH | Feature Flag ändern |
| `/api/admin/moderation-queue` | GET, PATCH | Moderationsqueue laden/statusändern |
| `/api/admin/moderate` | POST | Nutzer-/Moderationsaktion |
| `/api/admin/mods/[id]/approve` | POST | Mod freigeben |
| `/api/admin/mods/[id]/reject` | POST | Mod ablehnen |
| `/api/admin/mods/[id]/kill-switch` | POST | Mod deaktivieren/Kill-Switch |
| `/api/admin/gregscan-reports` | GET | Securityberichte laden |
| `/api/admin/gregscan-reports/[id]/resolve` | POST | Securitybericht auflösen |
| `/api/admin/users/[id]/role` | POST | Benutzerrolle ändern |
| `/api/admin/steam-sync` | POST | Steam-Workshop-Synchronisierung |
| `/api/admin/steam-import` | POST | Steam-Item als Mod importieren |
| `/api/admin/discord-bot-config` | GET, PATCH | Discord-Bot-Konfiguration |
| `/api/admin/discord-bot-logs` | GET | Discord-Bot-Logs |
| `/api/admin/discord-bot-status` | GET | Discord-Bot-Status |
| `/api/admin/discord-guild-roles` | GET | Discord-Guild-Rollen |
| `/api/admin/discord-guild-channels` | GET | Discord-Guild-Kanäle |
| `/api/admin/discord-role-mappings` | GET, POST, PATCH, DELETE | Rollen-/Berechtigungsmappings |
| `/api/admin/telegram-admins` | GET, POST, DELETE | Telegram-Admins |
| `/api/admin/telegram-test` | POST | Telegram-Testnachricht |

## 13. API-Inventar – interne Integrationen

Diese Routen sind nicht normale Benutzerfunktionen. Sie benötigen interne Schlüssel, HMAC-Signaturen, Bot-/Service-Authentifizierung oder spezielle Infrastruktur.

| Route | Methoden | Funktion |
| --- | --- | --- |
| `/api/internal/assets` | GET, POST | interne Assetverwaltung |
| `/api/internal/mods` | GET, POST, PATCH | interne Modverwaltung |
| `/api/internal/mods/[id]/verify` | GET, POST | interne Modverifikation |
| `/api/internal/downloads` | GET, POST | interne Download-/Zählerverarbeitung |
| `/api/internal/stats` | GET | interne Statistiken |
| `/api/internal/health` | GET | interner Healthcheck |
| `/api/internal/minio/upload-url` | GET, POST | interne Object-Storage-URLs |
| `/api/internal/scan/results` | POST | Scanresultate empfangen |
| `/api/internal/scanner/queue` | GET, POST | Scannerqueue steuern |
| `/api/internal/scanner/status` | GET, POST | Scannerstatus |
| `/api/internal/security/results` | POST | Security-Pipeline-Webhook |
| `/api/internal/bot/upload` | POST | Discord-Bot-Upload |
| `/api/internal/bot-tenants` | GET, POST, DELETE | Bot-Tenants verwalten |
| `/api/internal/bot/premium/[guildId]` | GET | Premiumstatus für Guild |
| `/api/internal/discord/announce` | POST | Discord-Ankündigung |
| `/api/internal/discord/config` | GET | interne Discordkonfiguration |
| `/api/internal/discord/role-sync` | POST | Discord-Rollensynchronisierung |
| `/api/internal/discord/webhooks` | GET, POST | Discord-Webhooks |
| `/api/internal/deploy` | POST | interne Deployment-Aktion |
| `/api/internal/deployment` | GET, POST | Deploymentstatus/-steuerung |
| `/api/webhooks/git` | POST | GitHub-/Git-Release-Webhook |
| `/api/forum/discord-webhook/post` | POST | Forumbeitrag über Discordintegration |
| `/api/steam/items` | GET, POST | Steam-Datenzugriff; je nach Aktion öffentlich/adminnah |
| `/api/setup` | POST | initiales Setup, nur solange noch kein Setup abgeschlossen ist |

## 14. Legacy-/Tauri-GMod-Publisher-Funktionen

Im selben Repository existiert zusätzlich eine separate Svelte-/Tauri-Anwendung unter `app/` und `src-tauri/`. Diese ist nicht mit den Next.js-Modstore-Seiten gleichzusetzen.

Nachweisbare Seiten unter `app/pages/`:

- `AddonSizeAnalyzer.svelte`: Addon-Größenanalyse.
- `ContentGenerator.svelte`: Content-/Addon-Generierung.
- `Downloader.svelte`: Downloads/Downloadjobs.
- `InstalledAddons.svelte`: installierte Addons verwalten.
- `MyWorkshop.svelte`: eigene Workshopitems.
- `Subscriptions.svelte`: abonnierte Workshopitems.

Nachweisbare Tauri-/Rust-Funktionsbereiche:

- GMA-Datei lesen, schreiben, extrahieren und Preview erzeugen.
- Content-Whitelist und Pfadsicherheit.
- Addon-Analyse.
- Downloadverwaltung.
- Spiel-Addonverwaltung.
- Steam-Users, Workshop, Downloads, Publishing und Subscriptions.
- Appdaten, Logging, CLI und Webview.
- Transaktionen/WebSocket.

**Entscheidung für den Desktop-Modmanager:** [TODO] Festlegen, ob diese Legacy-GMod-Publisher-Funktionen ebenfalls in `gregApp.Modmanager` übernommen werden sollen oder ob nur die Next.js-Modstore-Funktionen gemeint sind.

## 15. Paritätsanforderungen für gregApp.Modmanager

### Muss im eingeloggten Benutzerbereich vorhanden sein

- Vollständiger Session-State mit Login, Logout und Sessionverlust.
- Profilbild, Anzeigename, E-Mail und höchste aktive Rolle.
- Profil anzeigen/bearbeiten.
- Avatar ändern.
- E-Mail ändern.
- Benachrichtigungen.
- Dashboard.
- Eigene Mods.
- Upload-Assistent mit allen Metadatenfeldern.
- Uploadmethoden: Git, ZIP, DLL, Lua, Python, Go, Models, Textures, Animations.
- Direkter MinIO/S3-Upload mit Fortschritt und Fehlerdetails.
- Security-/Moderationsstatus nach Upload.
- Neue Releases/Versionen für bestehende Mods.
- Releasehistorie.
- Git-Import und Git-Webhook-basierte Updates.
- Mod-Dokumentation.
- Collections und Share-Links.
- Favoriten, Ratings, Kommentare und Watchlist.
- Download und `greg://install?modId=...`.
- Suche nach Mods, Plugins und Assets.
- Mod-/Asset-Detailseiten.

### Muss im Modstore-Bereich vorhanden sein

- Öffentliche Suche und Filter.
- Mods, Plugins und Assets getrennt browsen.
- Kategorien und Tags.
- Detailseiten mit Autor, Versionen, Abhängigkeiten, Bewertungen und Kommentaren.
- Download-/Installieren-Aktion.
- Favorisieren und Beobachten.
- Steam-Workshop-Daten, wo verknüpft.
- Öffentliche Profile und Collections.
- Forum, Tutorials und Dokumentation, wenn die App vollständige Webparität beansprucht.

### Rollenabhängig oder nicht in die normale Nutzer-App übernehmen

- Admin-Dashboard.
- Moderation und Security-Triage.
- Nutzerrollenverwaltung.
- Kill-Switch.
- API-Key-Administration.
- Discord-/Telegram-Administration.
- Steam-Sync/-Import.
- Storage-/Deployment-/Health-Administration.
- Interne Bot-/Security-/Scanner-Routen.

## 16. Wichtige Integrationsverträge

- **Desktop ↔ Webapp:** `greg://install?modId=<modId>` wird von `src/components/ModCard.tsx` erzeugt und vom Desktop registriert/verarbeitet.
- **Discord-Bot ↔ Webapp:** `/api/internal/bot/upload` mit internem Schlüssel und Multipart-Datei.
- **Security-Pipeline ↔ Webapp:** `/api/internal/security/results` mit signiertem Ergebnis-Webhook.
- **GitHub ↔ Webapp:** `/api/webhooks/git` für Release-/Tag-Updates.
- **Steam ↔ Webapp:** Steam-Web-API und Workshop-Sync über `src/lib/steam-workshop.ts` und Admin-Routen.
- **Object Storage:** Presigned URLs über `/api/upload-url` beziehungsweise interne MinIO-Routen.

## 17. Priorisierte Umsetzung im Desktop

### Phase 1 – zwingende Parität für Modmanager

1. Gemeinsamen Logged-in-State und sichere Session-Synchronisierung.
2. Profilbild, Benutzername, höchste Rolle und Dropdown-Menü.
3. Webapp-kompatibler Upload-Assistent mit allen Metadaten.
4. Bestehende Mods laden, bearbeiten und neue Versionen anlegen.
5. Releasehistorie und Uploadstatus.
6. Git-/GitHub-Import und Git-Release-Updates.
7. Eigene Mods, Benachrichtigungen und Profilverwaltung.

### Phase 2 – Modstore-Funktionen

1. Suche, Filter, Kategorien und Tags.
2. Mod-/Plugin-/Asset-Detailseiten.
3. Download, Installation und `greg://`-Weiterleitung.
4. Favoriten, Ratings, Kommentare und Watchlist.
5. Collections und öffentliche Profile.
6. Steam-Workshop-Integration.

### Phase 3 – Community und Creator-Erweiterungen

1. Forum und Threads.
2. Tutorials und Dokumentation.
3. Asset-Upload und Asset-Versionen.
4. Creator-Dashboard und Community-Dashboard.

### Phase 4 – Rollenabhängige Adminfunktionen

1. Moderation und Security-Queue.
2. Nutzer-/Rollenverwaltung.
3. Steam-Sync/-Import.
4. Discord-/Telegram-Verwaltung.
5. Storage, Audit, Health, Metrics und Deployment.

## 18. Reviewfragen

1. Soll die Desktop-App nur die Next.js-Modstore-Funktionen übernehmen oder auch die separate Legacy-GMod-Publisher-Anwendung unter `app/`/`src-tauri/`?
2. Soll die Desktop-App Forum, Tutorials, Dokumentation und öffentliche Profile vollständig nachbilden oder nur verlinken/öffnen?
3. Welche Rollen dürfen im Desktop-Adminbereich sichtbar sein: nur `admin`/`superadmin`, oder auch `moderator` und `appTester`?
4. Soll die App Asset-Uploads vollständig gleichwertig zu Mod-Uploads unterstützen?
5. Soll die Desktop-App eigene API-Keys verwalten oder bleiben API-Key-Funktionen Web-only?
6. Welche Webapp-Felder sind für die Steam-Workshop-Version verbindlich und welche bleiben nur für Web-Mods relevant?
7. Welcher Backend-Endpoint ist der verbindliche Desktop-Session-Vertrag: `/api/app-auth`, better-auth oder ein separater Desktop-Token-Flow?
8. Unterstützt die Webapp aktuell eine Avatar-URL in der Desktop-Sessionantwort, oder muss der Desktop das Profil separat über `/api/user/profile` laden?

## 19. Belege

- Webapp-Intent und Integrationsverträge: `PROJECT.md`, `README.md`.
- Navigation und Logged-in-Menü: `src/components/sidebar/SidebarClient.tsx`, `src/components/UserMenu.tsx`, `src/components/navbar/Navbar.tsx`.
- Upload: `src/components/upload/UploadWizard.tsx`, `src/app/api/mods/submit/route.ts`, `src/app/api/upload-url/route.ts`.
- Auth: `src/lib/auth.ts`, `src/app/api/auth/*`, `src/app/api/user/*`.
- Mod-/Steam-Logik: `src/lib/mods.ts`, `src/lib/steam-workshop.ts`, `src/app/api/mods/*`, `src/app/api/steam/items/route.ts`.
- Admin-/Moderation: `src/lib/admin.ts`, `src/app/[locale]/admin/*`, `src/app/api/admin/*`.
- Security-Integrationen: `security/*`, `src/app/api/internal/security/results/route.ts`, `src/app/api/internal/scan/results/route.ts`.
- Desktop-Integration: `src/components/ModCard.tsx`, `PROJECT.md`.

## 20. Desktop-Gap-Matrix (erster Auditstand)

| Funktionsgruppe | Webapp | Desktop-Quellcode | Status | Nächster Schritt |
|---|---|---|---|---|
| Game-Adapter/Erkennung | kein normaler Webapp-Bereich | `Services/GameAdapters/`, `AppSettings` | IMPLEMENTIERT | Deployment-/Profile-Funktionen darauf aufbauen |
| Workshop Browse/Download/Upload | Steam-Websync/Metadaten | `SteamWorkshopService`, `WorkshopDownloadService` | TEILWEISE_IMPLEMENTIERT | Katalog-API und native Installationsplanung verbinden |
| Auth/Login/Logout/Refresh | `/auth/login`, `/auth/token`, `/auth/logout` | `Services/Auth/` | TEILWEISE_IMPLEMENTIERT | Sessionverlust/Profile/Benachrichtigungen vollständig verdrahten |
| Modstore-Suche/Details | `/api/mods`, `/api/assets`, Web-Seiten | keine allgemeine Webapp-Katalogclient-Schicht | FEHLT | versionierten Read-Client ergänzen |
| Lokale Installation/Deinstallation | nicht Webapp-verantwortlich | `ModsFolderSyncService` | FEHLERHAFT/TEILWEISE | transaktionales Deployment und Besitzjournal |
| Profile/Lockfile | Webapp-Collections, keine lokale Instanzlogik | `ModCollectionService` nur Collections | FEHLT | Profile/Manifest/Lockfile P0 |
| Creator-Upload/Releases | Upload-Wizard, Presigned URLs, Moderation | Steam-Projekteditor | TEILWEISE_IMPLEMENTIERT | Webapp-kompatibler Upload-Assistent |
| Admin/Moderation/Community | zahlreiche Webapp-Routen | keine Desktop-Parität | NICHT_FÜR_DESKTOP_RELEVANT/FEHLT | gemäß Rolle und Roadmap später |
