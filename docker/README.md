# Container build and test

The repository has separate targets for compilation/testing, Windows-container
builds, and macOS cross-publish artifacts.

## Linux SDK container

These commands work with Docker Buildx on Linux, macOS, or Docker Desktop in
Linux-container mode:

```bash
# Restore, compile and run the .NET test suite.
docker buildx bake test

# Generate a Windows x64 publish directory. This does not create an EXE/MSI.
docker buildx bake windows-cross-publish

# Generate macOS publish directories.
docker buildx bake macos-x64-publish
docker buildx bake macos-arm64-publish
```

Artifacts are written to `artifacts/docker/`, which is ignored by Git.

## Windows container

`docker/Dockerfile.windows` is a real Windows-container build and test image.
It requires a Windows host with Docker configured for **Windows containers**
and a host version compatible with `windowsservercore-ltsc2022`.

The project targets `net9.0` while `global.json` selects SDK 10. The image
copies the .NET 9 runtime into the SDK image so its `test` target can execute
the test binaries without a manual image patch.

```powershell
docker build --file docker/Dockerfile.windows --target test .
docker build --file docker/Dockerfile.windows --target publish --output type=local,dest=artifacts/docker/windows-native .
```

The native Windows installer build remains a Windows-host task because it
requires Inno Setup, WiX and Authenticode signing. Use the existing
`build/scripts/build.ps1` workflow or the Windows GitHub Actions runner for
the EXE and MSI.

## macOS boundary

Docker has no supported macOS container runtime or macOS base images. The
Linux SDK container can cross-publish `osx-x64` and `osx-arm64`, but it cannot
execute or UI-test those binaries. Run macOS tests, signing, notarization, and
installer packaging on a macOS runner (for example GitHub Actions
`macos-latest`) after adding the macOS-specific project code required by
`AGENTS.md`.

## Manual macOS GUI test from Ubuntu

Use a real Mac on the local network or a remote macOS machine with SSH and a
logged-in desktop session. From this repository on Ubuntu:

```bash
chmod +x build/scripts/macos/run-remote-manual-test.sh
./build/scripts/macos/run-remote-manual-test.sh --host user@mac-host --arch arm64
```

The script synchronizes source without deleting remote files, runs restore,
build and tests on the Mac, then creates a launchable GUI build there. Connect
to the Mac with Screen Sharing (or use its physical desktop) and launch the
path printed by the script. This is required for real interaction with the
macOS window system; SSH and Docker alone cannot provide it.
