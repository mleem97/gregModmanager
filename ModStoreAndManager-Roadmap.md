# Funktionsliste für Mod-Manager und Modding-Plattform

> Stand: 29. Juli 2026. Ausgelegt für eine zunächst spielbezogene App mit späterer Multi-Game-Erweiterung.

## Legende

- **P0:** Konkurrenzfähiger Kern / MVP
- **P1:** Starke Wettbewerbsparität oder wichtige Differenzierung
- **P2:** Wachstums- und Komfortfunktion
- **P3:** Spätere Erweiterung
- **Premium-Kandidat:** Monetarisierungsoption; Sicherheits-, Kompatibilitäts- und Wiederherstellungsfunktionen sollten nicht paywalled sein.

## Inhaltsübersicht

- Premium-Funktionen: 16
- Web-Funktionen: 53
- App-Funktionen: 77
- Backend-Funktionen: 70
- Manifest- und Uploadmodelle: 13
- Vorhandene eigene Manifestbeispiele: 6

## Berücksichtigte Wettbewerber

| Ökosystem | Typ | Offizielle Plattformen | Schwerpunkte | Grenzen |
|---|---|---|---|---|
| CurseForge + App | Hosting + Manager | Windows, macOS; Linux offiziell nur eingeschränkt/Ubuntu und je Spiel unterschiedlich; kein Android-Deploy | Premium: werbefrei, keine Wartezeit, Themes/Layouts; App: Discovery, Profile/Modpacks, Updates; API: Games/Mods/Files/Fingerprints | Linux-/Spielabdeckung ist nicht überall gleich; Manifest stark Minecraft-/spielbezogen |
| Nexus Mods + Vortex | Hosting + Multi-Game Manager | Vortex offiziell Windows; Nexus erklärte 2026 Vortex zum Fokus; kein offizieller stabiler Linux/macOS-Client | Premium: uncapped/instant, werbefrei, Collections-Automation; Vortex: Profile, Konflikte, Load Order, Extensions; Upload API/Action | Kein universelles Einzelmodmanifest; Installation stark game-/extensionabhängig |
| Steam Workshop + Steam Client | Plattform-/Game-native UGC | Steam Windows/macOS/Linux/SteamOS; Android nur Community/Account, nicht generisches Mod-Deployment | Subscribe/Auto-Download, Bewertungen, Collections, Tags; ISteamUGC, Steamworks Web API, SteamCMD/VDF Upload | Funktionen und Dateiformat sind vom einzelnen Spiel abhängig; kein allgemeiner Resolver |
| Thunderstore + r2modman | Multi-Community Hosting + Manager | r2modman Windows/Linux; Thunderstore Mod Manager offiziell Windows; kein offizielles macOS/Android-Deploy | Profiles, Export/Share, Direct Install, Updates, Config Editor; einfaches manifest.json und API | Dependency-Modell ist vergleichsweise einfach; weniger tiefe Konflikt-/Loadorder-Funktionen |
| Modrinth + Modrinth App | Hosting + Minecraft-Instanzen | Windows, Linux, macOS; kein Android-Deploy | Modrinth+: werbefrei/Badge/Creator-Anteil; App: Instanzen, Backups/Rollback, Update All, Logs; starke REST API und .mrpack | Auf Minecraft fokussiert; nicht direkt als allgemeines Multi-Game-Modell übertragbar |
| Mod Organizer 2 | Lokaler Power-User-Manager | Windows offiziell | VFS, Profile, Prioritäten, Plugin-/Loadorder, Konfliktansicht, Tools und Erweiterungen | Hohe Lernkurve; kein eigenes Hosting/Creator-Backend |
| Wabbajack | Automatisierter Modlisten-Installer | Windows offiziell | Reproduzierbare Installationsanweisungen, Downloadcache, Gallery/Repositories, MO2-Integration | Authoring komplex; stark auf bestimmte Spiele/MO2-Workflows ausgerichtet |
| Prism Launcher | Minecraft Multi-Instance Launcher | Windows, Linux, macOS | Instanzen, Loader/Java, Modrinth-/CurseForge-Downloads, Import/Export, Welten | Kein allgemeines Hosting-/Moderationsbackend |
| GDLauncher | Minecraft Launcher + Serverfunktionen | Windows, Linux, macOS | Mehrere Plattformquellen, isolierte Instanzen, Loader/Java, Import/Export/Cloud Share, Serververwaltung | Minecraft-spezifisch; einige Cloud/Premiumdetails produktabhängig |

# Premium-Funktionen

**Anzahl Funktionen:** 16

## Nutzererlebnis

- [ ] **F-001 – Werbefreie Website und App**
  - **Einordnung:** P2 · Monetarisierung · Web, Windows, Linux, macOS, Android-Companion
  - **Beschreibung:** Entfernt Display-/Videoanzeigen auf Website und in der App; der Kern-Download bleibt kostenlos.
  - **Nutzerwert:** Weniger Ablenkung und höhere wahrgenommene Qualität
  - **Premium:** Ja
  - **Beispiele/Wettbewerber:** CurseForge Premium; Nexus Premium; Modrinth+
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://www.curseforge.com/premium)

## Downloads

- [ ] **F-002 – Unbegrenzte Downloadgeschwindigkeit**
  - **Einordnung:** P2 · Monetarisierung · Web/API/App
  - **Beschreibung:** Kein künstliches Bandbreitenlimit für zahlende Konten; Infrastruktur- und Fair-Use-Limits bleiben möglich.
  - **Nutzerwert:** Schnellere Installation großer Modpacks
  - **Premium:** Ja
  - **Beispiele/Wettbewerber:** Nexus Premium
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://www.nexusmods.com/premium)

- [ ] **F-003 – Keine Wartezeit / Direktdownload**
  - **Einordnung:** P2 · Monetarisierung · Web/App
  - **Beschreibung:** Premium überspringt Countdown, Interstitial oder zusätzliche Bestätigungsschritte.
  - **Nutzerwert:** Weniger Reibung beim Installieren
  - **Premium:** Ja
  - **Beispiele/Wettbewerber:** CurseForge Premium; Nexus Premium
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://www.curseforge.com/premium)

- [ ] **F-004 – Mehr parallele Downloads**
  - **Einordnung:** P2 · Monetarisierung · Windows, Linux, macOS
  - **Beschreibung:** Höheres Limit für gleichzeitige Downloads und Hintergrundinstallation.
  - **Nutzerwert:** Modpacks werden deutlich schneller installiert
  - **Premium:** Ja
  - **Beispiele/Wettbewerber:** Nexus Collections/Vortex
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://www.nexusmods.com/premium)

## Automatisierung

- [ ] **F-005 – Ein-Klick-Installation großer Sammlungen**
  - **Einordnung:** P1 · Sammlungen · Windows, Linux, macOS
  - **Beschreibung:** Alle Dateien einer kuratierten Sammlung werden ohne einzelne Downloadbestätigungen verarbeitet.
  - **Nutzerwert:** Einsteigerfreundlichkeit
  - **Premium:** Optional Premium
  - **Beispiele/Wettbewerber:** Nexus Premium Collections
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://www.nexusmods.com/premium)

- [ ] **F-009 – Erweiterte Update-Regeln**
  - **Einordnung:** P2 · Automatisierung · Windows, Linux, macOS
  - **Beschreibung:** Zeitpläne, Wartungsfenster, Auto-Update nur bei kompatiblen Releases und automatische Snapshots.
  - **Nutzerwert:** Weniger Wartungsaufwand
  - **Premium:** Ja
  - **Beispiele/Wettbewerber:** Marktchance
  - **Evidenz:** Produktempfehlung

## Cloud

- [ ] **F-006 – Cloud-Synchronisierung von Profilen**
  - **Einordnung:** P2 · Cloud · Windows, Linux, macOS, Web
  - **Beschreibung:** Synchronisiert Modliste, Pins, Konfigurationen und optional Saves zwischen Geräten.
  - **Nutzerwert:** Gerätewechsel ohne Neuaufbau
  - **Premium:** Ja
  - **Beispiele/Wettbewerber:** Marktchance; teilweise Launcher-Cloudfunktionen
  - **Evidenz:** Produktempfehlung

- [ ] **F-007 – Erweiterte Cloud-Backups**
  - **Einordnung:** P2 · Cloud · Windows, Linux, macOS
  - **Beschreibung:** Mehr Speicher, längere Aufbewahrung und mehr Wiederherstellungspunkte; lokale Backups bleiben kostenlos.
  - **Nutzerwert:** Schutz vor defekten Updates
  - **Premium:** Ja
  - **Beispiele/Wettbewerber:** Marktchance
  - **Evidenz:** Produktempfehlung

## Sammlungen

- [ ] **F-008 – Private Sammlungen und Teams**
  - **Einordnung:** P2 · Community · Web/App
  - **Beschreibung:** Private/unlisted Modpacks, Teamfreigaben, Rollen und Einladungen.
  - **Nutzerwert:** Freundesgruppen, Tester und Server-Teams
  - **Premium:** Ja
  - **Beispiele/Wettbewerber:** Marktchance; private Projekte bei Hosting-Plattformen
  - **Evidenz:** Produktempfehlung

## Support

- [ ] **F-010 – Priorisierter Support**
  - **Einordnung:** P3 · Betrieb · Web
  - **Beschreibung:** Schnellere Bearbeitung von Konto-, Installations- und Creator-Problemen; Sicherheitsmeldungen niemals paywallen.
  - **Nutzerwert:** Vertrauen und planbare Hilfe
  - **Premium:** Ja
  - **Beispiele/Wettbewerber:** Gängiges SaaS-Muster
  - **Evidenz:** Produktempfehlung

## Personalisierung

- [ ] **F-011 – Themes, Layouts und App-Icon**
  - **Einordnung:** P3 · Monetarisierung · Web/App
  - **Beschreibung:** Zusätzliche Designs, kompakte/komfortable Layouts und kosmetische Profilmerkmale.
  - **Nutzerwert:** Kosmetischer Mehrwert ohne Funktionsnachteil
  - **Premium:** Ja
  - **Beispiele/Wettbewerber:** CurseForge Premium; Modrinth+ Badge
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://modrinth.com/news/article/design-refresh)

## Community

- [ ] **F-012 – Premium-Profilabzeichen**
  - **Einordnung:** P3 · Monetarisierung · Web/App
  - **Beschreibung:** Sichtbares, optional ausblendbares Unterstützerabzeichen.
  - **Nutzerwert:** Status und Plattformunterstützung
  - **Premium:** Ja
  - **Beispiele/Wettbewerber:** Modrinth+
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://modrinth.com/news/article/design-refresh)

## Creator Economy

- [ ] **F-013 – Abo-Anteil für Creator**
  - **Einordnung:** P2 · Creator Economy · Backend/Web
  - **Beschreibung:** Ein transparenter Anteil des Abos wird anhand Nutzung oder Nutzerwahl an Creator verteilt.
  - **Nutzerwert:** Anreiz für hochwertige Mods
  - **Premium:** Ja
  - **Beispiele/Wettbewerber:** Modrinth+; CurseForge Autorunterstützung
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://modrinth.com/news/article/design-refresh)

- [ ] **F-014 – Creator erhalten Premium durch Meilensteine**
  - **Einordnung:** P3 · Creator Economy · Backend/Web
  - **Beschreibung:** Downloads oder Qualitätsmeilensteine gewähren Premiumzeit.
  - **Nutzerwert:** Creator-Bindung
  - **Premium:** Ja
  - **Beispiele/Wettbewerber:** Nexus Mod Author Benefits
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://www.nexusmods.com/about/mod-author-benefits)

## Beta

- [ ] **F-015 – Früher Zugang zu optionalen Funktionen**
  - **Einordnung:** P3 · Betrieb · Windows, Linux, macOS, Web
  - **Beschreibung:** Opt-in Preview-Kanal mit Rollback; Sicherheits- und Kompatibilitätsfunktionen bleiben für alle verfügbar.
  - **Nutzerwert:** Feedback und Finanzierung
  - **Premium:** Ja
  - **Beispiele/Wettbewerber:** Gängiges Produktmuster
  - **Evidenz:** Produktempfehlung

## Historie

- [ ] **F-016 – Längere Aktivitäts- und Versionshistorie**
  - **Einordnung:** P3 · Cloud · Web/App
  - **Beschreibung:** Erweiterte persönliche Historie, Diff-Ansichten und langfristige Cloud-Aufbewahrung.
  - **Nutzerwert:** Fehleranalyse und Wiederherstellung
  - **Premium:** Ja
  - **Beispiele/Wettbewerber:** Marktchance
  - **Evidenz:** Produktempfehlung

---

# Web-Funktionen

**Anzahl Funktionen:** 53

## Discovery

- [ ] **F-017 – Spiel-/Community-Startseite**
  - **Einordnung:** P0 · MVP · Responsive Web
  - **Beschreibung:** Eigene Landingpage je Spiel mit Neuheiten, Trends, Kategorien, Loadern und kuratierten Empfehlungen.
  - **Nutzerwert:** Schneller Einstieg
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** CurseForge, Nexus, Steam Workshop, Thunderstore, Modrinth
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://thunderstore.io/)

- [ ] **F-018 – Volltextsuche**
  - **Einordnung:** P0 · MVP · Responsive Web
  - **Beschreibung:** Suche über Name, Beschreibung, Autor, Tags und optional Changelog/README.
  - **Nutzerwert:** Mods werden zuverlässig gefunden
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Alle großen Plattformen
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://docs.modrinth.com/api/)

- [ ] **F-019 – Facettierte Filter**
  - **Einordnung:** P0 · MVP · Responsive Web
  - **Beschreibung:** Filter für Spielversion, Modloader, Client/Server, Kategorie, Aktualität, Lizenz, Release-Kanal und Kompatibilität.
  - **Nutzerwert:** Reduziert inkompatible Downloads
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** CurseForge, Modrinth, Thunderstore
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://support.curseforge.com/support/solutions/articles/9000193488-getting-started)

- [ ] **F-020 – Sortierung und Ranking**
  - **Einordnung:** P0 · MVP · Responsive Web
  - **Beschreibung:** Relevant, Trending, neu, zuletzt aktualisiert, Downloads, Bewertungen, Follower und personalisiert.
  - **Nutzerwert:** Bessere Auffindbarkeit
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Alle großen Plattformen
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://partner.steamgames.com/doc/features/workshop/implementation)

- [ ] **F-021 – Kompatibilitäts-Badges**
  - **Einordnung:** P0 · MVP · Responsive Web
  - **Beschreibung:** Klare Anzeige für Spielbuild, Loader, Betriebssystem, Client/Server und bekannte Inkompatibilitäten.
  - **Nutzerwert:** Vermeidet Fehlversuche
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Modrinth; CurseForge
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://docs.modrinth.com/api/)

- [ ] **F-022 – Bereits installiert/heruntergeladen markieren**
  - **Einordnung:** P1 · Bibliothek · Web/App
  - **Beschreibung:** Suchergebnisse zeigen installierte, früher geladene, gefolgte und ausgeblendete Mods.
  - **Nutzerwert:** Verhindert Doppelarbeit
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Nexus-Historie; Nutzerwunsch im Nexus-Roadmap-Thread
  - **Evidenz:** Öffentlich diskutierter Wunsch
  - **Quelle:** [Quelle](https://www.nexusmods.com/news/15433)

- [ ] **F-023 – Ähnliche und ergänzende Mods**
  - **Einordnung:** P2 · Empfehlungen · Web/App
  - **Beschreibung:** Empfehlungen anhand Tags, Abhängigkeiten, gemeinsamer Profile und Kompatibilität.
  - **Nutzerwert:** Entdeckung und Retention
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Modrinth/Nexus ähnliche Inhalte; Marktchance
  - **Evidenz:** Produktempfehlung

- [ ] **F-024 – Kuratierten Listen folgen**
  - **Einordnung:** P1 · Community · Web/App
  - **Beschreibung:** Redaktionelle, Creator- und Community-Listen abonnieren.
  - **Nutzerwert:** Qualitätssignal und Updates
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Nexus Collections; Steam Collections
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://help.nexusmods.com/article/115-guidelines-for-collections)

## Projektseite

- [ ] **F-025 – Projektbeschreibung / README**
  - **Einordnung:** P0 · MVP · Responsive Web
  - **Beschreibung:** Markdown-Seite mit Übersicht, Installation, Voraussetzungen, FAQ und Warnhinweisen.
  - **Nutzerwert:** Verständliche Nutzung
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Alle Plattformen
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://wiki.thunderstore.io/mods/creating-a-package)

- [ ] **F-026 – Galerie und eingebettete Medien**
  - **Einordnung:** P1 · Content · Responsive Web
  - **Beschreibung:** Screenshots, Videos, Vergleichsbilder und optionale Alterskennzeichnung.
  - **Nutzerwert:** Bewertung vor Installation
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Nexus, Modrinth, Steam
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://docs.modrinth.com/api/)

- [ ] **F-027 – Versionen und Dateien**
  - **Einordnung:** P0 · MVP · Responsive Web
  - **Beschreibung:** Liste aller Releases mit Kanal, Datum, Spielversion, Dateigröße, Hash und Downloadstatus.
  - **Nutzerwert:** Gezielte Versionwahl
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Alle Plattformen
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://support.curseforge.com/support/solutions/articles/9000197242-file-project-types-and-additional-fields)

- [ ] **F-028 – Changelog pro Version**
  - **Einordnung:** P0 · MVP · Responsive Web
  - **Beschreibung:** Markdown-Changelog, Breaking-Change-Hinweis und Upgrade-Anleitung.
  - **Nutzerwert:** Sichere Updates
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** CurseForge, Nexus, Modrinth, Steam
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://support.curseforge.com/en/support/solutions/articles/9000197241-creating-and-submitting-a-project)

- [ ] **F-029 – Abhängigkeiten und Dependants**
  - **Einordnung:** P0 · MVP · Responsive Web
  - **Beschreibung:** Erforderliche, optionale, eingebettete, inkompatible und Tool-Abhängigkeiten plus Rückwärtsliste.
  - **Nutzerwert:** Transparente Installation
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** CurseForge, Thunderstore, Modrinth
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://support.curseforge.com/support/solutions/articles/9000197242-file-project-types-and-additional-fields)

- [ ] **F-030 – Konflikte / Ersetzt / Bietet**
  - **Einordnung:** P1 · Kompatibilität · Web/App
  - **Beschreibung:** Deklarierte Konflikte, virtuelle Fähigkeiten (provides) und Ersatzpakete.
  - **Nutzerwert:** Besserer Resolver
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Linux-Paketmanager-Muster; Marktchance
  - **Evidenz:** Produktempfehlung

- [ ] **F-031 – Lizenz und Berechtigungen**
  - **Einordnung:** P0 · MVP · Responsive Web
  - **Beschreibung:** SPDX-Lizenz oder benutzerdefinierte Rechte; Weiterverteilung, Modpacks, Forks und KI-Nutzung getrennt.
  - **Nutzerwert:** Rechtssicherheit
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Nexus Permissions; Modrinth License
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://docs.modrinth.com/api/)

- [ ] **F-032 – Quellcode-, Issue- und Support-Links**
  - **Einordnung:** P1 · Creator · Responsive Web
  - **Beschreibung:** Verlinkung zu Repository, Bugtracker, Dokumentation, Discord und Spenden.
  - **Nutzerwert:** Supportwege sichtbar
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Modrinth, Thunderstore, Nexus
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://wiki.thunderstore.io/mods/creating-a-package)

- [ ] **F-033 – Dateibaum und Archivvorschau**
  - **Einordnung:** P1 · Sicherheit · Web/App
  - **Beschreibung:** Sichere Vorschau des Archivinhalts, Installationszielen und ausführbarer Dateien.
  - **Nutzerwert:** Vertrauen und Diagnose
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** MO2 Archivvorschau; Marktchance webseitig
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://github.com/ModOrganizer2/modorganizer/releases)

- [ ] **F-034 – Sicherheitsstatus**
  - **Einordnung:** P0 · MVP · Responsive Web
  - **Beschreibung:** Scanstatus, Signatur, Hash, Scanzeitpunkt und transparente Warnungen anzeigen.
  - **Nutzerwert:** Vertrauen
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Nexus/CurseForge Virenscan
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://support.curseforge.com/en/support/solutions/articles/9000210425-curseforge-file-processor-errors-per-game)

- [ ] **F-035 – Status: aktiv, archiviert, veraltet, deprecated**
  - **Einordnung:** P0 · MVP · Responsive Web
  - **Beschreibung:** Lebenszykluskennzeichnung mit Ersatzempfehlung.
  - **Nutzerwert:** Verhindert tote Installationen
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Thunderstore deprecated; Nexus archived
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://thunderstore.io/)

- [ ] **F-036 – Mehrsprachige Beschreibung**
  - **Einordnung:** P2 · Lokalisierung · Responsive Web
  - **Beschreibung:** Übersetzungen pro Sprache mit Fallback und Kennzeichnung maschineller Übersetzung.
  - **Nutzerwert:** Internationale Reichweite
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Steam Workshop Sprachfelder
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://partner.steamgames.com/doc/features/workshop/implementation)

## Community

- [ ] **F-037 – Kommentare und Threads**
  - **Einordnung:** P1 · Community · Responsive Web
  - **Beschreibung:** Versionierbare Diskussionen mit Pinning, Moderation, Spoilern und Creator-Antworten.
  - **Nutzerwert:** Support und Feedback
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Nexus, Steam, CurseForge
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://partner.steamgames.com/doc/features/workshop/implementation)

- [ ] **F-038 – Bewerten, liken, endorse**
  - **Einordnung:** P1 · Community · Responsive Web
  - **Beschreibung:** Ein nachvollziehbares Qualitätssignal, getrennt von Downloadzahl.
  - **Nutzerwert:** Bessere Auswahl
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Steam Votes; Nexus Endorsements; Thunderstore Likes
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://partner.steamgames.com/doc/api/ISteamUGC)

- [ ] **F-039 – Folgen und Benachrichtigungen**
  - **Einordnung:** P1 · Community · Web/E-Mail/Push
  - **Beschreibung:** Projekt, Autor, Sammlung, Spielversion oder Kategorie folgen.
  - **Nutzerwert:** Updates nicht verpassen
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Modrinth followers; Nexus tracking
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://docs.modrinth.com/api/)

- [ ] **F-040 – Bugtracker / strukturierte Issues**
  - **Einordnung:** P2 · Community · Responsive Web
  - **Beschreibung:** Vorlagen, betroffene Version, Logs, Reproduktion, Status und Duplikaterkennung.
  - **Nutzerwert:** Weniger Supportchaos
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Nexus Bugs (spiel-/projektabhängig); Marktchance
  - **Evidenz:** Produktempfehlung

- [ ] **F-041 – Wiki / Guides / FAQ**
  - **Einordnung:** P2 · Content · Responsive Web
  - **Beschreibung:** Projektbezogene Dokumentation mit Versionierung und Berechtigungen.
  - **Nutzerwert:** Komplexe Mods werden nutzbar
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Thunderstore Wiki; Nexus articles
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://wiki.thunderstore.io/mods/creating-a-package)

- [ ] **F-042 – Melden, blockieren, stummschalten**
  - **Einordnung:** P0 · MVP · Responsive Web
  - **Beschreibung:** Meldungen für Inhalte, Malware, Rechteverletzung und Belästigung; Nutzer-/Tag-Mutes.
  - **Nutzerwert:** Sichere Community
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Alle großen Plattformen
  - **Evidenz:** Wettbewerberparität

## Konten

- [ ] **F-043 – Registrierung, Login und OAuth**
  - **Einordnung:** P0 · MVP · Web/App
  - **Beschreibung:** E-Mail, OAuth/OIDC, Device Flow für Desktop und optional Passkeys.
  - **Nutzerwert:** Niedrige Einstiegshürde
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Alle Plattformen
  - **Evidenz:** Wettbewerberparität

- [ ] **F-044 – 2FA, Passkeys und Sitzungsverwaltung**
  - **Einordnung:** P0 · MVP · Web/App
  - **Beschreibung:** TOTP/WebAuthn, aktive Geräte, Tokenwiderruf und Risikoalarme.
  - **Nutzerwert:** Kontosicherheit
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** SaaS-Standard
  - **Evidenz:** Produktempfehlung

- [ ] **F-045 – Bibliothek und Downloadhistorie**
  - **Einordnung:** P1 · Bibliothek · Web/App
  - **Beschreibung:** Geladene, installierte, gefolgte, bewertete und ausgeblendete Inhalte mit Suche.
  - **Nutzerwert:** Wiederfinden und Audit
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Nexus Download History
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://www.nexusmods.com/news/15433)

- [ ] **F-046 – Datenschutz- und NSFW-Einstellungen**
  - **Einordnung:** P0 · MVP · Responsive Web
  - **Beschreibung:** Alters-/Inhaltsfilter, Profil-Sichtbarkeit, Telemetrie-Opt-out und personalisierte Empfehlungen.
  - **Nutzerwert:** Kontrolle und Compliance
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Thunderstore NSFW; Nexus adult settings
  - **Evidenz:** Wettbewerberparität

- [ ] **F-047 – Datenexport und Kontolöschung**
  - **Einordnung:** P0 · MVP · Responsive Web
  - **Beschreibung:** Maschinenlesbarer Export und nachvollziehbare Lösch-/Aufbewahrungsfristen.
  - **Nutzerwert:** DSGVO-Konformität
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Rechtliche Basisfunktion
  - **Evidenz:** Produktempfehlung

## Creator

- [ ] **F-048 – Projekt erstellen und Draft speichern**
  - **Einordnung:** P0 · MVP · Responsive Web
  - **Beschreibung:** Wizard für Spiel, Typ, Name, Namespace, Lizenz, Beschreibung und Sichtbarkeit.
  - **Nutzerwert:** Schnelles Publishing
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** CurseForge, Modrinth, Nexus, Thunderstore
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://support.curseforge.com/en/support/solutions/articles/9000197241-creating-and-submitting-a-project)

- [ ] **F-049 – Teams, Co-Autoren und Rollen**
  - **Einordnung:** P1 · Creator · Responsive Web
  - **Beschreibung:** Owner, Maintainer, Uploader, Moderator, Analyst und Billing; fein granulare Rechte.
  - **Nutzerwert:** Skalierbare Zusammenarbeit
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Thunderstore Teams; Nexus Co-Autoren
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://www.nexusmods.com/news/15454)

- [ ] **F-050 – Web-Upload mit Drag-and-drop**
  - **Einordnung:** P0 · MVP · Responsive Web
  - **Beschreibung:** Mehrteiliger/resumierbarer Upload mit Fortschritt, Hashing und Abbruchfortsetzung.
  - **Nutzerwert:** Zuverlässiges Veröffentlichen
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Nexus Upload API/Cloud; alle Plattformen
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://www.nexusmods.com/news/15454)

- [ ] **F-051 – Manifest-Editor und Validator**
  - **Einordnung:** P0 · MVP · Responsive Web/App/CLI
  - **Beschreibung:** Formular- und JSON/YAML-Modus mit Schema, Autocomplete, Linting und verständlichen Fehlern.
  - **Nutzerwert:** Weniger fehlerhafte Pakete
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Thunderstore Validator; Modrinth mrpack
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://thunderstore.io/tools/manifest-v1-validator/)

- [ ] **F-052 – Release-Kanäle**
  - **Einordnung:** P0 · MVP · Web/App/API
  - **Beschreibung:** Stable/Release, Beta, Alpha/Nightly; Filter und Update-Regeln je Kanal.
  - **Nutzerwert:** Kontrollierte Updates
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** CurseForge Release/Beta/Alpha
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://support.curseforge.com/support/solutions/articles/9000197242-file-project-types-and-additional-fields)

- [ ] **F-053 – Kompatibilitätsmatrix pflegen**
  - **Einordnung:** P0 · MVP · Responsive Web/API
  - **Beschreibung:** Spielversionen, Loader, OS, Client/Server und DLCs deklarieren; Bulk-Edit möglich.
  - **Nutzerwert:** Korrekte Installationen
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** CurseForge, Modrinth
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://docs.modrinth.com/api/operations/createversion/)

- [ ] **F-054 – Abhängigkeiten per Suche hinzufügen**
  - **Einordnung:** P0 · MVP · Responsive Web/API
  - **Beschreibung:** Required/optional/incompatible/embedded/tools auswählen, Versionbereich festlegen.
  - **Nutzerwert:** Maschinenlesbarer Graph
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** CurseForge relations; Thunderstore dependencies
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://support.curseforge.com/support/solutions/articles/9000197242-file-project-types-and-additional-fields)

- [ ] **F-055 – Changelog und Upgrade-Hinweise**
  - **Einordnung:** P1 · Creator · Responsive Web/API
  - **Beschreibung:** Vorlagen, automatisch aus Git-Releases importierbar und pro Sprache.
  - **Nutzerwert:** Bessere Updates
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** CurseForge/Steam change note
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://partner.steamgames.com/doc/features/workshop/implementation)

- [ ] **F-056 – Draft, Unlisted, Private, Public**
  - **Einordnung:** P1 · Creator · Web/App/API
  - **Beschreibung:** Mehrstufige Sichtbarkeit für Tests, Freundesgruppen und Veröffentlichung.
  - **Nutzerwert:** Sicherer Releaseprozess
  - **Premium:** Teilweise
  - **Beispiele/Wettbewerber:** Steam visibility; Modrinth draft
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://partner.steamgames.com/doc/features/workshop/implementation)

- [ ] **F-057 – Geplante Veröffentlichung und Embargo**
  - **Einordnung:** P2 · Creator · Web/API
  - **Beschreibung:** Zeitgesteuertes Publishing nach abgeschlossener Prüfung.
  - **Nutzerwert:** Koordinierte Releases
  - **Premium:** Optional Premium
  - **Beispiele/Wettbewerber:** Marktchance
  - **Evidenz:** Produktempfehlung

- [ ] **F-058 – Staged Rollout und Rollback**
  - **Einordnung:** P2 · Creator · Web/API/App
  - **Beschreibung:** Prozentuale Ausrollung, automatische Stoppkriterien und Zurückziehen ohne Metadatenverlust.
  - **Nutzerwert:** Reduziert Update-Risiko
  - **Premium:** Optional Premium
  - **Beispiele/Wettbewerber:** App-Store-Muster; Marktchance
  - **Evidenz:** Produktempfehlung

- [ ] **F-059 – Version deprecaten/yanken**
  - **Einordnung:** P1 · Creator · Web/API
  - **Beschreibung:** Neue Installationen stoppen, bestehende Lockfiles bleiben reproduzierbar; Grund angeben.
  - **Nutzerwert:** Sicherheit ohne Historienbruch
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Paketregistry-Muster; Thunderstore deprecated
  - **Evidenz:** Produktempfehlung

- [ ] **F-060 – Moderationsstatus und Einspruch**
  - **Einordnung:** P0 · MVP · Responsive Web
  - **Beschreibung:** Status-Timeline, konkrete Regelverletzung, Nachreichung und Appeal-Workflow.
  - **Nutzerwert:** Transparenz
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** CurseForge Moderation; Modrinth moderation
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://support.curseforge.com/en/support/solutions/articles/9000197241-creating-and-submitting-a-project)

- [ ] **F-061 – Analytics-Dashboard**
  - **Einordnung:** P1 · Creator · Responsive Web
  - **Beschreibung:** Downloads, aktive Installationen, Updatequote, Retention, Fehler, Länder nur datenschutzkonform.
  - **Nutzerwert:** Produktverbesserung
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Nexus/CurseForge Creator Analytics; Marktchance
  - **Evidenz:** Wettbewerberparität

- [ ] **F-062 – Revenue-, Spenden- und Payout-Dashboard**
  - **Einordnung:** P2 · Creator Economy · Responsive Web
  - **Beschreibung:** Einnahmen, Attribution, Auszahlungsstatus, Steuer-/KYC-Prozess und Export.
  - **Nutzerwert:** Creator Economy
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Nexus Rewards; Modrinth+ Anteil
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://modrinth.com/news/article/design-refresh)

## Sammlungen

- [ ] **F-063 – Webbasierter Modpack-/Collection-Builder**
  - **Einordnung:** P1 · Sammlungen · Responsive Web
  - **Beschreibung:** Mods suchen, Versionen sperren, Reihenfolge/Optionen konfigurieren und Validierung ausführen.
  - **Nutzerwert:** Einfaches Kuratieren
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Nexus Collections; Steam Collections
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://help.nexusmods.com/article/115-guidelines-for-collections)

- [ ] **F-064 – Versionierte Revisionen und Diff**
  - **Einordnung:** P1 · Sammlungen · Web/App/API
  - **Beschreibung:** Jede Änderung erzeugt Revision; hinzugefügt, entfernt, aktualisiert, Konfigurationsänderung sichtbar.
  - **Nutzerwert:** Vertrauen in Updates
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Nexus Collections; Modpack-Versionen
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://help.nexusmods.com/article/115-guidelines-for-collections)

- [ ] **F-065 – Installationsoptionen**
  - **Einordnung:** P2 · Sammlungen · Web/App
  - **Beschreibung:** Optionale Gruppen, Varianten, Auswahlregeln und empfohlene Defaults.
  - **Nutzerwert:** Ein Modpack für mehrere Vorlieben
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Wabbajack/Collections teilweise; Marktchance
  - **Evidenz:** Produktempfehlung
  - **Quelle:** [Quelle](https://www.wabbajack.org/)

- [ ] **F-066 – Ein-Klick-/Deep-Link in App**
  - **Einordnung:** P0 · MVP · Web → App
  - **Beschreibung:** Webseite öffnet Desktop-App direkt auf Projekt/Version/Sammlung.
  - **Nutzerwert:** Nahtloser Funnel
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** CurseForge; Nexus; Wabbajack
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://support.curseforge.com/support/solutions/articles/9000193488-getting-started)

- [ ] **F-067 – Share-Code und Ablaufdatum**
  - **Einordnung:** P1 · Sammlungen · Web/App
  - **Beschreibung:** Kurzer Code für privates Profil oder Sammlung; widerrufbar und optional zeitlich begrenzt.
  - **Nutzerwert:** Teilen ohne Dateien
  - **Premium:** Optional Premium
  - **Beispiele/Wettbewerber:** CurseForge Profilcode
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://support.curseforge.com/support/solutions/articles/9000198501-exporting-modpacks)

## Mobile

- [ ] **F-068 – Responsive/PWA-Browsing**
  - **Einordnung:** P1 · Mobile · Android/iOS Web
  - **Beschreibung:** Suche, Projektseiten, Bibliothek, Benachrichtigungen und Creator-Status mobil.
  - **Nutzerwert:** Unterwegs recherchieren
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Webseiten der Plattformen
  - **Evidenz:** Wettbewerberparität

- [ ] **F-069 – Companion-App statt lokaler Modinstallation**
  - **Einordnung:** P2 · Mobile · Android/iOS
  - **Beschreibung:** Installationen an verknüpften Desktop senden, Warteschlange verwalten und Status/Fehler ansehen.
  - **Nutzerwert:** Android-Nutzen ohne unsicheren Dateizugriff
  - **Premium:** Optional Premium
  - **Beispiele/Wettbewerber:** Marktchance
  - **Evidenz:** Produktempfehlung

---

# App-Funktionen

**Anzahl Funktionen:** 77

## Onboarding

- [ ] **F-070 – Automatische Spielsuche**
  - **Einordnung:** P0 · MVP · Windows; später Linux/macOS
  - **Beschreibung:** Erkennt Installationen aus Steam, Epic, GOG, Microsoft Store und benutzerdefinierten Pfaden.
  - **Nutzerwert:** Schneller Start
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** CurseForge App; Vortex
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://support.curseforge.com/support/solutions/articles/9000193488-getting-started)

- [ ] **F-071 – Manuelles Hinzufügen und Pfadvalidierung**
  - **Einordnung:** P0 · MVP · Windows, Linux, macOS
  - **Beschreibung:** Nutzer kann Pfad wählen; App prüft Executable, Version und Schreibrechte.
  - **Nutzerwert:** Funktioniert bei exotischen Installationen
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** CurseForge App
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://support.curseforge.com/support/solutions/articles/9000193488-getting-started)

- [ ] **F-072 – Store- und Installationsvarianten erkennen**
  - **Einordnung:** P0 · MVP · Windows, Linux, macOS
  - **Beschreibung:** Steam/Epic/GOG/Game Pass, portable Versionen, DLCs und Testbranches unterscheiden.
  - **Nutzerwert:** Verhindert falsche Zielpfade
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Vortex/CurseForge spielabhängig
  - **Evidenz:** Wettbewerberparität

- [ ] **F-073 – Geführte Erstinstallation**
  - **Einordnung:** P0 · MVP · Windows, Linux, macOS
  - **Beschreibung:** Schrittfolge: Spiel erkennen, Profil anlegen, Loader prüfen, Backup erstellen, erste Mod installieren.
  - **Nutzerwert:** Reduziert Abbrüche
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Launcher-Muster
  - **Evidenz:** Produktempfehlung

## Spielstart

- [ ] **F-074 – Spiel direkt starten**
  - **Einordnung:** P0 · MVP · Windows, Linux, macOS
  - **Beschreibung:** Startet Vanilla oder ausgewähltes Profil mit passenden Argumenten/Umgebungsvariablen.
  - **Nutzerwert:** Ein zentraler Einstiegspunkt
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** CurseForge; r2modman; Vortex
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://thunderstore.io/c/patch-quest/p/ebkr/r2modman/)

- [ ] **F-075 – Vanilla-/Safe-Mode-Start**
  - **Einordnung:** P0 · MVP · Windows, Linux, macOS
  - **Beschreibung:** Temporär alle Mods deaktivieren, ohne Profil zu verändern.
  - **Nutzerwert:** Schnelle Fehlerdiagnose
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** r2modman normal Steam launch; Marktchance
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://thunderstore.io/c/patch-quest/p/ebkr/r2modman/)

- [ ] **F-076 – Launcher-Argumente und Tools**
  - **Einordnung:** P1 · Power User · Windows, Linux, macOS
  - **Beschreibung:** Profilbezogene Startargumente, externe Tools, Server, Debugger und Editors verwalten.
  - **Nutzerwert:** Power-User-Workflow
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** MO2 executables; Vortex tools
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://github.com/Nexus-Mods/Vortex)

## Installation

- [ ] **F-077 – Ein-Klick-Installation aus Katalog**
  - **Einordnung:** P0 · MVP · Windows; später Linux/macOS
  - **Beschreibung:** Projekt/Version wählen; Download, Prüfung, Dependencies und Deployment laufen transaktional.
  - **Nutzerwert:** Kernnutzen
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Alle Manager
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://www.nexusmods.com/about/vortex/)

- [ ] **F-078 – Manueller Import / Drag-and-drop**
  - **Einordnung:** P0 · MVP · Windows, Linux, macOS
  - **Beschreibung:** Lokale Archive, Ordner oder URL importieren; Quelle als unmanaged markieren.
  - **Nutzerwert:** Unterstützt externe Mods
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Vortex, MO2, Prism
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://github.com/Nexus-Mods/Vortex)

- [ ] **F-079 – Batch-Installation**
  - **Einordnung:** P1 · Sammlungen · Windows, Linux, macOS
  - **Beschreibung:** Mehrere Mods/Sammlungen in einer geplanten Transaktion installieren.
  - **Nutzerwert:** Spart Zeit
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Vortex Collections; Modrinth App
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://www.nexusmods.com/premium)

- [ ] **F-080 – Transaktionale Installation**
  - **Einordnung:** P0 · MVP · Windows, Linux, macOS
  - **Beschreibung:** Zuerst Download/Validierung/Staging, dann atomarer Commit; bei Fehler vollständiges Rollback.
  - **Nutzerwert:** Keine halb installierten Zustände
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Modrinth App Rollback; Paketmanager-Muster
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://modrinth.com/news/changelog)

- [ ] **F-081 – Saubere Deinstallation**
  - **Einordnung:** P0 · MVP · Windows, Linux, macOS
  - **Beschreibung:** Entfernt nur Dateien, die der Mod gehört; gemeinsam genutzte oder veränderte Dateien werden geschützt.
  - **Nutzerwert:** Keine beschädigte Installation
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** MO2 VFS; moderne Manager
  - **Evidenz:** Produktempfehlung

- [ ] **F-082 – Deployment-Strategien**
  - **Einordnung:** P0 · MVP · Windows, Linux, macOS
  - **Beschreibung:** Copy, hardlink, symlink, junction, VFS oder game-native Loader; Capability-Test und Fallback.
  - **Nutzerwert:** Performance und Sicherheit
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Vortex deployment; MO2 VFS
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://github.com/Nexus-Mods/Vortex)

- [ ] **F-083 – Installationsvorschau**
  - **Einordnung:** P1 · Sicherheit · Windows, Linux, macOS
  - **Beschreibung:** Zeigt Dateiziele, Überschreibungen, Scripts, Berechtigungen und geplante Änderungen vor Commit.
  - **Nutzerwert:** Vertrauen
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** MO2 archive preview; Marktchance
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://github.com/ModOrganizer2/modorganizer/releases)

## Downloads

- [ ] **F-084 – Downloadmanager**
  - **Einordnung:** P0 · MVP · Windows, Linux, macOS
  - **Beschreibung:** Queue, Fortschritt, Geschwindigkeit, ETA, Pause, Resume, Retry, Cancel und Fehlergrund.
  - **Nutzerwert:** Grundkomfort
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Alle Manager
  - **Evidenz:** Wettbewerberparität

- [ ] **F-085 – Parallele Downloads**
  - **Einordnung:** P1 · Performance · Windows, Linux, macOS
  - **Beschreibung:** Konfigurierbare Gleichzeitigkeit; Host-/Accountlimits respektieren.
  - **Nutzerwert:** Schnellere Modpacks
  - **Premium:** Optional Premium
  - **Beispiele/Wettbewerber:** Vortex Premium Collections
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://www.nexusmods.com/premium)

- [ ] **F-086 – Resumable/Chunked Downloads**
  - **Einordnung:** P0 · MVP · Windows, Linux, macOS
  - **Beschreibung:** Fortsetzen nach Neustart oder Netzwerkwechsel; ETag/Range und Hashprüfung.
  - **Nutzerwert:** Zuverlässigkeit
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Marktchance; moderne Downloadclients
  - **Evidenz:** Produktempfehlung

- [ ] **F-087 – Bandbreiten- und Zeitplanregeln**
  - **Einordnung:** P2 · Power User · Windows, Linux, macOS
  - **Beschreibung:** Globales Limit, nur nachts, metered network und Battery-Saver.
  - **Nutzerwert:** Kontrolle
  - **Premium:** Optional Premium
  - **Beispiele/Wettbewerber:** Marktchance
  - **Evidenz:** Produktempfehlung

- [ ] **F-088 – Lokaler Cache und Deduplizierung**
  - **Einordnung:** P0 · MVP · Windows, Linux, macOS
  - **Beschreibung:** Content-addressed Cache nach Hash; identische Dateien zwischen Profilen nur einmal speichern.
  - **Nutzerwert:** Weniger Speicher/Traffic
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Wabbajack Download Cache; Launcher-Muster
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://wiki.wabbajack.org/modlist_author_documentation/Pre-Compilation.html)

- [ ] **F-089 – Mirror-/CDN-Fallback**
  - **Einordnung:** P1 · Backend · Windows, Linux, macOS
  - **Beschreibung:** Wählt gesunden Mirror, wechselt bei Fehler und prüft unverändert Hash/Signatur.
  - **Nutzerwert:** Robuste Downloads
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Paketmanager-Muster
  - **Evidenz:** Produktempfehlung

## Versionen

- [ ] **F-090 – Update-Erkennung**
  - **Einordnung:** P0 · MVP · Windows, Linux, macOS
  - **Beschreibung:** Vergleicht installierte Version/Hash mit kompatiblen Releases und zeigt Changelog.
  - **Nutzerwert:** Aktuell bleiben
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** CurseForge; r2modman; Vortex
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://thunderstore.io/c/patch-quest/p/ebkr/r2modman/)

- [ ] **F-091 – Update All mit Vorschau**
  - **Einordnung:** P0 · MVP · Windows, Linux, macOS
  - **Beschreibung:** Kompatible Updates bündeln; inkompatible oder Breaking Updates separat bestätigen.
  - **Nutzerwert:** Komfort ohne Blindflug
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Vortex update all; Modrinth App
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://www.nexusmods.com/about/vortex/)

- [ ] **F-092 – Version pinnen / ignorieren**
  - **Einordnung:** P0 · MVP · Windows, Linux, macOS
  - **Beschreibung:** Bestimmte Version oder Bereich sperren; Update ausblenden mit Begründung.
  - **Nutzerwert:** Stabile Setups
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Gängige Manager
  - **Evidenz:** Wettbewerberparität

- [ ] **F-093 – Downgrade und Versionswechsel**
  - **Einordnung:** P1 · Versionierung · Windows, Linux, macOS
  - **Beschreibung:** Frühere kompatible Version auswählen; Snapshot vor Wechsel.
  - **Nutzerwert:** Recovery und Tests
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Modrinth App; GDLauncher
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://modrinth.com/news/changelog)

- [ ] **F-094 – Release-Kanal pro Mod/Profil**
  - **Einordnung:** P1 · Versionierung · Windows, Linux, macOS
  - **Beschreibung:** Stable/Beta/Alpha/Nightly getrennt wählbar.
  - **Nutzerwert:** Kontrollierte Experimente
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** CurseForge Kanäle
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://support.curseforge.com/support/solutions/articles/9000197242-file-project-types-and-additional-fields)

## Abhängigkeiten

- [ ] **F-095 – Automatische Pflichtabhängigkeiten**
  - **Einordnung:** P0 · MVP · Windows, Linux, macOS
  - **Beschreibung:** Resolver installiert required dependencies in kompatibler Version.
  - **Nutzerwert:** Weniger Fehler
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** CurseForge; Thunderstore; Modrinth
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://support.curseforge.com/support/solutions/articles/9000197242-file-project-types-and-additional-fields)

- [ ] **F-096 – Optionale Dependencies anbieten**
  - **Einordnung:** P1 · Kompatibilität · Windows, Linux, macOS
  - **Beschreibung:** Optionen mit Erklärung, empfohlener Default und Größen-/Kompatibilitätsprüfung.
  - **Nutzerwert:** Personalisierung
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** CurseForge optional relations; Marktchance
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://support.curseforge.com/support/solutions/articles/9000197242-file-project-types-and-additional-fields)

- [ ] **F-097 – Version-Range-Solver**
  - **Einordnung:** P0 · MVP · Windows, Linux, macOS
  - **Beschreibung:** SemVer/spielbezogene Constraints, Backtracking und verständliche Konflikterklärung.
  - **Nutzerwert:** Löst komplexe Packs
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Paketmanager-Muster; Plattformen meist einfacher
  - **Evidenz:** Produktempfehlung

- [ ] **F-098 – Dependants beim Entfernen warnen**
  - **Einordnung:** P0 · MVP · Windows, Linux, macOS
  - **Beschreibung:** Zeigt, welche Mods durch Deinstallation brechen; alternative Aktion deaktivieren.
  - **Nutzerwert:** Verhindert kaputte Profile
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Modrinth App dependency deletion warning
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://modrinth.com/news/changelog)

- [ ] **F-099 – Virtuelle Fähigkeiten / Provides**
  - **Einordnung:** P2 · Multi-Game · Windows, Linux, macOS
  - **Beschreibung:** Mehrere Implementierungen können dieselbe Fähigkeit erfüllen.
  - **Nutzerwert:** Flexibler Ökosystem-Resolver
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Paketmanager-Muster
  - **Evidenz:** Produktempfehlung

## Konflikte

- [ ] **F-100 – Dateikonflikte erkennen**
  - **Einordnung:** P0 · MVP · Windows, Linux, macOS
  - **Beschreibung:** Ermittelt mehrere Mods, die denselben Zielpfad liefern; Gewinner und Quelle anzeigen.
  - **Nutzerwert:** Sichtbarkeit
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Vortex conflict management; MO2 priorities
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://github.com/Nexus-Mods/Vortex)

- [ ] **F-101 – Konfliktregeln Before/After**
  - **Einordnung:** P1 · Kompatibilität · Windows, Linux, macOS
  - **Beschreibung:** Nutzer oder Autor definiert Priorität mit graphischer Regelansicht und Zyklenerkennung.
  - **Nutzerwert:** Kontrollierbare Überschreibungen
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Vortex rules; Nexus Nutzerwunsch nach Load-Order-Metadaten
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://www.nexusmods.com/news/15433)

- [ ] **F-102 – Plugin-/Load-Order-Verwaltung**
  - **Einordnung:** P1 · Spieladapter · Windows, Linux, macOS
  - **Beschreibung:** Game-Adapter verwaltet Plugins, Regeln, Groups und Auto-Sort, sofern das Spiel dies benötigt.
  - **Nutzerwert:** Spiel startet korrekt
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Vortex/LOOT; MO2
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://github.com/Nexus-Mods/Vortex)

- [ ] **F-103 – Bekannte Inkompatibilitäten**
  - **Einordnung:** P1 · Kompatibilität · Web/App
  - **Beschreibung:** Serverseitige Regeln plus lokale Erkennung; Ersatz/Kompatibilitätspatch vorschlagen.
  - **Nutzerwert:** Weniger Crashes
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** CurseForge incompatible relation; Marktchance
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://support.curseforge.com/support/solutions/articles/9000197242-file-project-types-and-additional-fields)

- [ ] **F-104 – Konfigurations-Merge**
  - **Einordnung:** P2 · Power User · Windows, Linux, macOS
  - **Beschreibung:** Strukturierte Merge-Strategien für JSON/YAML/INI statt blindem Überschreiben.
  - **Nutzerwert:** Updates erhalten Nutzereinstellungen
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Marktchance
  - **Evidenz:** Produktempfehlung

## Profile

- [ ] **F-105 – Mehrere Profile/Instanzen**
  - **Einordnung:** P0 · MVP · Windows, Linux, macOS
  - **Beschreibung:** Isolierte Modlisten je Spielversion, Server oder Spielstil.
  - **Nutzerwert:** Kernwunsch erfahrener Nutzer
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Vortex profiles; r2modman; Prism
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://thunderstore.io/c/patch-quest/p/ebkr/r2modman/)

- [ ] **F-106 – Profil duplizieren und Gruppen**
  - **Einordnung:** P1 · Profile · Windows, Linux, macOS
  - **Beschreibung:** Klonen, benennen, ordnen, archivieren und als Vorlage verwenden.
  - **Nutzerwert:** Schnelle Experimente
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** GDLauncher/Modrinth App groups
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://gdlauncher.com/docs/gdlauncher-vs-gdlauncher-carbon)

- [ ] **F-107 – Profilbezogene Konfigurationen**
  - **Einordnung:** P0 · MVP · Windows, Linux, macOS
  - **Beschreibung:** Configs und INIs werden isoliert oder per Overlay verwaltet.
  - **Nutzerwert:** Keine gegenseitige Beeinflussung
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** r2modman export configs; MO2 profiles
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://thunderstore.io/c/patch-quest/p/ebkr/r2modman/)

- [ ] **F-108 – Profilbezogene Saves**
  - **Einordnung:** P1 · Profile · Windows, Linux, macOS
  - **Beschreibung:** Optional separate Savegames mit Warnung vor inkompatiblen Modlisten.
  - **Nutzerwert:** Schützt Spielstände
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Vortex profile-specific saves
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://github.com/Nexus-Mods/Vortex)

- [ ] **F-109 – Profil teilen/exportieren**
  - **Einordnung:** P0 · MVP · Windows, Linux, macOS
  - **Beschreibung:** Portables Lockfile/Pack mit Manifest, Konfigurationen und Overrides; keine unerlaubten Binärdateien.
  - **Nutzerwert:** Reproduzierbare Setups
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** CurseForge, Thunderstore, Modrinth, r2modman
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://support.curseforge.com/support/solutions/articles/9000198501-exporting-modpacks)

- [ ] **F-110 – Share-Code / Remote-Import**
  - **Einordnung:** P1 · Profile · Windows, Linux, macOS
  - **Beschreibung:** Profil serverseitig speichern und per kurzem Code importieren.
  - **Nutzerwert:** Einfaches Teilen
  - **Premium:** Optional Premium
  - **Beispiele/Wettbewerber:** CurseForge/r2modman Profile Codes
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://support.curseforge.com/support/solutions/articles/9000198501-exporting-modpacks)

- [ ] **F-111 – Lockfile und Reproduzierbarkeit**
  - **Einordnung:** P0 · MVP · Windows, Linux, macOS
  - **Beschreibung:** Exakte Projekt-/Datei-IDs, Hashes, Quellen und Installationsoptionen festschreiben.
  - **Nutzerwert:** Gleiches Ergebnis auf jedem Gerät
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Modrinth mrpack; Wabbajack
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://support.modrinth.com/en/articles/8802351-modrinth-modpack-format-mrpack)

## Backup

- [ ] **F-112 – Automatischer Snapshot vor Änderung**
  - **Einordnung:** P0 · MVP · Windows, Linux, macOS
  - **Beschreibung:** Vor Install/Update/Remove/Loaderwechsel werden Manifest, Konfiguration und betroffene Dateien gesichert.
  - **Nutzerwert:** Sicheres Experimentieren
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Modrinth App backups/rollback
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://modrinth.com/news/changelog)

- [ ] **F-113 – Ein-Klick-Rollback**
  - **Einordnung:** P0 · MVP · Windows, Linux, macOS
  - **Beschreibung:** Letzte Transaktion oder beliebigen Snapshot wiederherstellen; Savegame separat behandeln.
  - **Nutzerwert:** Schnelle Recovery
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Modrinth App transactional rollback
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://modrinth.com/news/changelog)

- [ ] **F-114 – Integritätsprüfung und Reparatur**
  - **Einordnung:** P1 · Stabilität · Windows, Linux, macOS
  - **Beschreibung:** Vergleicht Besitzdaten/Hashes, findet fehlende oder fremd veränderte Dateien und repariert selektiv.
  - **Nutzerwert:** Diagnose
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Steam verify + Manager-Metadaten; Marktchance
  - **Evidenz:** Produktempfehlung

- [ ] **F-115 – Speicherbereinigung**
  - **Einordnung:** P1 · Stabilität · Windows, Linux, macOS
  - **Beschreibung:** Nicht referenzierte Downloads, alte Snapshots und Duplikate sicher anzeigen und löschen.
  - **Nutzerwert:** Platz sparen
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Launcher/Manager-Muster
  - **Evidenz:** Wettbewerberparität

## Sicherheit

- [ ] **F-116 – Hashprüfung vor Installation**
  - **Einordnung:** P0 · MVP · Windows, Linux, macOS
  - **Beschreibung:** SHA-256 oder stärker für jedes Artefakt; Hash aus signiertem Manifest/Backend.
  - **Nutzerwert:** Manipulation erkennen
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Modrinth SHA1/SHA512; Plattform-Scanner
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://support.modrinth.com/en/articles/8802351-modrinth-modpack-format-mrpack)

- [ ] **F-117 – Malware-/Archive-Scan**
  - **Einordnung:** P0 · MVP · Web/App/Backend
  - **Beschreibung:** Server- und optional lokaler Scan; Archive Bombs, Pfadtraversal, Executables und Skripte prüfen.
  - **Nutzerwert:** Kernvertrauen
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** CurseForge/Nexus Scanner
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://support.curseforge.com/en/support/solutions/articles/9000210425-curseforge-file-processor-errors-per-game)

- [ ] **F-118 – Signierte Manifeste und Publisher-Keys**
  - **Einordnung:** P1 · Sicherheit · Web/App/CLI/API
  - **Beschreibung:** Ed25519/Sigstore-ähnliche Signaturen, Key-Rotation und Widerruf; Signaturstatus sichtbar.
  - **Nutzerwert:** Supply-Chain-Sicherheit
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Marktchance; Paketregistry-Muster
  - **Evidenz:** Produktempfehlung

- [ ] **F-119 – Quarantäne und Freigabe**
  - **Einordnung:** P0 · MVP · Windows, Linux, macOS
  - **Beschreibung:** Verdächtige Downloads werden nicht deployed; Nutzer kann Bericht sehen, aber keine Warnung blind umgehen.
  - **Nutzerwert:** Verhindert Schadcode
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Nexus quarantine; CurseForge processing
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://support.curseforge.com/en/support/solutions/articles/9000210425-curseforge-file-processor-errors-per-game)

- [ ] **F-120 – Installations-Sandbox / beschränkte Scripts**
  - **Einordnung:** P1 · Sicherheit · Windows, Linux, macOS
  - **Beschreibung:** Installer-Skripte deklarativ oder in Sandbox mit minimalen Rechten; Netzwerkzugriff standardmäßig aus.
  - **Nutzerwert:** Reduziert Risiko
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Marktchance
  - **Evidenz:** Produktempfehlung

## Konfiguration

- [ ] **F-121 – In-App-Konfigurationseditor**
  - **Einordnung:** P1 · Power User · Windows, Linux, macOS
  - **Beschreibung:** Schema-basierte Formulare plus Rohtext für JSON/YAML/TOML/INI; Validierung und Backup.
  - **Nutzerwert:** Mods ohne Dateisuche konfigurieren
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** r2modman config editor
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://thunderstore.io/c/patch-quest/p/ebkr/r2modman/)

- [ ] **F-122 – Dateieditor mit Suche/Diff**
  - **Einordnung:** P2 · Power User · Windows, Linux, macOS
  - **Beschreibung:** Syntaxhighlighting, Find/Replace, Vergleich gegen Default und Restore.
  - **Nutzerwert:** Power-User-Komfort
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Modrinth App Files tab
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://modrinth.com/news/changelog)

## Diagnose

- [ ] **F-123 – Logviewer und Exportbundle**
  - **Einordnung:** P0 · MVP · Windows, Linux, macOS
  - **Beschreibung:** Spiel-/Loader-/App-Logs filtern, sensible Daten schwärzen und als Supportbundle exportieren.
  - **Nutzerwert:** Schnellere Hilfe
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Modrinth App logs; Manager-Muster
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://modrinth.com/news/changelog)

- [ ] **F-124 – Crash-Erkennung und Mod-Bisect**
  - **Einordnung:** P2 · Diagnose · Windows, Linux, macOS
  - **Beschreibung:** Letzten Crash erkennen, geänderte Mods markieren und optional binäre Suche über Modgruppen automatisieren.
  - **Nutzerwert:** Findet problematische Mods
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Marktchance
  - **Evidenz:** Produktempfehlung

- [ ] **F-125 – Kompatibilitäts-Preflight**
  - **Einordnung:** P0 · MVP · Windows, Linux, macOS
  - **Beschreibung:** Vor Start: fehlende Dependencies, falscher Loader, falsche Architektur, defekte Pfade und bekannte Konflikte prüfen.
  - **Nutzerwert:** Weniger Startfehler
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Alle guten Manager teilweise
  - **Evidenz:** Produktempfehlung

- [ ] **F-126 – Health Dashboard**
  - **Einordnung:** P1 · Stabilität · Windows, Linux, macOS
  - **Beschreibung:** Status für Spiel, Loader, Profil, Scan, Updates, Konflikte, Speicher und letzte Sicherung.
  - **Nutzerwert:** Sofortige Übersicht
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Marktchance
  - **Evidenz:** Produktempfehlung

## Creator Tools

- [ ] **F-127 – Package Builder**
  - **Einordnung:** P0 · Creator MVP · Windows, Linux, macOS
  - **Beschreibung:** Ordner wählen, Include/Exclude-Regeln, Manifest, Icon, README und Overrides zu validem Paket bauen.
  - **Nutzerwert:** Niedrige Publishing-Hürde
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Thunderstore package format; CurseForge export
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://wiki.thunderstore.io/mods/creating-a-package)

- [ ] **F-128 – Manifest-Editor mit Schema**
  - **Einordnung:** P0 · Creator MVP · Windows, Linux, macOS
  - **Beschreibung:** Autocomplete, Dokumentation, Dependency Picker, SemVer-Prüfung und Migrationsassistent.
  - **Nutzerwert:** Weniger Uploadfehler
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Thunderstore validator; mrpack schema
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://thunderstore.io/tools/manifest-v1-validator/)

- [ ] **F-129 – Lokaler Testmodus**
  - **Einordnung:** P1 · Creator · Windows, Linux, macOS
  - **Beschreibung:** Paket in temporäre Testinstanz installieren, Smoke-Test starten und Ergebnisbericht erzeugen.
  - **Nutzerwert:** Bessere Releases
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Steam Workshop test; Modding-Tool-Muster
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://partner.steamgames.com/doc/features/workshop/implementation)

- [ ] **F-130 – Publish aus App**
  - **Einordnung:** P1 · Creator · Windows, Linux, macOS
  - **Beschreibung:** Login/Token, Projekt auswählen, Paket hochladen, Metadaten/Changelog setzen und Status verfolgen.
  - **Nutzerwert:** Ein durchgängiger Workflow
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Steam in-game uploader; Nexus Publish API
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://www.nexusmods.com/news/15454)

- [ ] **F-131 – Watch-Folder / Dev Link**
  - **Einordnung:** P2 · Creator · Windows, Linux, macOS
  - **Beschreibung:** Lokales Build wird per Link/Overlay in Testprofil eingebunden; automatische Reload-Hooks spielabhängig.
  - **Nutzerwert:** Schneller Entwicklungszyklus
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** MO2/VFS- und Dev-Tool-Muster
  - **Evidenz:** Produktempfehlung

- [ ] **F-132 – CLI-Integration aus App erzeugen**
  - **Einordnung:** P1 · Creator · Windows, Linux, macOS
  - **Beschreibung:** App generiert reproduzierbaren CLI-Befehl/Config und API-Token mit minimalem Scope.
  - **Nutzerwert:** Brücke zu CI/CD
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Nexus GitHub Action; SteamCMD
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://github.com/Nexus-Mods/upload-action)

## UX

- [ ] **F-133 – Globale und profilspezifische Einstellungen**
  - **Einordnung:** P0 · MVP · Windows, Linux, macOS
  - **Beschreibung:** Sprache, Theme, Downloadpfad, Cache, Telemetrie, Startverhalten und Regeln sauber trennen.
  - **Nutzerwert:** Vorhersehbares Verhalten
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Alle Manager
  - **Evidenz:** Wettbewerberparität

- [ ] **F-134 – Barrierefreiheit**
  - **Einordnung:** P0 · MVP · Windows, Linux, macOS
  - **Beschreibung:** Tastaturnavigation, Screenreader, skalierbare Schrift, reduzierte Bewegung, hoher Kontrast.
  - **Nutzerwert:** Breitere Nutzbarkeit
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Produktstandard
  - **Evidenz:** Produktempfehlung

- [ ] **F-135 – Benachrichtigungszentrum**
  - **Einordnung:** P1 · MVP+ · Windows, Linux, macOS
  - **Beschreibung:** Updates, Konflikte, abgebrochene Downloads, Moderation und Sicherheitswarnungen mit Priorität.
  - **Nutzerwert:** Nichts Wichtiges übersehen
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Manager-Muster
  - **Evidenz:** Produktempfehlung

- [ ] **F-136 – Offline-Modus**
  - **Einordnung:** P1 · Stabilität · Windows, Linux, macOS
  - **Beschreibung:** Installierte Profile starten, lokale Pakete verwalten und Queues für später vorbereiten.
  - **Nutzerwert:** Resilienz
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Prism/Launcher-Muster
  - **Evidenz:** Wettbewerberparität

- [ ] **F-137 – Deep Links und Dateizuordnungen**
  - **Einordnung:** P0 · MVP · Windows, Linux, macOS
  - **Beschreibung:** Eigene URI-Schemata sowie .modpack/.manifest-Dateien öffnen sicher die passende Ansicht.
  - **Nutzerwert:** Nahtlose Website-Integration
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** CurseForge deeplinks; Wabbajack
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://support.curseforge.com/support/solutions/articles/9000193488-getting-started)

## Windows

- [ ] **F-138 – Windows-Integration**
  - **Einordnung:** P0 · MVP · Windows 10/11
  - **Beschreibung:** Registry-/Store-Erkennung, Long Paths, UAC-Vermeidung, Defender-kompatible Staging-Pfade, Installer und Auto-Update.
  - **Nutzerwert:** Stabile Hauptplattform
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** CurseForge/Vortex/MO2/Wabbajack
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://www.nexusmods.com/about/vortex/)

## Linux

- [ ] **F-139 – Native Linux- und Steam-Deck-Unterstützung**
  - **Einordnung:** P1 · Cross-Platform · Linux/SteamOS
  - **Beschreibung:** XDG-Pfade, Flatpak-Portals, AppImage/Flatpak, Proton-Präfixe, case-sensitive Dateisysteme und Gamescope.
  - **Nutzerwert:** Wachsende Zielgruppe
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** CurseForge Linux eingeschränkt; r2modman/Prism/Modrinth
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://support.curseforge.com/support/solutions/articles/9000193488-getting-started)

## macOS

- [ ] **F-140 – Native macOS-Unterstützung**
  - **Einordnung:** P2 · Cross-Platform · macOS 12+
  - **Beschreibung:** Universal Binary, notarisiert, Sandbox/File-Bookmarks, case sensitivity und Gatekeeper verständlich behandeln.
  - **Nutzerwert:** Mac-Nutzer nicht ausschließen
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** CurseForge/Modrinth/Prism/GDLauncher
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://support.curseforge.com/support/solutions/articles/9000193488-getting-started)

## Android

- [ ] **F-141 – Android nur bei game-native Unterstützung**
  - **Einordnung:** P3 · Mobile · Android
  - **Beschreibung:** Lokale Installation nur, wenn das konkrete Android-Spiel und dessen Modloader dies offiziell ermöglichen; Storage Access Framework nutzen.
  - **Nutzerwert:** Vermeidet falsche Versprechen
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Keine allgemeine Desktop-Mod-Parität
  - **Evidenz:** Produktempfehlung

- [ ] **F-142 – Desktop-Companion-Funktionen**
  - **Einordnung:** P2 · Mobile · Android/iOS
  - **Beschreibung:** Browse, Queue-to-PC, Push, Profilstatus, Remote-Start optional und sichere Gerätefreigabe.
  - **Nutzerwert:** Sinnvoller mobiler Mehrwert
  - **Premium:** Optional Premium
  - **Beispiele/Wettbewerber:** Marktchance
  - **Evidenz:** Produktempfehlung

## Multi-Game

- [ ] **F-143 – Game-Adapter/Plugin-System**
  - **Einordnung:** P0 · Architektur · Alle Desktop-OS
  - **Beschreibung:** Jedes Spiel liefert Erkennung, Version, Installationsziele, Loader, Konfliktmodell, Start und Save-Handling über stabile Interfaces.
  - **Nutzerwert:** Skalierbare Erweiterung
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Vortex game extensions; MO2 plugins
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://github.com/Nexus-Mods/Vortex)

- [ ] **F-144 – Capability-basiertes UI**
  - **Einordnung:** P0 · Architektur · Alle Desktop-OS
  - **Beschreibung:** UI zeigt nur Funktionen, die der Game-Adapter unterstützt, statt universelle Annahmen zu erzwingen.
  - **Nutzerwert:** Weniger Sonderfälle
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Produktempfehlung
  - **Evidenz:** Produktempfehlung

- [ ] **F-145 – Adapter-SDK und Testkit**
  - **Einordnung:** P2 · Multi-Game · Windows, Linux, macOS
  - **Beschreibung:** Versioniertes SDK, Mock-Spiel, Contract-Tests und Kompatibilitätsmatrix für Community-Adapter.
  - **Nutzerwert:** Schneller Spieleausbau
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Vortex extension ecosystem
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://github.com/Nexus-Mods/Vortex/wiki)

## Server

- [ ] **F-146 – Serverprofil und Headless-Installation**
  - **Einordnung:** P2 · Server · Windows/Linux Server
  - **Beschreibung:** Separate Serverinstanz, CLI/Daemon, Config-Templates und Mod-Sync zu Clients.
  - **Nutzerwert:** Mehr Anwendungsfälle
  - **Premium:** Optional Premium
  - **Beispiele/Wettbewerber:** Steam dedicated servers; GDLauncher server manager
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://partner.steamgames.com/doc/features/workshop/implementation)

---

# Backend-Funktionen

**Anzahl Funktionen:** 70

## Architektur

- [ ] **F-147 – Game-Adapter-Servicevertrag**
  - **Einordnung:** P0 · Architektur · Backend/SDK
  - **Beschreibung:** Versioniertes Interface für Spielsuche, Versionen, Installationsplan, Loader, Start, Saves, Konflikte und Uninstall.
  - **Nutzerwert:** Basis für spätere Spiele
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Vortex extensions; Steam game-defined content
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://github.com/Nexus-Mods/Vortex/wiki)

- [ ] **F-148 – Capability Registry**
  - **Einordnung:** P0 · Architektur · Backend/API/App
  - **Beschreibung:** Pro Spiel und OS deklarierte Fähigkeiten, Versionen und Grenzen; Client kann UI dynamisch anpassen.
  - **Nutzerwert:** Verhindert falsche Annahmen
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Produktempfehlung
  - **Evidenz:** Produktempfehlung

- [ ] **F-149 – Multi-Tenant-Spielkatalog**
  - **Einordnung:** P0 · MVP · Backend/API
  - **Beschreibung:** Spiele, Editionen, Stores, Builds, DLCs, Loader und Aliasnamen zentral modellieren.
  - **Nutzerwert:** Saubere Skalierung
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** CurseForge Games API; Nexus games
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://docs.curseforge.com/rest-api/)

## Identität

- [ ] **F-150 – OAuth 2.1/OIDC und Device Flow**
  - **Einordnung:** P0 · MVP · Backend/API
  - **Beschreibung:** Web-Login, Desktop-Device-Code, Refresh-Token-Rotation und PKCE.
  - **Nutzerwert:** Sicherer App-Login
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** API-Plattformstandard
  - **Evidenz:** Produktempfehlung

- [ ] **F-151 – API Keys und Personal Access Tokens**
  - **Einordnung:** P0 · Creator MVP · Backend/API/CLI
  - **Beschreibung:** Scopes, Ablaufdatum, IP-/App-Bindung, Rotation und letzter Gebrauch.
  - **Nutzerwert:** Sichere Automatisierung
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** CurseForge/Nexus APIs
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://docs.curseforge.com/rest-api/)

- [ ] **F-152 – Rollen und Rechte**
  - **Einordnung:** P0 · MVP · Backend/Web
  - **Beschreibung:** Organisation, Team, Projekt- und Versionsrollen mit Least Privilege.
  - **Nutzerwert:** Creator-Teams
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Thunderstore Teams; Nexus Co-Autoren
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://www.nexusmods.com/news/15454)

- [ ] **F-153 – 2FA/Passkeys und Risikomodell**
  - **Einordnung:** P0 · MVP · Backend/Web/App
  - **Beschreibung:** WebAuthn/TOTP, verdächtige Logins, Session-Widerruf und Recovery Codes.
  - **Nutzerwert:** Kontosicherheit
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Sicherheitsstandard
  - **Evidenz:** Produktempfehlung

## Katalog

- [ ] **F-154 – Projekt-/Versions-/Datei-Datenmodell**
  - **Einordnung:** P0 · MVP · Backend/API
  - **Beschreibung:** Stabile IDs, veränderbare Slugs, Projekttypen, Releasekanäle, Sichtbarkeit und Status.
  - **Nutzerwert:** API-Stabilität
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Modrinth stable IDs; CurseForge files
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://docs.modrinth.com/api/)

- [ ] **F-155 – Kompatibilitätsmodell**
  - **Einordnung:** P0 · MVP · Backend/API/App
  - **Beschreibung:** Spielbuilds, Loader, OS, Architektur, Client/Server, DLC und Edition als normalisierte Constraints.
  - **Nutzerwert:** Verhindert falsche Updates
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Modrinth/CurseForge
  - **Evidenz:** Wettbewerberparität

- [ ] **F-156 – Taxonomie und Tags**
  - **Einordnung:** P0 · MVP · Backend/API/Web
  - **Beschreibung:** Globale und spielspezifische Kategorien, Synonyme, Hierarchie, Moderation und Übersetzungen.
  - **Nutzerwert:** Bessere Suche
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Steam Workshop Tags; CurseForge categories
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://partner.steamgames.com/doc/features/workshop/tags)

- [ ] **F-157 – Lizenz-/Permission-Modell**
  - **Einordnung:** P0 · MVP · Backend/API/Web
  - **Beschreibung:** SPDX plus granulare Rechte für Weitergabe, Modpacks, Assets, Forks und kommerzielle Nutzung.
  - **Nutzerwert:** Rechtssicherheit
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Plattformvergleich
  - **Evidenz:** Produktempfehlung

## Suche

- [ ] **F-158 – Suchindex und Facetten**
  - **Einordnung:** P0 · MVP · Backend/API/Web
  - **Beschreibung:** Indexiert Projekte, Versionen, Autoren, Tags und Kompatibilität; near-real-time Updates.
  - **Nutzerwert:** Schnelle Discovery
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Alle Plattformen
  - **Evidenz:** Wettbewerberparität

- [ ] **F-159 – Ranking und Anti-Gaming**
  - **Einordnung:** P1 · Discovery · Backend
  - **Beschreibung:** Relevanz, Aktualität, Qualität, Retention und Sicherheitsstatus; Bot-/Download-Manipulation erkennen.
  - **Nutzerwert:** Faire Auffindbarkeit
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Plattformstandard
  - **Evidenz:** Produktempfehlung

- [ ] **F-160 – Personalisierte Empfehlungen**
  - **Einordnung:** P2 · Empfehlungen · Backend/API
  - **Beschreibung:** Opt-in Embeddings/Collaborative Signals, negative Signale und transparente Warum-Erklärung.
  - **Nutzerwert:** Passendere Mods
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Marktchance
  - **Evidenz:** Produktempfehlung

## Dateien

- [ ] **F-161 – Objektspeicher und CDN**
  - **Einordnung:** P0 · MVP · Backend
  - **Beschreibung:** Immutable Artefakte in Object Storage, geo-verteilte CDN-Auslieferung, Origin-Schutz.
  - **Nutzerwert:** Skalierbare Downloads
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Alle Hosting-Plattformen
  - **Evidenz:** Wettbewerberparität

- [ ] **F-162 – Direkt-/resumierbarer Upload**
  - **Einordnung:** P0 · MVP · Backend/API/Web/CLI
  - **Beschreibung:** Multipart Upload, presigned URLs, Fortschritt, Resume, Größenlimits und Finalize-Schritt.
  - **Nutzerwert:** Robuster Creator-Upload
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Nexus direct cloud upload
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://www.nexusmods.com/news/15454)

- [ ] **F-163 – Content-addressed Storage**
  - **Einordnung:** P0 · MVP · Backend/App
  - **Beschreibung:** SHA-256/512 als Identität, Deduplizierung und immutable Blob-Referenzen.
  - **Nutzerwert:** Integrität und Kosten
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Modrinth hashes; Wabbajack reproducibility
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://support.modrinth.com/en/articles/8802351-modrinth-modpack-format-mrpack)

- [ ] **F-164 – Download-URL-Service**
  - **Einordnung:** P0 · MVP · Backend/API
  - **Beschreibung:** Kurzlebige signierte URLs, Region/Mirror, Accounttier, Rate Limit und Audit.
  - **Nutzerwert:** Sichere Auslieferung
  - **Premium:** Optional Premium
  - **Beispiele/Wettbewerber:** CurseForge Get Download URL; Nexus
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://docs.curseforge.com/rest-api/)

- [ ] **F-165 – Range Requests und Resume**
  - **Einordnung:** P0 · MVP · Backend/CDN
  - **Beschreibung:** HTTP Range, ETag, If-Match und stabile Content-Length; Teilstücke nach Hash prüfen.
  - **Nutzerwert:** Zuverlässige große Downloads
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Webstandard
  - **Evidenz:** Produktempfehlung

- [ ] **F-166 – Delta-/Patch-Updates**
  - **Einordnung:** P3 · Performance · Backend/App
  - **Beschreibung:** Optional binäre Deltas bei großen Paketen, mit vollständigem Fallback und Hashprüfung.
  - **Nutzerwert:** Weniger Traffic
  - **Premium:** Optional Premium
  - **Beispiele/Wettbewerber:** Marktchance
  - **Evidenz:** Produktempfehlung

## Manifest

- [ ] **F-167 – Versioniertes Manifest-Schema**
  - **Einordnung:** P0 · MVP · Backend/API/App/CLI
  - **Beschreibung:** JSON Schema mit schemaVersion, stabilen IDs, Dateien, Hashes, Zielpfaden, Dependencies, Kompatibilität und Optionen.
  - **Nutzerwert:** Portabilität
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Thunderstore; Modrinth; CurseForge modpacks
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://support.modrinth.com/en/articles/8802351-modrinth-modpack-format-mrpack)

- [ ] **F-168 – Lockfile getrennt vom Autorenmanifest**
  - **Einordnung:** P0 · MVP · Backend/App/CLI
  - **Beschreibung:** Autorenmanifest darf Ranges enthalten; Resolver erzeugt Lockfile mit exakten Versionen, Quellen und Hashes.
  - **Nutzerwert:** Reproduzierbarkeit und Updates
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Paketmanager-Muster; Wabbajack/mrpack
  - **Evidenz:** Produktempfehlung

- [ ] **F-169 – Schema Registry und Migration**
  - **Einordnung:** P0 · MVP · Backend/API/SDK
  - **Beschreibung:** Alle Schemaversionen, JSON Schema, Beispiele und automatische Upgrades; alte Clients erhalten klare Mindestversion.
  - **Nutzerwert:** Langfristige Kompatibilität
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Produktempfehlung
  - **Evidenz:** Produktempfehlung

- [ ] **F-170 – Override- und Installationsregeln**
  - **Einordnung:** P0 · MVP · Backend/App
  - **Beschreibung:** Explizite Zielwurzel, include/exclude, client/server overrides, Merge-Strategie und keine Pfadtraversal.
  - **Nutzerwert:** Sichere Pakete
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Modrinth overrides/server-overrides; CurseForge overrides
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://support.modrinth.com/en/articles/8802351-modrinth-modpack-format-mrpack)

- [ ] **F-171 – Dependency-Syntax**
  - **Einordnung:** P0 · MVP · Backend/API/App
  - **Beschreibung:** Required, optional, incompatible, embedded, tools; SemVer/Build-Ranges und Feature-Flags.
  - **Nutzerwert:** Maschinenlesbarer Resolver
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** CurseForge relations; Thunderstore strings
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://support.curseforge.com/support/solutions/articles/9000197242-file-project-types-and-additional-fields)

- [ ] **F-172 – Environment-Metadaten**
  - **Einordnung:** P0 · MVP · Backend/API/App
  - **Beschreibung:** Client, Server oder beide; required/optional/unsupported; OS/Architektur wenn nötig.
  - **Nutzerwert:** Korrekte Zielauswahl
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Modrinth mrpack env
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://support.modrinth.com/en/articles/8802351-modrinth-modpack-format-mrpack)

- [ ] **F-173 – Publisher-Signatur**
  - **Einordnung:** P1 · Sicherheit · Backend/App/CLI
  - **Beschreibung:** Manifest und Artefaktliste signieren; Key-ID, Zeitstempel, Transparency Log und Revocation.
  - **Nutzerwert:** Supply-Chain-Sicherheit
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Produktempfehlung
  - **Evidenz:** Produktempfehlung

## Resolver

- [ ] **F-174 – Dependency-Graph-Service**
  - **Einordnung:** P0 · MVP · Backend/API/App
  - **Beschreibung:** Graphabfragen für direkte/transitive Dependencies, Dependants, Konflikte und Zyklen.
  - **Nutzerwert:** Installationsgrundlage
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Thunderstore/CurseForge/Modrinth
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://wiki.thunderstore.io/sharing-your-mods/modpacks-and-profiles)

- [ ] **F-175 – Constraint Solver**
  - **Einordnung:** P0 · MVP · Backend/App
  - **Beschreibung:** Findet kompatible Versionen unter Spiel-/Loader-/OS-Constraints und erklärt unlösbare Kerne.
  - **Nutzerwert:** Zuverlässige Modpacks
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Produktempfehlung
  - **Evidenz:** Produktempfehlung

- [ ] **F-176 – Update Planner**
  - **Einordnung:** P0 · MVP · Backend/App
  - **Beschreibung:** Berechnet minimalen sicheren Änderungsplan, Diff, Downloads, Speicher und Rollbackpunkt.
  - **Nutzerwert:** Verständliche Updates
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Produktempfehlung
  - **Evidenz:** Produktempfehlung

- [ ] **F-177 – Conflict/Load-Order Rules**
  - **Einordnung:** P1 · Spieladapter · Backend/App
  - **Beschreibung:** Dateikonflikte, before/after, Gruppen, autorisierte Patches und Zyklenerkennung.
  - **Nutzerwert:** Spielabhängige Korrektheit
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Vortex/MO2
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://github.com/Nexus-Mods/Vortex)

## Sicherheit

- [ ] **F-178 – Upload-Pipeline**
  - **Einordnung:** P0 · MVP · Backend
  - **Beschreibung:** Quarantäne → Entpackprüfung → MIME/Magic → Hash → Malware → Policy → Moderation → Publish.
  - **Nutzerwert:** Sicheres Hosting
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** CurseForge/Nexus
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://support.curseforge.com/en/support/solutions/articles/9000210425-curseforge-file-processor-errors-per-game)

- [ ] **F-179 – Archive-Schutz**
  - **Einordnung:** P0 · MVP · Backend/App
  - **Beschreibung:** Größen-/Dateizahllimits, Zip Slip, Symlinks, verschachtelte Archive, Bomb Ratio und verschlüsselte Archive behandeln.
  - **Nutzerwert:** Schützt Infrastruktur und Client
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Modrinth path security; Nexus archive rules
  - **Evidenz:** Produktempfehlung
  - **Quelle:** [Quelle](https://support.modrinth.com/en/articles/8802351-modrinth-modpack-format-mrpack)

- [ ] **F-180 – Malware-Scanning und Re-Scan**
  - **Einordnung:** P0 · MVP · Backend
  - **Beschreibung:** Mehrere Engines/Heuristiken, Sandbox, IOC-Updates und Re-Scan veröffentlichter Dateien.
  - **Nutzerwert:** Langfristiger Schutz
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** CurseForge/Nexus
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://support.curseforge.com/en/support/solutions/articles/9000210425-curseforge-file-processor-errors-per-game)

- [ ] **F-181 – SBOM und Binäranalyse**
  - **Einordnung:** P2 · Sicherheit · Backend/Web
  - **Beschreibung:** Optional CycloneDX/SPDX, Bibliotheken, native Binaries, Signaturen und bekannte CVEs erfassen.
  - **Nutzerwert:** Transparenz
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Marktchance
  - **Evidenz:** Produktempfehlung

- [ ] **F-182 – Abuse-/Rechteverletzungs-Workflow**
  - **Einordnung:** P0 · MVP · Backend/Web
  - **Beschreibung:** DMCA/Urheberrecht, Malware, impersonation, personenbezogene Daten und Appeals mit Beweissicherung.
  - **Nutzerwert:** Plattformschutz
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Hosting-Plattformstandard
  - **Evidenz:** Produktempfehlung

## Moderation

- [ ] **F-183 – Automatische und manuelle Prüfung**
  - **Einordnung:** P0 · MVP · Backend/Web
  - **Beschreibung:** Regelengine priorisiert Risiken; Moderator sieht Diff, Scanresultate, Historie und Team.
  - **Nutzerwert:** Skalierbare Qualität
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** CurseForge moderation
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://support.curseforge.com/en/support/solutions/articles/9000197241-creating-and-submitting-a-project)

- [ ] **F-184 – Statusmaschine**
  - **Einordnung:** P0 · MVP · Backend/API/Web
  - **Beschreibung:** Draft, processing, needs changes, approved, rejected, quarantined, deprecated, archived, removed.
  - **Nutzerwert:** Klare Prozesse
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Plattformvergleich
  - **Evidenz:** Produktempfehlung

- [ ] **F-185 – Audit Log**
  - **Einordnung:** P0 · MVP · Backend/Web
  - **Beschreibung:** Unveränderliche Historie für Uploads, Metadaten, Rollen, Moderation, Tokens und Auszahlungen.
  - **Nutzerwert:** Nachvollziehbarkeit
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Sicherheitsstandard
  - **Evidenz:** Produktempfehlung

## API

- [ ] **F-186 – Öffentliche REST API**
  - **Einordnung:** P1 · API · Backend/API
  - **Beschreibung:** Games, Projekte, Versionen, Dateien, Suche, Dependencies, Hash-Lookups, Nutzerbibliothek nach Scope.
  - **Nutzerwert:** Ökosystem und Apps
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** CurseForge/Modrinth/Nexus/Steam
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://docs.curseforge.com/rest-api/)

- [ ] **F-187 – Optional GraphQL**
  - **Einordnung:** P3 · API · Backend/API
  - **Beschreibung:** Für komplexe Webabfragen; persisted queries, Depth-/Cost-Limits und klare Versionierung.
  - **Nutzerwert:** Effiziente Clients
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Marktchance
  - **Evidenz:** Produktempfehlung

- [ ] **F-188 – Hash-/Fingerprint-Endpunkte**
  - **Einordnung:** P0 · MVP · Backend/API/App
  - **Beschreibung:** Datei anhand Hash identifizieren, Updates für Hashmenge prüfen und lokale Imports zuordnen.
  - **Nutzerwert:** Migration und Updateprüfung
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** CurseForge fingerprints; Modrinth hash updates
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://docs.curseforge.com/rest-api/)

- [ ] **F-189 – Rate Limits und Quotas**
  - **Einordnung:** P0 · MVP · Backend/API
  - **Beschreibung:** Pro IP, Nutzer, Token und Endpoint; Headers für Limit/Reset; höhere Quoten beantragbar.
  - **Nutzerwert:** Stabilität und Fairness
  - **Premium:** Optional Premium/Partner
  - **Beispiele/Wettbewerber:** Nexus/Modrinth
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://help.nexusmods.com/article/105-i-have-reached-a-daily-or-hourly-limit-api-requests-have-been-consumed-rate-limit-exceeded-what-does-this-mean)

- [ ] **F-190 – Idempotency und ETags**
  - **Einordnung:** P0 · MVP · Backend/API
  - **Beschreibung:** Sichere Retries für Upload/Publish; Conditional Requests für Katalog und Downloads.
  - **Nutzerwert:** Robuste Integrationen
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** API-Standard
  - **Evidenz:** Produktempfehlung

- [ ] **F-191 – Webhooks**
  - **Einordnung:** P1 · API · Backend/API
  - **Beschreibung:** project.version.published, moderation.changed, scan.failed, dependency.yanked, collection.updated; signierte Zustellung.
  - **Nutzerwert:** CI/CD und Bots
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Marktchance
  - **Evidenz:** Produktempfehlung

- [ ] **F-192 – Event-/Feed-Endpunkte**
  - **Einordnung:** P1 · API · Backend/API
  - **Beschreibung:** Cursor-basierte Änderungsfeeds für Clients und Mirrors, statt Vollkatalog regelmäßig zu laden.
  - **Nutzerwert:** Skalierbare Synchronisierung
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Paketregistry-Muster
  - **Evidenz:** Produktempfehlung

- [ ] **F-193 – SDKs und OpenAPI**
  - **Einordnung:** P1 · API · Backend/SDK
  - **Beschreibung:** OpenAPI-Spezifikation plus gepflegte TypeScript, C#, Rust/Go SDKs und Mockserver.
  - **Nutzerwert:** Niedrige Integrationskosten
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** CurseForge API docs; Steamworks SDK
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://docs.curseforge.com/rest-api/)

## Upload

- [ ] **F-194 – Web-, App-, CLI- und API-Upload**
  - **Einordnung:** P0 · Creator MVP · Backend/Web/App/CLI
  - **Beschreibung:** Gleicher Validierungs- und Publish-Workflow unabhängig vom Kanal.
  - **Nutzerwert:** Creator wählen ihren Workflow
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Steam, Nexus, Plattformen
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://www.nexusmods.com/news/15454)

- [ ] **F-195 – CLI publish/validate/pack**
  - **Einordnung:** P1 · Creator · CLI/API
  - **Beschreibung:** Deterministisches Packen, dry-run, Manifestprüfung, Hashing, Upload und Statuspolling.
  - **Nutzerwert:** Automatisierung
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** SteamCMD; Nexus Action; Thunderstore tools
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://github.com/Nexus-Mods/upload-action)

- [ ] **F-196 – CI/CD-Integration**
  - **Einordnung:** P1 · Creator · CI/API
  - **Beschreibung:** Official GitHub Action und generische CLI; OIDC trusted publishing statt langlebiger Secrets.
  - **Nutzerwert:** Sichere Releases
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Nexus GitHub Action
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://github.com/Nexus-Mods/upload-action)

- [ ] **F-197 – Direct-to-cloud und Resume**
  - **Einordnung:** P0 · MVP · Backend/API
  - **Beschreibung:** Client lädt Blob direkt zum Storage; Control Plane erhält nur Metadaten und Finalize.
  - **Nutzerwert:** Skalierbar und zuverlässig
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Nexus Upload API
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://www.nexusmods.com/news/15454)

- [ ] **F-198 – Duplicate Detection**
  - **Einordnung:** P0 · MVP · Backend
  - **Beschreibung:** Exakte Hashduplikate und sehr ähnliche Archive erkennen; legitime Reuploads mit Grund erlauben.
  - **Nutzerwert:** Weniger Spam und Speicher
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** CurseForge duplicate hash
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://support.curseforge.com/en/support/solutions/articles/9000210425-curseforge-file-processor-errors-per-game)

- [ ] **F-199 – Immutable Versions**
  - **Einordnung:** P0 · MVP · Backend/API
  - **Beschreibung:** Veröffentlichte Artefakte nicht still ersetzen; neue Version oder Yank; Metadatenänderungen auditieren.
  - **Nutzerwert:** Reproduzierbarkeit
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Paketregistry-Muster; Thunderstore Version bump
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://wiki.thunderstore.io/mods/updating-a-package)

- [ ] **F-200 – Preflight- und serverseitige Validierung**
  - **Einordnung:** P0 · MVP · Backend/App/CLI/Web
  - **Beschreibung:** Gleiche JSON-Schema-/Policy-Engine lokal und serverseitig; Fehler mit JSON Pointer und Lösung.
  - **Nutzerwert:** Weniger Iterationen
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Thunderstore manifest validator
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://thunderstore.io/tools/manifest-v1-validator/)

## Sammlungen

- [ ] **F-201 – Collection/Modpack Service**
  - **Einordnung:** P1 · Sammlungen · Backend/API/Web/App
  - **Beschreibung:** Versionierte Sammlung mit Lockfile, Optionen, Overrides, Autoren, Lizenzchecks und Installationsstatistik.
  - **Nutzerwert:** Reproduzierbare Setups
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Nexus/Steam/CurseForge/Modrinth
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://help.nexusmods.com/article/115-guidelines-for-collections)

- [ ] **F-202 – Lizenzkonforme Referenzierung**
  - **Einordnung:** P0 · MVP · Backend/App
  - **Beschreibung:** Standardmäßig nur Quellen/IDs/Hashes referenzieren; Binärdateien nur bei expliziter Berechtigung einbetten.
  - **Nutzerwert:** Respektiert Creator-Rechte
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Wabbajack reproduziert ohne Mod-Reupload
  - **Evidenz:** Offiziell belegt
  - **Quelle:** [Quelle](https://www.wabbajack.org/)

- [ ] **F-203 – Revisionen, Diff und Migration**
  - **Einordnung:** P1 · Sammlungen · Backend/App/Web
  - **Beschreibung:** Immutable Revisionen, Upgradepfad, optionale Migrationsscripts deklarativ/sandboxed.
  - **Nutzerwert:** Sichere Packupdates
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Nexus Collections; Modpack-Versionen
  - **Evidenz:** Wettbewerberparität

## Sync

- [ ] **F-204 – Geräte- und Profil-Sync**
  - **Einordnung:** P2 · Cloud · Backend/App/Web
  - **Beschreibung:** CRDT-/Versionsmodell für Pins, Einstellungen und Profilmetadaten; Konflikte sichtbar auflösen.
  - **Nutzerwert:** Mehrere Geräte
  - **Premium:** Optional Premium
  - **Beispiele/Wettbewerber:** Marktchance
  - **Evidenz:** Produktempfehlung

- [ ] **F-205 – Cloud-Backup-Service**
  - **Einordnung:** P2 · Cloud · Backend/App/Web
  - **Beschreibung:** Verschlüsselte Backups, Quoten, Retention, Restore-Test und getrennte Savegame-Einwilligung.
  - **Nutzerwert:** Recovery
  - **Premium:** Optional Premium
  - **Beispiele/Wettbewerber:** Marktchance
  - **Evidenz:** Produktempfehlung

## Benachrichtigungen

- [ ] **F-206 – Notification Service**
  - **Einordnung:** P1 · Community · Backend/Web/App
  - **Beschreibung:** In-App, Web Push und E-Mail; dedupliziert, priorisiert, digestfähig und granular konfigurierbar.
  - **Nutzerwert:** Relevante Updates
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Plattformstandard
  - **Evidenz:** Produktempfehlung

## Analytics

- [ ] **F-207 – Datenschutzfreundliche Produktanalytik**
  - **Einordnung:** P0 · MVP · Backend/App/Web
  - **Beschreibung:** Opt-in/legitime Interessen sauber trennen, Events minimieren, Pseudonymisierung und Löschpfad.
  - **Nutzerwert:** Produktverbesserung ohne Vertrauensverlust
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** DSGVO-Standard
  - **Evidenz:** Produktempfehlung

- [ ] **F-208 – Creator-Metriken**
  - **Einordnung:** P1 · Creator · Backend/Web
  - **Beschreibung:** Downloads, erfolgreiche Installationen, aktive Profile, Update-/Rollbackrate und Crash-Korrelation aggregiert.
  - **Nutzerwert:** Qualitätsfeedback
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Marktchance
  - **Evidenz:** Produktempfehlung

## Economy

- [ ] **F-209 – Creator-Rewards und Attribution**
  - **Einordnung:** P2 · Creator Economy · Backend/Web
  - **Beschreibung:** Transparente Formel, Fraud Detection, Mindestschwelle, KYC/Steuern, Payout Ledger und Einspruch.
  - **Nutzerwert:** Nachhaltiges Ökosystem
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Nexus Rewards; Modrinth+ creator share
  - **Evidenz:** Wettbewerberparität
  - **Quelle:** [Quelle](https://modrinth.com/news/article/design-refresh)

## Betrieb

- [ ] **F-210 – Job Queue und Event Bus**
  - **Einordnung:** P0 · MVP · Backend
  - **Beschreibung:** Uploads, Scans, Indexierung, Benachrichtigungen und Analytics entkoppeln; idempotente Worker.
  - **Nutzerwert:** Skalierbarkeit
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Architekturstandard
  - **Evidenz:** Produktempfehlung

- [ ] **F-211 – Observability**
  - **Einordnung:** P0 · MVP · Backend
  - **Beschreibung:** Metriken, strukturierte Logs, Traces, SLOs, CDN-/Upload-/Resolver-Dashboards und Alerting.
  - **Nutzerwert:** Stabiler Betrieb
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Betriebsstandard
  - **Evidenz:** Produktempfehlung

- [ ] **F-212 – Feature Flags und gestufte Releases**
  - **Einordnung:** P0 · MVP · Backend/App/Web
  - **Beschreibung:** Server-/Clientflags, Kohorten, Kill Switch und schema-kompatible Rollouts.
  - **Nutzerwert:** Sicheres Ausrollen
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Produktstandard
  - **Evidenz:** Produktempfehlung

- [ ] **F-213 – Backups und Disaster Recovery**
  - **Einordnung:** P0 · MVP · Backend
  - **Beschreibung:** Datenbank PITR, Object-Versioning, Restore-Drills und dokumentierte RPO/RTO.
  - **Nutzerwert:** Geschäftskontinuität
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Betriebsstandard
  - **Evidenz:** Produktempfehlung

- [ ] **F-214 – Statusseite und Incident-Kommunikation**
  - **Einordnung:** P1 · Betrieb · Web/Backend
  - **Beschreibung:** Öffentliche Komponentenstatus, Verlauf, Wartungen und Client-Fallback.
  - **Nutzerwert:** Vertrauen bei Störungen
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** SaaS-Standard
  - **Evidenz:** Produktempfehlung

## Compliance

- [ ] **F-215 – DSGVO, Datenaufbewahrung und Datenresidenz**
  - **Einordnung:** P0 · MVP · Backend/Web
  - **Beschreibung:** Zweckbindung, Löschfristen, AVV/Subprozessoren, Export/Löschung und EU-Optionen.
  - **Nutzerwert:** Rechtssicherheit
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** EU-Rechtsanforderung
  - **Evidenz:** Produktempfehlung

- [ ] **F-216 – Transparenz- und Moderationsberichte**
  - **Einordnung:** P2 · Betrieb · Web/Backend
  - **Beschreibung:** Aggregierte Meldungen, Maßnahmen, Appeals, Scan-False-Positives und Ausfallzahlen.
  - **Nutzerwert:** Plattformvertrauen
  - **Premium:** Nein
  - **Beispiele/Wettbewerber:** Marktchance
  - **Evidenz:** Produktempfehlung

---

# Manifeste, Paketformate und Upload-Workflows

> Steam Workshop und Nexus-Einzelmods besitzen kein universelles spielübergreifendes Pflichtmanifest. Die Installationssemantik wird dort überwiegend vom Spiel, Uploader oder Game-Adapter bestimmt.

## CurseForge – Minecraft Modpack

- **Manifest oder Paket:** manifest.json + overrides/
- **Struktur:** ZIP im Root: manifest.json und overrides/; zusätzliche ausgewählte Profilinhalte im Overrides-Ordner.
- **Wichtige Felder:** manifestType, manifestVersion, name, version, author, minecraft.version, minecraft.modLoaders[], files[], overrides
- **Abhängigkeiten:** files: projectID, fileID, required; Plattformrelationen zusätzlich required/optional/embedded/tools/incompatible.
- **Kompatibilität:** Minecraft-Version und Loader-ID; andere Spiele nutzen andere Formate/Uploads.
- **Client/Server:** Nicht als allgemeines Feld im Minecraft-v1-Manifest; Inhalt liegt in Overrides.
- **Hashes und Signaturen:** Projekt-/File-ID; serverseitige Datei-Hashes/Fingerprints, aber kein standardmäßiges Publisher-Signaturfeld im Manifest.
- **Overrides und Installation:** overrides wird in die Instanzwurzel kopiert.
- **Dateiformate:** ZIP; Dateigrößen- und projektspezifische Regeln.
- **Upload-Kanäle:** Website; App-Export; API für Katalog/Dateien, Upload je nach freigegebenen Schnittstellen/Tools.
- **Upload-Ablauf:** Projekt erstellen → Datei hochladen → Release/Beta/Alpha, Versionen, Changelog und Relations setzen → Verarbeitung/Moderation → Freigabe.
- **Validierung und Moderation:** Archiv-/Malware-/Duplikatprüfung und Moderation; erforderliche Dependencies werden in integrierten Clients automatisch geladen.
- **Versionierung:** Neue Datei/Version; Releasekanäle; zusätzliche Dateien sind nicht immer clientinstallierbar.
- **Quelle:** [Quelle](https://support.curseforge.com/support/solutions/articles/9000198500-exporting-a-modpack-for-curseforge-project-submission)

## Nexus Mods – Einzelmod

- **Manifest oder Paket:** Kein universelles Pflichtmanifest für Modarchive
- **Struktur:** Mod wird typischerweise als Archiv plus Webmetadaten veröffentlicht; Installation ist spiel- und Installer-Extension-abhängig.
- **Wichtige Felder:** Projekt-/Modmetadaten und File-Version/Changelog im Backend; Vortex verwaltet zusätzlich lokale Metadaten.
- **Abhängigkeiten:** Keine plattformweit einheitliche Dependency-Syntax für alle Spiele; Collections/Installerregeln können Anforderungen ausdrücken.
- **Kompatibilität:** Spiel, Kategorie, Version und game-spezifische Installer-/Load-Order-Regeln.
- **Client/Server:** Spielabhängig.
- **Hashes und Signaturen:** Serverseitige Hashes/Scans; kein allgemeines eingebettetes Signaturfeld für jedes Modarchiv.
- **Overrides und Installation:** Durch Vortex Installer-/Deployment-Logik, nicht durch ein universelles Paketmanifest.
- **Dateiformate:** Viele Archivtypen; password-geschützte Archive sind problematisch/nicht scanbar.
- **Upload-Kanäle:** Web; seit 2026 Upload API (Beta), offizielle GitHub Action und Tool-Integrationen.
- **Upload-Ablauf:** Mod anlegen → Datei direkt/cloud hochladen → Metadaten/Changelog → Scan/Quarantäne/Moderation → veröffentlichen.
- **Validierung und Moderation:** Virus-/Sicherheitsprüfung; Quarantäne bei Problemen; API unterstützt automatisierte Uploads.
- **Versionierung:** Neue File-Versionen; veröffentlichte Dateien sollten nicht still ersetzt werden; Collections sind revisioniert.
- **Quelle:** [Quelle](https://www.nexusmods.com/news/15454)

## Nexus Mods – Collections

- **Manifest oder Paket:** Plattformverwaltetes Collection-/Revision-Manifest
- **Struktur:** Referenziert Mods/Dateien und Installationsanweisungen; nicht als allgemein portabler offener Standard behandeln.
- **Wichtige Felder:** Collection-Metadaten, Revision, Regeln/Optionen und referenzierte Nexus-Dateien.
- **Abhängigkeiten:** Abhängigkeiten ergeben sich aus Collection-Inhalten und Mod-/Installerregeln.
- **Kompatibilität:** Spielspezifisch; Vortex-Extension bestimmt Installation.
- **Client/Server:** Spielspezifisch.
- **Hashes und Signaturen:** Dateien werden über Nexus-IDs und Plattformprüfung bezogen.
- **Overrides und Installation:** Vortex führt Installations-/Deployment-Schritte aus.
- **Dateiformate:** Plattformformat, nicht als freies ZIP-Reupload-Modpack gedacht.
- **Upload-Kanäle:** Web-Builder/Collection-Seite; Deep Link zu Vortex.
- **Upload-Ablauf:** Collection erstellen → Revision validieren → veröffentlichen → Nutzer installiert Revision in Vortex.
- **Validierung und Moderation:** Guidelines, Lizenz-/Inhaltsregeln und Moderation.
- **Versionierung:** Immutable/erkennbare Revisionen und Updates.
- **Quelle:** [Quelle](https://help.nexusmods.com/article/115-guidelines-for-collections)

## Steam Workshop – UGC Item

- **Manifest oder Paket:** Kein universelles Modmanifest; Content-Ordner + UGC-Metadaten
- **Struktur:** Steam speichert einen vom Spiel definierten Content-Ordner. Das Spiel entscheidet Dateiformat und Installation.
- **Wichtige Felder:** PublishedFileId, title, description, visibility, tags/KV-Tags, preview, metadata; App-ID und Change Note.
- **Abhängigkeiten:** Optional App-Dependencies und spieldefinierte Abhängigkeiten; keine universelle Paketresolver-Syntax.
- **Kompatibilität:** App-ID, optional Item-Versioning/Spielbranch; Semantik ist spielabhängig.
- **Client/Server:** Spielabhängig.
- **Hashes und Signaturen:** Steam-Backend identifiziert UGC-Item; Integrität/Depotmechanismen, aber kein universelles Publisher-Signaturfeld im Content.
- **Overrides und Installation:** Vom Spiel/Workshop-Implementierer bestimmt.
- **Dateiformate:** Beliebiger vom Spiel akzeptierter Ordnerinhalt.
- **Upload-Kanäle:** In-Game/Editor-Uploader über ISteamUGC; SteamCMD mit VDF; Steamworks Web API für bestimmte Operationen.
- **Upload-Ablauf:** CreateItem → PublishedFileId speichern → StartItemUpdate → Content/Preview/Metadaten setzen → SubmitItemUpdate → rechtliche Vereinbarung ggf. akzeptieren.
- **Validierung und Moderation:** Spiel kann lokal validieren; Steam verarbeitet Upload; Moderation/Visibility nach Spielmodell.
- **Versionierung:** Update desselben PublishedFileId mit Change Note; Item-Versioning kann Kompatibilität mit alten Gamebranches schützen.
- **Quelle:** [Quelle](https://partner.steamgames.com/doc/features/workshop/implementation)

## SteamCMD Workshop Upload

- **Manifest oder Paket:** VDF-Datei workshopitem
- **Struktur:** Lokale VDF verweist auf Contentfolder und Previewfile.
- **Wichtige Felder:** appid, publishedfileid, contentfolder, previewfile, visibility, title, description, changenote
- **Abhängigkeiten:** Nicht Teil des VDF-Standards; spielabhängig.
- **Kompatibilität:** appid; Item-Versionierung optional über Workshop-Implementierung.
- **Client/Server:** Nicht universell.
- **Hashes und Signaturen:** Steam verarbeitet Upload; keine universelle Manifest-Signatur im VDF.
- **Overrides und Installation:** Contentfolder wird vollständig hochgeladen; Spiel interpretiert Inhalt.
- **Dateiformate:** VDF + beliebige Content-Dateien.
- **Upload-Kanäle:** steamcmd +workshop_build_item <vdf>
- **Upload-Ablauf:** VDF schreiben → SteamCMD authentifizieren → Build/Upload → Ergebnis prüfen.
- **Validierung und Moderation:** Spiel-/Steam-Validierung; rechtliche Vereinbarung kann Veröffentlichung blockieren.
- **Versionierung:** publishedfileid für Updates wiederverwenden; changenote pflegen.
- **Quelle:** [Quelle](https://partner.steamgames.com/doc/features/workshop/implementation)

## Thunderstore – Package Manifest v1

- **Manifest oder Paket:** manifest.json im ZIP-Root
- **Struktur:** Typisch: manifest.json, README.md, icon.png und Moddateien im Root/Unterordner.
- **Wichtige Felder:** name, version_number, website_url, description, dependencies[]
- **Abhängigkeiten:** dependencies als Strings im Format Namespace-Package-Version; Modpacks bestehen im Wesentlichen aus Dependencies plus optional Configs.
- **Kompatibilität:** Community/Spiel durch Uploadziel; Kompatibilität häufig über Kategorien/Loaderkonventionen.
- **Client/Server:** Nicht im Kernmanifest standardisiert; Community-/Paketkonventionen.
- **Hashes und Signaturen:** Plattform berechnet Metadaten/Downloads; kein standardmäßiges Publisher-Signaturfeld.
- **Overrides und Installation:** Dateien werden nach Community-/Loaderkonvention installiert; Modpacks/Profile können Configs enthalten.
- **Dateiformate:** ZIP; Icon typischerweise PNG; README Markdown.
- **Upload-Kanäle:** Website Upload; Community-Tools/CLI; Manager installieren über API/Repository.
- **Upload-Ablauf:** Team/Namespace wählen → ZIP bauen → Validator → Upload → neue Paketversion sichtbar.
- **Validierung und Moderation:** Manifest-/Paketvalidator; Versionsnummer muss erhöht werden; Plattformregeln.
- **Versionierung:** version_number bei jedem Update erhöhen; veröffentlichte Versionen sind adressierbar.
- **Quelle:** [Quelle](https://wiki.thunderstore.io/mods/creating-a-package)

## Modrinth – Einzelprojektversion

- **Manifest oder Paket:** Version-Metadaten + hochgeladene Dateien
- **Struktur:** Dateien werden einer Projektversion zugeordnet; API akzeptiert u. a. JAR, ZIP, LITEMOD und MRPACK je Projekttyp.
- **Wichtige Felder:** name/version_number, changelog, dependencies, game_versions, loaders, version_type, featured; files mit filename, primary, hashes, URL, size.
- **Abhängigkeiten:** Dependencies mit project/version/file IDs und dependency_type; API-Modell.
- **Kompatibilität:** game_versions und loaders; Projekt enthält client_side/server_side Support.
- **Client/Server:** Projekt-/Version-Metadaten kennzeichnen Client/Server.
- **Hashes und Signaturen:** SHA-1 und SHA-512 pro Datei; Hash-Endpunkte für Updateprüfung.
- **Overrides und Installation:** Für Einzelmods loader-/spielabhängig.
- **Dateiformate:** Je Projekttyp zulässige Formate.
- **Upload-Kanäle:** Website und REST API; CI kann gegen API publizieren.
- **Upload-Ablauf:** Projekt als Draft anlegen → Version separat erstellen → Dateien hochladen/zuordnen → Metadaten/Dependencies → Moderation/Publish.
- **Validierung und Moderation:** Datei-/Metadatenvalidierung und Moderation.
- **Versionierung:** Separate immutable Versionen; Dateien einer Version sind primär/zusätzlich.
- **Quelle:** [Quelle](https://docs.modrinth.com/api/operations/createversion/)

## Modrinth – .mrpack

- **Manifest oder Paket:** ZIP mit modrinth.index.json
- **Struktur:** Root: modrinth.index.json; optional overrides/ und server-overrides/.
- **Wichtige Felder:** formatVersion, game, versionId, name, summary, files[], dependencies{}
- **Abhängigkeiten:** files sind direkte Artefakte; dependencies enthält Minecraft und Loader-Versionen; Projektabhängigkeiten sind über Download-URLs/Hashes aufgelöst.
- **Kompatibilität:** game='minecraft'; Loader in dependencies; files[].env mit client/server required\|optional\|unsupported.
- **Client/Server:** files[].env; server-overrides werden nach overrides angewendet.
- **Hashes und Signaturen:** files[].hashes: sha1 und sha512; downloads nur HTTPS; fileSize.
- **Overrides und Installation:** overrides in Instanz; server-overrides überschreibt serverseitig nach allgemeinen Overrides.
- **Dateiformate:** .mrpack (ZIP); strenge Pfadsicherheit, keine Pfadtraversal.
- **Upload-Kanäle:** App-Export; Website-Projektupload; REST API Version/File Upload.
- **Upload-Ablauf:** Profil exportieren → Index/Hashes/URLs erzeugen → lokal importieren oder als Modpack-Projektversion hochladen → Moderation.
- **Validierung und Moderation:** Schema-/Pfad-/Domainregeln und Moderation.
- **Versionierung:** versionId + Projektversionen; Lockfile-artige exakte Artefakte.
- **Quelle:** [Quelle](https://support.modrinth.com/en/articles/8802351-modrinth-modpack-format-mrpack)

## Wabbajack – .wabbajack

- **Manifest oder Paket:** Kompiliertes Installationsarchiv
- **Struktur:** Compiler analysiert MO2-Setup, Download-Metadaten und Dateiquellen und erzeugt Installationsanweisungen; Mods werden möglichst nicht neu verteilt.
- **Wichtige Felder:** Modlist-Metadaten, Downloadquellen, Hashes, Installations-/Patch-Anweisungen und inkludierte erlaubte Dateien.
- **Abhängigkeiten:** Ergebnis des kompilierten Setups; nicht primär ein einfacher Dependency-Resolver.
- **Kompatibilität:** Vor allem Bethesda-/MO2-Workflows; Spiel und Quelle in Compiler-Konfiguration.
- **Client/Server:** Spielspezifisch.
- **Hashes und Signaturen:** Hashes sind zentral für reproduzierbare Downloads und Patch-Anweisungen.
- **Overrides und Installation:** Stock Game / MO2-Struktur und detaillierte Installationsanweisungen.
- **Dateiformate:** .wabbajack plus Downloadcache/Metadaten.
- **Upload-Kanäle:** Wabbajack Compiler; Modlist-Repositories/Gallery; Nexus Deep Links.
- **Upload-Ablauf:** Portable MO2-Liste vorbereiten → Compilation Settings → kompilieren/validieren → .wabbajack und Metadaten veröffentlichen.
- **Validierung und Moderation:** Compiler prüft Downloads und Reproduzierbarkeit; Hosting-/Repositoryregeln.
- **Versionierung:** Neue kompilierte Modlist-Version; Update-Metadaten über Repository.
- **Quelle:** [Quelle](https://wiki.wabbajack.org/modlist_author_documentation/Compilation%20Settings.html)

## Mod Organizer 2 – lokales Profil

- **Manifest oder Paket:** meta.ini + Profiltextdateien, kein Hosting-Paketstandard
- **Struktur:** Pro Mod/Download Metadaten; Profile halten Modaktivierung, Plugin-/Loadorder und optionale Saves/INIs.
- **Wichtige Felder:** meta.ini je Mod/Download; profile/modlist.txt, plugins.txt, loadorder.txt und weitere game-spezifische Daten.
- **Abhängigkeiten:** Abhängigkeiten eher durch Nutzer/Plugins/Metadaten, kein universeller Solver.
- **Kompatibilität:** Game plugin bestimmt Support.
- **Client/Server:** Profiloptionen für Saves/INIs.
- **Hashes und Signaturen:** Download-Metadaten und Nexus-IDs/Hashes teilweise; keine allgemeine Signatur.
- **Overrides und Installation:** VFS legt Prioritäten fest, ohne den Spielordner direkt zu überschreiben.
- **Dateiformate:** Lokale Ordner/Archive.
- **Upload-Kanäle:** Manueller Download/Nexus-Integration; kein eigener öffentlicher Modhosting-Upload.
- **Upload-Ablauf:** Mod installieren → Separator/Priorität → Plugins sortieren → Profil testen.
- **Validierung und Moderation:** Lokale Konflikt-/Archivansicht; keine zentrale Moderation.
- **Versionierung:** Lokale Mod-/Profilversionierung durch Ordner/Metadaten.
- **Quelle:** [Quelle](https://github.com/ModOrganizer2/modorganizer/releases)

## Prism Launcher – Instanz

- **Manifest oder Paket:** instance.cfg + mmc-pack.json; Importformate
- **Struktur:** Instanz enthält Minecraft-/Loader-Komponenten, Mods, Configs, Worlds und launcherbezogene Einstellungen.
- **Wichtige Felder:** Komponenten/Versionen in mmc-pack.json, Instanzeinstellungen in instance.cfg; importiert Modrinth/CurseForge Packs.
- **Abhängigkeiten:** Mod-/Pack-Quellen liefern Dependencies; Launcher verwaltet Komponenten.
- **Kompatibilität:** Minecraft-Version und Loader-Komponenten.
- **Client/Server:** Client-Instanz; Server getrennt.
- **Hashes und Signaturen:** Quellen-/Packformatabhängige Hashes.
- **Overrides und Installation:** Isolierte Instanzordner.
- **Dateiformate:** Prism/PolyMC-Packs sowie CurseForge-/Modrinth-Importe.
- **Upload-Kanäle:** GUI-Import und integrierter Download aus Modrinth/CurseForge.
- **Upload-Ablauf:** Instanz erstellen/importieren → Komponenten/Java setzen → Mods laden → starten/exportieren.
- **Validierung und Moderation:** Launcher-/Formatvalidierung; Hostingplattform moderiert Inhalte.
- **Versionierung:** Instanzen duplizierbar; Packupdates formatabhängig.
- **Quelle:** [Quelle](https://prismlauncher.org/wiki/getting-started/download-mods/)

## GDLauncher – Modpack/Instanz

- **Manifest oder Paket:** Unterstützt .gdlpack, .mrpack und weitere Importe
- **Struktur:** Isolierte Instanz mit Modliste, Loader, Einstellungen und optional Cloud-Share.
- **Wichtige Felder:** Formatabhängig; GDLauncher exportiert eigene und offene Packformate.
- **Abhängigkeiten:** Abhängigkeiten werden beim Plattformdownload berücksichtigt.
- **Kompatibilität:** Minecraft-Version und Forge/Fabric/Quilt/NeoForge.
- **Client/Server:** Client; Serververwaltung separat verfügbar.
- **Hashes und Signaturen:** Format-/Plattformabhängig.
- **Overrides und Installation:** Isolierte Instanzen.
- **Dateiformate:** .gdlpack, .mrpack, ZIP und Launcherimporte.
- **Upload-Kanäle:** App-Export/Import; Cloud-Share-Code.
- **Upload-Ablauf:** Instanz konfigurieren → exportieren oder Cloud teilen → Empfänger importiert.
- **Validierung und Moderation:** Clientvalidierung und Plattformregeln.
- **Versionierung:** Packversion wechseln/aktualisieren, Instanz duplizieren.
- **Quelle:** [Quelle](https://gdlauncher.com/docs/gdlauncher-vs-gdlauncher-carbon)

## Empfohlenes eigenes Format

- **Manifest oder Paket:** author-manifest.json + lock.json + overrides/
- **Struktur:** Author manifest bleibt lesbar und deklarativ; Resolver erzeugt exaktes Lockfile. Optional signatures/ und migrations/.
- **Wichtige Felder:** schemaVersion, packageId, name, version, game, compatibility, dependencies, conflicts, provides, files, install, permissions, entrypoints, options.
- **Abhängigkeiten:** SemVer-Ranges im Autorenmanifest; exakte package/version/file IDs und Hashes im Lockfile.
- **Kompatibilität:** Game-Adapter-ID, Spielbuildbereich, Loader/API-Version, OS/Arch optional.
- **Client/Server:** client/server/both je Datei oder Komponente.
- **Hashes und Signaturen:** SHA-256/512 je Blob; Ed25519/Sigstore-Signatur über kanonisches Manifest und Merkle Root.
- **Overrides und Installation:** Explizite roots; overrides, server-overrides, merge rules; path traversal verboten.
- **Dateiformate:** ZIP/Zstd-Tar optional; portables JSON; keine ausführbaren Installerskripte ohne Sandbox.
- **Upload-Kanäle:** Web, App, CLI, REST API, CI/OIDC trusted publishing.
- **Upload-Ablauf:** validate → pack deterministisch → sign → upload chunks → finalize → scan → review → publish → webhook.
- **Validierung und Moderation:** Lokale und serverseitige identische Validatoren; Malware/SBOM/Policy; Statusmaschine und Appeal.
- **Versionierung:** Artefakte immutable; neue SemVer-Version; Yank/Deprecate statt Replace; Revisionen für Sammlungen.

# Bereits vorhandene Manifesttypen der App

Die vorhandenen Beispiele trennen Mods, Plugins, Extensions und Bibliotheken bereits sinnvoll.

## `01_simple_mod.json` – Simple Balance Tweaks

- **ID:** `simple-balance-tweaks`
- **Typ:** `game_mod`
- **Version:** `1.0.0`
- **Autor:** BalanceTeam
- **Beschreibung:** Tweaks weapon and upgrade costs for better game progression
- **Spiele:** datacenter
- **Lizenz:** `MIT`

## `02_gregcore_mod_with_deps.json` – Advanced Gameplay System

- **ID:** `advanced-gameplay`
- **Typ:** `gregcore_mod`
- **Version:** `2.0.0`
- **Autor:** GameDesigners
- **Beschreibung:** Adds new gameplay mechanics with gregCore support and advanced customization options
- **Spiele:** datacenter
- **gregCore mindestens:** `2.0.0`
- **Lizenz:** `MIT`
- **Tags:** `gameplay`, `gregcore`, `advanced`
- **Pflichtabhängigkeiten:**
  - `gregcore` (>= 2.0.0)
  - `common-utils` (>= 1.0.0)
- **Optionale Abhängigkeiten:**
  - `advanced-ui-plugin` (>= 1.0.0)

## `03_universal_plugin.json` – Universal Debug Plugin

- **ID:** `universal-debug-plugin`
- **Typ:** `melonloader_plugin`
- **Version:** `1.5.0`
- **Autor:** DebugTeam
- **Beschreibung:** In-game debug console and profiling tools for any MelonLoader game
- **Spiele:** Universell / nicht auf ein Spiel beschränkt
- **MelonLoader mindestens:** `0.6.0`
- **MelonLoader maximal:** `0.7.x`
- **Lizenz:** `MIT`
- **Tags:** `debugging`, `profiling`, `universal`

## `04_game_specific_plugin.json` – Data Center Performance Optimizer

- **ID:** `datacenter-perf-optimizer`
- **Typ:** `melonloader_plugin`
- **Version:** `1.0.0`
- **Autor:** PerfTeam
- **Beschreibung:** Optimizes rendering and physics performance specifically for Data Center
- **Spiele:** datacenter
- **MelonLoader mindestens:** `0.6.0`
- **Lizenz:** `MIT`
- **Tags:** `performance`, `optimization`, `datacenter`

## `05_gregcore_extension.json` – Custom Game State Extension

- **ID:** `custom-game-state`
- **Typ:** `gregcore_extension`
- **Version:** `1.0.0`
- **Autor:** ExtensionTeam
- **Beschreibung:** Extends gregCore with advanced game state tracking and event system
- **gregCore mindestens:** `2.0.0`
- **Lizenz:** `MIT`
- **Pflichtabhängigkeiten:**
  - `gregcore` (>= 2.0.0)

## `06_userlib.json` – Common Utilities Library

- **ID:** `common-utils`
- **Typ:** `userlib`
- **Version:** `2.0.0`
- **Autor:** UtilityTeam
- **Beschreibung:** Shared utilities and helper libraries for mods and plugins
- **Lizenz:** `MIT`
- **Tags:** `utilities`, `library`, `shared`
- **Konflikte:**
  - `common-utils` (<= 1.x) – API incompatible - requires update to v2.0+

# Empfohlene Manifest-Funktionsliste für die eigene Plattform

- [ ] **Identität:** `schemaVersion`, stabile `packageId`, `namespace`, `name`, `displayName`, `version`, `type`
- [ ] **Spielziel:** `game.adapterId`, Spiel-ID, Edition, Store, Spielbuild- oder Versionsbereich
- [ ] **Loader und Runtime:** Loader-ID, Loader-Version, Runtime/.NET/Java-Version, Architektur und Betriebssystem
- [ ] **Umgebung:** Client, Server oder beide; jeweils required, optional oder unsupported
- [ ] **Abhängigkeiten:** required, optional, incompatible, embedded und tool dependencies mit Versionsbereichen
- [ ] **Konflikte:** `conflicts`, `replaces`, `provides`, bekannte Inkompatibilitäten und Begründungen
- [ ] **Dateien:** Dateiliste mit Quellpfad, Zielroot, Zielpfad, Größe, Hash, Ausführbarkeit und Plattform
- [ ] **Overrides:** allgemeine Overrides, Client-/Server-Overrides, Include-/Exclude-Regeln und Merge-Strategien
- [ ] **Konfiguration:** Schemas für JSON, YAML, TOML oder INI sowie Defaultwerte und Migrationen
- [ ] **Installationsplan:** deklarative Installationsschritte; freie Skripte nur in einer eingeschränkten Sandbox
- [ ] **Sicherheit:** SHA-256/512, Publisher-Signatur, Key-ID, SBOM-Referenz, Scanstatus und benötigte Berechtigungen
- [ ] **Reproduzierbarkeit:** separates Lockfile mit exakten Versionen, Datei-IDs, Downloadquellen und Hashes
- [ ] **Lifecycle:** Release-Kanal, Mindest-App-Version, Deprecated-/Replacement-Angaben und Upgrade-Hinweise
- [ ] **Creator-Metadaten:** Autorenteam, Lizenz, Quellcode, Homepage, Issue-Tracker, Spendenlink und Kontakt
- [ ] **Darstellung:** Beschreibung, Kurzbeschreibung, Tags, Kategorien, Icon, Screenshots und Changelog

# Empfohlener Upload- und Veröffentlichungsworkflow

- [ ] Projekt oder Paket im Web, in der App oder per CLI auswählen beziehungsweise anlegen.
- [ ] Manifest lokal gegen eine veröffentlichte JSON-Schema-Version validieren.
- [ ] Paket deterministisch erstellen; identischer Inhalt muss denselben Paket-Hash erzeugen.
- [ ] Dateien und Manifest hashen; optional mit einem Publisher-Key signieren.
- [ ] Große Dateien per Multipart- oder resumierbarem Direktupload in den Objektspeicher übertragen.
- [ ] Upload mit einer idempotenten Finalize-Anfrage abschließen.
- [ ] Archivstruktur, Dateipfade, Symlinks, ausführbare Dateien und Archive-Bomb-Risiken prüfen.
- [ ] Malware-, Policy-, Lizenz- und Duplikatprüfung durchführen.
- [ ] Abhängigkeiten, Spielversionen, Loader, Betriebssysteme und Client-/Server-Ziele validieren.
- [ ] Moderationsstatus als Draft, Processing, Needs Changes, Approved, Rejected oder Quarantined anzeigen.
- [ ] Creator konkrete Fehler mit Feldpfad, Ursache und Lösungsvorschlag zurückgeben.
- [ ] Nach Freigabe eine unveränderliche Version veröffentlichen.
- [ ] Änderungen nur als neue Version veröffentlichen; problematische Versionen deprecaten oder yanken statt ersetzen.
- [ ] Webhooks für Veröffentlichung, Scanfehler, Moderationsänderung und Deprecation senden.
- [ ] Clients über Katalog-, Hash- und Update-Endpunkte informieren.

# Priorisierte Roadmap

## P0

- [ ] **Game-Adapter + Capability Registry** — Architektur · MVP
  - **Umfang:** Das aktuelle Spiel vollständig über ein Adapter-Interface kapseln; keine Spielpfade oder Loaderregeln im UI/Backend hart codieren.
  - **Akzeptanzkriterium:** Contract-Tests decken Detect, PlanInstall, Deploy, Uninstall, Launch, SavePaths und Compatibility ab.

- [ ] **Manifest v1 + Lockfile v1** — Manifest · MVP
  - **Umfang:** Ein lesbares Autorenmanifest und ein exaktes Lockfile mit IDs, Quellen, Hashes und Installationszielen.
  - **Akzeptanzkriterium:** JSON Schema veröffentlicht; 20 gültige/ungültige Fixtures; deterministischer Pack-Hash.

- [ ] **Transaktionales Deployment + Rollback** — Sicherheit · MVP
  - **Umfang:** Jede Änderung wird geplant, gestaged, validiert und atomar committed.
  - **Akzeptanzkriterium:** Absichtlich abgebrochene Installationen hinterlassen exakt den vorherigen Zustand.

- [ ] **Upload-Scanpipeline** — Sicherheit · MVP
  - **Umfang:** Quarantäne, Archive-Schutz, Hash/Dedupe, Malware-Scan, Policy und Moderationsstatus.
  - **Akzeptanzkriterium:** Keine Datei ist vor abgeschlossenem Scan installierbar; Scanstatus ist öffentlich nachvollziehbar.

- [ ] **Profile/Instanzen** — Nutzer · MVP
  - **Umfang:** Mehrere isolierte Modlisten mit eigener Config und optionalen Saves.
  - **Akzeptanzkriterium:** Profilwechsel verändert keine Dateien eines anderen Profils; Duplikation funktioniert ohne Vollduplikat großer Blobs.

- [ ] **Ein-Klick-Install + Dependency Resolver** — Nutzer · MVP
  - **Umfang:** Mod auswählen, kompatible Versionen lösen, Plan anzeigen und installieren.
  - **Akzeptanzkriterium:** Required Dependencies vollständig; unlösbare Konflikte werden verständlich erklärt.

- [ ] **Update, Pin, Downgrade, Snapshot** — Nutzer · MVP
  - **Umfang:** Updates mit Changelog/Diff; einzelne Mods pinnen; Snapshot vor Änderung.
  - **Akzeptanzkriterium:** Update All überspringt Pins und inkompatible Releases; Rollback in einem Schritt.

- [ ] **Katalog, Suche, Projekt-/Versionsseiten** — Web · MVP
  - **Umfang:** Kompatibilität, Abhängigkeiten, Changelog, Lizenz, Scanstatus und Deep Link sichtbar.
  - **Akzeptanzkriterium:** Suche kann nach aktueller Spielversion und installierbarer Version filtern.

- [ ] **Projekt-, Webupload- und Moderationsworkflow** — Creator · MVP
  - **Umfang:** Draft → Upload → Processing → Needs changes/Approved → Publish.
  - **Akzeptanzkriterium:** Creator sieht konkrete Validierungsfehler und Statushistorie; Moderatoraktionen sind auditiert.

- [ ] **Auth, Rollen, Audit, Observability und DSGVO** — Betrieb · MVP
  - **Umfang:** Device Flow, Projektrollen, strukturierte Logs, SLOs, Export/Löschung.
  - **Akzeptanzkriterium:** Token sind scoped/revocable; kritische Aktionen im Auditlog; Datenexport testbar.

## P1

- [ ] **Linux-Unterstützung** — Plattform · Nach MVP
  - **Umfang:** Native Paketierung, XDG, Proton/Steam Deck und case-sensitive Pfade.
  - **Akzeptanzkriterium:** Install/Update/Rollback bestehen dieselbe Adapter-Testmatrix wie Windows.

- [ ] **macOS-Unterstützung** — Plattform · Nach MVP
  - **Umfang:** Notarisierung, Universal Binary, File Bookmarks und Gatekeeper-UX.
  - **Akzeptanzkriterium:** Installation ohne manuelles Abschalten von Sicherheitsfunktionen.

- [ ] **Dateikonflikte + Load Order** — Kompatibilität · Nach MVP
  - **Umfang:** Besitzdaten, Before/After-Regeln, Gruppen und game-spezifische Sortierung.
  - **Akzeptanzkriterium:** Zyklen werden blockiert; Nutzer sieht exakt, welche Datei gewinnt.

- [ ] **Versionierte Collections/Modpacks** — Sammlungen · Nach MVP
  - **Umfang:** Lockfile, Optionsgruppen, Overrides, Diff, Deep Link und Share Code.
  - **Akzeptanzkriterium:** Eine veröffentlichte Revision ist reproduzierbar und kann sicher aktualisiert werden.

- [ ] **CLI + CI/OIDC Publishing** — Creator · Nach MVP
  - **Umfang:** validate, pack, sign, publish, status; offizielle GitHub Action.
  - **Akzeptanzkriterium:** Release aus CI ohne langlebigen API-Key möglich; Upload ist idempotent/resumierbar.

- [ ] **Öffentliche REST API + Webhooks** — API · Nach MVP
  - **Umfang:** Katalog, Hashlookup, Bibliothek, Upload und Events mit OpenAPI.
  - **Akzeptanzkriterium:** Drittclient kann vollständigen Installationsplan aufbauen; Webhooks sind signiert/retrybar.

- [ ] **Logbundle + Preflight + Health Dashboard** — Diagnose · Nach MVP
  - **Umfang:** Automatisch sensible Daten schwärzen und häufige Fehler vor Start erkennen.
  - **Akzeptanzkriterium:** Supportbundle enthält Versionen, Manifest, Konflikte und relevante Logs ohne Tokens/Pfade.

## P2

- [ ] **Profil-Sync und Backups** — Cloud · Wachstum
  - **Umfang:** Metadaten synchronisieren; Config/Saves opt-in verschlüsselt sichern.
  - **Akzeptanzkriterium:** Konfliktauflösung und Restore-Test; klare Quoten/Retention.

- [ ] **Companion/PWA** — Mobile · Wachstum
  - **Umfang:** Browse, Queue-to-PC, Push, Profilstatus; keine pauschale Android-Modinstallation versprechen.
  - **Akzeptanzkriterium:** Gerätepaarung ist widerrufbar; Remote-Aktionen benötigen Bestätigung/Scope.

- [ ] **Rewards und faire Premium-Stufe** — Creator Economy · Monetarisierung
  - **Umfang:** Werbefrei, Cloud, Komfortautomation und Creator-Anteil; Kernschutz bleibt kostenlos.
  - **Akzeptanzkriterium:** Dependency Resolver, Scans, Rollback und manuelle Installation sind nie paywalled.

- [ ] **Adapter SDK + zweites Spiel** — Multi-Game · Expansion
  - **Umfang:** SDK, Dokumentation, Mock und Contract Tests; zweites Spiel validiert Architektur.
  - **Akzeptanzkriterium:** Zweites Spiel benötigt keinen Fork der Kern-App und keine schema-brechende Änderung.

- [ ] **Empfehlungen und Kuratierung** — Discovery · Wachstum
  - **Umfang:** Personalisierung opt-in, erklärbar und mit Mute/Reset; redaktionelle Listen.
  - **Akzeptanzkriterium:** Nutzer kann Gründe sehen und Signale deaktivieren; Sicherheits-/Kompatibilitätsfilter dominieren Ranking.

## P3

- [ ] **Delta-Updates und verteilte Mirrors** — Performance · Später
  - **Umfang:** Nur nach stabiler Hash-/Cache-/Resume-Basis.
  - **Akzeptanzkriterium:** Vollpaket-Fallback; Deltas sind signiert und messen echte Einsparung.

- [ ] **Community Game-Adapter Marketplace** — Ökosystem · Später
  - **Umfang:** Signierte Adapter, Review, Permission-Sandbox und API-Kompatibilitätsmatrix.
  - **Akzeptanzkriterium:** Adapter kann keine beliebigen Dateipfade/Netzwerkrechte ohne deklarierte Permission nutzen.

# Produktprinzipien

- Sicherheit, Malwareprüfung, Dependency-Auflösung, saubere Deinstallation und Rollback bleiben kostenlos.
- Premium umfasst primär Werbefreiheit, Cloudspeicher, zusätzliche Automatisierung, kosmetische Anpassungen und Creator-Unterstützung.
- Die erste öffentliche Version kann auf ein Spiel beschränkt sein, der Kern muss aber vollständig über einen Game-Adapter abstrahiert sein.
- Installationen und Updates müssen transaktional und reproduzierbar sein.
- Veröffentlichte Artefakte sind unveränderlich; Updates erhalten neue Versionen.
- Jede installierte Datei benötigt Besitz- und Herkunftsmetadaten.
- Kompatibilität muss als strukturierte Daten vorliegen und darf nicht nur in Beschreibungen stehen.
- Android sollte zunächst als Companion- oder Queue-to-PC-App behandelt werden; lokale Modinstallation nur bei offiziell unterstützten Android-Spielen.
