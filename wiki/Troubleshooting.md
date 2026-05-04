# Troubleshooting

## Avalonia project does not run
- Ensure `.NET 9 SDK` is installed.
- Run `dotnet restore` and check NuGet feed access.

## Linux package build fails
- Verify Docker is installed or `nfpm` is available in PATH.
- Ensure publish output exists before packaging.

## Steam upload blocked unexpectedly
- Check cooldown message and wait until retry window expires.
- Ensure Steam client is running and user is logged in.

## Duplicate title bars
- Confirm window settings in `MainWindow.axaml`:
  - `SystemDecorations="None"`
  - `ExtendClientAreaToDecorationsHint="True"`
  - `ExtendClientAreaChromeHints="NoChrome"`

## UI appears too large
- Validate compact style values in `Styles/TerminalCore.axaml`.
- Reduce headline and panel padding for high-DPI scenarios.
