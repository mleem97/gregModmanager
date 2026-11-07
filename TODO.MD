# TODO – gregModmanager

> Abgeleitet aus `CONCEPT.md` (Aktueller Implementierungsstand, Stand: 2026-10-29, Version 1.7.7).

## Offene Konzepte und Umsetzungspfade

### Statusleiste / Problems

- [ ] Konzeptionelle Verschiebung des `Problems`-Bereichs aus der Navigationsleiste in die globale Statusleiste final abschließen.
- [ ] Automatische Fehlermeldung (Probleme, Warnungen, Fehler) in die globale Statusleiste integrieren und priorisieren.

### Workshop / Modentwicklung

- [ ] Finale Konsolidierung der Bereiche `Browse`, `Subscribed` und `Favorites` im Workshop-Bereich gegen dem Konzept „Umlagerung in den Modstore“.
- [ ] Mod-Creation-Assistenten abschließen: Validierung vollständiger Projektdateien und Einstellungen, Ausweitung auf Multi-Plattform-Projekte.

### Modstore

- [ ] Webapp-Katalog-Parität: vollständige Abdeckung des Datacentermods.com-Katalogs inkl. direkter Installationen.
- [ ] Versionsvergleich Steam-Workshop vs. Datacentermods.com robust und nachvollziehbar abschließen (Priorisierung neuer Versionen).

### Game-Launch / Runtime

- [ ] Vollständiger Review der Stop-Funktion (Zuständigkeit, Redundanz, Verhalten bei abgebrochenen Starts).
- [ ] Profile und Lockfiles, automatisches Laden von Konflikten / Load Order, In-App-Updater umsetzen (laut `README.md` noch nicht released).

### Authentifizierung / Report-Workflow

- [ ] Discord-/GitHub-Login-Integration im Desktop finalisieren.
- [ ] Backend-seitige Report-Intake (GitHub App / Service-Account) finalisieren; env-gated Issue-Erstellung ist bereits implementiert.

### Plattform

- [ ] macOS: Native Mac-Validierung, Signing und Notarization abschließen (Stand: experimentell).

## Nächste Schritte

1. Offene Konzept-Entscheidungen aus `CONCEPT.md` Abschnitt 11 (Implementierungsstand) priorisieren.
2. Desktop-/Webapp-Paritäts-Lücken aus `desktop-gap-matrix.md` gegen `TODO.md`-Punkten abschließen.
3. Neue Funktionen nur mit bestehendem Rust-Stack (`greg-core`, `greg-platform`, `greg-steam`, `greg-modstore`, `greg-loader`, `greg-app`, `greg-cli`) umsetzen.

## Update-Mechanismus (GitHub Releases)

> Konkreter Implementierungsplan (Umfang, Abhängigkeiten, Risiken): `UPDATE_MECHANISM.md`.

- [ ] Update-Mechanismus basierend auf GitHub Releases planen und implementieren (In-App-Updater).
- [ ] Versionsabgleich (lokale Versionsangabe ↔ GitHub Release) umsetzen.
- [ ] Update-Assets je Plattform und Checksum-Verfahren finalisieren.
- [ ] Installationslogik inkl. Backup und Neustart nach Update umsetzen.
- [ ] Manuelles und automatisches Update in der UI bereitstellen.
- [ ] Stable-/Beta-Kanal-Trennung und Release-Pipeline mit GitHub Releases verknüpfen.
