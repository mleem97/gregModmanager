# GitHub release automation

The GitHub Actions package workflow runs checks and builds on pull requests,
manual dispatches, and pushes to `main`. Only a successful push to `main` can
publish a release. It publishes a stable, latest GitHub Release tagged
`v<VERSION>` after the quality, Linux and Windows jobs succeed. Pull requests
and manual dispatches only produce test artifacts.

`VERSION` and the Cargo workspace version must match and contain a stable
three-part version. A previously published stable release is left unchanged;
increment both versions to publish a new release. A pre-existing tag, draft,
prerelease or unexpected API error stops publication rather than replacing it.
An incomplete draft requires maintainer inspection before retrying.
If a run fails after creating its tag, inspect the workflow log, the draft on
the repository's Releases page and `git ls-remote --tags origin 'refs/tags/v<VERSION>'`.
Resolve any partial draft or tag only after verifying its commit and assets;
the job deliberately does not overwrite or delete them on retry.

The release contains Windows x64 Setup EXE, MSI and portable ZIP plus Linux
x64 DEB, RPM, Arch and portable tarball, each with a SHA-256 file. Linux builds
run on Ubuntu 24.04. The Alpine APK is built separately for diagnostics and is
not distributed as a stable asset because it does not run on Alpine.

Windows installers are not signed by default. Ubuntu DEB and Windows installers
have undergone limited installation testing; RPM/Arch, Steam/game integration
and native Windows upgrade paths still need further validation. GitHub Actions
stores the prerelease-independent build artifacts in the workflow run as well.
