//! Keep a Changelog parser plus Semantic Versioning validation.
//!
//! Lets the manager read the newest changelog entry for an update without
//! manual input. Port of `KeepAChangelogParser`.

use std::cmp::Ordering;

use crate::error::{CoreError, Result};

/// One `## [x.y.z]` section (or `## [Unreleased]`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangelogEntry {
    /// Normalized version or `"Unreleased"`.
    pub version: String,
    /// Trailing date text, if present.
    pub date: Option<String>,
    /// Section body without the heading and link definitions.
    pub body: String,
}

/// Strict SemVer 2.0.0 check. A single leading `v` is accepted and ignored.
pub fn is_valid_sem_version(version: &str) -> bool {
    match normalize_version(version) {
        Some(v) => semver::Version::parse(&v).is_ok(),
        None => false,
    }
}

/// Trims whitespace and a single leading `v`. Returns `None` when empty.
pub fn normalize_version(version: &str) -> Option<String> {
    let v = version.trim();
    let v = if v.len() > 1 && (v.starts_with('v') || v.starts_with('V')) {
        v[1..].trim()
    } else {
        v
    };
    if v.is_empty() {
        None
    } else {
        Some(v.to_string())
    }
}

/// Parses all `##` sections. Errors on empty input, missing sections, or
/// duplicate versions.
pub fn parse_entries(markdown: &str) -> Result<Vec<ChangelogEntry>> {
    if markdown.trim().is_empty() {
        return Err(CoreError::Changelog("Changelog is empty.".into()));
    }
    let mut entries: Vec<ChangelogEntry> = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    let mut current: Option<(String, Option<String>)> = None;
    let mut body: Vec<String> = Vec::new();

    let flush = |current: &mut Option<(String, Option<String>)>,
                 body: &mut Vec<String>,
                 entries: &mut Vec<ChangelogEntry>| {
        if let Some((version, date)) = current.take() {
            let text = strip_link_definitions(&body.join("\n"));
            entries.push(ChangelogEntry {
                version,
                date,
                body: text,
            });
            body.clear();
        }
    };

    for raw_line in markdown.lines() {
        if let Some((token, date)) = parse_section_heading(raw_line) {
            flush(&mut current, &mut body, &mut entries);
            if token.eq_ignore_ascii_case("Unreleased") {
                current = Some(("Unreleased".to_string(), None));
            } else if let Some(normalized) = normalize_version(&token) {
                if semver::Version::parse(&normalized).is_err() {
                    // Not a version section — skip until the next `##`.
                    current = None;
                    continue;
                }
                if seen.iter().any(|v| v.eq_ignore_ascii_case(&normalized)) {
                    return Err(CoreError::Changelog(format!(
                        "Duplicate changelog section for version {normalized}."
                    )));
                }
                seen.push(normalized.clone());
                current = Some((normalized, date));
            } else {
                current = None;
            }
            continue;
        }
        if current.is_some() {
            body.push(raw_line.to_string());
        }
    }
    flush(&mut current, &mut body, &mut entries);

    if entries.is_empty() {
        return Err(CoreError::Changelog(
            "No '## [x.y.z]' or '## [Unreleased]' section found (Keep a Changelog format).".into(),
        ));
    }
    Ok(entries)
}

/// Section body for an exact SemVer version.
/// Falls back to `[Unreleased]` when it names that version.
pub fn entry_for_version(markdown: &str, version: &str) -> Option<ChangelogEntry> {
    let wanted = normalize_version(version)?;
    let entries = parse_entries(markdown).ok()?;
    for entry in &entries {
        if entry.version.eq_ignore_ascii_case(&wanted) && !entry.body.trim().is_empty() {
            return Some(entry.clone());
        }
    }
    let unreleased = entries.iter().find(|e| {
        e.version == "Unreleased" && !e.body.trim().is_empty() && mentions_version(&e.body, &wanted)
    });
    unreleased.cloned()
}

/// Newest versioned section by SemVer precedence (ignores Unreleased/empty).
pub fn latest_release_entry(markdown: &str) -> Option<ChangelogEntry> {
    let entries = parse_entries(markdown).ok()?;
    entries
        .into_iter()
        .filter(|e| e.version != "Unreleased" && !e.body.trim().is_empty())
        .max_by(|a, b| compare_semver(&a.version, &b.version))
}

/// The `[Unreleased]` section, if it has content.
pub fn unreleased_entry(markdown: &str) -> Option<ChangelogEntry> {
    let entries = parse_entries(markdown).ok()?;
    entries
        .into_iter()
        .find(|e| e.version == "Unreleased" && !e.body.trim().is_empty())
}

/// Compares two (optionally `v`-prefixed) SemVer strings. Release > prerelease.
pub fn compare_semver(a: &str, b: &str) -> Ordering {
    let na = normalize_version(a).unwrap_or_default();
    let nb = normalize_version(b).unwrap_or_default();
    match (semver::Version::parse(&na), semver::Version::parse(&nb)) {
        (Ok(pa), Ok(pb)) => pa.cmp(&pb),
        _ => na.cmp(&nb),
    }
}

/// Parses a `## …` heading into `(token, date)`. Returns `None` otherwise.
fn parse_section_heading(line: &str) -> Option<(String, Option<String>)> {
    let trimmed = line.trim();
    let rest = trimmed.strip_prefix("##")?;
    // Exactly level 2: `###` must not match.
    if rest.starts_with('#') {
        return None;
    }
    let rest = rest.trim_start();
    if rest.is_empty() {
        return None;
    }
    let (token, after) = if let Some(inner) = rest.strip_prefix('[') {
        let end = inner.find(']')?;
        (inner[..end].trim().to_string(), inner[end + 1..].trim())
    } else {
        let mut parts = rest.splitn(2, char::is_whitespace);
        let token = parts.next().unwrap_or("").to_string();
        let after = parts.next().unwrap_or("").trim();
        (token, after)
    };
    if token.is_empty() {
        return None;
    }
    let date = after
        .strip_prefix('-')
        .map(|d| d.trim().to_string())
        .filter(|d| !d.is_empty());
    Some((token, date))
}

/// Removes trailing `[label]: url` link definitions from a section body.
fn strip_link_definitions(body: &str) -> String {
    body.lines()
        .filter(|line| !is_link_definition(line))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

fn is_link_definition(line: &str) -> bool {
    let t = line.trim();
    if !t.starts_with('[') {
        return false;
    }
    match t.find("]:") {
        Some(i) => t[i + 2..].trim_start().starts_with("http"),
        None => false,
    }
}

fn mentions_version(body: &str, version: &str) -> bool {
    let lower = body.to_lowercase();
    let want = version.to_lowercase();
    lower.contains(&want) || lower.contains(&format!("v{want}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "# Changelog\n\nThe format is based on Keep a Changelog.\n\n## [Unreleased]\n\n### Added\n\n- Planned feature.\n\n## [1.1.0] - 2026-09-27\n\n### Fixed\n\n- Fixed crash on startup.\n\n## [1.0.0] - 2026-09-01\n\n### Added\n\n- Initial release.\n\n[1.1.0]: https://example.com/compare/1.0.0...1.1.0\n[1.0.0]: https://example.com/releases/1.0.0\n";

    #[test]
    fn validates_semver() {
        for v in ["1.0.0", "v1.0.0", "1.2.3-beta.1", "1.0.0+build.5", "0.0.1"] {
            assert!(is_valid_sem_version(v), "{v}");
        }
        for v in ["", "1.0", "1.0.0.0", "01.0.0", "latest"] {
            assert!(!is_valid_sem_version(v), "{v}");
        }
    }

    #[test]
    fn parses_all_sections() {
        let entries = parse_entries(SAMPLE).unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].version, "Unreleased");
        assert_eq!(entries[1].version, "1.1.0");
        assert_eq!(entries[1].date.as_deref(), Some("2026-09-27"));
        assert_eq!(entries[2].version, "1.0.0");
        assert!(!entries[2].body.contains("example.com"));
        assert!(entries[2].body.contains("Initial release"));
    }

    #[test]
    fn finds_exact_and_v_prefixed() {
        let e = entry_for_version(SAMPLE, "1.1.0").unwrap();
        assert!(e.body.contains("crash"));
        let e = entry_for_version(SAMPLE, "v1.0.0").unwrap();
        assert_eq!(e.version, "1.0.0");
        assert!(entry_for_version(SAMPLE, "9.9.9").is_none());
    }

    #[test]
    fn picks_latest_release() {
        let e = latest_release_entry(SAMPLE).unwrap();
        assert_eq!(e.version, "1.1.0");
    }

    #[test]
    fn rejects_duplicates_and_empty() {
        let dup = format!("{SAMPLE}\n## [1.0.0] - 2026-10-01\n\n- Dup.\n");
        assert!(parse_entries(&dup).is_err());
        assert!(parse_entries("# Just a readme\n").is_err());
        assert!(parse_entries("   ").is_err());
    }

    #[test]
    fn orders_releases_above_prereleases() {
        assert_eq!(compare_semver("1.0.0", "1.0.0-beta.1"), Ordering::Greater);
        assert_eq!(compare_semver("1.0.1", "1.0.0"), Ordering::Greater);
        assert_eq!(compare_semver("v2.0.0", "2.0.0"), Ordering::Equal);
    }
}
