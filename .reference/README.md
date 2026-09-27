# .reference/

Reference material for the Rust rewrite. Not built, not extended.

## modmanager-old/

Frozen C# / Avalonia (.NET) implementation of gregModmanager, moved here when
the project restarted in Rust. See `RUST_GRAPH.md` (repo root) for the new
workspace plan and `AGENTS.md` for current agent instructions.

- Buildable snapshot: `modmanager-old/GregModmanager.sln` (`global.json` pins the SDK).
- Layout: `src/` (Core, Avalonia UI, platform shims, Melons) + `tests/`.
- History is preserved via `git mv` — use `git log --follow` on any file.
- Do not extend. Bugfixes, if ever needed, go through the maintainer first.
