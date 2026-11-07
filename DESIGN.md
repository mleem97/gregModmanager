# gregDesktop Design System — Terminal Core

Dieses Dokument definiert die strikten Design-Vorgaben fuer alle Desktop-Anwendungen im gregFramework-Ökosystem, die auf Slint basieren. Es erweitert und konkretisiert die Architekturregeln aus `AGENTS.md` fuer alle visuellen und interaktiven Aspekte.

---

## 1. Design-Philosophie

- **Terminal Core**: Die UI soll sich angemessen, praegnant und informationsdicht anfuehlen — nicht verspielt.
- **Funktion vor Form**: Jedes Element muss einen Zweck erfuellen. Dekoration ohne Nutzen wird entfernt.
- **Konsistenz ueber Customizing**: Alle greg-Apps teilen dieselben Tokens, Abstaende und Interaktionsmuster.

---

## 2. Farbsystem

Alle Farbwerte sind absolut und duerfen nicht interpoliert oder veraendert werden.

| Token | Hex | Verwendung |
| :--- | ---: | --- |
| `SurfaceBase` | `#051424` | Haupt-Hintergrund aller Fenster und Seiten |
| `SurfaceElevated` | `#0A1E2E` | Karten, Panels, Modale, Flyouts |
| `SurfaceOverlay` | `#122131` | Hover-States, Trennlinien, Subtle Hervorhebungen |
| `Primary` | `#8AEBFF` | Aktive Aktionen, Links, Fokus-Ringe, Ladeindikatoren |
| `Secondary` | `#4DE082` | Erfolgszustaende, positive Badges, gespeicherte Indikatoren |
| `OnSurface` | `#D4E4FA` | Primaerer Text, Icons im Normalzustand |
| `OnSurfaceMuted` | `#8A9DB3` | Sekundaerer Text, Placeholder, Disabled-Labels |
| `Danger` | `#FF5370` | Fehler, Destruktive Aktionen, Rate-Limit-Timer |
| `Warning` | `#FFD166` | Warnungen, ungespeicherte Aenderungen |
| `BorderSubtle` | `#122131` | Trennlinien, Rahmen von Eingabefeldern |
| `BorderActive` | `#8AEBFF` | Fokussierte Eingabefelder, aktive Tabs |

**Regeln:**

- Keine transparenten Overlays (`Opacity < 1.0`) fuer Hintergruende.
- Keine Verlaeufe (Gradients).
- Keine Schwaeche (Drop Shadows).
- Kontrastverhaeltnisse muessen mindestens WCAG AA fuer Text erfuellen (Primary auf SurfaceBase: 10.8:1).

---

## 3. Typografie

| Rolle | Schriftart | Groesse | Gewicht | Zeilenabstand |
| :--- | --- | ---: | --- | ---: |
| Display | Space Grotesk | 24 px | 700 | 32 px |
| Headline | Space Grotesk | 18 px | 600 | 24 px |
| Title | Inter | 14 px | 600 | 20 px |
| Body | Inter | 13 px | 400 | 18 px |
| Caption | Inter | 11 px | 400 | 14 px |
| Mono (Status, Metadaten) | JetBrains Mono | 12 px | 400 | 16 px |

**Regeln:**

- Text-Transform: Nie `uppercase` fuer Labels oder Buttons. Nur fuer kleine statische Badges erlaubt.
- Text-Trimming: Ellipsis bei Pfaden und langen Titeln; Tooltip mit Volltext bei Hover.

---

## 4. Layout-Grid & Abstaende

- **Basis-Einheit**: 8 px.
- **Seitenraender**: 24 px (Desktop), 16 px (kompakte Fenster < 900 px Breite).
- **Panel-Padding**: 16 px.
- **Karten-Padding**: 16 px.
- **Element-Abstand**: 8 px, 16 px, 24 px (Vielfache von 8).
- **Trennlinien**: 1 px, Farbe `BorderSubtle`, horizontal oder vertikal.

**Regeln:**

- Keine halben Einheiten (z.B. 4 px nur fuer Hairlines/Trennlinien erlaubt).
- Kein zentrierter Fliesstext; Linksbündig ausgerichtet.
- Listen und Tabellen nutzen volle verfuegbare Breite; letzte Spalte expandiert.

---

## 5. Geometrie & Radien

| Komponente | Radius |
| :--- | ---: |
| Fenster (root) | 0 px (scharfe Kanten) |
| Karten / Panels | 4 px |
| Buttons | 4 px |
| Eingabefelder | 4 px |
| Dropdowns / ContextMenus | 4 px |
| Modale / Dialoge | 8 px |
| Avatare / Thumbnails | 0 px (quadratisch) |
| Tabs | 4 px (oben) |
| Badges | 2 px |

**Regeln:**

- Keine abgerundeten "Bubble"-UIs.
- Keine vollstaendig runden Buttons (Pill-Shape).
- Bilder und Thumbnails sind immer quadratisch mit 0 px Radius.

---

## 6. Fenster-Chrome

- Das Slint `Window` ist frameless (custom chrome); der Titelzeilen-Header ist ein
  eigenes Element mit Hintergrund `SurfaceElevated`, Hoehe 40 px.
- Header reagiert auf Drag-Events (native WindowMoveArea / Titelleisten-Muster) —
  Wayland-safe, kein manuelles `set_position`.
- Fenster-Controls (Minimieren, Maximieren, Schliessen) sind eigene Buttons rechts
  oben.
- Schliessen-Button zeigt `Danger`-Farbe nur bei Hover.

---

## 7. Komponenten

### 7.1 Buttons

| Variante | Hintergrund | Rahmen | Text |
| :--- | --- | --- | --- |
| Primary | `Primary` | none | `SurfaceBase` |
| Secondary | `SurfaceElevated` | `BorderSubtle` | `OnSurface` |
| Danger | `Danger` | none | `SurfaceBase` |
| Ghost | transparent | none | `OnSurfaceMuted` |

- Padding: 8 px vertikal, 16 px horizontal.
- Hover: Helligkeit um 10 % erhoehen (durch feste Farbe, nicht Opacity).
- Disabled: `OnSurfaceMuted` Text, kein Rahmen, keine Aktion.
- Focus-Ring: 2 px `Primary` mit 2 px Offset (innerer Ring `SurfaceBase`).

### 7.2 Eingabefelder

- Hintergrund: `SurfaceBase`.
- Rahmen: 1 px `BorderSubtle`; bei Fokus 1 px `BorderActive`.
- Padding: 8 px.
- Placeholder: `OnSurfaceMuted`.
- Disabled: Rahmen `BorderSubtle`, Text `OnSurfaceMuted`.

### 7.3 Dropdowns / ComboBoxes

- Dropdown-Panel: `SurfaceElevated`, Radius 4 px, 1 px Rahmen `BorderSubtle`.
- Ausgewaehltes Item: Hintergrund `SurfaceOverlay`.
- Hover-Item: Hintergrund `SurfaceOverlay`.

### 7.4 Listen & DataGrids

- Zeilen-Hoehe: 40 px.
- Abwechselnde Zeilenfarben: Nein. Stattdessen Trennlinien zwischen Zeilen.
- Hover-Zeile: Hintergrund `SurfaceOverlay`.
- Ausgewaehlte Zeile: Hintergrund `SurfaceOverlay` + linker 3 px Rand `Primary`.
- Header: JetBrains Mono, 11 px, `OnSurfaceMuted`, Hintergrund `SurfaceElevated`.

### 7.5 Karten (Cards)

- Hintergrund: `SurfaceElevated`.
- Rahmen: 1 px `BorderSubtle`.
- Radius: 4 px.
- Padding: 16 px.
- Kein Schatten.

### 7.6 Badges

- Hintergrund: `SurfaceOverlay`.
- Text: JetBrains Mono, 11 px.
- Padding: 2 px 8 px.
- Radius: 2 px.

### 7.7 Tooltips

- Hintergrund: `SurfaceElevated`.
- Rahmen: 1 px `BorderSubtle`.
- Text: Caption.
- Verzoegerung: 400 ms.

---

## 8. Dialoge & Modale

- Nie native `MessageBox` verwenden.
- Pflicht: dedizierte Dialog-Elemente im Slint UI mit eigenen Buttons
  (Primary rechts, destruktive links als Ghost-Button).
- Dialog-Hintergrund: halbtransparentes `SurfaceBase` (nur hier ist leichte
  Transparenz erlaubt: `#051424E6`).
- Dialog-Panel: `SurfaceElevated`, Radius 8 px, max. Breite 560 px.
- Trennung von Titel, Inhalt und Aktionen durch Hairlines.
- Primaere Aktion immer rechts unten; destruktive Aktion links davon als Ghost-Button.
- Dialoge muessen scrollbar sein, wenn Inhalt > 70 % der Fensterhoehe.

---

## 9. Statusanzeigen & Feedback

### 9.1 Fortschritt

- Linearer Fortschrittsbalken, Hoehe 4 px, Radius 2 px.
- Track: `SurfaceOverlay`; Fill: `Primary`.
- Keine prozentuale Animation in der Breite (nur bei Wert-Aenderung).

### 9.2 Ladezustaende

- Spinner: 16 px, 2 px Strichstaerke, Farbe `Primary`.
- Skelett-Loading nur fuer Listen erlaubt (rechteckige Bloecke in `SurfaceOverlay`).

### 9.3 Rate Limiter (Steam)

- Expliziter Cooldown-Timer in Sekunden, Mono-Schrift.
- Farbe: `Danger`, solange blockiert; `Secondary`, sobald verfuegbar.
- Sichtbarkeit: Banner oberhalb der primaeren Aktion, nicht als Toast.

---

## 10. Icons

- Stil: Strichstaerke 1.5 px, scharfe Enden, keine ausgefuellten Flaechen.
- Groesse: 16 px (Standard), 20 px (Toolbar), 24 px (leere Zustaende).
- Farbe: `OnSurface` (Standard), `Primary` (aktiv), `OnSurfaceMuted` (disabled).
- Keine Emojis als Icon-Ersatz.

---

## 11. Lokalisierung im UI

- Alle UI-Strings muessen ueber die `greg-core`-L10n-Module (Englisch plus Deutsch
  mindestens) aufgeloest werden.
- Keine hardcodierten Strings in den Slint-Views.
- Neue Keys muessen zuerst in die L10n-Strukturen von `greg-core` eingetragen werden.
- Textlaengen-Unterschiede (z.B. Deutsch +30 %) muessen getestet werden; UI darf
  nicht brechen.

---

## 12. Dateien & Assets

- Slint-`global` Variablen / Tokens fuer Farben, Abstaende, Radien.
- Bilder: PNG mit Alpha; keine JPEGs fuer UI-Elemente.

---

## 13. Animationen

- Max. Dauer: 150 ms fuer UI-Transitions (Fade, Slide).
- Easing: `CubicEaseOut`.
- Keine Animationen fuer Listen-Scroll-Updates (Performance).

---

## 14. Responsives Verhalten

- Minimale Fenstergroesse: 1024 x 640 px.
- Bei < 1024 px Breite: Seitenpanel kollabiert zu Icons-only.

---

## 15. Verbotene Muster

| Muster | Ersetzen durch |
| :--- | --- |
| Native `MessageBox` | Dedizierte Dialog-Elemente im Slint UI |
| Vollstaendig runde Buttons | Rechtecke mit 4 px Radius |
| Schlagschatten | Hairlines oder kontrastierende Rahmen |
| Verlaeufe | Flaechenfarben |
| Emojis | Vektor-Icons (Lucide) |
| Halbe Pixel-Werte (4 px ausser Trennlinien) | 8 px Grid |

---

## 16. Validierung

Bevor ein PR gemerged wird, muss geprueft werden:

1. [ ] Alle neuen UI-Strings sind in den `greg-core`-L10n-Strukturen (Englisch + Deutsch).
2. [ ] Keine hardcodierten Farben ausser den definierten Tokens.
3. [ ] Keine nativen `MessageBox`-Aufrufe.
4. [ ] Fenster ist bei 1024x640 px und bei 1920x1080 px bedienbar.
5. [ ] Screenshot-Diff bei visuellen Aenderungen (manuell oder via CI).

---

###### Letzte Aktualisierung: 2026-10-28
