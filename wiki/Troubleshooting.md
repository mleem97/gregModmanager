# Troubleshooting

## Avalonia project does not run
- Ensure `.NET 9 SDK` is installed.
- Run `dotnet restore` and check NuGet feed access.

## Linux package build fails
- Install `nfpm` via GoReleaser apt repo:

```bash
echo 'deb [trusted=yes] https://repo.goreleaser.com/apt/ /' | sudo tee /etc/apt/sources.list.d/goreleaser.list
sudo apt update
sudo apt install nfpm
```

- Verify `nfpm --version` works and `nfpm` is in `PATH`.
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
