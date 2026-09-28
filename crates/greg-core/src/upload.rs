//! Upload-readiness gate (`UploadDependencyChecker` port, file-aware).
//!
//! `README.md` / `CHANGELOG.md` in the project root are honored exactly like
//! the publisher uses them, so checks never disagree with an upload.

use std::path::Path;

use crate::docs::{self, project_docs};
use crate::limits::{MAX_DESCRIPTION_LENGTH, MAX_PREVIEW_IMAGE_BYTES, MAX_TITLE_LENGTH};
use crate::models::{UploadCheckResult, WorkshopMetadata};

/// Runs all readiness checks for a project.
/// Includes the Security-Pipeline preflight (secrets, executables, symlinks,
/// size hard-limit) so the desktop never uploads what the web pipeline would
/// reject.
pub fn check(
    lang: &str,
    project_root: &Path,
    meta: &WorkshopMetadata,
    manual_changelog: Option<&str>,
) -> Vec<UploadCheckResult> {
    let _ = lang;
    let mut results = Vec::new();
    check_content_folder(project_root, &mut results);
    if project_root.join("content").is_dir() {
        check_native_config(project_root, &mut results);
    }
    check_metadata_fields(meta, &mut results);
    check_version(meta, &mut results);
    check_project_docs(project_root, meta, &mut results);
    check_preview_image(project_root, meta, &mut results);
    check_tags(meta, &mut results);
    check_content_size(project_root, &mut results);
    check_security_preflight(project_root, &mut results);
    check_changelog(project_root, meta, manual_changelog, &mut results);
    check_greg_dependency(project_root, meta, &mut results);
    results
}

fn check_content_folder(project_root: &Path, results: &mut Vec<UploadCheckResult>) {
    let content = project_root.join("content");
    if !content.is_dir() {
        results.push(UploadCheckResult::error(
            "Content folder",
            "content/ folder is missing. Create it and add your files before uploading.",
        ));
        return;
    }
    let count = walkdir_count(&content);
    if count == 0 {
        results.push(UploadCheckResult::error(
            "Content folder",
            "content/ folder is empty. Add files to upload.",
        ));
    } else {
        results.push(UploadCheckResult::ok(
            "Content folder",
            format!("content/ contains {count} file(s)."),
        ));
    }
}

fn walkdir_count(dir: &Path) -> usize {
    let mut n = 0;
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&d) else {
            continue;
        };
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                stack.push(p);
            } else {
                n += 1;
            }
        }
    }
    n
}

fn check_native_config(project_root: &Path, results: &mut Vec<UploadCheckResult>) {
    if project_root.join("content").join("config.json").is_file() {
        results.push(UploadCheckResult::ok(
            "config.json",
            "Native mod definition present under content/.",
        ));
    } else {
        results.push(UploadCheckResult::warning(
            "config.json",
            "content/config.json is missing. It is required for native shop/static items when you ship vanilla assets.",
        ));
    }
}

fn check_metadata_fields(meta: &WorkshopMetadata, results: &mut Vec<UploadCheckResult>) {
    if meta.title.trim().is_empty() {
        results.push(UploadCheckResult::error(
            "Title",
            "Title is empty. Steam requires a title.",
        ));
    } else if meta.title.chars().count() > MAX_TITLE_LENGTH {
        results.push(UploadCheckResult::error(
            "Title",
            format!(
                "Title exceeds {MAX_TITLE_LENGTH} characters ({}).",
                meta.title.chars().count()
            ),
        ));
    } else {
        results.push(UploadCheckResult::ok(
            "Title",
            format!(
                "\"{}\" ({}/{MAX_TITLE_LENGTH})",
                meta.title,
                meta.title.chars().count()
            ),
        ));
    }

    if meta.description.trim().is_empty() {
        results.push(UploadCheckResult::warning(
            "Description",
            "Description is empty. Recommended for discoverability.",
        ));
    } else if meta.description.chars().count() > MAX_DESCRIPTION_LENGTH {
        results.push(UploadCheckResult::error(
            "Description",
            format!("Description exceeds {MAX_DESCRIPTION_LENGTH} characters."),
        ));
    } else {
        results.push(UploadCheckResult::ok(
            "Description",
            format!(
                "{}/{} characters.",
                meta.description.chars().count(),
                MAX_DESCRIPTION_LENGTH
            ),
        ));
    }

    match meta.visibility.as_str() {
        "Public" | "FriendsOnly" | "Private" => {
            results.push(UploadCheckResult::ok("Visibility", meta.visibility.clone()))
        }
        other => results.push(UploadCheckResult::warning(
            "Visibility",
            format!("Unknown visibility \"{other}\". Expected: Public, FriendsOnly, or Private."),
        )),
    }
}

fn check_version(meta: &WorkshopMetadata, results: &mut Vec<UploadCheckResult>) {
    let version = meta.version.trim();
    if version.is_empty() {
        results.push(UploadCheckResult::error(
            "Version",
            "Version is empty. Use Semantic Versioning (e.g. 1.0.0).",
        ));
        return;
    }
    if !docs::is_valid_sem_version(version) {
        results.push(UploadCheckResult::error(
            "Version",
            format!("\"{version}\" is not valid Semantic Versioning (expected MAJOR.MINOR.PATCH)."),
        ));
        return;
    }
    results.push(UploadCheckResult::ok(
        "Version",
        format!(
            "v{} (SemVer).",
            docs::normalize_version(version).unwrap_or_default()
        ),
    ));
}

fn check_project_docs(
    project_root: &Path,
    meta: &WorkshopMetadata,
    results: &mut Vec<UploadCheckResult>,
) {
    if let Some(path) = project_docs::find_readme(project_root) {
        let converted =
            project_docs::resolve_steam_description(project_root, &meta.description).text;
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("README.md");
        if converted.chars().count() > MAX_DESCRIPTION_LENGTH {
            results.push(UploadCheckResult::error(
                "README.md",
                format!(
                    "{name} found — Steam uses converted BBCode, but it exceeds {MAX_DESCRIPTION_LENGTH} characters. Modstore keeps native Markdown."
                ),
            ));
        } else {
            results.push(UploadCheckResult::ok(
                "README.md",
                format!(
                    "{name} found — Steam uses converted BBCode ({}/{}). Modstore keeps native Markdown.",
                    converted.chars().count(),
                    MAX_DESCRIPTION_LENGTH
                ),
            ));
        }
    } else {
        results.push(UploadCheckResult::warning(
            "README.md",
            "No README.md in the project root — Steam uses the metadata description. Add README.md (Markdown) to share one source with the Modstore.",
        ));
    }

    let Some(path) = project_docs::find_changelog(project_root) else {
        return;
    };
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("CHANGELOG.md");
    let markdown = match std::fs::read_to_string(&path) {
        Ok(m) => m,
        Err(_) => {
            results.push(UploadCheckResult::warning(
                "CHANGELOG.md",
                format!("Found {name} but could not read it."),
            ));
            return;
        }
    };
    match docs::parse_entries(&markdown) {
        Err(e) => results.push(UploadCheckResult::error(
            "CHANGELOG.md",
            format!("{name} is not Keep a Changelog format: {e}"),
        )),
        Ok(_) => {
            let version = docs::normalize_version(&meta.version).unwrap_or_else(|| "1.0.0".into());
            match docs::entry_for_version(&markdown, &version) {
                Some(entry) => {
                    let from = if entry.version == "Unreleased" {
                        "[Unreleased] (mentions this version)".to_string()
                    } else {
                        format!("[{}]", entry.version)
                    };
                    results.push(UploadCheckResult::ok(
                        "CHANGELOG.md",
                        format!("{name} {from} will be used for v{version} — no manual input needed."),
                    ));
                }
                None => results.push(UploadCheckResult::warning(
                    "CHANGELOG.md",
                    format!("{name} has no section for v{version}. Add '## [{version}] - YYYY-MM-DD' (Keep a Changelog) or enter notes manually."),
                )),
            }
        }
    }
}

fn check_preview_image(
    project_root: &Path,
    meta: &WorkshopMetadata,
    results: &mut Vec<UploadCheckResult>,
) {
    if meta.preview_image_relative_path.trim().is_empty() {
        results.push(UploadCheckResult::error(
            "Preview image",
            "A preview image is required for every Workshop upload and update.",
        ));
        return;
    }
    let path = project_root.join(&meta.preview_image_relative_path);
    match std::fs::metadata(&path) {
        Err(_) => results.push(UploadCheckResult::error(
            "Preview image",
            format!(
                "File not found: {}. Select a preview image before uploading.",
                meta.preview_image_relative_path
            ),
        )),
        Ok(m) => {
            let size = m.len() as i64;
            if size > MAX_PREVIEW_IMAGE_BYTES {
                results.push(UploadCheckResult::error(
                    "Preview image",
                    format!(
                        "Preview image is {} ({} bytes). Steam allows at most 1 MiB.",
                        crate::util::format_bytes(size),
                        size
                    ),
                ));
            } else {
                results.push(UploadCheckResult::ok(
                    "Preview image",
                    format!(
                        "{} ({})",
                        meta.preview_image_relative_path,
                        crate::util::format_bytes(size)
                    ),
                ));
            }
        }
    }
}

fn check_tags(meta: &WorkshopMetadata, results: &mut Vec<UploadCheckResult>) {
    if meta.tags.is_empty() {
        results.push(UploadCheckResult::warning(
            "Tags",
            "No tags set. Tags help users find your content.",
        ));
    } else {
        results.push(UploadCheckResult::ok("Tags", meta.tags.join(", ")));
    }
}

fn check_content_size(project_root: &Path, results: &mut Vec<UploadCheckResult>) {
    let content = project_root.join("content");
    if !content.is_dir() {
        return;
    }
    let mut total: i64 = 0;
    let mut stack = vec![content];
    while let Some(d) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&d) else {
            continue;
        };
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                stack.push(p);
            } else if let Ok(m) = entry.metadata() {
                total += m.len() as i64;
            }
        }
    }
    const WARN_THRESHOLD: i64 = 100 * 1024 * 1024;
    // Hard limit mirrors `POST /api/upload-url` (MAX_MOD_BYTES = 500 MiB).
    const MAX_THRESHOLD: i64 = 500 * 1024 * 1024;
    if total > MAX_THRESHOLD {
        results.push(UploadCheckResult::error(
            "Content size",
            format!(
                "Total content size is {} — exceeds the 500 MiB upload limit. Remove files before uploading.",
                crate::util::format_bytes(total)
            ),
        ));
    } else if total > WARN_THRESHOLD {
        results.push(UploadCheckResult::warning(
            "Content size",
            format!(
                "Total content size is {}. Large uploads take longer.",
                crate::util::format_bytes(total)
            ),
        ));
    } else {
        results.push(UploadCheckResult::ok(
            "Content size",
            crate::util::format_bytes(total),
        ));
    }
}

/// Security-Pipeline preflight (mirrors the web Security-Pipeline UploadChecker).
///
/// Blocks what the server would reject or quarantine:
/// secrets/credentials, Windows/Unix executables outside the mod allowlist,
/// symlinks (zip-slip/jail-break risk) and `.git/` payloads.
fn check_security_preflight(project_root: &Path, results: &mut Vec<UploadCheckResult>) {
    const SECRET_FILES: &[&str] = &[
        ".env",
        ".env.local",
        ".env.production",
        "id_rsa",
        "id_ed25519",
        "credentials.json",
        "secrets.json",
    ];
    const SECRET_EXTS: &[&str] = &[".pem", ".key", ".pfx", ".p12", ".kdbx"];
    // Executables that must never ship inside mod content.
    // `.dll` stays allowed (native Data Center mods); everything that can
    // run standalone is blocked.
    const BLOCKED_EXTS: &[&str] = &[
        ".exe", ".msi", ".bat", ".cmd", ".com", ".scr", ".pif", ".vbs", ".vbe", ".jse", ".wsf",
        ".wsh", ".ps1", ".sh",
    ];

    let content = project_root.join("content");
    if !content.is_dir() {
        return;
    }
    let mut secrets: Vec<String> = Vec::new();
    let mut blocked: Vec<String> = Vec::new();
    let mut symlinks: Vec<String> = Vec::new();
    let mut git_payload = false;

    let mut stack = vec![content.clone()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let rel = path
                .strip_prefix(project_root)
                .map(|r| r.to_string_lossy().to_string())
                .unwrap_or_else(|_| path.to_string_lossy().to_string());
            if entry.file_type().map(|t| t.is_symlink()).unwrap_or(false) {
                symlinks.push(rel);
                continue;
            }
            if path.is_dir() {
                if path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n == ".git")
                    .unwrap_or(false)
                {
                    git_payload = true;
                }
                stack.push(path);
                continue;
            }
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_lowercase();
            let is_secret = SECRET_FILES.iter().any(|s| name == *s)
                || SECRET_EXTS.iter().any(|e| name.ends_with(e));
            if is_secret {
                secrets.push(rel.clone());
            }
            if BLOCKED_EXTS.iter().any(|e| name.ends_with(e)) {
                blocked.push(rel);
            }
        }
    }

    if secrets.is_empty() && blocked.is_empty() && symlinks.is_empty() && !git_payload {
        results.push(UploadCheckResult::ok(
            "Security preflight",
            "No secrets, executables or symlinks in content/.",
        ));
        return;
    }
    if !secrets.is_empty() {
        results.push(UploadCheckResult::error(
            "Security preflight",
            format!(
                "Remove credential files before uploading: {}.",
                secrets.iter().take(5).cloned().collect::<Vec<_>>().join(", ")
            ),
        ));
    }
    if !blocked.is_empty() {
        results.push(UploadCheckResult::error(
            "Security preflight",
            format!(
                "Blocked executables in content/ (ship .dll/.zip/.lua/.py/.go only): {}.",
                blocked.iter().take(5).cloned().collect::<Vec<_>>().join(", ")
            ),
        ));
    }
    if !symlinks.is_empty() {
        results.push(UploadCheckResult::error(
            "Security preflight",
            format!(
                "Symlinks are not uploaded (extraction risk): {}.",
                symlinks.iter().take(5).cloned().collect::<Vec<_>>().join(", ")
            ),
        ));
    }
    if git_payload {
        results.push(UploadCheckResult::warning(
            "Security preflight",
            "content/ contains .git/ — it will bloat the upload. Remove it or move the project root.",
        ));
    }
}

fn check_changelog(
    project_root: &Path,
    meta: &WorkshopMetadata,
    manual: Option<&str>,
    results: &mut Vec<UploadCheckResult>,
) {
    let is_first_publish = meta.published_file_id == 0;
    let resolved = project_docs::resolve_steam_changelog(project_root, &meta.version, manual);
    if resolved.text.trim().is_empty() {
        // Optional even for the first publish: Steam accepts empty change
        // notes, and maintainers upload work-in-progress snapshots.
        results.push(UploadCheckResult::warning(
            "Changelog",
            if is_first_publish {
                "No changelog provided. Recommended for the first release so subscribers know what this is."
            } else {
                "No changelog provided. Recommended so subscribers know what changed."
            },
        ));
        return;
    }
    let source = match resolved.source {
        project_docs::ChangelogSource::Manual => "manual input".to_string(),
        project_docs::ChangelogSource::FileVersion => format!(
            "CHANGELOG.md [{}]",
            resolved.entry_version.unwrap_or_default()
        ),
        project_docs::ChangelogSource::FileUnreleased => "CHANGELOG.md [Unreleased]".to_string(),
        project_docs::ChangelogSource::None => "manual input".to_string(),
    };
    results.push(UploadCheckResult::ok(
        "Changelog",
        format!("{} characters ({source}).", resolved.text.chars().count()),
    ));
}

fn check_greg_dependency(
    project_root: &Path,
    meta: &WorkshopMetadata,
    results: &mut Vec<UploadCheckResult>,
) {
    let desc = project_docs::resolve_steam_description(project_root, &meta.description).text;
    let desc_mentions = contains_greg_hint(&desc);
    let tags_suggest = meta.tags.iter().any(|t| tag_suggests_greg(t));
    if meta.needsgreg {
        if desc_mentions {
            results.push(UploadCheckResult::ok(
                "GregFramework (greg)",
                "Requires gregCoreModFramework / GregFramework — description mentions the framework.",
            ));
        } else {
            results.push(UploadCheckResult::warning(
                "GregFramework (greg)",
                "Marked as requiring gregCoreModFramework, but the description does not mention it yet. A standard notice is still appended automatically on upload if missing.",
            ));
        }
    } else if tags_suggest || desc_mentions {
        results.push(UploadCheckResult::warning(
            "GregFramework (greg)",
            "Tags or description suggest greg — enable \"Needs gregCoreModFramework\" if players must install GregFramework.",
        ));
    } else {
        results.push(UploadCheckResult::ok(
            "GregFramework (greg)",
            "Not flagged as requiring gregCoreModFramework (GregFramework).",
        ));
    }
}

fn tag_suggests_greg(tag: &str) -> bool {
    let t = tag.trim().to_lowercase();
    matches!(
        t.as_str(),
        "greg" | "gregcore-mod-framework" | "gregframework" | "gregcore" | "gregcoremodframework"
    ) || t.contains("gregcoremod")
        || t.contains("gregframework")
}

fn contains_greg_hint(text: &str) -> bool {
    let lower = text.to_lowercase();
    lower.contains("gregcoremodframework")
        || lower.contains("gregframework")
        || lower.contains("greg tools")
        || lower.contains("gregcore mod framework")
        || has_word_greg(&lower)
}

/// Case-insensitive whole-word `greg` (mirrors `\bgreg\b`).
fn has_word_greg(lower: &str) -> bool {
    let bytes = lower.as_bytes();
    let mut i = 0;
    while i + 4 <= bytes.len() {
        if &bytes[i..i + 4] == b"greg" {
            let before = if i == 0 { None } else { Some(bytes[i - 1]) };
            let after = bytes.get(i + 4).copied();
            let boundary = |c: Option<u8>| {
                c.map(|b| !(b.is_ascii_alphanumeric() || b == b'_'))
                    .unwrap_or(true)
            };
            if boundary(before) && boundary(after) {
                return true;
            }
        }
        i += 1;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::is_ready_to_upload;

    #[test]
    fn greg_word_boundaries() {
        // `has_word_greg` takes lowercased input (callers lowercase first).
        assert!(has_word_greg("needs greg here"));
        assert!(has_word_greg("greg."));
        assert!(!has_word_greg("gregframework"));
        assert!(!has_word_greg("aggregation"));
        assert!(tag_suggests_greg("gregCore-mod-framework"));
        assert!(!tag_suggests_greg("vanilla"));
    }

    #[test]
    fn empty_project_fails() {
        let dir = std::env::temp_dir().join(format!(
            "greg-upload-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let meta = WorkshopMetadata::default();
        let results = check("en", &dir, &meta, None);
        assert!(!is_ready_to_upload(&results));
        assert!(results.iter().any(|r| {
            r.label == "Content folder" && r.severity == crate::models::UploadCheckSeverity::Error
        }));
        std::fs::remove_dir_all(&dir).ok();
    }

    fn temp_project(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "greg-upload-{name}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(dir.join("content")).unwrap();
        dir
    }

    #[test]
    fn security_preflight_blocks_secrets_and_executables() {
        let dir = temp_project("sec");
        std::fs::write(dir.join("content").join("mod.dll"), b"dll").unwrap();
        std::fs::write(dir.join("content").join(".env"), b"SECRET=x").unwrap();
        std::fs::write(dir.join("content").join("run.exe"), b"mz").unwrap();
        let mut results = Vec::new();
        check_security_preflight(&dir, &mut results);
        assert!(results.iter().any(|r| {
            r.label == "Security preflight"
                && r.severity == crate::models::UploadCheckSeverity::Error
                && r.detail.contains(".env")
        }));
        assert!(results.iter().any(|r| {
            r.label == "Security preflight"
                && r.severity == crate::models::UploadCheckSeverity::Error
                && r.detail.contains("run.exe")
        }));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn security_preflight_passes_clean_content() {
        let dir = temp_project("clean");
        std::fs::write(dir.join("content").join("mod.dll"), b"dll").unwrap();
        std::fs::write(dir.join("content").join("config.json"), b"{}").unwrap();
        let mut results = Vec::new();
        check_security_preflight(&dir, &mut results);
        assert!(results.iter().any(|r| {
            r.label == "Security preflight"
                && r.severity == crate::models::UploadCheckSeverity::Ok
        }));
        std::fs::remove_dir_all(&dir).ok();
    }
}
