//! File-based project docs.
//!
//! `README.md` is the description source, `CHANGELOG.md` (Keep a Changelog +
//! SemVer) the changelog source. Steam gets converted BBCode/plain text, the
//! Modstore keeps native Markdown. Port of `ProjectDocsResolver`.

use std::path::{Path, PathBuf};

use crate::docs::{bbcode, changelog};
use crate::l10n;
use crate::models::WorkshopMetadata;

/// Maximum accepted docs file size (matches the Steam preview cap logic).
pub const MAX_DOCS_FILE_BYTES: u64 = 400_000;

/// Where a resolved changelog came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangelogSource {
    /// Nothing usable found.
    None,
    /// Explicit manual text.
    Manual,
    /// `CHANGELOG.md` section for the requested version.
    FileVersion,
    /// `CHANGELOG.md` `[Unreleased]` section.
    FileUnreleased,
}

/// Resolved description text.
#[derive(Debug, Clone)]
pub struct ResolvedDescription {
    /// Effective text (BBCode for Steam, Markdown for the Modstore).
    pub text: String,
    /// True when read from `README.md`.
    pub from_file: bool,
    /// Source file, if any.
    pub file_path: Option<PathBuf>,
}

/// Resolved changelog text.
#[derive(Debug, Clone)]
pub struct ResolvedChangelog {
    /// Effective text.
    pub text: String,
    /// Where it came from.
    pub source: ChangelogSource,
    /// Source file, if any.
    pub file_path: Option<PathBuf>,
    /// Section version (`"Unreleased"` possible).
    pub entry_version: Option<String>,
}

const README_CANDIDATES: &[&str] = &[
    "README.md",
    "readme.md",
    "README.markdown",
    "readme.markdown",
];
const CHANGELOG_CANDIDATES: &[&str] = &[
    "CHANGELOG.md",
    "changelog.md",
    "CHANGELOG.markdown",
    "changelog.markdown",
];

fn find_first(project_root: &Path, candidates: &[&str]) -> Option<PathBuf> {
    if !project_root.is_dir() {
        return None;
    }
    candidates
        .iter()
        .map(|name| project_root.join(name))
        .find(|p| p.is_file())
}

/// Finds `README.md` in the project root (case variants included).
pub fn find_readme(project_root: &Path) -> Option<PathBuf> {
    find_first(project_root, README_CANDIDATES)
}

/// Finds `CHANGELOG.md` in the project root (case variants included).
pub fn find_changelog(project_root: &Path) -> Option<PathBuf> {
    find_first(project_root, CHANGELOG_CANDIDATES)
}

fn read_docs_file(path: &Path) -> Option<String> {
    let meta = std::fs::metadata(path).ok()?;
    if meta.len() == 0 || meta.len() > MAX_DOCS_FILE_BYTES {
        return None;
    }
    std::fs::read_to_string(path).ok()
}

/// Steam description: `README.md` converted to BBCode when present,
/// otherwise the metadata fallback.
pub fn resolve_steam_description(project_root: &Path, fallback: &str) -> ResolvedDescription {
    match find_readme(project_root).and_then(|p| read_docs_file(&p).map(|md| (p, md))) {
        Some((path, markdown)) => {
            let bbcode = bbcode::convert(&markdown);
            if bbcode.trim().is_empty() {
                ResolvedDescription {
                    text: fallback.trim().to_string(),
                    from_file: false,
                    file_path: None,
                }
            } else {
                ResolvedDescription {
                    text: bbcode.trim().to_string(),
                    from_file: true,
                    file_path: Some(path),
                }
            }
        }
        None => ResolvedDescription {
            text: fallback.trim().to_string(),
            from_file: false,
            file_path: None,
        },
    }
}

/// Modstore description: native Markdown, no BBCode conversion.
pub fn resolve_modstore_description(project_root: &Path, fallback: &str) -> ResolvedDescription {
    match find_readme(project_root).and_then(|p| read_docs_file(&p).map(|md| (p, md))) {
        Some((path, markdown)) => {
            let text = markdown.trim().to_string();
            if text.is_empty() {
                ResolvedDescription {
                    text: fallback.trim().to_string(),
                    from_file: false,
                    file_path: None,
                }
            } else {
                ResolvedDescription {
                    text,
                    from_file: true,
                    file_path: Some(path),
                }
            }
        }
        None => ResolvedDescription {
            text: fallback.trim().to_string(),
            from_file: false,
            file_path: None,
        },
    }
}

/// Steam changelog priority: manual input > `CHANGELOG.md` section for
/// `version` > `[Unreleased]` section.
pub fn resolve_steam_changelog(
    project_root: &Path,
    version: &str,
    manual: Option<&str>,
) -> ResolvedChangelog {
    if let Some(text) = manual {
        if !text.trim().is_empty() {
            return ResolvedChangelog {
                text: text.trim().to_string(),
                source: ChangelogSource::Manual,
                file_path: None,
                entry_version: None,
            };
        }
    }
    let Some(path) = find_changelog(project_root) else {
        return ResolvedChangelog {
            text: String::new(),
            source: ChangelogSource::None,
            file_path: None,
            entry_version: None,
        };
    };
    let Some(markdown) = read_docs_file(&path) else {
        return ResolvedChangelog {
            text: String::new(),
            source: ChangelogSource::None,
            file_path: Some(path),
            entry_version: None,
        };
    };
    let wanted = changelog::normalize_version(version).unwrap_or_else(|| "1.0.0".into());
    if let Some(entry) = changelog::entry_for_version(&markdown, &wanted) {
        let source = if entry.version == "Unreleased" {
            ChangelogSource::FileUnreleased
        } else {
            ChangelogSource::FileVersion
        };
        return ResolvedChangelog {
            text: entry.body.trim().to_string(),
            source,
            file_path: Some(path),
            entry_version: Some(entry.version),
        };
    }
    if let Some(entry) = changelog::unreleased_entry(&markdown) {
        return ResolvedChangelog {
            text: entry.body.trim().to_string(),
            source: ChangelogSource::FileUnreleased,
            file_path: Some(path),
            entry_version: Some(entry.version),
        };
    }
    ResolvedChangelog {
        text: String::new(),
        source: ChangelogSource::None,
        file_path: Some(path),
        entry_version: None,
    }
}

/// Modstore changelog: same resolution, raw Markdown body (no BBCode).
pub fn resolve_modstore_changelog(
    project_root: &Path,
    version: &str,
    manual: Option<&str>,
) -> ResolvedChangelog {
    resolve_steam_changelog(project_root, version, manual)
}

/// Prefixes the Steam change note with the version unless already present.
pub fn format_steam_change_note(version: &str, body: &str) -> String {
    let v = {
        let t = version.trim();
        if t.is_empty() {
            "1.0.0".to_string()
        } else {
            t.to_string()
        }
    };
    let b = body.trim();
    if b.is_empty() {
        return format!("v{v}");
    }
    let lower = b.to_lowercase();
    if lower.starts_with(&format!("v{}", v.to_lowercase())) || lower.starts_with(&v.to_lowercase())
    {
        return b.to_string();
    }
    format!("v{v}: {b}")
}

/// Effective Steam description including auto-appended requirement notices.
pub fn build_effective_steam_description(
    lang: &str,
    project_root: &Path,
    meta: &WorkshopMetadata,
) -> String {
    let mut desc = resolve_steam_description(project_root, &meta.description).text;
    if meta.needs_melon_loader && !desc.to_lowercase().contains("melonloader") {
        desc.push_str("\n\n---\n");
        desc.push_str(&l10n::get(lang, "Editor_MelonLoaderNotice"));
    }
    if meta.needsgreg && !desc.to_lowercase().contains("gregcoremodframework") {
        desc.push_str("\n\n---\n");
        desc.push_str(&l10n::get(lang, "Editor_gregNotice"));
    }
    desc.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("greg-docs-test-{nanos}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn readme_prefers_bbcode_for_steam() {
        let root = temp_root();
        std::fs::write(root.join("README.md"), "# My Mod\n\n**Bold** feature.").unwrap();
        let resolved = resolve_steam_description(&root, "fallback");
        assert!(resolved.from_file);
        assert!(
            resolved.text.contains("[h1]My Mod[/h1]"),
            "{}",
            resolved.text
        );
        assert!(resolved.text.contains("[b]Bold[/b]"), "{}", resolved.text);
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn modstore_keeps_markdown() {
        let root = temp_root();
        std::fs::write(root.join("README.md"), "# My Mod\n\n**Bold** feature.").unwrap();
        let resolved = resolve_modstore_description(&root, "fallback");
        assert!(resolved.from_file);
        assert!(resolved.text.contains("**Bold**"));
        assert!(!resolved.text.contains("[b]"));
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn changelog_resolution_order() {
        let root = temp_root();
        std::fs::write(
            root.join("CHANGELOG.md"),
            "# Changelog\n\n## [1.1.0] - 2026-09-27\n\n### Fixed\n\n- From file.\n",
        )
        .unwrap();
        let manual = resolve_steam_changelog(&root, "1.1.0", Some("Manual note"));
        assert_eq!(manual.source, ChangelogSource::Manual);
        let file = resolve_steam_changelog(&root, "1.1.0", None);
        assert_eq!(file.source, ChangelogSource::FileVersion);
        assert!(file.text.contains("From file"));
        let missing = resolve_steam_changelog(&root, "1.1.0", None);
        let _ = missing;
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn formats_change_notes() {
        assert_eq!(
            format_steam_change_note("1.1.0", "Fixed X"),
            "v1.1.0: Fixed X"
        );
        assert_eq!(
            format_steam_change_note("1.1.0", "v1.1.0: Fixed X"),
            "v1.1.0: Fixed X"
        );
        assert_eq!(format_steam_change_note("1.0.0", ""), "v1.0.0");
    }
}
