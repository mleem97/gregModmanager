# Greg Modmanager Concept

# Data Center Modmanager – Product Concept and Functional Requirements

## 1. Objectives

I develop with **Rust and Slint** a cross-platform mod manager for the game *Data Center*.

The goal is a powerful all-in-one tool for end users, modders and mod publishers. The application shall cover the entire lifecycle of mods: from installation and management, over development and publishing, to error analysis and maintenance.

The mod manager shall become a central part of the Data Center mod ecosystem and must remain manageable, performant and easy to use even with an increasing number of installed mods.

The user interface and information architecture must therefore be consistently designed for scalability, clear responsibilities and short interaction paths.

## 2. Navigation

The application currently has four functional main areas, presented via an **icon-based left navigation bar** (Slint layout including `theme.slint` and `icons.slint`, see `crates/greg-app/ui/main.slint`):

1. **Mod manager** – installation and management for end users.
2. **Workshop** – development, validation and publishing (upload) for modders and publishers.
3. **Mod store** – central search, discovery and installation of mods from the Steam Workshop and Datacentermods.com.
4. **Settings** – configuration of the application, game paths, working directories and integrations.

The assignment of areas to navigation icons and the sub-sections per area are defined via `on_icon` and `on_nav` in `crates/greg-app/src/actions.rs` as well as `AppState::apply_icon_state` in `crates/greg-app/src/state.rs`.

### Current sub-sections per area

- The **Mod manager** (`modmanager` icon) uses a sub-section bar: **My Mods**, **My Plugins**, **My Libs**, **Modpacks / Collections**, **Problems** and **Report**.
- The **Workshop** (`workshop` icon) uses a sub-section bar: **Projects**, **New Project**, **Browse**, **Subscribed** and **Favorites**.
- The **Mod store** (`modstore` icon) and **Settings** (`settings` icon) do not use a sub-section bar; the functionally appropriate pages (mod store surface including updates and installations, settings page) serve as central surfaces.

This structure must be reviewed and, if necessary, reworked. Functions should be assigned to areas where they logically belong. Duplicate functions and redundant navigation elements are to be avoided.

**Status (2026-10-29, version 1.7.7):** The icon-based navigation is implemented. The implementation is done via the Slint pages `page-local`, `page-packs`, `page-projects`, `page-editor`, `page-browse`, `page-store`, `page-settings`, `page-problems` and `page-report`. Individual concept decisions (e.g. status bar instead of `Problems` main navigation) are only partially implemented; details are in the respective sections and in the implementation status (section 11).

## 3. Mod manager

The mod manager is primarily aimed at end users and bundles all functions for managing installed mods.

### Current sub-sections

- **My Mods:** Installed and managed mods, including active and disabled entries as well as dependency and conflict hints.
- **My Plugins:** Installed plugins, in particular MelonLoader plugins, including browsing and automatic installation of the official IL2CPP Assembly Fixer.
- **My Libs:** Libraries and dependencies required by mods, including installation of the gregFramework libraries into the `{GameRoot}/UserLibs` directory.
- **Modpacks / Collections:** Collections of several mods.
- **Problems:** Current problems, warnings and errors.
- **Report:** Reporting of errors, improvement suggestions and feature requests via the Universal Report Assistant.

### Requirements and open points

**Mod management**

The application must remain manageable even with a large number of mods. Suitable filter, search, sort and grouping functions are to be provided.

**Plugins and libraries**

Mods, plugins and libraries must be managed separately from each other logically. Dependencies and possible conflicts should be presented transparently.

**Modpacks and collections**

The distinction between modpacks and collections must be conceptually defined.

In this regard, it should be checked whether both terms actually need different functions or whether a shared implementation with different use cases would be more sensible.

**Problems and status messages**

The former `Problems` area should be removed from the main navigation and integrated into a global status and notification bar at the bottom of the window.

This status bar must provide information for different user groups, including:

- End users: installation errors, missing dependencies, mod conflicts and compatibility issues.
- Modders: build errors, project issues and validation errors.
- Publishers: upload errors, synchronization issues and publishing status.

The display should be context-dependent and only highlight relevant information.

**Status (2026-10-29):** The global status bar has meanwhile been implemented and is visible at the bottom of the window. It provides status LEDs for Steam connection, gregAPI connection and MelonLoader status including version display. The factual problems flow remains currently a sub-area of the mod manager in the navigation bar; the conceptual shift into the status bar is therefore only partially implemented.

**Report**

Bug reporting as well as reporting of errors, improvement suggestions and feature requests shall be integrated into the central report workflow of the Greg Framework. Reports must be assigned to the correct mod and repository and must be processed via the intended Git integrations.

## 4. Workshop and mod development

The area is currently primarily implemented as a workshop/upload area with a focus on the Steam Workshop.

This designation is too narrow long-term, since the area shall also support development, validation and publishing of mods for multiple platforms.

A suitable new name still has to be determined. The area shall be designed as the central development and publishing environment regardless of the later chosen name.

### New Project

The former `New Project` function was replaced by a guided **Mod-Creation-Assistant**.

The assistant shall guide modders through the complete initial setup of a project and ensure that all required files, metadata and configurations are created correctly.

The assistant must consider at least the following use cases:

- Creation of a new mod project.
- Configuration of the required metadata.
- Setup of the project structure and local workspace.
- Validation of the required files and settings.
- Preparation for the Steam Workshop.
- Preparation for the Modstore on Datacentermods.com.
- Support for projects that should be published on multiple platforms.

The project structure and publishing process should use as many shared components as possible, so that mods do not have to be maintained completely separately for each platform.

**Status (2026-10-29):** The assistant is implemented as a dialog with project folder input and project template (`show-new-modal` in `crates/greg-app/ui/main.slint`). The final validation of files and settings and the further expansion to multi-platform projects are part of the ongoing implementation.

### My Projects

Here all mod projects managed by the user are brought together. Own uploads are managed via the PublishedFileId-based project assignment; the former separate "My Uploads" surface is therefore integrated into the project management.

The project overview shall represent the current development and publishing status and enable direct access to the associated tools.

Plausible information:

- Project status and validation results.
- Local workspace and project files.
- Detected problems and warnings.
- Publishing status per platform.
- Available updates and synchronization status.

### Further functions

The former areas `Browse`, `Subscribed` and `Favorites` should be removed from the workshop area and reassigned.

- **Browse:** To be moved into the Modstore.
- **Subscribed:** To be integrated into the regular mod management, since subscribed mods should already be managed via `My Mods`.
- **Favorites:** To be made centrally available in the mod manager and the Modstore, so that favorites can be both managed and discovered.

Duplicate subpages with identical functional scope should be avoided.

**Status (2026-10-29):** The conceptual relocation is only partially implemented. The workshop icon continues to use `Browse`, `Subscribed` and `Favorites` (as a list within the browse page), while the Modstore additionally provides its own browse, updates and installation surfaces. The final consolidation is open.

## 5. Modstore

The mod store shall be the central point of entry for searching, discovering and installing mods.

It combines two sources:

- Steam Workshop.
- Datacentermods.com mod store.

The current implementation is functionally built: The browse surface primarily works via the Steam Workshop (via the native Steamworks library or via the Steam web API fallback), offers update and installation surfaces with multi-selection and mass installation. Publications can additionally be submitted to the mod store via `gregcli publish --to modstore` (API token via `--token` / `GREG_MODSTORE_TOKEN`) and via an editor button; the mod store identifier is persisted in `metadata.json` (`modstore_id`).

The connection to the complete Datacentermods.com catalog including direct installations from Datacentermods.com remains currently open.

### Requirements

**Mod search and discovery**

The mod store needs an appropriate surface for searching, filtering and displaying available mods from both sources.

**Unified mod representation**

Mods should be presented uniformly regardless of their source. Source-specific information and functions must be available when needed.

**Installation sources and prioritization**

If the same mod is available both in the Steam Workshop and on Datacentermods.com, the Steam version should be preferred by default.

An exception applies if the version available on Datacentermods.com is demonstrably newer. In this case, the version on Datacentermods.com should be preferred.

The decision must be based on a traceable version comparison. A mere comparison of release timestamps is not sufficient if no reliable version information is available.

If a version cannot be clearly classified as newer, the application must not without further ado prefer a possibly outdated or incompatible version.

**Favorites**

Favorite mods must be accessible in the mod store. Changes to favorites should be centrally managed and taken over in the other relevant areas.

**Integration into mod management**

Installed mods must appear uniformly in the mod manager regardless of their original download source.

The source of a mod must remain traceable so that updates, dependencies and conflicts can be handled correctly.

## 6. Settings

The `Settings` area bundles all configuration options of the application.

The settings must be organized by functional areas in an orderly manner.

### Language and user interface

- Selection of the application language.
- Further global display settings, if required.

### Workspace and project management

The user must be able to configure the storage location for mod projects.

Each project must have a uniquely assigned workspace that contains all files required for development, validation and upload.

The workspaces must be organized so that multiple projects can be managed independently of each other.

### Game directory

The mod manager must automatically detect the installation directory of Data Center where possible on the respective platform.

The following installation types must be considered:

- **Windows:** Detection within existing Steam libraries.
- **Linux:** Detection of the Steam installation including relevant Proton environments.
- **macOS:** Detection of native Steam installations.
- **Custom installations:** Manual configuration of external installation paths, e.g. for GOG or Epic Games, if the respective installation is supported.

Automatically detected paths must be verifiable. If automatic detection fails, the user must be able to select the directory manually.

The application must not assume that the game is always installed in the Steam default directory.

### Integrations and telemetry

The settings shall also enable configuration of the Greg Framework integrations.

This includes especially:

- Grafana for monitoring and telemetry.
- Bugsink for error capture and error tracking.
- Central logging of relevant application errors.
- Configuration of the connection to the intended backend services.

Local error diagnosis and central telemetry must be logically separated. The transmission of diagnostic information must be controllable and data-protection compliant.

**Status (2026-10-29):** The settings page is implemented and includes working directory, game directory, language, mod store API URL, Steam web API key, diagnostics/support bundle, health checks and log directory.

## 7. Global top bar

In addition to the main navigation, the application has a global top bar with the following functions:

- `Start Game`
- `WithMods`
- `Stop`
- Login for the mod store

The functions must be reviewed regarding their responsibilities and their actual behavior.

### Start Game

Starts Data Center without the mods managed by the mod manager.

### WithMods

Starts Data Center with the configured mod environment.

The activated mods and required dependencies must be taken into account.

### Stop

Ends the game instance managed by the mod manager and resets the Steamworks state afterwards.

If a game is running, the stop button replaces the start buttons. A start without mods is done via **Start Game**, a start with mods via **With Mods**. If a start was interrupted by a kill, temporarily disabled mod assemblies are restored automatically at the next start (vanilla mode).

**Status (2026-10-29):** The stop function is implemented (`stop_game` in `crates/greg-app/src/actions.rs`, stop button in the header during `game_running`). A complete review of responsibilities and redundancy has not yet taken place.

### Login

The login enables access to authentication-dependent functions of the mod store.

The authentication shall be made available long-term also for the integration of Datacentermods.com and further functions of the Greg Framework.

## 8. Global status bar

The status bar at the bottom of the window shall serve as a central information surface for the entire mod manager.

It replaces the former `Problems` main area and presents relevant status messages from all application areas.

This includes among other:

- Errors and warnings.
- Installation and validation issues.
- Mod and dependency conflicts.
- Build and upload status.
- Synchronization issues.
- Publishing status.
- Relevant messages from the Greg Framework and Datacentermods.com.

Messages must be prioritized, categorized and, if necessary, assigned to a specific mod or project.

A click on a message should, if possible, lead directly to the affected component or the corresponding error diagnosis.

**Status (2026-10-29):** The status bar is implemented and visible at the bottom of the window. It includes status LEDs for Steam, gregAPI and MelonLoader plus a version display. Problems are currently still resolved via the sub-section bar of the mod manager area; an automatic error message in the status bar is open.

## 9. Cross-cutting architecture and UX requirements

The revision shall not merely shift existing functions between tabs. The goal is a consistent information architecture for the entire mod ecosystem.

The following requirements apply:

- **Clear responsibilities:** Each function has a unique primary use.
- **No redundant areas:** Functions such as subscribed mods and favorites are not implemented independently of each other anymore.
- **Cross-platform support:** Windows, Linux and macOS must be considered.
- **Scalability:** The manageability must be retained even with very many mods and projects.
- **Unified data models:** Mods, plugins, libraries, projects and releases must be managed consistently.
- **Traceable status messages:** Errors and warnings must be understandable and context-related.
- **Modular implementation:** Existing Rust crates and compatible libraries should be preferred, provided that they meet the requirements.
- **Extensibility:** New download sources, publishing platforms and Greg Framework integrations must be addable later without restructuring the architecture.

## 10. Work order

Analyze the existing Rust/Slint application and develop a complete concept for the future information architecture and functional structure based on these requirements.

Proceed in the following order:

1. Analyze the existing tabs, sub-pages, actions and data models.
2. Identify redundant, misplaced, incomplete or not clearly defined functions.
3. Develop a consistent target structure for navigation, sub-pages, top bar and status bar.
4. Define the factual differences between mods, plugins, libraries, modpacks, collections and mod projects.
5. Describe the necessary data flows between mod manager, workshop/development area, mod store and settings.
6. Define requirements for cross-platform path detection, download sources, version comparison and start logic.
7. Check which existing implementations and Rust crates can be reused.
8. Create a prioritized implementation plan that gradually improves existing functions without unnecessary new developments or redundant implementations.

**Important:** Clearly distinguish between existing functions, planned functions and still open concept decisions. Missing details should first be analyzed and decided on a factual basis, rather than treating unverified assumptions as existing requirements.

## 11. Current implementation status

**Status: 2026-10-29 · Version 1.7.7**

This concept describes the target architecture and functional requirements. The following overview summarizes the implementation status of the current Rust/Slint application and serves as a reference for aligning concept, plan and implementation.

### Implemented

- **Shell:** Slint desktop app (`crates/greg-app`) with frameless custom chrome (title bar, window move area, window controls), icon-based left navigation bar, global top bar logic and global status bar with status LEDs.
- **Game launch:** Supervised start (`Start Game` / `WithMods`) and stop via managed process instances; vanilla mode with temporary deactivation of mod assemblies; MelonLoader gating for mod start.
- **Workshop:** Native Steamworks integration (`ISteamUGC`) including web API fallback, preview images, change notes and rate limiter; projects including editor with readiness checks, publish to Steam and mod store.
- **Modstore:** Browse, updates and installations (including multi-selection and mass install); mod store publish via `gregcli publish --to modstore`; `modstore_id` persistence.
- **Report:** Universal Report Assistant with categories, Markdown preview, export, attachments, log collection and env-gated GitHub issue creation including local export fallback.
- **Localization:** Complete internationalization via `greg-core::l10n` (English plus nine further languages) including Slint `Tr` global and live switching.
- **Design:** Design system from `DESIGN.md` (tokens, Lucide icons, dialogs, buttons, lists); UI redesign based on the shared components.

### Open / still open

- Webapp catalog parity: The desktop currently primarily uses the Steam Workshop as a source; full catalog parity with Datacentermods.com including direct installations is open.
- Automatic conflict resolution, load order automation, profiles and lock files, in-app updater (according to `README.md` not yet released).
- Discord/GitHub login integration in the desktop not complete.
- macOS: Native Mac validation, signing and notarization (according to `CHANGELOG.md` experimental).

### Sources of the current status

- `CHANGELOG.md` under `[Unreleased]` and release tagging via `VERSION`.
- `README.md` (features, compatibility, not yet released features).
- `docs/` (platforms, CI/CD, troubleshooting, roadmap).
- `desktop-gap-matrix.md` (desktop/web app parity status).

## 12. Planned update mechanism

**Status: 2026-10-29:** An in-app update mechanism is not yet implemented; the following points are to be understood as concept, planning and TODO basis.

### Goal

The application shall enable users to update to new versions via GitHub Releases – manually and optionally automatically.

### Sources

- GitHub Releases of the repository `mleem97/gregModmanager` (tags `v*`).
- Platform artifacts: Windows (portable ZIP, Inno Setup), Linux (portable tar.gz, DEB/RPM/APK/Arch), macOS (experimental).

### Planned procedure

1. Compare local version information with the latest GitHub Release.
2. Select the appropriate release asset for the installed platform.
3. Validate integrity (checksums/SHA-256) before installation.
4. Perform installation including backup of the previous version.
5. Restart the application after successful update.

### Security and quality requirements

- Checksum validation before every update download.
- Separation of stable and beta/pre-release channels.
- No updates to read-only system paths.
- Update source exclusively official GitHub Releases.

### Open points

- Final artifact format and checksum procedure per platform.
- Link release pipeline with GitHub Releases.
- Provide UI for manual and automatic update.

# Extensions: Authentication, Git integration and centralized bug reporting

## 1. Authentication and account integrations

Three authentication methods shall be integrated:

- **Discord Login:** Primary for bug reports, error messages and feature requests.
- **GitHub Login:** For Git imports and creating issues on behalf of the authenticated user. Long-term, GitHub login shall also be usable as an authentication method for Datacentermods.com.
- **Datacentermods Git Login:** For Git imports and creating issues on behalf of the authenticated user. Long-term, this authentication shall also be made available for Datacentermods.com.

The authentication methods shall be modularly built so that they can be extended independently of each other and used in other applications.

## 2. Centralized bug reporting and issue creation

Bug reports, error messages and feature requests shall preferably be created directly as issues in the respective Git remote.

The following priority applies:

1. **Authenticated user:** When possible, the Git integration of the user will be used to create the issue in the user's name and with the user's permissions.
2. **Not authenticated or insufficiently authorized user:** A central, yet to be implemented GitHub App, GitHub Action or comparable workflow takes over issue creation via an authorized service account.

The implementation must ensure that issues are assigned to the correct repository and mod. Credentials and tokens must never be passed on to clients or stored insecurely.

## 3. Central report tab in the mod manager

The mod manager receives a central **report tab** through which end users can report errors, submit improvement suggestions and create feature requests.

The reports shall not be stored in isolation but shall be integrated directly into the existing development and maintenance workflow.

The integration includes:

- Automatic assignment of reports to mods and Git repositories.
- Creation of issues in the respective Git remotes.
- Support for bug reports, feature requests and general feedback.
- Tracking of processing status and associated issues.
- Provision of the reports for further processing by the Greg Framework, gregLint and the VS Code extension.

## 4. Integration in gregLint, VS Code extension and Datacentermods.com

The report system must work with the existing and planned components of the Greg Framework.

- **gregLint:** Provision and processing of relevant reports for mods.
- **VS Code extension:** Display, creation and editing of reports directly in the development environment.
- **Datacentermods.com:** Central synchronization and assignment of reports to the respective mods.
- **Mod manager:** Display of associated reports and available processing status for the end user.

Datacentermods.com shall serve as the central interface through which reports are synchronized between the mod manager, VS Code extension and the respective Git repositories.

The synchronization must recognize which mods have open or new reports and make this information available to the respective applications.

## 5. AI-assisted report assistant

The report tab shall receive an AI assistant that supports end users in creating meaningful bug reports and feature requests.

The assistant shall:

- Accept and structure user input.
- Create traceable reports from incomplete error descriptions.
- Take relevant project information and available error data into account.
- Assign reports automatically to the appropriate mod and repository if the required information is available.
- Make the final report reviewable by the user before submission.

In addition, developers shall be able to request a summary of the current project status. For this, the assistant must have access to the currently released project information.

## 6. Support for external AI agents and APIs

Users shall be able to connect their own AI models and agents that are compatible with the OpenAI API or the Anthropic Claude API.

The integration shall enable at least the following:

- Configuration of own API endpoints and API keys.
- Support for compatible OpenAI API endpoints as well as the Claude API.
- Creation of project summaries based on the available project information.
- Support in the analysis and structuring of error messages.
- Generation of complete bug reports directly on the end device of the user.

The architecture shall support different providers and models via a shared interface without binding the application to a single provider.

## 7. Local report creation and vision support

End users shall be able to prepare and create reports fully locally, insofar as the required project and error data are available locally.

The report creation must fundamentally work in a **text-based** manner. Image analysis shall be optionally supported if the configured model provides vision capabilities.

Accordingly, the following requirements result:

- Text-based report creation as standard.
- Optional analysis of screenshots and images with vision-capable models.
- No mandatory dependency on vision models.
- Clear separation between local report creation, external AI processing and subsequent delivery to the Git remote.

## 8. Architecture guidelines

All functions shall be implemented modularly, extensibly and reusably.

Existing npm packages and established integrations should be preferred, insofar as they are compatible with the existing stack and fully cover the requirements.

New custom developments are to be limited to functions for which no suitable existing solution is available.

The implementation shall treat authentication, Git providers, report processing, AI binding and synchronization as clearly separated components. This shall enable future extensions – especially the integration into Datacentermods.com – without major changes to the existing architecture.
