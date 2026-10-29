# Greg Modmanager Konzept

# Data Center Modmanager – Produktkonzept und funktionale Anforderungen

## 1. Zielsetzung

Ich entwickle mit **Rust und Slint** einen plattformübergreifenden Modmanager für das Spiel *Data Center*.

Das Ziel ist ein leistungsfähiges All-in-One-Werkzeug für Enduser, Modder und Modpublisher. Die Anwendung soll den gesamten Lebenszyklus von Mods abdecken: von der Installation und Verwaltung über die Entwicklung und Veröffentlichung bis hin zur Fehleranalyse und Wartung.

Der Modmanager soll ein zentraler Bestandteil des Mod-Ökosystems von Data Center werden und mit steigender Anzahl installierter Mods übersichtlich, performant und einfach bedienbar bleiben.

Die Benutzeroberfläche und Informationsarchitektur müssen deshalb konsequent auf Skalierbarkeit, klare Zuständigkeiten und kurze Bedienwege ausgelegt sein.

## 2. Hauptnavigation

Die Anwendung besitzt derzeit vier fachliche Hauptbereiche, dargestellt über eine **icon-basierte linke Navigationsleiste** (Slint-Layout inkl. `theme.slint` und `icons.slint`, siehe `crates/greg-app/ui/main.slint`):

1. **Modmanager** – Installation und Verwaltung für Enduser.
2. **Workshop** – Entwicklung, Validierung und Veröffentlichung (Upload) für Modder und Publisher.
3. **Modstore** – Zentrale Suche, Entdeckung und Installation von Mods aus Steam Workshop und Datacentermods.com.
4. **Settings** – Konfiguration der Anwendung, Spielpfade, Arbeitsverzeichnisse und Integrationen.

Die Zuordnung der Bereiche zu den Navigations-Icons sowie die Unterseiten je Bereich werden über `on_icon` und `on_nav` in `crates/greg-app/src/actions.rs` sowie `AppState::apply_icon_state` in `crates/greg-app/src/state.rs` definiert.

### Aktuelle Unterseiten je Bereich

- **Modmanager** (`modmanager`-Icon) führt über eine Unterseitenleiste: **My Mods**, **My Plugins**, **My Libs**, **Modpacks / Collections**, **Problems** und **Report**.
- **Workshop** (`workshop`-Icon) führt über die Unterseitenleiste: **Projects**, **New Project**, **Browse**, **Subscribed** und **Favorites**.
- **Modstore** (`modstore`-Icon) und **Settings** (`settings`-Icon) nutzen keine Unterseitenleiste; die fachlich passenden Seiten (Modstore-Fläche inkl. Updates und Installationen, Settings-Seite) dienen als zentrale Flächen.

Diese Struktur muss bei Bedarf überprüft und überarbeitet werden. Funktionen sollen dort eingeordnet werden, wo sie fachlich hingehören. Doppelte Funktionen und redundante Navigationselemente sind zu vermeiden.

**Stand (2026-10-29, Version 1.7.7):** Die icon-basierte Navigation ist implementiert. Die Umsetzung erfolgt über die Slint-Seiten `page-local`, `page-packs`, `page-projects`, `page-editor`, `page-browse`, `page-store`, `page-settings`, `page-problems` und `page-report`. Einzelne Konzept-Entscheidungen (z. B. Statusleiste statt `Problems`-Hauptnavigation) sind nur teilweise umgesetzt; die Details dazu stehen in den jeweiligen Abschnitten und im Implementierungsstatus (Abschnitt 11).

## 3. Modmanager

Der Modmanager richtet sich primär an Enduser und bündelt alle Funktionen zur Verwaltung der installierten Mods.

### Aktuelle Unterbereiche

- **My Mods:** Installierte und verwaltete Mods, einschließlich aktiver und deaktivierter Einträge sowie Abhängigkeits- und Konflikthinweise.
- **My Plugins:** Installierte Plugins, insbesondere MelonLoader-Plugins, inklusive Browsing und automatischer Installation des offiziellen IL2CPP Assembly Fixers.
- **My Libs:** Bibliotheken und Abhängigkeiten, die von Mods benötigt werden, inklusive Installation der gregFramework-Libraries in das `{GameRoot}/UserLibs`-Verzeichnis.
- **Modpacks / Collections:** Zusammenstellungen mehrerer Mods.
- **Problems:** Aktuelle Probleme, Warnungen und Fehler.
- **Report:** Meldung von Fehlern, Verbesserungsvorschlägen und Feature Requests über den Universal Report Assistant.

### Anforderungen und offene Punkte

**Modverwaltung**

Die Anwendung muss auch bei einer großen Anzahl von Mods übersichtlich bleiben. Dazu sollen geeignete Filter-, Such-, Sortier- und Gruppierungsfunktionen vorgesehen werden.

**Plugins und Libraries**

Mods, Plugins und Bibliotheken müssen logisch voneinander getrennt verwaltet werden. Abhängigkeiten und mögliche Konflikte sollen nachvollziehbar dargestellt werden.

**Modpacks und Collections**

Die Unterscheidung zwischen Modpacks und Collections muss konzeptionell definiert werden.

Dabei soll geprüft werden, ob beide Begriffe tatsächlich unterschiedliche Funktionen benötigen oder ob eine gemeinsame Implementierung mit unterschiedlichen Anwendungsfällen sinnvoller ist.

**Problems und Statusmeldungen**

Der bisherige Bereich `Problems` soll aus der Hauptnavigation entfernt und in eine globale Status- und Benachrichtigungsleiste am unteren Fensterrand integriert werden.

Diese Statusleiste muss Informationen für unterschiedliche Benutzergruppen bereitstellen, darunter:

- Enduser: Installationsfehler, fehlende Abhängigkeiten, Modkonflikte und Kompatibilitätsprobleme.
- Modder: Build-Fehler, Projektprobleme und Validierungsfehler.
- Publisher: Upload-Fehler, Synchronisierungsprobleme und Veröffentlichungsstatus.

Die Anzeige soll kontextabhängig sein und nur relevante Informationen hervorheben.

**Stand (2026-10-29):** Die globale Statusleiste ist inzwischen implementiert und am unteren Fensterrand als Status-Leds für Steam-Verbindung, gregAPI-Verbindung sowie MelonLoader-Status mit Versionsanzeige ausgebildet. Der fachliche Problems-Flow bleibt derzeit als Unterbereich des Modmanager-Bereichs in der Navigationsleiste erhalten; die konzeptionelle Verschiebung in die Statusleiste ist damit nur teilweise umgesetzt.

**Report**

Das Bugreporting sowie die Meldung von Fehlern, Verbesserungsvorschlägen und Feature Requests soll in den zentralen Report-Workflow des Greg Frameworks integriert werden. Reports müssen dem richtigen Mod und Repository zugeordnet und über die vorgesehenen Git-Integrationen verarbeitet werden können.

## 4. Workshop und Modentwicklung

Der Hauptbereich wird aktuell primär als Workshop-/Upload-Bereich mit Fokus auf den Steam-Workshop umgesetzt.

Diese Bezeichnung ist langfristig zu eng, da der Bereich auch die Entwicklung, Validierung und Veröffentlichung von Mods für mehrere Plattformen unterstützen soll.

Ein geeigneter neuer Name muss noch festgelegt werden. Der Bereich soll unabhängig vom später gewählten Namen als zentrale Entwicklungs- und Publishing-Umgebung konzipiert werden.

### Neues Projekt

Die bisherige Funktion `Neues Projekt` wurde durch einen geführten **Mod-Creation-Assistenten** ersetzt.

Der Assistent soll Modder durch die vollständige Ersteinrichtung eines Projekts führen und sicherstellen, dass alle erforderlichen Dateien, Metadaten und Konfigurationen korrekt angelegt werden.

Dabei muss er mindestens folgende Anwendungsfälle berücksichtigen:

- Erstellung eines neuen Mod-Projekts.
- Konfiguration der erforderlichen Metadaten.
- Einrichtung der Projektstruktur und des lokalen Workspaces.
- Validierung der erforderlichen Dateien und Einstellungen.
- Vorbereitung für den Steam Workshop.
- Vorbereitung für den Modstore auf Datacentermods.com.
- Unterstützung für Projekte, die auf mehreren Plattformen veröffentlicht werden sollen.

Die Projektstruktur und der Veröffentlichungsprozess sollen möglichst viele gemeinsame Komponenten verwenden, damit Mods nicht für jede Plattform vollständig separat gepflegt werden müssen.

**Stand (2026-10-29):** Der Assistent ist als Dialog mit Projektordner-Eingabe und Projekt-Template umgesetzt (`show-new-modal` in `crates/greg-app/ui/main.slint`). Die abschließende Validierung von Dateien und Einstellungen sowie die weitere Ausweitung auf Multi-Plattform-Projekte sind Bestandteil der laufenden Umsetzung.

### Meine Projekte

Hier werden alle vom Benutzer verwalteten Mod-Projekte zusammengeführt. Eigene Uploads werden dabei über die PublishedFileId-basierte Projektverknüpfung verwaltet; die frühere separate „My Uploads“-Fläche ist damit in die Projektbearbeitung integriert.

Die Projektübersicht soll den aktuellen Entwicklungs- und Veröffentlichungsstatus darstellen und den direkten Zugriff auf die zugehörigen Werkzeuge ermöglichen.

Denkbare Informationen sind:

- Projektstatus und Validierungsergebnisse.
- Lokaler Workspace und Projektdateien.
- Erkannte Fehler und Warnungen.
- Veröffentlichungsstatus je Plattform.
- Verfügbare Updates und Synchronisierungsstatus.

### Weitere Funktionen

Die bisherigen Bereiche `Browse`, `Subscribed` und `Favorites` sollen aus dem Workshop-Bereich herausgelöst und neu zugeordnet werden.

- **Browse:** Wird in den Modstore verschoben.
- **Subscribed:** Wird in die reguläre Modverwaltung integriert, da abonnierte Mods bereits über `Meine Mods` verwaltet werden sollen.
- **Favorites:** Werden zentral im Modmanager und im Modstore verfügbar gemacht, damit Favoriten sowohl verwaltet als auch entdeckt werden können.

Doppelte Unterseiten mit identischem Funktionsumfang sollen vermieden werden.

**Stand (2026-10-29):** Die konzeptionelle Verschiebung ist nur teilweise umgesetzt. Das Workshop-Icon führt weiterhin über `Browse`, `Subscribed` und `Favorites` (als Liste innerhalb der Browse-Seite), während der Modstore zusätzlich eigene Browse-, Updates- und Installationsflächen bereitstellt. Die finale Konsolidierung ist offen.

## 5. Modstore

Der Modstore soll die zentrale Anlaufstelle für die Suche, Entdeckung und Installation von Mods werden.

Er kombiniert zwei Quellen:

- Steam Workshop.
- Datacentermods.com Modstore.

Die aktuelle Implementierung ist funktional aufgebaut: Die Browse-Fläche arbeitet primär über den Steam Workshop (via native Steamworks-Bibliothek bzw. Steam-Web-API-Fallback), bietet Updates- und Installationsflächen mit Mehrfachauswahl und Mass-Installation. Veröffentlichungen können zusätzlich über `gregcli publish --to modstore` (API-Token via `--token` / `GREG_MODSTORE_TOKEN`) und über einen Editor-Button an den Modstore übermittelt werden; der Modstore-Identifier wird in `metadata.json` (`modstore_id`) persistiert.

Die Anbindung an den vollständigen Datacentermods.com-Katalog inklusive direkter Installationen aus Datacentermods.com bleibt derzeit offen.

### Anforderungen

**Modsuche und Entdeckung**

Der Modstore benötigt eine geeignete Oberfläche zur Suche, Filterung und Anzeige verfügbarer Mods aus beiden Quellen.

**Einheitliche Moddarstellung**

Mods sollen unabhängig von ihrer Quelle einheitlich dargestellt werden. Quellenspezifische Informationen und Funktionen müssen bei Bedarf zusätzlich verfügbar sein.

**Installationsquellen und Priorisierung**

Ist derselbe Mod sowohl im Steam Workshop als auch auf Datacentermods.com verfügbar, soll standardmäßig die Steam-Version bevorzugt werden.

Eine Ausnahme gilt, wenn die auf Datacentermods.com verfügbare Version nachweislich neuer ist. In diesem Fall soll diese Version bevorzugt werden.

Die Entscheidung muss auf einem nachvollziehbaren Versionsvergleich basieren. Ein bloßer Vergleich der Veröffentlichungszeitpunkte reicht nicht aus, wenn keine verlässlichen Versionsinformationen vorliegen.

Wenn eine Version nicht eindeutig als neuer eingestuft werden kann, darf die Anwendung nicht ohne Weiteres eine möglicherweise veraltete oder inkompatible Version bevorzugen.

**Favoriten**

Favorisierte Mods müssen im Modstore zugänglich sein. Änderungen an Favoriten sollen zentral verwaltet und in den anderen relevanten Bereichen übernommen werden.

**Integration in die Modverwaltung**

Installierte Mods müssen unabhängig von ihrer ursprünglichen Downloadquelle einheitlich im Modmanager erscheinen.

Die Quelle eines Mods muss nachvollziehbar bleiben, damit Updates, Abhängigkeiten und Konflikte korrekt behandelt werden können.

## 6. Settings

Der Bereich `Settings` bündelt sämtliche Konfigurationsmöglichkeiten der Anwendung.

Die Einstellungen müssen übersichtlich nach Funktionsbereichen gegliedert werden.

### Sprache und Benutzeroberfläche

- Auswahl der Anwendungssprache.
- Weitere globale Darstellungseinstellungen, sofern erforderlich.

### Workspace und Projektverwaltung

Der Benutzer muss den Speicherort für seine Mod-Projekte konfigurieren können.

Jedes Projekt muss einen eindeutig zugeordneten Workspace besitzen, der alle für Entwicklung, Validierung und Upload erforderlichen Dateien enthält.

Die Workspaces müssen so organisiert sein, dass mehrere Projekte unabhängig voneinander verwaltet werden können.

### Spielverzeichnis

Der Modmanager muss das Installationsverzeichnis von Data Center automatisch erkennen, soweit dies auf der jeweiligen Plattform möglich ist.

Folgende Installationsarten müssen berücksichtigt werden:

- **Windows:** Erkennung innerhalb vorhandener Steam-Bibliotheken.
- **Linux:** Erkennung der Steam-Installation einschließlich relevanter Proton-Umgebungen.
- **macOS:** Erkennung nativer Steam-Installationen.
- **Benutzerdefinierte Installationen:** Manuelle Konfiguration externer Installationspfade, beispielsweise für GOG oder Epic Games, sofern die jeweilige Installation unterstützt wird.

Automatisch erkannte Pfade müssen überprüfbar sein. Wenn die automatische Erkennung fehlschlägt, muss der Benutzer das Verzeichnis manuell auswählen können.

Die Anwendung darf nicht davon ausgehen, dass das Spiel immer im Standardverzeichnis von Steam installiert ist.

### Integrationen und Telemetrie

Die Einstellungen sollen außerdem die Konfiguration der Greg-Framework-Integrationen ermöglichen.

Dazu gehören insbesondere:

- Grafana für Monitoring und Telemetrie.
- Bugsink für Fehlererfassung und Error-Tracking.
- Zentrale Protokollierung relevanter Anwendungsfehler.
- Konfiguration der Verbindung zu den vorgesehenen Backend-Diensten.

Dabei müssen lokale Fehlerdiagnose und zentrale Telemetrie logisch getrennt bleiben. Die Übertragung von Diagnoseinformationen muss kontrollierbar und datenschutzgerecht implementiert werden.

**Stand (2026-10-29):** Die Settings-Seite ist implementiert und umfasst Arbeitsverzeichnis, Spielverzeichnis, Sprache, Modstore-API-URL, Steam-Web-API-Key, Diagnose-/Support-Bundle, Health-Checks sowie Log-Verzeichnis.

## 7. Globale Top-Bar

Neben der Hauptnavigation verfügt die Anwendung über eine globale Top-Bar mit folgenden Funktionen:

- `Start Game`
- `WithMods`
- `Stop`
- Login für den Modstore

Die Funktionen müssen hinsichtlich ihrer Zuständigkeiten und ihres tatsächlichen Verhaltens überprüft werden.

### Start Game

Startet Data Center ohne die vom Modmanager verwalteten Mods.

### WithMods

Startet Data Center mit der konfigurierten Mod-Umgebung.

Dabei müssen die aktivierten Mods und erforderlichen Abhängigkeiten berücksichtigt werden.

### Stop

Beendet die vom Modmanager überwachten Spielinstanz bzw. den Spiel-Prozess und setzt den Steamworks-Zustand danach zurück.

Ist ein Spiel aktiv, ersetzt die Stop-Schaltfläche die Start-Schaltflächen. Ein Start ohne Mods erfolgt über **Start Game**, ein Start mit Mods über **With Mods**. Wird ein Start durch einen Kill/Abbruch unterbrochen, werden die temporär deaktivierten Mod-Assemblys beim nächsten Start automatisch wiederhergestellt (vanilla-Modus).

**Stand (2026-10-29):** Die Stop-Funktion ist implementiert (`stop_game` in `crates/greg-app/src/actions.rs`, Stop-Button im Header während `game_running` ist). Einen kompletten Review der Zuständigkeiten und Redundanz hat es damit noch nicht gegeben.

### Login

Der Login ermöglicht den Zugriff auf authentifizierungsabhängige Funktionen des Modstores.

Die Authentifizierung soll langfristig auch für die Integration von Datacentermods.com und weitere Funktionen des Greg Frameworks nutzbar sein.

## 8. Globale Statusleiste

Die Statusleiste am unteren Fensterrand soll als zentrale Informationsfläche für den gesamten Modmanager dienen.

Sie ersetzt den bisherigen `Problems`-Hauptbereich und stellt relevante Statusmeldungen aus allen Anwendungsbereichen dar.

Dazu gehören unter anderem:

- Fehler und Warnungen.
- Installations- und Validierungsprobleme.
- Mod- und Abhängigkeitskonflikte.
- Build- und Upload-Status.
- Synchronisierungsprobleme.
- Veröffentlichungsstatus.
- Relevante Meldungen aus Greg Framework und Datacentermods.com.

Meldungen müssen priorisiert, kategorisiert und bei Bedarf einem konkreten Mod oder Projekt zugeordnet werden.

Ein Klick auf eine Meldung soll nach Möglichkeit direkt zur betroffenen Komponente oder zur entsprechenden Fehlerdiagnose führen.

**Stand (2026-10-29):** Die Statusleiste ist implementiert und am unteren Fensterrand sichtbar. Sie bindet Status-Leds für Steam, gregAPI und MelonLoader sowie eine Versionsanzeige ein. Probleme werden aktuell noch über die Unterseitenleiste des Modmanager-Bereichs aufgelöst; eine automatische Fehlermeldung in die Statusleiste ist offen.

## 9. Übergreifende Architektur- und UX-Anforderungen

Die Überarbeitung soll nicht lediglich bestehende Funktionen zwischen Tabs verschieben. Ziel ist eine konsistente Informationsarchitektur für das gesamte Mod-Ökosystem.

Dabei gelten folgende Anforderungen:

- **Klare Zuständigkeiten:** Jede Funktion hat einen eindeutigen primären Einsatzort.
- **Keine redundanten Bereiche:** Funktionen wie abonnierte Mods und Favoriten werden nicht unabhängig voneinander mehrfach implementiert.
- **Plattformübergreifende Unterstützung:** Windows, Linux und macOS müssen berücksichtigt werden.
- **Skalierbarkeit:** Die Bedienbarkeit muss auch bei sehr vielen Mods und Projekten erhalten bleiben.
- **Einheitliche Datenmodelle:** Mods, Plugins, Libraries, Projekte und Veröffentlichungen müssen konsistent verwaltet werden.
- **Nachvollziehbare Statusmeldungen:** Fehler und Warnungen müssen verständlich und kontextbezogen angezeigt werden.
- **Modulare Implementierung:** Bestehende Rust-Crates und kompatible Bibliotheken sollen bevorzugt genutzt werden, sofern sie die Anforderungen erfüllen.
- **Erweiterbarkeit:** Neue Downloadquellen, Publishing-Plattformen und Greg-Framework-Integrationen müssen später ohne grundlegende Neustrukturierung ergänzt werden können.

## 10. Arbeitsauftrag

Analysiere die bestehende Rust-/Slint-Anwendung und entwickle auf Grundlage dieser Anforderungen ein vollständiges Konzept für die zukünftige Informationsarchitektur und Funktionsstruktur.

Gehe dabei in folgender Reihenfolge vor:

1. Analysiere die vorhandenen Tabs, Unterseiten, Aktionen und Datenmodelle.
2. Identifiziere redundante, falsch platzierte, unvollständige oder nicht eindeutig definierte Funktionen.
3. Entwickle eine konsistente Zielstruktur für Navigation, Unterseiten, Top-Bar und Statusleiste.
4. Definiere die fachlichen Unterschiede zwischen Mods, Plugins, Libraries, Modpacks, Collections und Mod-Projekten.
5. Beschreibe die notwendigen Datenflüsse zwischen Modmanager, Workshop-/Entwicklungsbereich, Modstore und Settings.
6. Definiere die Anforderungen an plattformübergreifende Pfaderkennung, Downloadquellen, Versionsvergleich und Startlogik.
7. Prüfe, welche bestehenden Implementierungen und Rust-Crates weiterverwendet werden können.
8. Erstelle einen priorisierten Implementierungsplan, der bestehende Funktionen schrittweise verbessert, ohne unnötige Neuentwicklungen oder redundante Implementierungen zu erzeugen.

**Wichtig:** Unterscheide klar zwischen bereits vorhandenen Funktionen, geplanten Funktionen und noch offenen Konzeptentscheidungen. Fehlende Details sollen zunächst analysiert und begründet entschieden werden, anstatt unbelegte Annahmen als bestehende Anforderungen zu behandeln.

## 11. Aktueller Implementierungsstand

**Stand: 2026-10-29 · Version 1.7.7**

Dieses Konzept beschreibt die Zielarchitektur und funktionalen Anforderungen. Der folgende Überblick fasst den Umsetzungsstand der aktuellen Rust-/Slint-Anwendung zusammen und dient als Referenz für die Abstimmung zwischen Konzept, Plan und Implementierung.

### Implementiert

- **Shell:** Slint-Desktop-App (`crates/greg-app`) mit frameless Custom-Chrome (Titelzeile, WindowMoveArea, Fenstersteuerung), icon-basierter linker Navigationsleiste, globalem Top-Bar-Logik und globaler Statusleiste mit Status-Leds.
- **Game-Launch:** Supervised Start (`Start Game` / `With Mods`) und Stop über verwachte Prozess-Instanzen; Vanilla-Modus mit temporärer Deaktivierung von Mod-Assemblys; MelonLoader-Gating für den Mod-Start.
- **Workshop:** Native Steamworks-Integration (`ISteamUGC`) inkl. Web-API-Fallback, Preview-Images, Change Notes und Rate Limiter; Projekte inkl. Editor mit Readiness-Checks, Publish an Steam und Modstore.
- **Modstore:** Browse, Updates und Installationen (inkl. Mehrfachauswahl und Mass Install); Modstore-Publish über `gregcli publish --to modstore`; `modstore_id`-Persistierung.
- **Report:** Universal Report Assistant mit Kategorien, Markdown-Preview, Export, Attachments, Log-Sammlung und env-gated GitHub-Issue-Erstellung inkl. lokaler Export-Fallback.
- **Lokalisierung:** Komplette Internationalisierung über `greg-core::l10n` (Englisch plus neun weitere Sprachen) inkl. Slint-`Tr`-Global und Live-Wechsel.
- **Design:** Design-System aus `DESIGN.md` (Tokens, Lucide-Icons, Dialoge, Buttons, Listen); UI-Redesign auf Basis der gemeinsamen Komponenten.

### Offen / weiterhin offen

- Webapp-Katalog-Parität: Der Desktop nutzt aktuell primär den Steam Workshop als Quelle; vollständige Katalog-Parität mit Datacentermods.com inkl. direkter Installationen ist offen.
- Automatische Konfliktlösung, Load-Order-Automatisierung, Profile und Lockfiles, In-App-Updater (laut `README.md` noch nicht released).
- Discord-/GitHub-Login-Integration im Desktop vollständig.
- macOS: Native Mac-Validierung, Signing und Notarization (laut `CHANGELOG.md` experimentell).

### Quellen des aktuellen Stands

- `CHANGELOG.md` unter `[Unreleased]` sowie Release-Tagging über `VERSION`.
- `README.md` (Features, Kompatibilität, noch nicht released Features).
- `docs/` (PLATFORMS, CI/CD, TROUBLESHOOTING, ROADMAP).
- `desktop-gap-matrix.md` (Desktop-/Webapp-Paritäts-Status).

# Erweiterungen: Authentifizierung, Git-Integration und zentralisiertes Bugreporting

## 1. Authentifizierung und Account-Integrationen

Es sollen drei Authentifizierungsmöglichkeiten integriert werden:

- **Discord Login:** Primär für Bugreports, Fehlermeldungen und Feature Requests.
- **GitHub Login:** Für Git-Importe und das Erstellen von Issues im Namen des authentifizierten Users. Perspektivisch soll GitHub Login auch als Authentifizierungsmethode für Datacentermods.com dienen.
- **Datacentermods Git Login:** Für Git-Importe und das Erstellen von Issues im Namen des authentifizierten Users. Perspektivisch soll diese Authentifizierung ebenfalls für Datacentermods.com verfügbar sein.

Die Authentifizierungsmethoden sollen modular aufgebaut sein, damit sie unabhängig voneinander erweitert und in weiteren Anwendungen verwendet werden können.

## 2. Zentralisiertes Bugreporting und Issue-Erstellung

Bugreports, Fehlermeldungen und Feature Requests sollen möglichst direkt als Issues im jeweils zuständigen Git-Remote erstellt werden.

Dabei gilt folgende Priorität:

1. **Authentifizierter User:** Wenn möglich, wird die Git-Integration des Users verwendet, um das Issue in dessen Namen und mit dessen Berechtigungen zu erstellen.
2. **Nicht authentifizierter oder nicht ausreichend berechtigter User:** Eine zentrale, noch zu implementierende GitHub App, GitHub Action oder ein vergleichbarer Workflow übernimmt die Issue-Erstellung über einen autorisierten Service-Account.

Die Implementierung muss sicherstellen, dass Issues dem richtigen Repository und Mod zugeordnet werden. Zugangsdaten und Tokens dürfen niemals an Clients weitergegeben oder ungeschützt gespeichert werden.

## 3. Zentraler Report-Tab im Modmanager

Der Modmanager erhält einen zentralen **Report-Tab**, über den Enduser Fehler melden, Verbesserungsvorschläge einreichen und Feature Requests erstellen können.

Die Meldungen sollen nicht isoliert gespeichert werden, sondern direkt in den bestehenden Entwicklungs- und Wartungsworkflow integriert werden.

Die Integration umfasst:

- Automatische Zuordnung von Meldungen zu Mods und Git-Repositories.
- Erstellung von Issues in den zuständigen Git-Remotes.
- Unterstützung für Bugreports, Feature Requests und allgemeine Rückmeldungen.
- Nachverfolgung des Bearbeitungsstatus und der zugehörigen Issues.
- Bereitstellung der Meldungen für die weitere Verarbeitung durch Greg Framework, gregLint und die VS Code Extension.

## 4. Integration in gregLint, VS Code Extension und Datacentermods.com

Das Report-System muss mit den bestehenden und geplanten Komponenten des Greg Frameworks zusammenarbeiten.

- **gregLint:** Bereitstellung und Verarbeitung relevanter Meldungen zu Mods.
- **VS Code Extension:** Anzeige, Erstellung und Bearbeitung von Reports direkt in der Entwicklungsumgebung.
- **Datacentermods.com:** Zentrale Synchronisierung und Zuordnung von Meldungen zu den jeweiligen Mods.
- **Modmanager:** Anzeige der zugehörigen Reports und des verfügbaren Bearbeitungsstatus für den Enduser.

Datacentermods.com soll als zentrale Schnittstelle dienen, über die Reports zwischen Modmanager, VS Code Extension und den zuständigen Git-Repositories synchronisiert werden.

Die Synchronisierung muss erkennen, welche Mods offene oder neue Meldungen haben, und diese Informationen den jeweiligen Anwendungen zur Verfügung stellen.

## 5. KI-gestützter Report-Assistent

Der Report-Tab soll einen KI-Assistenten erhalten, der Enduser bei der Erstellung aussagekräftiger Bugreports und Feature Requests unterstützt.

Der Assistent soll:

- Nutzereingaben entgegennehmen und strukturieren.
- Aus unvollständigen Fehlerbeschreibungen nachvollziehbare Reports erstellen.
- Relevante Projektinformationen und verfügbare Fehlerdaten berücksichtigen.
- Reports automatisch dem passenden Mod und Repository zuordnen, sofern die erforderlichen Informationen vorliegen.
- Den finalen Report vor dem Absenden durch den User überprüfbar machen.

Zusätzlich sollen Entwickler den aktuellen Projektstatus zusammenfassen lassen können. Dafür muss der Assistent auf die jeweils freigegebenen Projektinformationen zugreifen können.

## 6. Unterstützung externer KI-Agenten und APIs

User sollen eigene KI-Modelle und Agenten anbinden können, die mit der OpenAI-API oder der Anthropic-Claude-API kompatibel sind.

Die Integration soll mindestens Folgendes ermöglichen:

- Konfiguration eigener API-Endpunkte und API-Keys.
- Unterstützung kompatibler OpenAI-API-Endpunkte sowie der Claude-API.
- Erstellung von Projektzusammenfassungen auf Basis der verfügbaren Projektinformationen.
- Unterstützung bei der Analyse und Strukturierung von Fehlermeldungen.
- Generierung vollständiger Bugreports direkt auf dem Endgerät des Users.

Die Architektur soll unterschiedliche Provider und Modelle über eine gemeinsame Schnittstelle unterstützen, ohne die Anwendung an einen einzelnen Anbieter zu binden.

## 7. Lokale Report-Erstellung und Vision-Unterstützung

Enduser sollen Reports vollständig lokal vorbereiten und erstellen können, soweit die erforderlichen Projekt- und Fehlerdaten lokal verfügbar sind.

Die Report-Erstellung muss grundsätzlich **textbasiert** funktionieren. Bildanalyse soll optional unterstützt werden, wenn das konfigurierte Modell Vision-Funktionen bereitstellt.

Daraus ergeben sich folgende Anforderungen:

- Textbasierte Report-Erstellung als Standard.
- Optionale Analyse von Screenshots und Bildern mit Vision-fähigen Modellen.
- Keine verpflichtende Abhängigkeit von Vision-Modellen.
- Klare Trennung zwischen lokaler Report-Erstellung, externer KI-Verarbeitung und dem anschließenden Versand an das Git-Remote.

## 8. Architekturvorgaben

Alle Funktionen sollen modular, erweiterbar und wiederverwendbar implementiert werden.

Vorhandene npm-Pakete und etablierte Integrationen sollen bevorzugt werden, sofern sie mit dem bestehenden Stack kompatibel sind und die Anforderungen vollständig abdecken.

Neue Eigenentwicklungen sind auf Funktionen zu beschränken, für die keine geeignete bestehende Lösung verfügbar ist.

Die Umsetzung soll Authentifizierung, Git-Provider, Report-Verarbeitung, KI-Anbindung und Synchronisierung als klar getrennte Komponenten behandeln. Dadurch müssen zukünftige Erweiterungen – insbesondere die Integration in Datacentermods.com – ohne grundlegende Änderungen an der bestehenden Architektur möglich sein.
