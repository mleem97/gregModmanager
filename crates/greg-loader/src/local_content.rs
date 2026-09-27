//! Locally installed content: mods, plugins, libs (`My Mods` pages).
//!
//! Scans the game folders, toggles assemblies via `.disabled` renames and
//! removes files. All paths stay inside the known game folders.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{io_err, LoaderError, Result};

/// Local content kinds for the three pages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocalContentKind {
    /// `{root}/Mods` assemblies.
    Mods,
    /// `{root}/Plugins` + `{root}/greg/Plugins` assemblies.
    Plugins,
    /// `{root}/Plugins/Dependencies` + `{root}/Userlibs` files.
    Libs,
}

impl LocalContentKind {
    /// Stable id used by the UI.
    pub fn id(self) -> &'static str {
        match self {
            Self::Mods => "mymods",
            Self::Plugins => "myplugins",
            Self::Libs => "mylibs",
        }
    }

    /// Page title.
    pub fn title(self) -> &'static str {
        match self {
            Self::Mods => "My Mods",
            Self::Plugins => "My Plugins",
            Self::Libs => "My Libs",
        }
    }
}

/// One installed file entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LocalContentEntry {
    /// File name as shown.
    pub name: String,
    /// Detail line (size + location).
    pub detail: String,
    /// False when renamed to `.disabled`.
    pub enabled: bool,
    /// Absolute path.
    pub path: PathBuf,
}

/// Folders scanned per kind (may not exist).
fn folders(game_root: &Path, kind: LocalContentKind) -> Vec<PathBuf> {
    match kind {
        LocalContentKind::Mods => vec![game_root.join("Mods")],
        LocalContentKind::Plugins => vec![
            game_root.join("Plugins"),
            game_root.join("greg").join("Plugins"),
        ],
        LocalContentKind::Libs => vec![
            game_root.join("Plugins").join("Dependencies"),
            game_root.join("Userlibs"),
        ],
    }
}

/// Scans installed entries (`.dll` + `.dll.disabled`; libs: any file).
pub fn scan(game_root: &Path, kind: LocalContentKind) -> Vec<LocalContentEntry> {
    let mut entries = Vec::new();
    for folder in folders(game_root, kind) {
        if !folder.is_dir() {
            continue;
        }
        let Ok(read) = std::fs::read_dir(&folder) else {
            continue;
        };
        let location = folder
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
            .to_string();
        for entry in read.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };
            let (display, enabled) = match name.strip_suffix(".disabled") {
                Some(base) => (base.to_string(), false),
                None => (name.clone(), true),
            };
            let is_dll = display.to_lowercase().ends_with(".dll");
            if kind != LocalContentKind::Libs && !is_dll {
                continue;
            }
            let size = entry
                .metadata()
                .map(|m| greg_core::util::format_bytes(m.len() as i64))
                .unwrap_or_default();
            entries.push(LocalContentEntry {
                name: display,
                detail: format!("{size} · {location}"),
                enabled,
                path,
            });
        }
    }
    entries.sort_by_key(|a| a.name.to_lowercase());
    entries
}

/// Cheap change signature for a kind (`name|size|mtime|enabled` per file).
/// Poll this to rescan only when something actually changed.
pub fn signature(game_root: &Path, kind: LocalContentKind) -> String {
    let mut parts: Vec<String> = Vec::new();
    for folder in folders(game_root, kind) {
        if !folder.is_dir() {
            continue;
        }
        let Ok(read) = std::fs::read_dir(&folder) else {
            continue;
        };
        for entry in read.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?")
                .to_string();
            let (size, mtime) = entry
                .metadata()
                .map(|m| {
                    (
                        m.len(),
                        m.modified()
                            .ok()
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|d| d.as_secs())
                            .unwrap_or(0),
                    )
                })
                .unwrap_or((0, 0));
            parts.push(format!("{name}|{size}|{mtime}"));
        }
    }
    parts.sort();
    parts.join("\n")
}

/// Toggles an entry (`.dll` ↔ `.dll.disabled`). Non-DLL files cannot toggle.
pub fn set_enabled(entry: &LocalContentEntry, enabled: bool) -> Result<PathBuf> {
    if entry.enabled == enabled {
        return Ok(entry.path.clone());
    }
    let target = if enabled {
        entry
            .path
            .to_string_lossy()
            .strip_suffix(".disabled")
            .map(PathBuf::from)
            .ok_or_else(|| LoaderError::Other("not a disabled assembly".into()))?
    } else {
        PathBuf::from(format!("{}.disabled", entry.path.display()))
    };
    if target.exists() {
        return Err(LoaderError::Other(format!(
            "target exists: {}",
            target.display()
        )));
    }
    std::fs::rename(&entry.path, &target).map_err(io_err)?;
    Ok(target)
}

/// Removes an installed file. Callers confirm first (UI dialog).
pub fn remove(entry: &LocalContentEntry) -> Result<()> {
    if !entry.path.is_file() {
        return Err(LoaderError::Other("file is already gone".into()));
    }
    std::fs::remove_file(&entry.path).map_err(io_err)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game_root(name: &str) -> PathBuf {
        let base = std::env::temp_dir().join(format!(
            "greg-local-{name}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(base.join("Mods")).unwrap();
        std::fs::create_dir_all(base.join("Plugins").join("Dependencies")).unwrap();
        std::fs::write(base.join("Mods").join("a.dll"), b"12345").unwrap();
        std::fs::write(base.join("Mods").join("b.dll.disabled"), b"12").unwrap();
        std::fs::write(base.join("Mods").join("notes.txt"), b"hi").unwrap();
        std::fs::write(
            base.join("Plugins").join("Dependencies").join("lib.dll"),
            b"123",
        )
        .unwrap();
        base
    }

    #[test]
    fn scans_and_toggles() {
        let root = game_root("scan");
        let mods = scan(&root, LocalContentKind::Mods);
        assert_eq!(mods.len(), 2);
        assert!(mods.iter().any(|e| e.name == "a.dll" && e.enabled));
        assert!(mods.iter().any(|e| e.name == "b.dll" && !e.enabled));

        let a = mods.iter().find(|e| e.name == "a.dll").unwrap().clone();
        let disabled = set_enabled(&a, false).unwrap();
        assert!(disabled.to_string_lossy().ends_with(".disabled"));
        let back = LocalContentEntry {
            path: disabled,
            ..a
        };
        let re = set_enabled(
            &LocalContentEntry {
                enabled: false,
                ..back
            },
            true,
        )
        .unwrap();
        assert!(re.to_string_lossy().ends_with("a.dll"));

        let libs = scan(&root, LocalContentKind::Libs);
        assert_eq!(libs.len(), 1);
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn signature_tracks_changes() {
        let root = game_root("sig");
        let before = signature(&root, LocalContentKind::Mods);
        std::fs::write(root.join("Mods").join("new.dll"), b"123").unwrap();
        let after = signature(&root, LocalContentKind::Mods);
        assert_ne!(before, after);
        // Toggle renames the file → signature changes too.
        let mods = scan(&root, LocalContentKind::Mods);
        let entry = mods.iter().find(|e| e.name == "new.dll").unwrap();
        set_enabled(entry, false).unwrap();
        assert_ne!(after, signature(&root, LocalContentKind::Mods));
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn removes_files() {
        let root = game_root("remove");
        let mods = scan(&root, LocalContentKind::Mods);
        let a = mods.iter().find(|e| e.name == "a.dll").unwrap();
        remove(a).unwrap();
        assert!(!a.path.exists());
        std::fs::remove_dir_all(&root).ok();
    }
}
