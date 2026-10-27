# TOOLS.md — Build, CI, and Operations

## Build

- Local build script: `build/scripts/build.ps1`.
- Interactive builders: `build/builder.ps1` and `build/builder.sh`.
- CI workflows: `.forgejo/workflows/` (Forgejo Actions) and `.gitea/workflows/`
  (Gitea Actions) — both directories are byte-identical by convention (enforced
  by the `parity` job), Linux-only runners; Windows artifacts are cross-built.
  GitHub is a read-only archive mirror — `.github/` contains no workflows.
- Keep script and workflow changes aligned unless the PR explicitly documents a staged migration.

## Steam Workshop

- Steam AppID for Data Center: `4170200`.
- `SteamPublishRateLimiter.Shared` enforces cooldown behavior for publish attempts.
- UI cooldown copy should show seconds when rate-limit state is visible.

## Signing

- Windows signing uses `build/installer/sign-authenticode.ps1`.
- `auto`/`self-signed` creates a fresh build certificate; use `-SigningMode pfx`
  with `CODE_SIGN_THUMBPRINT`, or `CODE_SIGN_PFX` plus `CODE_SIGN_PFX_PASSWORD`,
  to reuse an existing certificate.
- Do not include `steam_api64.dll` in signing loops unless the maintainer confirms the vendor binary is safe to sign.

## Telemetry

- Telemetry implementation: `TelemetryService.cs`.
- Telemetry must respect `AppSettings.TelemetryEnabled`.
- Installation tracking uses an anonymized machine identifier.

## Code coverage (Codacy)

- Coverage wird bei jedem Push auf `main`/`dev` und bei Pull Requests (aus diesem
  Repo, nicht von Forks) mit [cargo-tarpaulin](https://github.com/xd009642/tarpaulin)
  als workspace-weites LCOV-Report erzeugt und über den
  [Codacy Coverage Reporter](https://github.com/codacy/codacy-coverage-reporter)
  an Codacy hochgeladen (`.forgejo/workflows/rust.yml` / `.gitea/workflows/rust.yml`,
  Job `coverage`).
- Es wird ein einzelnes workspace-weites LCOV-Report hochgeladen
  (`cargo tarpaulin --workspace --out lcov`), sodass auch unveränderte Module
  (Monorepo) abgedeckt sind. Siehe die Codacy-Docs zu mehreren Reports
  (`-r` mehrfach / `--partial` + `final`) und zu nicht unterstützten Sprachen
  (`-l <Sprache> --force-language`).
- Authentifizierung: das Repository-Geheimnis `CODACY_PROJECT_TOKEN` setzen
  (Codacy → Project settings → API token). Wird es nicht gesetzt, wird Coverage
  nur lokal erzeugt, nicht hochgeladen. Bei Codacy Self-hosted zusätzlich
  `CODACY_API_BASE_URL` (und ggf. `CODACY_REPORTER_VERSION`) setzen.

- Lokal:
  `./build/scripts/linux/coverage-codacy.sh` erzeugt ein workspace-weites
  LCOV-Report (cargo-tarpaulin) und lädt es über den Codacy Coverage Reporter
  hoch — identisch mit dem CI-Job. Ohne `CODACY_PROJECT_TOKEN` bricht das
  Skript mit einer klaren Fehlermeldung ab.
  Optionen: `--check` (Voraussetzungen prüfen), `--install-only` (nur
  Reporter + tarpaulin installieren), `--partial` / `--final` (mehrere Reports
  nacheinander, erweitertes Szenario), `--force-language -l <Sprache>` (für
  nicht unterstützte Sprachen), `-r <datei>` (explizites Report-File).

## Troubleshooting

- Duplicate Avalonia title bars: check custom-chrome settings.
- Steam upload blocked: inspect rate-limit state and cooldown copy.
- JSON trimming warnings: verify source-generation registration in `AppJsonContext`.
- Linux package build failures: verify external packaging tools listed in `EXTERNAL_DEPENDENCIES.md`.
- Codacy zeigt keine Coverage: `CODACY_PROJECT_TOKEN` geheim gesetzt? Der letzte
  Commit des Branches musste bereits statisch von Codacy analysiert sein, bevor
  Coverage-Metriken erscheinen (siehe Codacy-Docs „Validating that the coverage setup is complete“).
