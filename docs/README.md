# gregModmanager documentation

This directory is the repository's canonical documentation. Its files are
tracked in this repository; contributors must edit `docs/`, not a nonexistent
`wiki/` directory.

## Start here

| Audience | Document | What it covers |
| --- | --- | --- |
| Application user | [End-user guide](01_END_USER_GUIDE.md) | supported installation paths and the current UI |
| Mod or plugin creator | [Creator guide](02_MOD_CREATOR_GUIDE.md) | integration boundary and safe publishing expectations |
| Contributor or maintainer | [Contributor guide](03_CONTRIBUTOR_GUIDE.md) | setup, tests, builds, releases, and documentation changes |
| Everyone | [Index](INDEX.md) | task-oriented navigation |

## Documentation contract

Documentation describes behaviour implemented in the checked-in source. Mark
unreleased work as **planned** rather than presenting it as available. Test each
command before publishing it, keep secret values out of examples, and update
the affected document when a command, artifact name, supported platform, or
user-visible behaviour changes.

The JSON files in `examples/manifests/` are retained as historical samples; they
are not a validated input contract for the current desktop client. See their
[status note](examples/manifests/README.md) before using them.

## Maintainer references

- [Build scripts](../build/scripts/README.md)
- [Docker build and test containers](../docker/README.md)
- [Code signing](../build/installer/CODE_SIGNING.md)
- [Dependency inventory](../EXTERNAL_DEPENDENCIES.md)
- [Codebase reference](codebase/STRUCTURE.md)
