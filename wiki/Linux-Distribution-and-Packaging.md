# Linux Distribution and Packaging

## Supported Targets
Primary distro families for gamers and IT users:
- Debian/Ubuntu (`.deb`)
- Fedora/RHEL (`.rpm`)
- Arch (`.pkg.tar.zst`)
- Universal fallback (`.tar.gz`)

## Scripts
- `scripts/linux/build-avalonia-packages.sh`
- `scripts/linux/build-avalonia-packages.ps1`

## Flatpak
- Manifest: `scripts/linux/flatpak/com.gregframework.gregModmanager.yml`
- Launcher: `scripts/linux/flatpak/gregmodmanager-flatpak-launcher.sh`

## CI/CD
Workflow: `.github/workflows/avalonia-linux-packages.yml`
- Builds Linux artifacts
- Uploads packages as workflow artifacts
- Publishes a runtime container to GHCR

## Notes
- Desktop launcher entry is included.
- Package dependencies are explicitly declared.
- Flatpak is optional but recommended for broad desktop compatibility.
