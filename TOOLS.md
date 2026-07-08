# TOOLS.md — Build, CI, and Operations

## Build

- Local build script: `build/scripts/build.ps1`.
- Interactive builders: `build/builder.ps1` and `build/builder.sh`.
- CI workflow: `.github/workflows/build-and-release.yml`.
- Keep script and workflow changes aligned unless the PR explicitly documents a staged migration.

## Steam Workshop

- Steam AppID for Data Center: `4170200`.
- `SteamPublishRateLimiter.Shared` enforces cooldown behavior for publish attempts.
- UI cooldown copy should show seconds when rate-limit state is visible.

## Signing

- Windows signing uses `build/installer/sign-authenticode.ps1`.
- Supported signing inputs: `CODE_SIGN_THUMBPRINT`, or `CODE_SIGN_PFX` plus `CODE_SIGN_PFX_PASSWORD`.
- Do not include `steam_api64.dll` in signing loops unless the maintainer confirms the vendor binary is safe to sign.

## Telemetry

- Telemetry implementation: `TelemetryService.cs`.
- Telemetry must respect `AppSettings.TelemetryEnabled`.
- Installation tracking uses an anonymized machine identifier.

## Troubleshooting

- Duplicate Avalonia title bars: check custom-chrome settings.
- Steam upload blocked: inspect rate-limit state and cooldown copy.
- JSON trimming warnings: verify source-generation registration in `AppJsonContext`.
- Linux package build failures: verify external packaging tools listed in `EXTERNAL_DEPENDENCIES.md`.
