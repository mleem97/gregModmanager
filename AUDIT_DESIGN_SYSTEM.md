# Audit Design System — Slint Implementation vs. Design Goals

> Owner: gregModmanager maintainers. Evidence: source-tree audit (2026-10-28).
> Status: **Audit only** — no implementation. Visual verification still pending.

## 1. Design Source of Truth

- `DESIGN.md` (root, last updated 2026-05-05) is written against the **frozen
  Avalonia/C# implementation** (`AppStrings.resx`, `IDialogService`, `MessageBox`,
  `ThemeVariant`, `.axaml` styles, `ReflectionBindingExtension`).
- The active implementation is **Slint** (`slint 1.18.1`, `lucide-slint 1.49.1`).
- Token *names* were re-ported into a Slint global `Palette`, but tooling,
  machinery and several values drifted.

## 2. Token Audit (implemented vs. DESIGN.md)

### 2.1 Colors

| Token | DESIGN.md (Avalonia-era) | Implemented (`crates/greg-app/ui/theme.slint`) | Finding |
|---|---|---|---|
| Surface base | `#051424` | `surface: #051424` | Match |
| Surface elevated | `#0A1E2E` | `container-low: #0D1C2D` | Naming shift / value drift |
| Surface overlay | `#122131` | `container: #122131` | Naming shift (value reused) |
| Primary | `#8AEBFF` | `primary: #8AEBFF` | Match |
| Secondary | `#4DE082` | `secondary: #4DE082` | Match |
| On-surface | `#D4E4FA` | `on-surface: #D4E4FA` | Match |
| On-surface muted | `#8A9DB3` | `on-variant: #BBC9CD` | Value/naming drift |
| Danger | `#FF5370` | `error: #FFB4AB` (+ `error-container`) | Value/naming drift |
| Warning | `#FFD166` | not in palette; raw `#E8B93E` inline (`page-problems.slint:55`) | Missing token |
| Border active | `#8AEBFF` | `outline: #3C494C` | Missing token |

- Alpha usage: `top-highlight: #FFFFFF0D`, `soft-shadow: #00000045`; DESIGN
  discourages transparent overlays < 1.0 for backgrounds — deviation.
- Raw hardcoded colors outside the palette: `#00000099` scrim (`main.slint:735,
  782`), `#5A2530` / `#7A1F1F` danger hovers (`theme.slint:96,218`), `#FFFFFF`
  WinBtn hover icon (`theme.slint:228`).

### 2.2 Radii / spacing / typography

- Implemented radii: `radius: 6px`, `radius-lg: 10px` (global Palette);
  DESIGN: 4px cards/buttons/inputs, 8px modals.
- Page padding implemented ~18px (e.g. `page-local.slint:27–28`);
  DESIGN: 24px page margin, 16px panel/card padding.
- Dialog scrim `#00000099` vs DESIGN `#051424E6`.
- Buttons: implemented height 32px (`GregButton`), DESIGN: padding 8/16.

### 2.3 Typography

- Implemented families: `"Nexa Black"` (header, `font-weight: 900`) and
  `"monospace"` (log/editor log). All other text uses Slint default font.
- Implemented sizes: 10–15 px range observed; DESIGN specifies Space Grotesk /
  Inter / JetBrains Mono with specific sizes — not implemented.
- Fonts exist only as vendored assets in the frozen C# reference; not loaded by
  the Slint app.

### 2.4 Components / layout / feedback / dialogs / icons

- No `GridLayout` used anywhere; Linear layouts only.
- Custom `Rectangle` + `TouchArea` buttons (no native `StandardButton`);
  `ProgressIndicator` from std-widgets (DESIGN wanted custom 4px bar).
- Dialogs: inline conditional Rectangles with scrim + centered card (DESIGN
  wanted `IDialogService`/`Window.ShowDialog`) — not followed.
- Primary action position (right-bottom, destructive left) not followed;
  buttons horizontally centered in dialogs.
- Icons: Lucide via generated `lucide.slint`, `AppIcon`, stroke-width 1.8,
  ad-hoc sizes; DESIGN: stroke 1.5px, sizes 16/20/24.
- Hover: `drop-shadow` neumorphic lift implemented; DESIGN forbids shadows.
- Animation: max 150 ms intended; drop-shadow transitions observed.

### 2.5 Theming, accessibility, localization

- No dark/light theme switch; fixed global `Palette` (DESIGN allowed dark-only).
- **No accessibility usage** in any `.slint` (no accessible-name, focus,
  keyboard, hotkeys, Tab handling). DESIGN §11/§16 (resx, CompiledBindings) are
  Avalonia-era and not implementable.
- Localization: Rust string tables `STRINGS_EN`/`STRINGS_DE` (360 keys each) via
  `greg_core::l10n`; `SUPPORTED_LANGUAGES` includes 10 cultures but only EN/DE
  tables implemented (fallback English). Rust→Slint strings via property
  setters; many strings hardcoded English directly in `.slint`.

## 3. Design Audit Findings

| Severity | Finding | Evidence | Implication for URA |
|---|---|---|---|
| P1 | `DESIGN.md` does not describe the Slint implementation | `DESIGN.md` vs `crates/greg-app/ui/theme.slint`, `main.slint` | URA must be designed against the actual Slint tokens/patterns, not DESIGN.md verbatim |
| P2 | Design tokens drift from documented values | token table above | Design decisions for URA should use implemented `Palette` names |
| P3 | No accessibility implementation | no a11y usage in `.slint` | URA dialogs should add accessible names/focus handling from scratch or via Slint a11y APIs |
| P3 | No visual verification yet | no screenshots in audit | Design section of URA plan must be validated with headless screenshot harness (`crates/greg-app/src/shots.rs`) before implementation |

## 4. Audit Conclusion

The implemented design system is a fixed, dark-only, custom-Chrome Slint
implementation with hand-rolled global palette, custom Rectangle/TouchArea
components, Lucide icons, and inline dialogs. It deviates from `DESIGN.md` in
tokens, typography, spacing, shadows, icons, and tooling assumptions. Any new
UI for the URA should reuse the Slint components (`GregButton`, `ListCard`,
`IconButton`, `NavButton`, `StatusLed`, …) and the implemented palette.

## 5. Open Design Questions

- Whether and when to migrate token naming to a documented canonical set.
- Accessibility implementation status in Slint 1.18.1 (planned vs. available).
- Font loading strategy for `"Nexa Black"` and system default.
