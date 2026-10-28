# Architektur — gregModmanager

> gregModmanager is a cross-platform desktop client for the gregFramework modding ecosystem. It manages local Workshop projects, publishes them to the Steam Workshop via native Steamworks calls, browses and subscribes Mod Store items, and launches Data Center with or without mods — on Windows and Linux.

## Komponenten

- **Einstieg:** siehe [QUICKSTART.md](../QUICKSTART.md).
- **Crates:** [`crates/`](../crates/) — Domain, Platform, Steam, Modstore, Loader, App, CLI.
- **Build-Orchestrierung:** [`xtask/`](../xtask/) — `cargo xtask check|setup|dist`.
- **Skripte:** [`scripts/`](../scripts/) — Automatisierung rund um Build/Test/Release.
- **Tests:** [`crates/`](../crates/) — Rust Workspace-Tests.
- **Referenzen:** [`references/`](../references/) — externe Referenzen und Material.
- **Beispiele:** [`docs/examples/`](../docs/examples/) — lauffähige Minimalbeispiele.

## Datenflüsse

Projektspezifisch dokumentieren (Komponente A → B, Formate, Schnittstellen).
Änderungen hier + [`CHANGELOG.md`](../CHANGELOG.md) (Unreleased) nachtragen.
