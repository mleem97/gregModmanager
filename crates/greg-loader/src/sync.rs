//! Workshop-to-game folder sync (`ModsFolderSyncService` port).
//!
//! Routes a downloaded item into the game tree by its marker mod type and
//! removes previously synced items by id.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::content::ModContentType;
use crate::error::{LoaderError, Result};
use crate::game::GameAdapterRegistry;

/// Outcome of one sync operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncOutcome {
    /// Success flag.
    pub success: bool,
    /// Destination dir (when successful).
    pub path: Option<PathBuf>,
    /// Error message (when failed).
    pub error: Option<String>,
}

impl SyncOutcome {
    /// Successful sync.
    pub fn ok(path: PathBuf) -> Self {
        Self {
            success: true,
            path: Some(path),
            error: None,
        }
    }

    /// Failed sync.
    pub fn fail(error: impl Into<String>) -> Self {
        Self {
            success: false,
            path: None,
            error: Some(error.into()),
        }
    }
}

/// Marker file content inside synced workshop content.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct ModTypeMarker {
    #[serde(rename = "modType", default)]
    mod_type: Option<String>,
}

/// Syncs one downloaded item (`steam_dir`) into `game_root`.
pub fn sync_item(
    registry: &GameAdapterRegistry,
    published_file_id: u64,
    steam_dir: &Path,
    game_root: &Path,
) -> SyncOutcome {
    match sync_item_inner(registry, published_file_id, steam_dir, game_root) {
        Ok(path) => SyncOutcome::ok(path),
        Err(e) => SyncOutcome::fail(e.to_string()),
    }
}

fn sync_item_inner(
    registry: &GameAdapterRegistry,
    published_file_id: u64,
    steam_dir: &Path,
    game_root: &Path,
) -> Result<PathBuf> {
    if !steam_dir.is_dir() {
        return Err(LoaderError::Game(format!(
            "download folder missing: {}",
            steam_dir.display()
        )));
    }
    let mod_type = read_mod_type(steam_dir);
    let dest = destination_path(registry, game_root, published_file_id, mod_type);
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(crate::error::io_err)?;
    }
    if dest.exists() {
        std::fs::remove_dir_all(&dest).map_err(crate::error::io_err)?;
    }
    copy_dir(steam_dir, &dest)?;
    Ok(dest)
}

/// Removes all synced folders belonging to `published_file_id`.
pub fn remove_item(
    registry: &GameAdapterRegistry,
    published_file_id: u64,
    game_root: &Path,
) -> bool {
    let id = published_file_id.to_string();
    let (_, installation) = match registry.detect(Some(game_root)) {
        Some(found) => found,
        None => return false,
    };
    let root = &installation.root_path;
    let candidates = [
        root.join("Data Center_Data")
            .join("StreamingAssets")
            .join("Mods")
            .join(&id),
        root.join("Mods").join("Workshop").join(&id),
        root.join("Mods").join(&id),
        root.join("Plugins").join(&id),
        root.join("Plugins").join("Dependencies").join(&id),
        root.join("Userlibs").join(&id),
    ];
    let mut removed = false;
    for candidate in candidates {
        if candidate.is_dir() && std::fs::remove_dir_all(&candidate).is_ok() {
            removed = true;
        }
    }
    removed
}

fn read_mod_type(steam_dir: &Path) -> ModContentType {
    let marker = steam_dir.join("greg-modmanager.meta.json");
    if let Ok(text) = std::fs::read_to_string(&marker) {
        if let Ok(parsed) = serde_json::from_str::<ModTypeMarker>(&text) {
            if let Some(t) = parsed.mod_type {
                return ModContentType::parse(&t);
            }
        }
    }
    ModContentType::PlacableObject
}

fn destination_path(
    registry: &GameAdapterRegistry,
    game_root: &Path,
    published_file_id: u64,
    mod_type: ModContentType,
) -> PathBuf {
    let id = published_file_id.to_string();
    let detected = registry.detect(Some(game_root));
    let root = detected
        .as_ref()
        .map(|(_, i)| i.root_path.clone())
        .unwrap_or_else(|| game_root.to_path_buf());
    // Adapter-aware defaults with plain fallbacks.
    let adapter_paths = detected
        .as_ref()
        .map(|(adapter, installation)| adapter.paths(&installation.root_path));
    match mod_type {
        ModContentType::MelonloaderPlugin => adapter_paths
            .map(|p| p.plugins.join(&id))
            .unwrap_or_else(|| root.join("Plugins").join(&id)),
        ModContentType::Userlib => adapter_paths
            .map(|p| p.user_libraries.join(&id))
            .unwrap_or_else(|| root.join("Plugins").join("Dependencies").join(&id)),
        ModContentType::DataCenterMod => adapter_paths
            .map(|p| p.mods.join(&id))
            .unwrap_or_else(|| root.join("Mods").join(&id)),
        ModContentType::PlacableObject => adapter_paths
            .map(|p| p.workshop.join(&id))
            .unwrap_or_else(|| {
                root.join("Data Center_Data")
                    .join("StreamingAssets")
                    .join("Mods")
                    .join(&id)
            }),
    }
}

fn copy_dir(source: &Path, dest: &Path) -> Result<()> {
    std::fs::create_dir_all(dest).map_err(crate::error::io_err)?;
    for entry in walkdir::WalkDir::new(source)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let rel = entry
            .path()
            .strip_prefix(source)
            .map_err(|e| LoaderError::Other(e.to_string()))?;
        let target = dest.join(rel);
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&target).map_err(crate::error::io_err)?;
        } else {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent).map_err(crate::error::io_err)?;
            }
            std::fs::copy(entry.path(), &target).map_err(crate::error::io_err)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_base(name: &str) -> PathBuf {
        let base = std::env::temp_dir().join(format!(
            "greg-sync-{name}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(base.join("src")).unwrap();
        std::fs::create_dir_all(base.join("game").join("Mods")).unwrap();
        std::fs::write(base.join("src").join("mod.dll"), "bytes").unwrap();
        base
    }

    #[test]
    fn routes_by_marker_type() {
        let base = temp_base("route");
        let registry = GameAdapterRegistry::with_defaults();
        // No marker → PlacableObject → StreamingAssets workshop folder.
        let out = sync_item(&registry, 7, &base.join("src"), &base.join("game"));
        assert!(out.success);
        let dest = out.path.unwrap();
        assert!(dest.join("mod.dll").is_file());
        assert!(dest.to_string_lossy().contains("StreamingAssets"));
        assert!(remove_item(&registry, 7, &base.join("game")));
        std::fs::remove_dir_all(&base).ok();
    }

    #[test]
    fn melon_plugin_goes_to_plugins() {
        let base = temp_base("melon");
        std::fs::write(
            base.join("src").join("greg-modmanager.meta.json"),
            r#"{"modType":"MelonloaderPlugin"}"#,
        )
        .unwrap();
        let registry = GameAdapterRegistry::with_defaults();
        let out = sync_item(&registry, 9, &base.join("src"), &base.join("game"));
        assert!(out.success);
        assert!(out.path.unwrap().to_string_lossy().contains("Plugins"));
        std::fs::remove_dir_all(&base).ok();
    }
}
