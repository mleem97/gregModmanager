//! MelonLoader source discovery (`MelonSourceDiscoveryService` port).

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Where external MelonLoader content was found.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MelonSourceKind {
    /// Next to the game executable.
    Game,
    /// StreamingAssets delivery.
    StreamingAssets,
    /// Downloaded Workshop item.
    Workshop,
    /// greg framework tree.
    Greg,
}

/// What kind of content a source holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MelonContentKind {
    /// MelonLoader mods.
    Mods,
    /// MelonLoader plugins.
    Plugins,
    /// greg plugins.
    GregPlugins,
    /// Shared libraries.
    UserLibs,
}

/// One discovered source folder.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MelonSourceLocation {
    /// Source kind.
    pub kind: MelonSourceKind,
    /// Content kind.
    pub content: MelonContentKind,
    /// Absolute path.
    pub path: PathBuf,
}

/// Discovers MelonLoader source folders below `game_root`.
pub fn discover(game_root: &std::path::Path) -> Vec<MelonSourceLocation> {
    let mut out = Vec::new();
    let mut push = |kind, content, path: PathBuf| {
        if path.is_dir() {
            out.push(MelonSourceLocation {
                kind,
                content,
                path,
            });
        }
    };
    push(
        MelonSourceKind::Game,
        MelonContentKind::Mods,
        game_root.join("Mods"),
    );
    push(
        MelonSourceKind::Game,
        MelonContentKind::Plugins,
        game_root.join("Plugins"),
    );
    push(
        MelonSourceKind::Game,
        MelonContentKind::UserLibs,
        game_root.join("Plugins").join("Dependencies"),
    );
    push(
        MelonSourceKind::Greg,
        MelonContentKind::GregPlugins,
        game_root.join("greg").join("Plugins"),
    );
    let streaming = game_root
        .join("Data Center_Data")
        .join("StreamingAssets")
        .join("mods");
    if streaming.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&streaming) {
            for entry in entries.flatten() {
                let dir = entry.path();
                if dir.is_dir() {
                    push(
                        MelonSourceKind::Workshop,
                        MelonContentKind::Mods,
                        dir.join("WorkshopUploadContent").join("Mods"),
                    );
                    push(
                        MelonSourceKind::Workshop,
                        MelonContentKind::Plugins,
                        dir.join("WorkshopUploadContent").join("Plugins"),
                    );
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovers_game_folders() {
        let base = std::env::temp_dir().join(format!(
            "greg-discover-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(base.join("Mods")).unwrap();
        std::fs::create_dir_all(base.join("greg").join("Plugins")).unwrap();
        let found = discover(&base);
        assert!(found.iter().any(|l| l.content == MelonContentKind::Mods));
        assert!(found
            .iter()
            .any(|l| l.content == MelonContentKind::GregPlugins));
        std::fs::remove_dir_all(&base).ok();
    }
}
