//! Project documentation handling: changelogs and descriptions.

pub mod bbcode;
pub mod changelog;
pub mod project_docs;

pub use bbcode::convert as markdown_to_bbcode;
pub use changelog::{
    compare_semver, entry_for_version, is_valid_sem_version, latest_release_entry,
    normalize_version, parse_entries, unreleased_entry, ChangelogEntry,
};
pub use project_docs::{
    build_effective_steam_description, find_changelog, find_readme, format_steam_change_note,
    resolve_modstore_changelog, resolve_modstore_description, resolve_steam_changelog,
    resolve_steam_description, ChangelogSource, ResolvedChangelog, ResolvedDescription,
    MAX_DOCS_FILE_BYTES,
};
