# gregWeb.Modstore – historical feature inventory

> **Scope:** This is an evidence snapshot of a separate `gregWeb.Modstore`
> checkout dated 2026-07-29. It does not describe released gregModmanager
> desktop functionality. Use the desktop user guide and source tree to make
> claims about this repository's application.

Stand: 2026-07-29  
Quelle: `/home/marvin/Dokumente/GregFramework/WebRepositories/gregWeb.Modstore/`

Dieses Dokument beschreibt Funktionen, die im aktuellen Quellcode als Seiten, Komponenten, API-Routen, Services oder Tests vorhanden sind. „Fertig“ bedeutet hier: implementiert und im Repository nachweisbar. Es bedeutet nicht automatisch, dass die Funktion auf dem Produktionssystem konfiguriert, mit allen externen Diensten verbunden oder vollständig manuell abgenommen ist.

## 1. Plattform und Oberfläche

- Next.js-App mit App Router und React.
- Mehrsprachige Seitenstruktur über `next-intl` mit Locale-Routen.
- Responsive Weboberfläche mit Navigation, Sidebar, Navbar, User-Menü und Admin-Sidebar.
- Öffentliche Landingpage mit Modstore-Einstieg, Kategorien, Suche, Upload- und Community-Einstiegen.
- Öffentliche Status-, Versions-, Privacy-, Imprint- und Maintenance-Seiten.
- API-Dokumentationsseite mit OpenAPI-Dokumenten.
- Fehler- und Zugriffszustände für nicht gefundene Inhalte, nicht authentifizierte Nutzer und gesperrte Bereiche.

## 2. Authentifizierung und Benutzerkonten

- Registrierung mit E-Mail und Passwort.
- Login und Logout über Better Auth.
- E-Mail-Verifikation.
- Passwort vergessen und Passwort zurücksetzen.
- GitHub- und Discord-Login.
- Account-Linking für vertrauenswürdige Login-Provider.
- Steam-Login über Steam OpenID inklusive Callback, Steam-ID-Auflösung und Profilbildübernahme.
- Passkey/WebAuthn:
  - Registrierung starten;
  - Registrierung verifizieren;
  - vorhandene Passkeys auflisten;
  - Passkeys löschen.
- Zwei-Faktor-Authentifizierung über das Better-Auth-Plugin.
- Session-Verwaltung mit serverseitiger Sessionprüfung.
- Rollen im User-/Session-Kontext: `user`, `moderator`, `admin`, `superadmin` sowie weitere im Code verwendete Rollen.
- Konfigurierbare Login- und Registrierungsfreigabe.
- Erstsetup für den ersten Admin-Benutzer.
- Onboarding-Seite für neue Benutzer.
- Benutzerprofil:
  - öffentlicher Profilzugriff per Username;
  - Anzeigename;
  - Bio und Profildaten;
  - Avatar;
  - Banner;
  - GitHub-, Discord-, Steam-, Twitch-, YouTube- und Website-Links;
  - E-Mail-Adresse ändern;
  - verknüpfte Konten verwalten.
- Benutzeravatar hochladen und speichern.
- Benachrichtigungen laden, als gelesen markieren und löschen.
- Responsive Account-Menü mit Dashboard, Profil, Collections, Upload, Adminbereich und Logout.

## 3. Öffentlicher Modstore

- Mods durchsuchen und anzeigen.
- Suche mit Query, Kategorien und Tags.
- Modkarten mit Titel, Autor, Kategorie, Status, Bild und Metadaten.
- Mod-Detailseiten mit:
  - Titel und Beschreibung;
  - Autor und Profil-Link;
  - Tags und Kategorie;
  - Versionen und Releases;
  - Abhängigkeiten;
  - Installationsinformationen;
  - Kompatibilität;
  - Homepage/GitHub/Support-Links;
  - Downloadaktion;
  - Steam-Workshop-Informationen, wenn verknüpft.
- `greg://install?modId=<id>`-Link zum installierten Desktop-Modmanager.
- Downloadzähler und Downloadzugriff.
- Zugriffsbeschränkung für nicht veröffentlichte, blockierte oder abgelehnte Inhalte.
- Mod melden.
- Favoriten setzen und entfernen.
- Watchlist setzen und entfernen.
- Kommentare laden, erstellen und löschen.
- Bewertungen laden, erstellen, aktualisieren und löschen.
- Öffentliche Profile und öffentliche Collections öffnen.

## 4. Assets und Plugins

- Plugin-Bereich.
- Asset-Bereich.
- Asset-Kategorien.
- Asset-Suche und Filter.
- Asset-Unterbereiche, darunter Models.
- Asset-Detailseiten mit Vorschaubildern und Metadaten.
- Asset-Download.
- Assets favorisieren.
- Asset-Bewertungen laden, erstellen und löschen.
- Asset-Abhängigkeiten laden, erstellen und löschen.
- Asset-Versionen laden und neue Versionen anlegen.
- Asset-Erstellung und Asset-Upload mit Berechtigungsprüfung.
- Unterstützte Uploadtypen im Upload-Assistenten umfassen Modelle, Texturen und Animationen mit den im Assistenten definierten Dateiformaten.

## 5. Mod-Upload und Creator-Workflow

- Geschützter Upload-Assistent.
- Uploadschritte:
  1. Uploadmethode;
  2. Metadaten;
  3. Bilder;
  4. Review;
  5. Ergebnis/Moderationsentscheidung.
- Uploadmethoden:
  - Git/GitHub-Release;
  - ZIP;
  - DLL;
  - Lua;
  - Python;
  - Go;
  - Modelldateien oder ZIP;
  - Texturen oder ZIP;
  - Animationen oder ZIP.
- Erfassbare Moddaten:
  - Titel;
  - Version;
  - Kategorie;
  - Beschreibung;
  - Lizenz und Lizenztyp;
  - Abhängigkeiten;
  - Kompatibilitätshinweise;
  - Sicherheitshinweise;
  - GitHub-URL;
  - Homepage;
  - Kontakt-E-Mail;
  - Spenden-URL;
  - minimale und empfohlene Spielversion;
  - Tags;
  - Installationsanleitung;
  - Support-URL;
  - Cover- und Galeriebilder.
- Rate-Limit für Mod-Submissions.
- Presigned Upload-URLs für Moddateien und Bilder.
- Direkter Upload in Object Storage.
- Erzeugung von Mod-, Release-, Bild- und Moderationsdatensätzen.
- Security-Scan-Einplanung nach dem Upload.
- Moderationsstatus und Ergebnisanzeige.
- Discord-/Telegram-Benachrichtigung an Moderatoren.
- Fehlerbehandlung für ungültige Daten, fehlende Dateien, ungültige JSON-Payloads und fehlgeschlagene Git-Imports.

## 6. Releases und Versionsverwaltung

- Releases zu Mods speichern.
- Releaseversionen und Changelogs speichern.
- Releasehistorie in Mod- und Creatoransichten.
- Neue Versionen im Creatorbereich beginnen.
- Releaseartefakte mit Hash und Dateigröße speichern.
- Verification-/Scanstatus eines Releases speichern.
- Releaseimport aus GitHub/Gitea.
- Git-Repository aus URL oder expliziten Host-/Owner-/Repository-Daten ermitteln.
- Release-Tag und Release-Asset abrufen.
- Importiertes Release in Quarantäne-Storage laden.
- SHA-256 und Dateigröße beim Git-Import erfassen.
- Git-Webhook-Route für spätere Release-/Tag-Updates.
- Dokumentationsupload für ein Modprojekt.

## 7. Dashboard und Creatorbereiche

- Persönliches Dashboard.
- Eigene Mods auflisten.
- Modstatus und Veröffentlichungsstatus anzeigen.
- Zu Moddetail, Dokumentation und Upload navigieren.
- Creator-Dashboard mit eigenen veröffentlichten und ausstehenden Mods.
- Creatorstatistiken und Statuskarten.
- Eigene Bibliothek.
- Community-Dashboard.
- Benachrichtigungsübersicht.
- API-Key-Verwaltung für den Benutzer.
- Einstellungen für Profil, Sicherheit und verbundene Accounts.

## 8. Collections

- Collections auflisten.
- Collections erstellen.
- Collection-Detailseiten.
- Collections bearbeiten.
- Collections löschen.
- Mods zu Collections hinzufügen.
- Mods aus Collections entfernen.
- Collectiontitel, Beschreibung, Slug und Sichtbarkeit speichern.
- Share-Links lesen und erzeugen.
- Öffentliche Collection-Share-Seiten.

## 9. Forum, Tutorials und Dokumentation

- Forumkategorien laden.
- Threads listen.
- Threaddetailseiten.
- Threads erstellen.
- Beiträge laden und erstellen.
- Öffentliche Tutorials listen.
- Tutorialdetailseiten.
- Tutorials erstellen, bearbeiten und löschen, wenn der Benutzer berechtigt ist.
- Dokumentationsübersicht.
- Dokumentationsseiten per Slug.
- Moddokumentation über Upload-API erzeugen beziehungsweise aktualisieren.

## 10. Admin- und Moderationsbereich

Der Adminbereich ist rollen- und berechtigungsabhängig.

- Admin-Dashboard.
- Systemkennzahlen und Metriken.
- App-Status und Betriebszustand.
- Moderationsqueue.
- Modfilter nach Status und Risiko.
- Mod freigeben.
- Mod ablehnen.
- Mod deaktivieren/Kill-Switch.
- Moderationsentscheidungen und Notizen.
- GregScan-/Securityberichte laden.
- Securityberichte freigeben oder ablehnen.
- Nutzerliste.
- Nutzerrollen ändern.
- Nutzerbezogene Moderationsaktionen.
- Feature Flags laden und ändern.
- Site-/App-Einstellungen nach Gruppen verwalten.
- Maintenance-Bypass setzen und entfernen.
- Admin-API-Keys erstellen, auflisten und löschen.
- Audit-Logs.
- Admin-Healthcheck.
- Storage-Metriken.
- Deploymentstatus.

## 11. Discord- und Telegram-Administration

- Discord-Bot-Konfiguration lesen und ändern.
- Discord-Botstatus prüfen.
- Discord-Botlogs laden.
- Guild-Rollen laden.
- Guild-Channels laden.
- Discord-Rollen-Mappings erstellen, ändern und löschen.
- Discord-Announcement/Testaktionen.
- Discord-Webhooks.
- Telegram-Admins auflisten, hinzufügen und löschen.
- Telegram-Testnachricht senden.

## 12. Steam-Integration der Webapp

- Steam-Workshop-Items über Steam-Web-API laden.
- Steam-Creator-Namen auflösen.
- Steam-Workshop-Metadaten in Moddaten übernehmen.
- Steam-Workshop-Item importieren.
- Steam-Workshop-Synchronisierung aus dem Adminbereich starten.
- Steam-App-/Workshop-Konfiguration über Admin-/Settings-Bereich verwalten, soweit aktiviert.

Der eigentliche lokale Steam-Workshop-Upload, Update, Download und Fortschritt bleibt im Desktop-Client eine native Steamworks-/`ISteamUGC`-Funktion.

## 13. Security- und Storage-Pipeline

- Quarantäne-Storage für untrusted Modartefakte.
- Presigned S3/MinIO-Upload-URLs.
- Datei- und Security-Scan-Queue.
- Scannerstatus und Queue-Verwaltung.
- AV-/EICAR-Testpfade in den E2E-Szenarien.
- Security-Webhook mit Signaturprüfung.
- Scanstatus `passed`, `rejected`, `needs_review`, `manual_required` und weitere im Code verwendete Zustände.
- Downloadbeschränkung für blockierte oder nicht freigegebene Artefakte.
- Security-Findings und Moderationsfreigabe.
- Redaction-/Logging-Hilfen für sensible Werte.
- Rate-Limits für Upload, Git-Import und weitere mutierende Aktionen.
- Zod-basierte Requestvalidierung in den zentralen Upload- und Moderationsrouten.

## 14. Interne Integrationen

- Interne Asset-, Mod- und Download-APIs.
- Interne MinIO-Upload-URL-Erzeugung.
- Interne Scanner- und Security-Callbacks.
- Discord-Bot-Upload.
- Bot-Tenant- und Premiumstatus-Verwaltung.
- Deployment- und Release-Status.
- Interne Healthchecks.
- Git-Webhook.
- Event-Bus/RabbitMQ-Anbindung.
- Redis-basierte Caches und Rate-Limits.
- PostgreSQL über Drizzle ORM.
- MinIO/S3 über AWS SDK und Presigned URLs.

## 15. Desktop-Integration

- Webapp erzeugt `greg://install?modId=<id>`-Installationslinks.
- Desktop registriert und verarbeitet das `greg://`-Protokoll.
- Desktop-Testprofil unterstützt:
  - Browser/Login: `https://datacentermods.home`;
  - API: `https://api.datacentermods.home`.
- Webapp-Desktop-Auth-Bridge im Quellcode:
  - `GET /auth/login`;
  - `POST /auth/token`;
  - `POST /auth/logout`.
- Die Bridge verwendet Better-Auth-Sessions und liefert keinen unabhängigen dauerhaften Benutzeraccount.

## 16. Nachweisbare API-Gruppen

### Benutzer- und öffentliche APIs

- `/api/assets` und Asset-Unterressourcen.
- `/api/collections` und Collection-Unterressourcen.
- `/api/docs`.
- `/api/forum/*`.
- `/api/mods/submit`.
- `/api/mods/[id]/*` für Status, Kommentare, Ratings, Favoriten, Watchlist, Dokumentation, Git-Import und Moderation.
- `/api/notifications`.
- `/api/report`.
- `/api/steam/items`.
- `/api/tutorials`.
- `/api/upload-url`.
- `/api/user/*`.
- `/api/keys`.
- `/api/status` und `/api/version`.
- Better-Auth-, Passkey- und Steam-Auth-Routen.

### Admin-APIs

- Admin-API-Keys.
- Appstatus.
- Auditlogs.
- Health und Metrics.
- Storage.
- Maintenance-Bypass.
- Feature Flags.
- Moderationsqueue und Moderationsaktionen.
- GregScan-Reports.
- Nutzerrollen.
- Steam-Sync und Steam-Import.
- Discord-Bot, Guild-Daten und Rollen-Mappings.
- Telegram-Administratoren und Testnachrichten.

### Interne APIs

- Interne Assets, Mods, Downloads und Statistiken.
- MinIO-Upload-URLs.
- Scanner-, Security- und Deployment-Endpunkte.
- Bot-Upload und Bot-Tenants.
- Discord-Internal-Integrationen.
- Git-Webhook und Discord-Forum-Webhook.

## 17. Verifikation und Grenzen

Im Repository vorhandene E2E-/Boundary-Tests decken unter anderem Landingpage, Moddetailseite, Suche, Uploadvalidierung, Downloadzähler, Rollen-/Adminzugriff, Security-Callbacks, Scanblockierung, Desktop-Protokollintegration, Bot-Upload und Bot-Ticketing ab.

Noch extern zu verifizieren sind insbesondere:

- produktive DNS-/TLS-Auflösung von `datacentermods.home` und `api.datacentermods.home`;
- lokale Webapp-Neustart/Deployment der neuen Desktop-Auth-Routen;
- echte PostgreSQL-, Redis-, MinIO-, Discord-, Telegram-, Steam- und Security-Service-Konfiguration;
- vollständiger End-to-End-Desktop-Login einschließlich Browser-Redirect auf `greg://`;
- vollständige Parität des Desktop-Clients mit allen Webapp-Seiten;
- Legacy-GMod-Publisher-Funktionen unter `app/` und `src-tauri/`.

## Belege

- Seiten: `src/app/[locale]/**`.
- Benutzeroberfläche: `src/components/**`.
- Auth: `src/lib/auth.ts`, `src/app/api/auth/**`, `src/app/api/user/**`.
- Upload: `src/components/upload/UploadWizard.tsx`, `src/app/api/mods/submit/route.ts`, `src/app/api/upload-url/route.ts`.
- Modstore: `src/app/[locale]/mod/**`, `src/app/api/mods/**`, `src/lib/mods.ts`.
- Assets: `src/app/[locale]/assets/**`, `src/app/api/assets/**`.
- Collections: `src/app/[locale]/collections/**`, `src/app/api/collections/**`.
- Admin: `src/app/[locale]/admin/**`, `src/app/api/admin/**`, `src/lib/admin.ts`.
- Security: `security/**`, `src/lib/security/**`, `src/app/api/internal/security/**`.
- Steam: `src/lib/steam-workshop.ts`, `src/app/api/steam/items/route.ts`.
- Desktop-Integration: `src/components/ModCard.tsx`, `src/app/auth/**`.
- Tests: `tests/e2e/**`.

## Desktop-Stand der Analyse (2026-07-29)

- `IMPLEMENTIERT`: Game-Adapter-Registry mit Data-Center-Adapter für Erkennung, plattformabhängige Pfade, Fähigkeiten und sichere Install-/Uninstall-/Launch-Pläne.
- `IMPLEMENTIERT`: Adapter wird von der lokalen Dependency-Prüfung und Workshop-Synchronisierung verwendet.
- `TEILWEISE_IMPLEMENTIERT`: Profile, direkter Spielstart, transaktionales Deployment, Besitzdaten, Manifest-/Lockfile-Resolver und Webapp-Katalogclient fehlen weiterhin.
- `FEHLERHAFT/RISIKO`: Der Desktop-Client nutzt bisher überwiegend Steam-native Browse-/Download-/Upload-Funktionen; die Webapp-Parität ist daher nicht gegeben.

Belege: `src/GregModmanager.Core/Services/GameAdapters/`, `ModDependencyService.cs`, `ModsFolderSyncService.cs`, `tests/GregModmanager.Tests/GameAdapterTests.cs`.

## Desktop-Erweiterungen dieser Arbeitseinheit (2026-07-29)

- `IMPLEMENTIERT`: My Uploads bietet neben Edit einen expliziten Add-Update-Einstieg; bestehende Projekte werden anhand der PublishedFileId wiederverwendet.
- `IMPLEMENTIERT`: Workshop-Import schlägt bei vorhandenen Projektordnern nicht mehr fehl: gleiche PublishedFileId wird wiederverwendet, Namenskonflikte erhalten einen freien Suffixordner.
- `IMPLEMENTIERT`: Dialoge verwenden rahmenlose MainWindow-Dekoration, Titelzeile, Farbpalette und Aktionsbutton-Stile des Hauptfensters.
- `IMPLEMENTIERT`: Lange Settings-/New-Project-Seiten und Fehlermeldungsinhalte sind scrollbar.
- `IMPLEMENTIERT`: Modstore zeigt Steam-Installationsstatus, Preview-Bilder, Listen-/Galerieansicht und Mehrfachauswahl mit Mass Install über den bestehenden Subscribe-/Workshop-Sync-Workflow.
