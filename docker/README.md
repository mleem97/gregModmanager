# Container build and test

The repository has a Rust build container for deterministic build/test and
cross-RID publish.

## Linux Rust SDK container

```bash
# Restore, compile and run the Rust test suite.
docker buildx bake test

# Generate a Windows x64 publish directory. This does not create an EXE/MSI.
docker buildx bake windows-cross-publish

# Generate macOS publish directories.
docker buildx bake macos-x64-publish
docker buildx bake macos-arm64-publish
```

Artifacts are written to `artifacts/docker/`, which is ignored by Git.

## Cross-build boundary

The Linux Rust container cross-compiles the Windows x64 target through the mingw
toolchain; macOS targets require a macOS host with the Apple SDK and cannot be
cross-built from Linux.

## macOS boundary

Docker has no supported macOS container runtime or macOS base images. The
Linux Rust container can cross-publish `osx-x64` and `osx-arm64`, but it cannot
execute or UI-test those binaries. Validate the current cross-published artifact
on a real Mac. No macOS CI job, signed package, notarization, or release artifact
exists yet; add and validate those independently before claiming macOS support.

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
binary printed by the script. This is required for real interaction with the
macOS window system; SSH and Docker alone cannot provide it.
