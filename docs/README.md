# gregModmanager Documentation

This folder contains comprehensive documentation for end users, mod creators, and contributors.

## 📖 Main Documentation Files

### 1. **End-User Guide** (`01_END_USER_GUIDE.md`)

Complete guide for everyday users of gregModmanager:

- Installation on Windows, Linux, macOS (planned)
- Getting started and first-run setup
- User interface overview
- Managing games and mods
- Troubleshooting and FAQ
- Localization and language support

**Audience:** All end users

### 2. **Mod & Plugin Creator Guide** (`02_MOD_CREATOR_GUIDE.md`)

Comprehensive guide for creators making mods, plugins, and extensions:

- Modding ecosystem overview (package types, MelonLoader, gregCore)
- Creating MelonLoader universal and game-specific plugins
- Creating game mods
- Creating gregCore mods and extensions
- Creating UserLibs / shared libraries
- Defining dependencies
- Plugin manifest format and validation
- Publishing to datacentermods.com

**Audience:** Mod developers, plugin creators

### 3. **Contributor Guide** (`03_CONTRIBUTOR_GUIDE.md`)

Technical guide for developers contributing to gregModmanager:

- Project architecture and structure
- Development environment setup
- Build system and scripts
- Coding standards and conventions
- Testing and debugging
- Contributing workflow and pull requests
- Localization system
- Release and versioning process
- Security guidelines

**Audience:** Contributors, maintainers, developers

### 4. **Documentation Index** (`INDEX.md`)

Quick navigation index for all documentation:

- Links organized by audience (users, creators, contributors)
- Topic-based table of contents
- Example resources
- External resources
- Documentation maintenance guide

**Audience:** Everyone (entry point)

---

## 📂 Examples Directory

### `examples/manifests/`

Ready-to-use manifest templates for different mod/plugin types:

| File | Type | Purpose |
|------|------|---------|
| `01_simple_mod.json` | Game Mod | Basic mod with no dependencies |
| `02_gregcore_mod_with_deps.json` | gregCore Mod | Complex mod with multiple dependencies |
| `03_universal_plugin.json` | MelonLoader Plugin | Works with any game |
| `04_game_specific_plugin.json` | MelonLoader Plugin | Game-specific optimization |
| `05_gregcore_extension.json` | gregCore Extension | Extends gregCore services |
| `06_userlib.json` | UserLib | Shared library with versioning |

**How to use:**

1. Copy the relevant manifest for your project type
2. Edit the fields (id, name, version, author, dependencies, etc.)
3. Validate the manifest using gregModmanager's **Validate Manifest** feature
4. Package and publish

### `examples/mods/`

Example mod structures and implementations (to be expanded with working examples):

- Complete folder structures
- Example code
- Configuration files
- Best practices demonstrations

---

## 🎯 Quick Navigation

### I'm a

**End User:**

1. Start with [End-User Guide](01_END_USER_GUIDE.md) § [Getting Started](01_END_USER_GUIDE.md#getting-started)
2. See [Troubleshooting](01_END_USER_GUIDE.md#troubleshooting--faq) if you hit issues
3. Check [FAQ](01_END_USER_GUIDE.md#faq) for common questions

**Mod Creator:**

1. Read [Mod Creator Guide](02_MOD_CREATOR_GUIDE.md) § [Modding Ecosystem Overview](02_MOD_CREATOR_GUIDE.md#modding-ecosystem-overview)
2. Follow the section for your package type (Game Mod, Plugin, gregCore, UserLib)
3. Use the example manifests in `examples/manifests/`
4. See [Publishing](02_MOD_CREATOR_GUIDE.md#publishing-mods-to-datacentermods.com) for distribution

**Contributor:**

1. Read [Contributor Guide](03_CONTRIBUTOR_GUIDE.md) § [Development Environment Setup](03_CONTRIBUTOR_GUIDE.md#development-environment-setup)
2. Follow the build instructions
3. Read [Coding Standards](03_CONTRIBUTOR_GUIDE.md#coding-standards--conventions)
4. Review [Contributing Workflow](03_CONTRIBUTOR_GUIDE.md#contributing-workflow--pull-requests)

---

## 📚 Documentation by Topic

| Topic | File | Section |
|-------|------|---------|
| Installation | End-User Guide | § Installation & Platform Support |
| Building locally | Contributor Guide | § Development Environment Setup |
| Creating mods | Mod Creator Guide | § Creating Game Mods |
| Creating plugins | Mod Creator Guide | § Creating MelonLoader Plugins |
| Dependencies | Mod Creator Guide | § Defining Dependencies |
| Manifests | Mod Creator Guide | § Plugin Manifest Files |
| Publishing | Mod Creator Guide | § Publishing to datacentermods.com |
| Architecture | Contributor Guide | § Project Overview & Architecture |
| Testing | Contributor Guide | § Testing |
| Contributing | Contributor Guide | § Contributing Workflow |
| Release process | Contributor Guide | § Release & Versioning Process |

---

## 🔗 Related Resources

### Inside This Repository

- **README.md** — Project overview (top-level)
- **RELEASENOTE.md** — Version history and changelog
- **EXTERNAL_DEPENDENCIES.md** — Third-party library inventory
- **AGENTS.md** — System architecture and build constraints
- **LICENSE** — MIT license

### External Links

- **[datacentermods.com](https://datacentermods.com)** — Mod repository
- **[gregframework.eu](https://gregframework.eu)** — gregCore documentation
- **[melonwiki.xyz](https://melonwiki.xyz)** — MelonLoader documentation
- **[GitHub Repository](https://github.com/mleem97/gregModmanager)** — Source code
- **[GitHub Issues](https://github.com/mleem97/gregModmanager/issues)** — Bug reports
- **[GitHub Discussions](https://github.com/mleem97/gregModmanager/discussions)** — Questions & ideas

---

## 📝 Contributing to Documentation

Found an error? Want to improve the docs? Here's how:

1. **Identify the issue** — Which section is unclear or incorrect?
2. **Create an issue** — [GitHub Issues](https://github.com/mleem97/gregModmanager/issues) with `documentation` label
3. **Submit a PR** — Fork, edit, and create a pull request
4. **Style guide:**
   - Use clear, concise language
   - Include code examples
   - Link to related sections
   - Test any instructions

---

## 📋 Version Information

**Documentation Version:** v1.5.1  
**Last Updated:** May 2026

This documentation covers:

- **gregModmanager** v1.5.1+
- **.NET** 9.0+
- **Avalonia UI** 11.2+
- **MelonLoader** 0.6.0+
- **gregCore** 2.0.0+

For documentation on older versions, see [GitHub Releases](https://github.com/mleem97/gregModmanager/releases).

---

**Need help?** Check the [Documentation Index](INDEX.md) or [GitHub Discussions](https://github.com/mleem97/gregModmanager/discussions).
