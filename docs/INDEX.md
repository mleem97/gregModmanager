# Documentation Index — gregModmanager

**Version:** v1.5.1  
**Last Updated:** May 2026

---

## Quick Navigation

### For Users
- **[End-User Guide](01_END_USER_GUIDE.md)** — Installation, configuration, UI usage, troubleshooting
- **[FAQ & Troubleshooting](01_END_USER_GUIDE.md#troubleshooting--faq)** — Common problems and solutions
- **[System Requirements](01_END_USER_GUIDE.md#system-requirements)** — Hardware and software requirements

### For Mod/Plugin Creators
- **[Mod Creator Guide](02_MOD_CREATOR_GUIDE.md)** — Creating and publishing mods
- **[Example Manifests](examples/manifests/)** — Ready-to-use metadata templates
- **[Dependency Management](02_MOD_CREATOR_GUIDE.md#defining-dependencies)** — How to declare mod dependencies
- **[Publishing to datacentermods.com](02_MOD_CREATOR_GUIDE.md#publishing-mods-to-datacentermods.com)** — Release workflow

### For Contributors
- **[Contributor Guide](03_CONTRIBUTOR_GUIDE.md)** — Setup, coding standards, testing, contributing
- **[Architecture Overview](03_CONTRIBUTOR_GUIDE.md#project-overview--architecture)** — Project structure and design
- **[Build Instructions](03_CONTRIBUTOR_GUIDE.md#build-system--scripts)** — How to build locally
- **[Testing Guide](03_CONTRIBUTOR_GUIDE.md#testing)** — Unit testing and validation
- **[PR Workflow](03_CONTRIBUTOR_GUIDE.md#contributing-workflow--pull-requests)** — Creating pull requests

### General Resources
- **[README.md](../README.md)** — Project overview and quick links
- **[RELEASENOTE.md](../RELEASENOTE.md)** — Version history and changelog
- **[EXTERNAL_DEPENDENCIES.md](../EXTERNAL_DEPENDENCIES.md)** — Third-party library inventory
- **[AGENTS.md](../AGENTS.md)** — System architecture and agent instructions
- **[LICENSE](../LICENSE)** — MIT license

---

## Documentation by Topic

### Installation & Setup
| Topic | Link | Audience |
|-------|------|----------|
| Windows installation | [End-User Guide § Windows](01_END_USER_GUIDE.md#windows) | End Users |
| Linux installation | [End-User Guide § Linux](01_END_USER_GUIDE.md#linux) | End Users |
| First-run setup | [End-User Guide § First-Run Setup](01_END_USER_GUIDE.md#first-run-setup) | End Users |
| Development environment | [Contributor Guide § Setup](03_CONTRIBUTOR_GUIDE.md#development-environment-setup) | Contributors |

### Building & Release
| Topic | Link | Audience |
|-------|------|----------|
| Build script reference | [Contributor Guide § Build System](03_CONTRIBUTOR_GUIDE.md#build-system--scripts) | Contributors |
| Release workflow | [Contributor Guide § Release Process](03_CONTRIBUTOR_GUIDE.md#release--versioning-process) | Maintainers |
| Packaging (installer, portable, Linux) | [build.ps1 source](../scripts/build.ps1) | Contributors |

### Modding & Package Management
| Topic | Link | Audience |
|-------|------|----------|
| Package types overview | [Mod Creator Guide § Modding Ecosystem](02_MOD_CREATOR_GUIDE.md#modding-ecosystem-overview) | Creators |
| Creating game mods | [Mod Creator Guide § Game Mods](02_MOD_CREATOR_GUIDE.md#creating-game-mods) | Creators |
| Creating MelonLoader plugins | [Mod Creator Guide § MelonLoader Plugins](02_MOD_CREATOR_GUIDE.md#creating-melonloader-universal--game-specific-plugins) | Creators |
| gregCore mods and extensions | [Mod Creator Guide § gregCore](02_MOD_CREATOR_GUIDE.md#creating-gregcore-mods) | Creators |
| UserLibs / shared libraries | [Mod Creator Guide § UserLibs](02_MOD_CREATOR_GUIDE.md#creating-userlibs--universal-extensions) | Creators |
| Dependency declarations | [Mod Creator Guide § Dependencies](02_MOD_CREATOR_GUIDE.md#defining-dependencies) | Creators |
| Manifest format reference | [Mod Creator Guide § Manifests](02_MOD_CREATOR_GUIDE.md#plugin-manifest-files) | Creators |

### UI & Features
| Topic | Link | Audience |
|-------|------|----------|
| UI overview | [End-User Guide § User Interface](01_END_USER_GUIDE.md#user-interface-overview) | End Users |
| Managing games | [End-User Guide § Managing Games](01_END_USER_GUIDE.md#managing-games) | End Users |
| Managing mods | [End-User Guide § Managing Mods](01_END_USER_GUIDE.md#managing-mods--plugins) | End Users |
| Mod profiles | [End-User Guide § Profiles](01_END_USER_GUIDE.md#managing-profiles) | End Users |

### Troubleshooting
| Topic | Link | Audience |
|-------|------|----------|
| Common errors | [End-User Guide § Troubleshooting](01_END_USER_GUIDE.md#common-problems--solutions) | End Users |
| Log file locations | [End-User Guide § Reading Logs](01_END_USER_GUIDE.md#reading-logs) | End Users |
| Reporting bugs | [End-User Guide § Reporting Issues](01_END_USER_GUIDE.md#reporting-issues) | End Users |
| Debugging code | [Contributor Guide § Debugging](03_CONTRIBUTOR_GUIDE.md#debugging--logging) | Contributors |

### Development
| Topic | Link | Audience |
|-------|------|----------|
| Architecture | [Contributor Guide § Architecture](03_CONTRIBUTOR_GUIDE.md#project-overview--architecture) | Contributors |
| Coding standards | [Contributor Guide § Coding Standards](03_CONTRIBUTOR_GUIDE.md#coding-standards--conventions) | Contributors |
| Testing | [Contributor Guide § Testing](03_CONTRIBUTOR_GUIDE.md#testing) | Contributors |
| Contributing workflow | [Contributor Guide § PR Workflow](03_CONTRIBUTOR_GUIDE.md#contributing-workflow--pull-requests) | Contributors |
| Localization | [Contributor Guide § Localization](03_CONTRIBUTOR_GUIDE.md#localization--translations) | Contributors |

---

## Example Resources

### Manifest Examples
Located in `docs/examples/manifests/`:

| File | Type | Use Case |
|------|------|----------|
| `01_simple_mod.json` | Game Mod | No dependencies, basic mod |
| `02_gregcore_mod_with_deps.json` | gregCore Mod | Complex dependencies, multiple requirements |
| `03_universal_plugin.json` | MelonLoader Plugin | Works with any game |
| `04_game_specific_plugin.json` | MelonLoader Plugin | Game-specific optimization |
| `05_gregcore_extension.json` | gregCore Extension | Extends gregCore services |
| `06_userlib.json` | UserLib | Shared library with versioning |

**How to use:** Copy an example manifest, modify fields for your mod, then validate and publish.

### Example Mod Structures
Located in `docs/examples/mods/` (to be populated with working examples)

---

## External Resources

### Official Ecosystems
- **[datacentermods.com](https://datacentermods.com)** — Central mod repository
- **[gregframework.eu](https://gregframework.eu)** — gregCore/gregFramework API documentation
- **[melonwiki.xyz](https://melonwiki.xyz)** — MelonLoader documentation and plugins

### Related Technologies
- **[Avalonia UI](https://docs.avaloniaui.net/)** — Cross-platform UI framework used
- **[.NET Documentation](https://learn.microsoft.com/en-us/dotnet/)** — .NET 9 API reference
- **[Inno Setup](https://jrsoftware.org/isinfo.php)** — Installer scripting language

### Community
- **[GitHub Issues](https://github.com/mleem97/gregModmanager/issues)** — Bug reports and feature requests
- **[GitHub Discussions](https://github.com/mleem97/gregModmanager/discussions)** — General questions and ideas
- **[GitHub Releases](https://github.com/mleem97/gregModmanager/releases)** — Download releases

---

## Documentation Maintenance

### Wiki (Submodule)

The `wiki/` folder is a Git submodule pointing to [gregModmanager.wiki](https://github.com/mleem97/gregModmanager.wiki).

**To update wiki:**

```bash
# Navigate to wiki submodule
cd wiki

# Make changes (using GitHub web or local editor)
git add .
git commit -m "docs: update [topic]"
git push origin main

# Return to main repo and commit submodule update
cd ..
git add wiki
git commit -m "chore: update wiki submodule"
git push origin main
```

### docs/ Folder

The `docs/` folder contains generated/supplementary documentation:

- `01_END_USER_GUIDE.md` — User-facing guide
- `02_MOD_CREATOR_GUIDE.md` — Modding documentation
- `03_CONTRIBUTOR_GUIDE.md` — Developer/contributor guide
- `examples/` — Code examples and templates
- `INDEX.md` — This file

**To update docs:**

```bash
# Edit .md files directly
git add docs/
git commit -m "docs: update [topic]"
git push origin main
```

---

## Contributing to Documentation

Want to improve the docs? Here's how:

1. **Identify the gap:** Missing info, outdated section, unclear explanation?
2. **Create an issue:** [GitHub Issues](https://github.com/mleem97/gregModmanager/issues) with label `documentation`
3. **Fork and edit:** Edit the relevant `.md` file
4. **Submit PR:** Include a clear description of changes
5. **Review:** Maintainers will review for clarity and accuracy

**Documentation style guide:**
- Use clear, concise language
- Include code examples where relevant
- Link to related sections
- Add screenshots for UI-heavy topics
- Test any instructions you add

---

## Version Compatibility

This documentation is for:
- **gregModmanager v1.5.1**
- **.NET 9.0+**
- **Avalonia UI 11.2+**
- **MelonLoader 0.6.0+**
- **gregCore 2.0.0+**

Documentation for other versions:
- **v1.5.0:** See [releases/tag/v1.5.0](https://github.com/mleem97/gregModmanager/releases/tag/v1.5.0)
- **Older versions:** Refer to Git history

---

**Last Updated:** May 2026  
**Next Review:** November 2026
