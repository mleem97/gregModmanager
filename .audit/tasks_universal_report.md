# Universal Report System — Tasklist

Stand: 2026-10-28, nach Implementierung von f25b222 / 88d5d5f / 9b5cabc
Stand: 2026-10-28, Master-Implementation (Abschlussrunde)

## Offene Arbeitspakete

### P1 — Report-Assistent (lokal implementierbar)
- [x] Report-Tab + Navigation
- [x] Kategorien + Validierung + Redaction + Markdown
- [x] Vorschau + lokaler Export
- [x] Submission-Bestätigung + env-gated GitHub-Submit
- [x] Versioniertes Schema + States + Modes
- [x] Log-Sammlung in Report-Draft (`report_collect_logs`) — **JETZT IN ARBEIT**
- [x] Lokale Persistenz gespeicherter Reports / Wiederaufnahme — **UMGESETZT** (`ReportStore`)
- [ ] Anhangs-Upload (Upload-Infrastruktur fehlt → Fallback dokumentieren)

### P2 — Identität (OAuth / Git / Datacentermods)
- [ ] Discord OAuth: Code + Config-Validierung + NotConfigured-Status — **UMGESETZT** (`greg-modstore::discord`)
- [ ] GitHub OAuth: Code + Config-Validierung + NotConfigured-Status — **UMGESETZT** (`greg-modstore::github` token surface + env config)
- [ ] Git Import: sichere Klonlogik + Validierung — **BLOCKED_BY_EXTERNAL_SETUP** (kein Git-Server-Backend im Repo)
- [ ] Datacentermods Provider-Vertrag: Interface + Modelle (API fehlt → BLOCKED)

### P3 — Produktübergreifend / Sync / KI
- [ ] gregLint-Integrationsvertrag — **UMGESETZT** (Schnittstelle in `greg-core::report`, Quellen: SOURCE_GREG_LINT)
- [ ] VSIX-Integrationsvertrag — **UMGESETZT** (Schnittstelle in `greg-core::report`, Quellen: SOURCE_VSIX)
- [ ] Datacentermods-Sync-Modell + Statusanzeige — **BLOCKED_BY_EXTERNAL_SETUP** (Sync-Status-Modelle vorhanden)
- [x] AI-Provider-Abstraktion (OpenAI-kompatiert, Anthropic-kompatiert, lokal) — **UMGESETZT** (`greg-ai`)
- [ ] Projektzusammenfassungen, Vision-Fallback, lokale KI — **UMGESETZT** (lokale Structurierung + Vision-Gate)

### P4 — Ops / Abschluss
- [x] `.env.example` finalisieren — **UMGESETZT** (+ operator/discord keys ergänzt)
- [ ] Setup/Docs finalisieren — **UMGESETZT** (`docs/setup/…`)
- [ ] Finale Security-/Qualitätsprüfung — **IN ARBEIT**
- [ ] Abschlussbericht finalisieren — **IN ARBEIT**

## Empfohlene Reihenfolge

1. Report-Assistent lokal abschließen (Logs, Persistenz). ✅
2. OAuth-/Identitäts-Code-Schranken auflisten und implementierbare Teile umsetzen. ✅
3. AI-/Sync-/Productivity-Schnittstellen nach Bedarf. ✅
4. Abschlussbericht + Docs. 🔄
