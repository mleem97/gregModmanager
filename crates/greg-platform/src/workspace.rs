//! Workspace management (`WorkspaceService` port).
//!
//! A workspace root is the manager base, containing configuration, logs,
//! publishing projects and managed game content. Game installs are never bases.

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
    active_game: Option<crate::game_links::GameModdingSlot>,
    active_game_root: Option<PathBuf>,
}

impl Workspace {
    /// Resolve a configured base, then environment/bootstrap settings, then
    /// `$HOME/GregModmanager`. A new custom directory need not exist yet.
    /// `game_root` is retained for source compatibility and is not a base.
    pub fn resolve(custom_path: Option<&str>, _game_root: Option<&Path>) -> Self {
        let root = custom_path
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(crate::paths::manager_base_dir);
        Self {
            root,
            active_game: None,
            active_game_root: None,
        }
    }

    /// Manager base, not the Workshop publishing-project directory.
    pub fn root(&self) -> &Path {
        &self.root
    }
    /// Publishing projects, each with `metadata.json` and `content/`.
    pub fn workshop_dir(&self) -> PathBuf {
        self.root.join("Workshop")
    }
    /// Manager configuration and internal settings database.
    pub fn configs_dir(&self) -> PathBuf {
        self.root.join("Configs")
    }
    /// Session logs and crash reports.
    pub fn logs_dir(&self) -> PathBuf {
        self.root.join("Logs")
    }
    /// Managed game content, separated from the game installation.
    pub fn game_modding_dir(&self) -> PathBuf {
        self.active_game
            .as_ref()
            .map(|game| game.path.clone())
            .unwrap_or_else(|| self.root.join("GameModding"))
    }
    /// Return a workspace view scoped to one registered game installation.
    /// Settings, logs, publishing projects and ModConfigs still use `root`.
    pub fn for_game(&self, game_id: &str, game_root: &Path) -> Result<Self> {
        let slot = crate::game_links::game_modding_slot(&self.root, game_id, game_root)?;
        let mut workspace = self.clone();
        workspace.active_game_root = Some(game_root.canonicalize().map_err(io_err)?);
        workspace.active_game = Some(slot);
        Ok(workspace)
    }
    /// The selected game installation root, when this workspace is scoped.
    pub fn active_game_root(&self) -> Option<&Path> {
        self.active_game_root.as_deref()
    }
    /// The selected game adapter ID, when this workspace is scoped.
    pub fn active_game_id(&self) -> Option<&str> {
        self.active_game.as_ref().map(|game| game.game_id.as_str())
    }
    /// Per-game, per-installation managed content root. The installation key
    /// is opaque so local absolute paths are not exposed in directory names.
    pub fn game_modding_dir_for(&self, game_id: &str, game_root: &Path) -> Result<PathBuf> {
        Ok(crate::game_links::game_modding_slot(&self.root, game_id, game_root)?.path)
    }
    /// Per-installation Mods link location.
    pub fn mods_dir_for(&self, game_id: &str, game_root: &Path) -> Result<PathBuf> {
        Ok(self.game_modding_dir_for(game_id, game_root)?.join("Mods"))
    }
    /// Per-installation MelonLoader plugin link location.
    pub fn melons_dir_for(&self, game_id: &str, game_root: &Path) -> Result<PathBuf> {
        Ok(self
            .game_modding_dir_for(game_id, game_root)?
            .join("Melons"))
    }
    /// Per-installation UserLibs link location.
    pub fn userlibs_dir_for(&self, game_id: &str, game_root: &Path) -> Result<PathBuf> {
        Ok(self
            .game_modding_dir_for(game_id, game_root)?
            .join("UserLibs"))
    }
    /// Future content namespaces stay inside the selected game installation's
    /// managed slot, with Objects established as the first Custom category.
    pub fn custom_objects_dir_for(&self, game_id: &str, game_root: &Path) -> Result<PathBuf> {
        Ok(self
            .game_modding_dir_for(game_id, game_root)?
            .join("Custom")
            .join("Objects"))
    }
    /// Managed mods.
    pub fn mods_dir(&self) -> PathBuf {
        self.game_modding_dir().join("Mods")
    }
    /// Managed user libraries.
    pub fn userlibs_dir(&self) -> PathBuf {
        self.game_modding_dir().join("UserLibs")
    }
    /// Whole game UserData link, when the selected adapter supports it.
    pub fn user_data_dir(&self) -> PathBuf {
        self.game_modding_dir().join("UserData")
    }
    /// Future custom content roots remain organized by capability.
    pub fn custom_dir(&self) -> PathBuf {
        self.game_modding_dir().join("Custom")
    }
    /// Managed MelonLoader plugins.
    pub fn melons_dir(&self) -> PathBuf {
        self.game_modding_dir().join("Melons")
    }
    /// Per-mod configuration, separate from manager configuration.
    pub fn mod_configs_dir(&self) -> PathBuf {
        self.root.join("ModConfigs")
    }
    /// Publishing templates.
    pub fn templates_dir(&self) -> PathBuf {
        self.workshop_dir().join(".templates")
    }

    /// Create the standardized base structure without changing existing files.
    pub fn ensure_structure(&self) -> Result<()> {
        for dir in [
            self.configs_dir(),
            self.logs_dir(),
            self.workshop_dir(),
            self.mods_dir(),
            self.userlibs_dir(),
            self.melons_dir(),
            self.mod_configs_dir(),
            self.templates_dir(),
        ] {
            std::fs::create_dir_all(dir).map_err(io_err)?;
        }
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

    /// Discover canonical publishing projects plus old projects directly in the
    /// base. Legacy project paths remain valid and no files are moved on startup.
    pub fn scan_projects(&self) -> Result<Vec<WorkshopProject>> {
        self.ensure_structure()?;
        let mut list = Vec::new();
        let mut names = std::collections::HashSet::new();
        let mut roots = vec![self.workshop_dir(), self.root.clone()];
        if crate::paths::default_manager_base_dir().as_deref() == Some(self.root()) {
            if let Some(legacy) = crate::paths::legacy_fallback_workspace() {
                if legacy != self.root {
                    roots.push(legacy);
                }
            }
        }
        for root in roots {
            let entries = match std::fs::read_dir(&root) {
                Ok(entries) => entries,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(error) => return Err(io_err(error)),
            };
            for entry in entries {
                let entry = entry.map_err(io_err)?;
                let dir = entry.path();
                if !entry.file_type().map_err(io_err)?.is_dir() {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with('.') {
                    continue;
                }
                // Manager directories and arbitrary folders are not projects.
                if !dir.join("content").is_dir() && !dir.join("metadata.json").is_file() {
                    continue;
                }
                if !names.insert(name.to_lowercase()) {
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
        }
        list.sort_by_key(|a| a.name.to_lowercase());
        Ok(list)
    }

    /// Find the local project owning a Steam Workshop item.
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

    /// Compatibility hook. Legacy projects are discovered in place; automatic
    /// startup must never move or delete a user's publishing sources.
    pub fn migrate_legacy_projects(&self) -> usize {
        0
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
        let mut stats = ContentStats {
            exists: true,
            ..Default::default()
        };
        for entry in walkdir::WalkDir::new(&content)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                if let Ok(metadata) = entry.metadata() {
                    stats.total_bytes += metadata.len() as i64;
                    stats.file_count += 1;
                }
            }
        }
        stats
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
        let Ok(mut file) = std::fs::File::open(&full) else {
            return String::new();
        };
        let Ok(metadata) = file.metadata() else {
            return String::new();
        };
        let rel = rel.to_string_lossy().replace('\\', "/");
        hasher.update(rel.as_bytes());
        hasher.update([0u8]);
        hasher.update(metadata.len().to_be_bytes());
        hasher.update([0u8]);
        if !hash_stream(&mut hasher, &mut file, metadata.len()) {
            return String::new();
        }
    }
    if !meta.preview_image_relative_path.trim().is_empty() {
        let preview = project_root.join(&meta.preview_image_relative_path);
        if preview.is_file() {
            let Ok(mut file) = std::fs::File::open(&preview) else {
                return String::new();
            };
            let Ok(metadata) = file.metadata() else {
                return String::new();
            };
            if !hash_stream(&mut hasher, &mut file, metadata.len()) {
                return String::new();
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

// Bound peak memory independently of mod size while retaining the existing
// publish-hash wire format. A changing file must not produce a false sync hash.
fn hash_stream(hasher: &mut Sha256, file: &mut std::fs::File, expected_len: u64) -> bool {
    use std::io::Read;
    let mut buffer = [0u8; 64 * 1024];
    let mut read_bytes = 0u64;
    loop {
        match file.read(&mut buffer) {
            Ok(0) => return read_bytes == expected_len,
            Ok(count) => {
                hasher.update(&buffer[..count]);
                read_bytes += count as u64;
            }
            Err(_) => return false,
        }
    }
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
        let ws = Workspace {
            root: root.clone(),
            active_game: None,
            active_game_root: None,
        };
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
        let ws = Workspace {
            root: root.clone(),
            active_game: None,
            active_game_root: None,
        };
        let projects = ws.scan_projects().unwrap();
        let names: Vec<_> = projects.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["a-proj", "b-proj"]);
        std::fs::remove_dir_all(&base).ok();
    }

    #[test]
    fn layout_preserves_existing_projects_and_separates_manager_directories() {
        let base = temp_root("layout");
        let ws = Workspace::resolve(
            Some(base.to_str().unwrap()),
            Some(Path::new("/irrelevant-game")),
        );
        assert_eq!(ws.root(), base);
        ws.ensure_structure().unwrap();
        for directory in [
            ws.configs_dir(),
            ws.logs_dir(),
            ws.workshop_dir(),
            ws.mods_dir(),
            ws.userlibs_dir(),
            ws.melons_dir(),
            ws.mod_configs_dir(),
        ] {
            assert!(directory.is_dir());
        }
        std::fs::create_dir_all(ws.workshop_dir().join("new-project/content")).unwrap();
        std::fs::create_dir_all(base.join("old-project/content")).unwrap();
        std::fs::write(base.join("old-project/content/mod.dll"), b"old").unwrap();
        std::fs::create_dir_all(base.join("unrelated")).unwrap();
        let projects = ws.scan_projects().unwrap();
        assert_eq!(projects.len(), 2);
        assert_eq!(ws.migrate_legacy_projects(), 0);
        assert_eq!(
            std::fs::read(base.join("old-project/content/mod.dll")).unwrap(),
            b"old"
        );
        std::fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn new_custom_base_does_not_need_to_exist() {
        let parent = temp_root("new-base");
        let base = parent.join("not-yet-created");
        let ws = Workspace::resolve(Some(base.to_str().unwrap()), None);
        assert_eq!(ws.root(), base);
        ws.ensure_structure().unwrap();
        assert!(base.join("Workshop").is_dir());
        std::fs::remove_dir_all(parent).unwrap();
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
