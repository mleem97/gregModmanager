//! Controlled Workshop imports into the manager-owned workspace.
//!
//! Steam keeps ownership of its download cache. Imports are copied into
//! `GameModding/{Mods,Melons,UserLibs}/{item-id}`; game folders are never targets.

use std::io::Read;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{io_err, LoaderError, Result};
use crate::game::GameAdapterRegistry;
use greg_core::models::ProjectContentKind;

const OWNER_FILE: &str = ".greg-managed-item.json";

/// Outcome of one import operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncOutcome {
    pub success: bool,
    pub path: Option<PathBuf>,
    pub error: Option<String>,
}

impl SyncOutcome {
    pub fn ok(path: PathBuf) -> Self {
        Self {
            success: true,
            path: Some(path),
            error: None,
        }
    }

    pub fn fail(error: impl Into<String>) -> Self {
        Self {
            success: false,
            path: None,
            error: Some(error.into()),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct ModTypeMarker {
    #[serde(rename = "modType", default)]
    mod_type: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct Ownership {
    schema_version: u32,
    published_file_id: u64,
}

/// Legacy game-tree copying is intentionally disabled. Call
/// [`sync_managed_item`] with the manager base instead.
pub fn sync_item(
    _registry: &GameAdapterRegistry,
    _published_file_id: u64,
    _steam_dir: &Path,
    _game_root: &Path,
) -> SyncOutcome {
    SyncOutcome::fail("direct Workshop installation into the game directory is disabled; import into the manager workspace")
}

/// Legacy game-tree removal is disabled: existing game files require explicit
/// migration/review, never implicit deletion during a workspace import.
pub fn remove_item(
    _registry: &GameAdapterRegistry,
    _published_file_id: u64,
    _game_root: &Path,
) -> bool {
    false
}

/// Imports a downloaded item into an explicitly selected manager base.
///
/// Mirror packages may contain `Mods`, `Plugins`/`Melons`, `UserLibs`, and
/// `UserData`/`ModConfigs`; the latter seed `ModConfigs/{id}` only once so user
/// settings survive Workshop updates. Flat/legacy packages route by modType.
/// Symlinks, special files and paths unsafe on Windows are rejected. Each
/// category is replaced atomically with rollback; unrelated files are untouched.
pub fn sync_managed_item(
    published_file_id: u64,
    steam_dir: &Path,
    manager_base: &Path,
) -> SyncOutcome {
    match import_inner(published_file_id, steam_dir, manager_base) {
        Ok(path) => SyncOutcome::ok(path),
        Err(error) => SyncOutcome::fail(error.to_string()),
    }
}

fn import_inner(id: u64, source: &Path, base: &Path) -> Result<PathBuf> {
    if id == 0 {
        return Err(LoaderError::InvalidArtefact(
            "Workshop item ID must be nonzero".into(),
        ));
    }
    reject_links(source)?;
    if !source.is_dir() {
        return Err(LoaderError::InvalidArtefact(
            "download folder is missing".into(),
        ));
    }
    // A mistakenly supplied game root must never become a managed target.
    if base.join("Data Center_Data").exists()
        || base.join("MelonLoader").exists()
        || crate::launch::GAME_EXE_NAMES
            .iter()
            .any(|name| base.join(name).exists())
    {
        return Err(LoaderError::InvalidArtefact(
            "manager base points to a game installation".into(),
        ));
    }
    ensure_plain_dir(base)?;
    let base = base.canonicalize().map_err(io_err)?;
    let source = source.canonicalize().map_err(io_err)?;
    if source.starts_with(&base) || base.starts_with(&source) {
        return Err(LoaderError::InvalidArtefact(
            "Workshop source and manager base must not overlap".into(),
        ));
    }
    let managed = base.join("GameModding");
    ensure_plain_dir(&managed)?;
    let _lock = ImportLock::acquire(managed.join(format!(".workshop-{id}.lock")))?;
    let plans = import_plan(&source)?;
    // Preflight every destination before replacing any category. A conflict in
    // an auxiliary library package must not partially update the mod package.
    for (folder, _, seed_only) in &plans {
        let category = if *seed_only {
            base.join(folder)
        } else {
            managed.join(folder)
        };
        let dest = package_destination(&category, id);
        reject_links(&dest)?;
        if !seed_only && dest.exists() && !owned_by(&dest, id) {
            return Err(LoaderError::InvalidArtefact(format!(
                "destination is not owned by this Workshop item: {}",
                dest.display()
            )));
        }
    }
    let mut primary = None;
    for (folder, src, seed_only) in plans {
        let category = if seed_only {
            base.join(folder)
        } else {
            managed.join(folder)
        };
        ensure_plain_dir(&category)?;
        let dest = package_destination(&category, id);
        if seed_only && dest.exists() {
            reject_links(&dest)?;
            continue;
        }
        replace_package(id, &src, &dest)?;
        if !seed_only && primary.is_none() {
            primary = Some(dest);
        }
    }
    primary
        .ok_or_else(|| LoaderError::InvalidArtefact("package contains no loadable content".into()))
}

fn package_destination(category: &Path, id: u64) -> PathBuf {
    let enabled = category.join(id.to_string());
    let disabled = category.join(format!("{id}.disabled"));
    if !enabled.exists() && disabled.exists() {
        disabled
    } else {
        enabled
    }
}

fn import_plan(source: &Path) -> Result<Vec<(&'static str, PathBuf, bool)>> {
    // Validate the complete source before changing any existing package.
    let mut portable_paths = std::collections::HashSet::new();
    for entry in walkdir::WalkDir::new(source).follow_links(false) {
        let entry = entry.map_err(|e| LoaderError::InvalidArtefact(e.to_string()))?;
        validate_entry(entry.path(), entry.file_type())?;
        let relative = entry
            .path()
            .strip_prefix(source)
            .map_err(|e| LoaderError::Other(e.to_string()))?;
        if !portable_paths.insert(relative.to_string_lossy().to_lowercase()) {
            return Err(LoaderError::InvalidArtefact(
                "case-colliding package paths are not portable".into(),
            ));
        }
    }
    let content = source.join("content");
    let payload = if content.is_dir() {
        content
    } else {
        source.to_path_buf()
    };
    let mirrors = [
        ("Mods", "Mods", false),
        ("Plugins", "Melons", false),
        ("Melons", "Melons", false),
        ("UserLibs", "UserLibs", false),
        ("UserData", "ModConfigs", true),
        ("ModConfigs", "ModConfigs", true),
    ];
    let mut plan = Vec::new();
    for (input, output, seed) in mirrors {
        let path = payload.join(input);
        if !path.is_dir() {
            continue;
        }
        if plan.iter().any(|(existing, _, _)| *existing == output) {
            return Err(LoaderError::InvalidArtefact(format!(
                "ambiguous package folders targeting {output}"
            )));
        }
        plan.push((output, path, seed));
    }
    if !plan.is_empty() {
        if plan.iter().all(|(_, _, seed)| *seed) {
            return Err(LoaderError::InvalidArtefact(
                "config-only Workshop package has no loadable content".into(),
            ));
        }
        return Ok(plan);
    }
    let marker = source.join("greg-modmanager.meta.json");
    let kind = if marker.is_file() {
        let value: ModTypeMarker =
            serde_json::from_str(&std::fs::read_to_string(marker).map_err(io_err)?)
                .map_err(crate::error::json_err)?;
        match value.mod_type.as_deref() {
            Some(value) => ProjectContentKind::parse(value).ok_or_else(|| {
                LoaderError::InvalidArtefact(format!("unsupported modType: {value}"))
            })?,
            None => ProjectContentKind::PlacableObject,
        }
    } else {
        ProjectContentKind::PlacableObject
    };
    let folder = kind.installation_dir();
    Ok(vec![(folder, payload, false)])
}

/// Rejects symlinked managed paths and parents before a managed operation.
pub fn validate_managed_path(path: &Path) -> Result<()> {
    reject_links(path)
}

fn reject_links(path: &Path) -> Result<()> {
    for parent in path.ancestors() {
        if let Ok(meta) = std::fs::symlink_metadata(parent) {
            if meta.file_type().is_symlink() {
                return Err(LoaderError::InvalidArtefact(format!(
                    "symbolic link is not allowed: {}",
                    parent.display()
                )));
            }
        }
    }
    Ok(())
}

fn ensure_plain_dir(path: &Path) -> Result<()> {
    reject_links(path)?;
    std::fs::create_dir_all(path).map_err(io_err)
}

fn validate_entry(path: &Path, kind: std::fs::FileType) -> Result<()> {
    if kind.is_symlink() || !(kind.is_file() || kind.is_dir()) {
        return Err(LoaderError::InvalidArtefact(format!(
            "unsupported filesystem entry: {}",
            path.display()
        )));
    }
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| LoaderError::InvalidArtefact("non-Unicode package path".into()))?;
    let windows_stem = name
        .split('.')
        .next()
        .unwrap_or_default()
        .to_ascii_uppercase();
    let reserved_device = matches!(windows_stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (windows_stem.len() == 4
            && (windows_stem.starts_with("COM") || windows_stem.starts_with("LPT"))
            && matches!(windows_stem.as_bytes()[3], b'1'..=b'9'));
    if name == OWNER_FILE
        || name.contains(['\\', ':', '<', '>', '"', '|', '?', '*'])
        || name.ends_with(['.', ' '])
        || reserved_device
        || name.chars().any(char::is_control)
    {
        return Err(LoaderError::InvalidArtefact(format!(
            "unsafe package path: {}",
            path.display()
        )));
    }
    Ok(())
}

fn owned_by(path: &Path, id: u64) -> bool {
    reject_links(&path.join(OWNER_FILE)).is_ok()
        && std::fs::read(path.join(OWNER_FILE))
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Ownership>(&bytes).ok())
            .is_some_and(|owner| owner.schema_version == 1 && owner.published_file_id == id)
}

fn replace_package(id: u64, source: &Path, dest: &Path) -> Result<()> {
    reject_links(dest)?;
    if dest.exists() && !owned_by(dest, id) {
        return Err(LoaderError::InvalidArtefact(format!(
            "destination is not owned by this Workshop item: {}",
            dest.display()
        )));
    }
    if dest.is_dir() && trees_equal(source, dest)? {
        return Ok(());
    }
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let staging = dest.with_file_name(format!(".{id}.stage-{}-{nonce}", std::process::id()));
    let backup = dest.with_file_name(format!(".{id}.backup-{}-{nonce}", std::process::id()));
    let result = (|| {
        std::fs::create_dir(&staging).map_err(io_err)?;
        copy_dir(source, &staging)?;
        preserve_disabled_files(dest, &staging)?;
        let owner = serde_json::to_vec(&Ownership {
            schema_version: 1,
            published_file_id: id,
        })
        .map_err(crate::error::json_err)?;
        std::fs::write(staging.join(OWNER_FILE), owner).map_err(io_err)?;
        if dest.exists() {
            std::fs::rename(dest, &backup).map_err(io_err)?;
        }
        if let Err(error) = std::fs::rename(&staging, dest) {
            if backup.exists() {
                std::fs::rename(&backup, dest).map_err(io_err)?;
            }
            return Err(io_err(error));
        }
        if backup.exists() {
            std::fs::remove_dir_all(&backup).map_err(io_err)?;
        }
        Ok(())
    })();
    if staging.exists() {
        let _ = std::fs::remove_dir_all(staging);
    }
    result
}

fn preserve_disabled_files(previous: &Path, staging: &Path) -> Result<()> {
    if !previous.is_dir() {
        return Ok(());
    }
    for entry in walkdir::WalkDir::new(staging).follow_links(false) {
        let entry = entry.map_err(|error| LoaderError::InvalidArtefact(error.to_string()))?;
        if !entry.file_type().is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy();
        if !name.to_ascii_lowercase().ends_with(".dll") {
            continue;
        }
        let relative = entry
            .path()
            .strip_prefix(staging)
            .map_err(|error| LoaderError::Other(error.to_string()))?;
        let previous_disabled = previous
            .join(relative)
            .with_file_name(format!("{name}.disabled"));
        reject_links(&previous_disabled)?;
        if previous_disabled.is_file() {
            let disabled = entry.path().with_file_name(format!("{name}.disabled"));
            if !disabled.exists() {
                std::fs::rename(entry.path(), disabled).map_err(io_err)?;
            }
        }
    }
    Ok(())
}

fn copy_dir(source: &Path, dest: &Path) -> Result<()> {
    for entry in walkdir::WalkDir::new(source).follow_links(false) {
        let entry = entry.map_err(|e| LoaderError::InvalidArtefact(e.to_string()))?;
        validate_entry(entry.path(), entry.file_type())?;
        let relative = entry
            .path()
            .strip_prefix(source)
            .map_err(|e| LoaderError::Other(e.to_string()))?;
        let target = dest.join(relative);
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(target).map_err(io_err)?;
        } else {
            std::fs::copy(entry.path(), target).map_err(io_err)?;
        }
    }
    Ok(())
}

fn trees_equal(source: &Path, dest: &Path) -> Result<bool> {
    let mut source_files = 0;
    for entry in walkdir::WalkDir::new(source).follow_links(false) {
        let entry = entry.map_err(|e| LoaderError::InvalidArtefact(e.to_string()))?;
        let relative = entry
            .path()
            .strip_prefix(source)
            .map_err(|e| LoaderError::Other(e.to_string()))?;
        let mut target = dest.join(relative);
        if !target.exists()
            && entry.file_type().is_file()
            && entry
                .path()
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("dll"))
        {
            let disabled = target.with_file_name(format!(
                "{}.disabled",
                target.file_name().unwrap_or_default().to_string_lossy()
            ));
            if disabled.is_file() {
                target = disabled;
            }
        }
        reject_links(&target)?;
        if entry.file_type().is_dir() {
            if !target.is_dir() {
                return Ok(false);
            }
        } else {
            source_files += 1;
            if !files_equal(entry.path(), &target)? {
                return Ok(false);
            }
        }
    }
    let dest_files = walkdir::WalkDir::new(dest)
        .follow_links(false)
        .into_iter()
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| LoaderError::InvalidArtefact(e.to_string()))?;
    if dest_files.iter().any(|e| e.file_type().is_symlink()) {
        return Err(LoaderError::InvalidArtefact(
            "symlink in managed package".into(),
        ));
    }
    Ok(source_files + 1
        == dest_files
            .iter()
            .filter(|e| e.file_type().is_file())
            .count())
}

fn files_equal(source: &Path, dest: &Path) -> Result<bool> {
    if !dest.is_file()
        || std::fs::metadata(source).map_err(io_err)?.len()
            != std::fs::metadata(dest).map_err(io_err)?.len()
    {
        return Ok(false);
    }
    let mut a = std::fs::File::open(source).map_err(io_err)?;
    let mut b = std::fs::File::open(dest).map_err(io_err)?;
    let mut ab = [0; 64 * 1024];
    let mut bb = [0; 64 * 1024];
    loop {
        let n = a.read(&mut ab).map_err(io_err)?;
        b.read_exact(&mut bb[..n]).map_err(io_err)?;
        if ab[..n] != bb[..n] {
            return Ok(false);
        }
        if n == 0 {
            return Ok(true);
        }
    }
}

struct ImportLock(PathBuf);
impl ImportLock {
    fn acquire(path: PathBuf) -> Result<Self> {
        std::fs::OpenOptions::new().write(true).create_new(true).open(&path)
            .map_err(|e| LoaderError::Other(format!("cannot lock Workshop import (another import or interrupted operation needs review): {e}")))?;
        Ok(Self(path))
    }
}
impl Drop for ImportLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// Removes manager-owned imported payloads only. ModConfigs deliberately stay.
pub fn remove_managed_item(id: u64, manager_base: &Path) -> Result<bool> {
    let root = manager_base.join("GameModding");
    reject_links(&root)?;
    let _lock = ImportLock::acquire(root.join(format!(".workshop-{id}.lock")))?;
    let mut removed = false;
    for folder in ["Mods", "Melons", "UserLibs"] {
        for name in [id.to_string(), format!("{id}.disabled")] {
            let path = root.join(folder).join(name);
            reject_links(&path)?;
            if !path.exists() {
                continue;
            }
            if !owned_by(&path, id) {
                return Err(LoaderError::InvalidArtefact(
                    "refusing to remove unowned content".into(),
                ));
            }
            std::fs::remove_dir_all(path).map_err(io_err)?;
            removed = true;
        }
    }
    Ok(removed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "greg-managed-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(dir.join("source")).unwrap();
        dir.canonicalize().unwrap()
    }

    #[test]
    fn mirror_routes_to_managed_tree_and_preserves_user_configs() {
        let root = setup();
        for folder in ["Mods", "Plugins", "UserLibs", "UserData"] {
            std::fs::create_dir_all(root.join("source/content").join(folder)).unwrap();
            std::fs::write(
                root.join("source/content").join(folder).join("test.dll"),
                b"first",
            )
            .unwrap();
        }
        let base = root.join("manager");
        let first = sync_managed_item(42, &root.join("source"), &base);
        assert!(first.success, "{:?}", first.error);
        for folder in ["Mods", "Melons", "UserLibs"] {
            assert!(base
                .join("GameModding")
                .join(folder)
                .join("42/test.dll")
                .is_file());
        }
        let config = base.join("ModConfigs/42/test.dll");
        std::fs::write(&config, b"user setting").unwrap();
        std::fs::write(root.join("source/content/Mods/test.dll"), b"second").unwrap();
        assert!(sync_managed_item(42, &root.join("source"), &base).success);
        assert_eq!(
            std::fs::read(base.join("GameModding/Mods/42/test.dll")).unwrap(),
            b"second"
        );
        assert_eq!(std::fs::read(&config).unwrap(), b"user setting");
        assert!(remove_managed_item(42, &base).unwrap());
        assert!(config.is_file());
        assert!(root.join("source/content/Mods/test.dll").is_file());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn idempotent_import_keeps_file_timestamp_and_rejects_unowned_target() {
        let root = setup();
        std::fs::write(root.join("source/mod.dll"), b"first").unwrap();
        let base = root.join("manager");
        let dest = sync_managed_item(1, &root.join("source"), &base)
            .path
            .unwrap();
        let before = std::fs::metadata(dest.join("mod.dll"))
            .unwrap()
            .modified()
            .unwrap();
        assert!(sync_managed_item(1, &root.join("source"), &base).success);
        assert_eq!(
            std::fs::metadata(dest.join("mod.dll"))
                .unwrap()
                .modified()
                .unwrap(),
            before
        );
        std::fs::remove_file(dest.join(OWNER_FILE)).unwrap();
        assert!(!sync_managed_item(1, &root.join("source"), &base).success);
        assert_eq!(std::fs::read(dest.join("mod.dll")).unwrap(), b"first");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn workshop_updates_keep_disabled_assemblies_disabled() {
        let root = setup();
        std::fs::write(root.join("source/mod.dll"), b"old").unwrap();
        let base = root.join("manager");
        let dest = sync_managed_item(1, &root.join("source"), &base)
            .path
            .unwrap();
        std::fs::rename(dest.join("mod.dll"), dest.join("mod.dll.disabled")).unwrap();
        assert!(sync_managed_item(1, &root.join("source"), &base).success);
        assert!(!dest.join("mod.dll").exists());
        std::fs::write(root.join("source/mod.dll"), b"new version").unwrap();
        assert!(sync_managed_item(1, &root.join("source"), &base).success);
        assert!(!dest.join("mod.dll").exists());
        assert_eq!(
            std::fs::read(dest.join("mod.dll.disabled")).unwrap(),
            b"new version"
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn game_tree_install_is_fail_closed() {
        let root = setup();
        assert!(
            !sync_item(
                &GameAdapterRegistry::with_defaults(),
                1,
                &root.join("source"),
                &root.join("game")
            )
            .success
        );
        assert!(!root.join("game").exists());
        std::fs::create_dir_all(root.join("manager/Data Center_Data")).unwrap();
        assert!(!sync_managed_item(1, &root.join("source"), &root.join("manager")).success);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlink_payload_without_replacing_previous_import() {
        let root = setup();
        std::fs::write(root.join("source/mod.dll"), b"first").unwrap();
        let base = root.join("manager");
        assert!(sync_managed_item(1, &root.join("source"), &base).success);
        std::os::unix::fs::symlink(root.join("source/mod.dll"), root.join("source/link.dll"))
            .unwrap();
        assert!(!sync_managed_item(1, &root.join("source"), &base).success);
        assert_eq!(
            std::fs::read(base.join("GameModding/Mods/1/mod.dll")).unwrap(),
            b"first"
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
