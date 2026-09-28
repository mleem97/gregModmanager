//! Workspace management (`WorkspaceService` port).
//!
//! Resolution order: custom path (preferences) → `<GameRoot>/workshop`
//! (passed in — game discovery lives in `greg-loader`/`greg-steam`) → legacy
//! `~/DataCenterWS` fallback.

use std::path::{Path, PathBuf};

use greg_core::limits::{self, MAX_DESCRIPTION_LENGTH, MAX_TITLE_LENGTH};
use greg_core::models::{ContentStats, ProjectSyncState, WorkshopMetadata, WorkshopProject};
use sha2::{Digest, Sha256};

use crate::error::{io_err, json_err, PlatformError, Result};

/// Preferences key for the user-configured workspace path.
pub const CUSTOM_WORKSPACE_PATH_KEY: &str = "CustomWorkspacePath";

const PREVIEW_EXTENSIONS: &[&str] = &[".png", ".jpg", ".jpeg", ".gif", ".webp"];

/// Workspace handle.
#[derive(Debug, Clone)]
pub struct Workspace {
    root: PathBuf,
}

impl Workspace {
    /// Resolves the root: custom path → game workshop dir → legacy fallback.
    pub fn resolve(custom_path: Option<&str>, game_root: Option<&Path>) -> Self {
        if let Some(custom) = custom_path {
            if !custom.trim().is_empty() {
                let p = PathBuf::from(custom.trim());
                if p.is_dir() {
                    return Self { root: p };
                }
            }
        }
        if let Some(game) = game_root {
            let ws = game.join("workshop");
            return Self { root: ws };
        }
        let fallback = crate::paths::legacy_fallback_workspace().unwrap_or_else(std::env::temp_dir);
        Self { root: fallback }
    }

    /// Resolved root.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// `<root>/.templates`.
    pub fn templates_dir(&self) -> PathBuf {
        self.root.join(".templates")
    }

    /// Creates the workspace structure plus a metadata sample.
    pub fn ensure_structure(&self) -> Result<()> {
        std::fs::create_dir_all(&self.root).map_err(io_err)?;
        std::fs::create_dir_all(self.templates_dir()).map_err(io_err)?;
        let sample = self.templates_dir().join("metadata.sample.json");
        if !sample.exists() {
            let meta = WorkshopMetadata {
                title: "My Mod".into(),
                description: "Description".into(),
                ..Default::default()
            };
            let text = serde_json::to_string_pretty(&meta).map_err(json_err)?;
            std::fs::write(sample, text).map_err(io_err)?;
        }
        Ok(())
    }

    /// Scans project folders (skips dot-folders), sorted by name.
    pub fn scan_projects(&self) -> Result<Vec<WorkshopProject>> {
        self.ensure_structure()?;
        let mut list = Vec::new();
        let entries = std::fs::read_dir(&self.root).map_err(io_err)?;
        for entry in entries.flatten() {
            let dir = entry.path();
            if !dir.is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                continue;
            }
            let content = dir.join("content");
            list.push(WorkshopProject {
                name,
                content_path: content.clone(),
                metadata_path: dir.join("metadata.json"),
                root_path: dir,
                is_valid_layout: content.is_dir(),
            });
        }
        list.sort_by_key(|a| a.name.to_lowercase());
        Ok(list)
    }

    /// Finds the local project owning a Steam Workshop item.
    pub fn find_project_by_published_file_id(
        &self,
        published_file_id: u64,
    ) -> Result<Option<WorkshopProject>> {
        if published_file_id == 0 {
            return Ok(None);
        }
        for project in self.scan_projects()? {
            let meta = load_metadata(&project.root_path)?;
            if meta.published_file_id == published_file_id {
                return Ok(Some(project));
            }
        }
        Ok(None)
    }

    /// Moves legacy `~/DataCenterWS` projects into this workspace.
    /// Returns the number of moved folders.
    pub fn migrate_legacy_projects(&self) -> usize {
        let Some(legacy) = crate::paths::legacy_fallback_workspace() else {
            return 0;
        };
        if legacy == self.root || !legacy.is_dir() {
            return 0;
        }
        let mut moved = 0;
        let Ok(entries) = std::fs::read_dir(&legacy) else {
            return 0;
        };
        for entry in entries.flatten() {
            let dir = entry.path();
            if !dir.is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                continue;
            }
            if !dir.join("metadata.json").is_file() && !dir.join("content").is_dir() {
                continue;
            }
            let dest = self.root.join(&name);
            if dest.exists() {
                continue;
            }
            if std::fs::create_dir_all(&self.root).is_err() {
                continue;
            }
            if std::fs::rename(&dir, &dest).is_ok() {
                moved += 1;
            }
        }
        moved
    }

    /// Compares current state against the recorded publish hash.
    /// `effective_description` must be the exact text sent to Steam
    /// (README-resolved + notices), like the publisher uses.
    pub fn sync_state(
        &self,
        project_root: &Path,
        effective_description: &str,
    ) -> Result<ProjectSyncState> {
        let meta = load_metadata(project_root)?;
        if !limits::is_usable_workshop_id(meta.published_file_id) {
            return Ok(ProjectSyncState::Unpublished);
        }
        if meta.last_published_hash.is_empty() {
            return Ok(ProjectSyncState::Unknown);
        }
        let current = compute_publish_hash(project_root, &meta, effective_description);
        if current.is_empty() {
            return Ok(ProjectSyncState::Unknown);
        }
        Ok(if current == meta.last_published_hash {
            ProjectSyncState::Synced
        } else {
            ProjectSyncState::Modified
        })
    }

    /// Total size of `content/` plus largest direct children.
    pub fn content_stats(&self, project_root: &Path) -> ContentStats {
        let content = project_root.join("content");
        if !content.is_dir() {
            return ContentStats::default();
        }
        ContentStats {
            exists: true,
            total_bytes: dir_size(&content),
            file_count: walkdir::WalkDir::new(&content)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .count() as i64,
        }
    }
}

/// Loads `metadata.json` (defaults + normalization + preview auto-detect).
pub fn load_metadata(project_root: &Path) -> Result<WorkshopMetadata> {
    let path = project_root.join("metadata.json");
    let mut meta = if path.is_file() {
        let text = std::fs::read_to_string(&path).map_err(io_err)?;
        serde_json::from_str::<WorkshopMetadata>(&text).unwrap_or_default()
    } else {
        WorkshopMetadata::default()
    };
    meta.normalize();
    auto_detect_preview(project_root, &mut meta);
    Ok(meta)
}

/// Saves `metadata.json`, enforcing Steam length limits.
pub fn save_metadata(project_root: &Path, metadata: &WorkshopMetadata) -> Result<()> {
    if metadata.title.chars().count() > MAX_TITLE_LENGTH {
        return Err(PlatformError::Project(format!(
            "Title exceeds {MAX_TITLE_LENGTH} characters."
        )));
    }
    if metadata.description.chars().count() > MAX_DESCRIPTION_LENGTH {
        return Err(PlatformError::Project(format!(
            "Description exceeds {MAX_DESCRIPTION_LENGTH} characters."
        )));
    }
    let text = serde_json::to_string_pretty(metadata).map_err(json_err)?;
    std::fs::write(project_root.join("metadata.json"), text).map_err(io_err)?;
    Ok(())
}

/// SHA-256 over everything a publish sends to Steam: content files
/// (relative paths + bytes, sorted), preview bytes, and the uploaded
/// metadata fields. Returns `""` when `content/` is missing.
pub fn compute_publish_hash(
    project_root: &Path,
    meta: &WorkshopMetadata,
    effective_description: &str,
) -> String {
    let content = project_root.join("content");
    if !content.is_dir() {
        return String::new();
    }
    let mut files: Vec<PathBuf> = Vec::new();
    for entry in walkdir::WalkDir::new(&content)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        files.push(entry.path().to_path_buf());
    }
    files.sort();
    let mut hasher = Sha256::new();
    for full in files {
        let Ok(rel) = full.strip_prefix(&content) else {
            return String::new();
        };
        let Ok(bytes) = std::fs::read(&full) else {
            return String::new();
        };
        let rel = rel.to_string_lossy().replace('\\', "/");
        hasher.update(rel.as_bytes());
        hasher.update([0u8]);
        hasher.update((bytes.len() as u64).to_be_bytes());
        hasher.update([0u8]);
        hasher.update(&bytes);
    }
    if !meta.preview_image_relative_path.trim().is_empty() {
        let preview = project_root.join(&meta.preview_image_relative_path);
        if preview.is_file() {
            match std::fs::read(&preview) {
                Ok(bytes) => hasher.update(&bytes),
                Err(_) => return String::new(),
            }
        }
    }
    let meta_line = format!(
        "{}\n{}\n{}\n{}\n{}",
        meta.title.trim(),
        effective_description,
        meta.tags.join(","),
        meta.visibility,
        meta.version
    );
    hasher.update(meta_line.as_bytes());
    hex::encode(hasher.finalize())
}

/// Copies a directory tree recursively.
pub fn copy_dir_recursive(source: &Path, dest: &Path) -> Result<()> {
    if !source.is_dir() {
        return Err(PlatformError::Project(format!(
            "source not found: {}",
            source.display()
        )));
    }
    std::fs::create_dir_all(dest).map_err(io_err)?;
    for entry in walkdir::WalkDir::new(source)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        // Never follow or materialize symlinks from foreign content
        // (workshop items): jail-break and arbitrary-read risk.
        if entry.path().is_symlink() {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(source)
            .map_err(|e| PlatformError::Other(e.to_string()))?;
        let target = dest.join(rel);
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&target).map_err(io_err)?;
        } else {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent).map_err(io_err)?;
            }
            std::fs::copy(entry.path(), &target).map_err(io_err)?;
        }
    }
    Ok(())
}

fn auto_detect_preview(project_root: &Path, meta: &mut WorkshopMetadata) {
    if project_root
        .join(&meta.preview_image_relative_path)
        .is_file()
    {
        return;
    }
    for ext in PREVIEW_EXTENSIONS {
        let candidate = format!("preview{ext}");
        if project_root.join(&candidate).is_file() {
            meta.preview_image_relative_path = candidate;
            return;
        }
    }
}

fn dir_size(dir: &Path) -> i64 {
    walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok().map(|m| m.len() as i64))
        .sum()
}

/// Re-exported pure helpers (single source lives in `greg-core`).
pub use greg_core::util::{format_bytes, sanitize_folder_name};

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "greg-ws-test-{nanos}-{}-{name}",
            std::process::id()
        ));
        std::fs::create_dir_all(dir.join("content")).unwrap();
        std::fs::write(dir.join("content").join("mod.dll"), "v1-bytes").unwrap();
        std::fs::write(dir.join("preview.png"), "preview-bytes").unwrap();
        dir
    }

    fn base_meta() -> WorkshopMetadata {
        WorkshopMetadata {
            published_file_id: 1_234_567_890,
            title: "Sync Test Mod".into(),
            description: "Plain description.".into(),
            ..Default::default()
        }
    }

    #[test]
    fn hash_is_deterministic_and_sensitive() {
        let root = temp_root("hash");
        let meta = base_meta();
        let h1 = compute_publish_hash(&root, &meta, &meta.description);
        let h2 = compute_publish_hash(&root, &meta, &meta.description);
        assert!(!h1.is_empty());
        assert_eq!(h1, h2);
        std::fs::write(root.join("content").join("mod.dll"), "v2").unwrap();
        let h3 = compute_publish_hash(&root, &meta, &meta.description);
        assert_ne!(h1, h3);
        let mut renamed = meta.clone();
        renamed.title = "Renamed".into();
        assert_ne!(
            h1,
            compute_publish_hash(&root, &renamed, &renamed.description)
        );
        assert_eq!(
            compute_publish_hash(&root.join("nope"), &meta, &meta.description),
            ""
        );
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn sync_state_roundtrip() {
        let root = temp_root("sync");
        let ws = Workspace { root: root.clone() };
        let mut meta = base_meta();
        meta.published_file_id = 0;
        save_metadata(&root, &meta).unwrap();
        assert_eq!(
            ws.sync_state(&root, &meta.description).unwrap(),
            ProjectSyncState::Unpublished
        );
        meta.published_file_id = 1_234_567_890;
        save_metadata(&root, &meta).unwrap();
        assert_eq!(
            ws.sync_state(&root, &meta.description).unwrap(),
            ProjectSyncState::Unknown
        );
        meta.last_published_hash = compute_publish_hash(&root, &meta, &meta.description);
        save_metadata(&root, &meta).unwrap();
        assert_eq!(
            ws.sync_state(&root, &meta.description).unwrap(),
            ProjectSyncState::Synced
        );
        std::fs::write(root.join("content").join("mod.dll"), "v3").unwrap();
        assert_eq!(
            ws.sync_state(&root, &meta.description).unwrap(),
            ProjectSyncState::Modified
        );
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn metadata_save_enforces_limits() {
        let root = temp_root("limits");
        let mut meta = base_meta();
        meta.title = "x".repeat(200);
        assert!(save_metadata(&root, &meta).is_err());
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn scan_skips_dotfolders() {
        let base = temp_root("scan-base");
        let root = base.join("ws");
        std::fs::create_dir_all(root.join("b-proj").join("content")).unwrap();
        std::fs::create_dir_all(root.join("a-proj").join("content")).unwrap();
        std::fs::create_dir_all(root.join(".templates")).unwrap();
        let ws = Workspace { root: root.clone() };
        let projects = ws.scan_projects().unwrap();
        let names: Vec<_> = projects.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["a-proj", "b-proj"]);
        std::fs::remove_dir_all(&base).ok();
    }

    #[test]
    fn copy_dir_recursive_works() {
        let base = temp_root("copy-base");
        let dest = base.join("dest");
        copy_dir_recursive(&base.join("content"), &dest).unwrap();
        assert!(dest.join("mod.dll").is_file());
        std::fs::remove_dir_all(&base).ok();
    }

    #[cfg(unix)]
    #[test]
    fn copy_skips_symlinks() {
        use std::os::unix::fs::symlink;
        let base = temp_root("copy-link");
        let content = base.join("content");
        std::fs::create_dir_all(&content).unwrap();
        std::fs::write(content.join("mod.dll"), b"1").unwrap();
        symlink(content.join("mod.dll"), content.join("evil.dll")).ok();
        symlink("/etc/hostname", content.join("escape")).ok();
        let dest = base.join("dest");
        copy_dir_recursive(&content, &dest).unwrap();
        assert!(dest.join("mod.dll").is_file());
        assert!(!dest.join("evil.dll").exists());
        assert!(!dest.join("escape").exists());
        std::fs::remove_dir_all(&base).ok();
    }
}
