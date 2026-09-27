//! Modpack apply logic (enable/disable members, subscribe workshop entries).
//!
//! Steam access arrives as closures so this crate never depends on
//! `greg-steam` (see `RUST_GRAPH.md`).

use std::path::Path;

use greg_core::models::ModCollectionDefinition;

use crate::error::Result;

/// What applying one pack did.
#[derive(Debug, Default)]
pub struct ApplyReport {
    /// Local files enabled.
    pub enabled_local: usize,
    /// Local files disabled.
    pub disabled_local: usize,
    /// Workshop ids subscribed.
    pub subscribed: Vec<u64>,
    /// Workshop ids unsubscribed.
    pub unsubscribed: Vec<u64>,
    /// Workshop ids skipped (no Steam).
    pub skipped_no_steam: Vec<u64>,
    /// Non-fatal errors.
    pub errors: Vec<String>,
}

impl ApplyReport {
    /// One-line summary for logs and the UI status.
    pub fn summary(&self) -> String {
        format!(
            "enabled {}, disabled {}, subscribed {}, unsubscribed {}, skipped {}, errors {}",
            self.enabled_local,
            self.disabled_local,
            self.subscribed.len(),
            self.unsubscribed.len(),
            self.skipped_no_steam.len(),
            self.errors.len()
        )
    }
}

/// Applies a pack: local members follow `entry.enabled`, workshop members
/// are (un)subscribed when Steam is available (`subscribe` is `None` without
/// Steam — those ids land in `skipped_no_steam`).
pub fn apply_pack(
    collection: &ModCollectionDefinition,
    game_root: &Path,
    subscribe: Option<&dyn Fn(u64) -> std::result::Result<(), String>>,
    unsubscribe: Option<&dyn Fn(u64) -> std::result::Result<(), String>>,
) -> Result<ApplyReport> {
    let _ = game_root;
    let mut report = ApplyReport::default();
    if !collection.enabled {
        report.errors.push(format!(
            "pack '{}' is disabled — nothing applied",
            collection.name
        ));
        return Ok(report);
    }
    for entry in &collection.items {
        if let Some(path) = entry.local_path.as_deref() {
            let path = Path::new(path);
            let probe = crate::local_content::LocalContentEntry {
                name: entry.title.clone(),
                detail: String::new(),
                enabled: !entry.enabled,
                path: path.to_path_buf(),
            };
            // set_enabled flips to the target state; probe carries the inverse.
            match crate::local_content::set_enabled(&probe, entry.enabled) {
                Ok(_) => {
                    if entry.enabled {
                        report.enabled_local += 1;
                    } else {
                        report.disabled_local += 1;
                    }
                }
                Err(e) => report.errors.push(format!("{}: {e}", entry.title)),
            }
            continue;
        }
        if entry.published_file_id == 0 {
            report.errors.push(format!(
                "{}: entry has neither file id nor path",
                entry.title
            ));
            continue;
        }
        let id = entry.published_file_id;
        if entry.enabled {
            match subscribe {
                Some(subscribe) => match subscribe(id) {
                    Ok(()) => report.subscribed.push(id),
                    Err(e) => report.errors.push(format!("subscribe {id}: {e}")),
                },
                None => report.skipped_no_steam.push(id),
            }
        } else {
            match unsubscribe {
                Some(unsubscribe) => match unsubscribe(id) {
                    Ok(()) => report.unsubscribed.push(id),
                    Err(e) => report.errors.push(format!("unsubscribe {id}: {e}")),
                },
                None => report.skipped_no_steam.push(id),
            }
        }
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use greg_core::models::{CollectionKind, ModCollectionEntry, ModCollectionService};

    fn pack_with(entries: Vec<ModCollectionEntry>) -> ModCollectionDefinition {
        let mut svc = ModCollectionService::new();
        let id = svc.create(CollectionKind::Pack, "Pack", "");
        for entry in entries {
            svc.add_item(&id, entry);
        }
        svc.get(&id).unwrap().clone()
    }

    #[test]
    fn toggles_local_files() {
        let dir = std::env::temp_dir().join(format!(
            "greg-pack-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let dll = dir.join("m.dll");
        std::fs::write(&dll, b"123").unwrap();
        let pack = pack_with(vec![ModCollectionEntry {
            title: "m.dll".into(),
            local_path: Some(dll.to_string_lossy().to_string()),
            enabled: false,
            ..Default::default()
        }]);
        let report = apply_pack(&pack, &dir, None, None).unwrap();
        assert_eq!(report.disabled_local, 1);
        assert!(dll.to_string_lossy().ends_with(".disabled") || !dll.exists());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn subscribes_workshop_entries() {
        let pack = pack_with(vec![
            ModCollectionEntry {
                published_file_id: 11,
                title: "on".into(),
                enabled: true,
                ..Default::default()
            },
            ModCollectionEntry {
                published_file_id: 22,
                title: "off".into(),
                enabled: false,
                ..Default::default()
            },
        ]);
        let report = apply_pack(
            &pack,
            Path::new("/nonexistent"),
            Some(&|_| Ok(())),
            Some(&|_| Ok(())),
        )
        .unwrap();
        assert_eq!(report.subscribed, vec![11]);
        assert_eq!(report.unsubscribed, vec![22]);
    }

    #[test]
    fn skips_workshop_without_steam() {
        let pack = pack_with(vec![ModCollectionEntry {
            published_file_id: 11,
            title: "on".into(),
            ..Default::default()
        }]);
        let report = apply_pack(&pack, Path::new("/nonexistent"), None, None).unwrap();
        assert_eq!(report.skipped_no_steam, vec![11]);
    }

    #[test]
    fn disabled_pack_applies_nothing() {
        let mut svc = ModCollectionService::new();
        let id = svc.create(CollectionKind::Pack, "Pack", "");
        svc.toggle(&id);
        let pack = svc.get(&id).unwrap().clone();
        let report = apply_pack(&pack, Path::new("/nonexistent"), None, None).unwrap();
        assert_eq!(report.errors.len(), 1);
        assert_eq!(report.enabled_local, 0);
    }
}
