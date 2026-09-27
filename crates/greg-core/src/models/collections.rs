//! Named mod collections (local catalog, JSON persisted).

use serde::{Deserialize, Serialize};

use crate::error::{json_err, Result};

/// Where a collection entry comes from.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModCollectionSourceKind {
    /// Manually assembled.
    #[default]
    Local,
    /// From the greg store.
    GregStore,
    /// From Steam Workshop.
    SteamWorkshop,
}

/// Purpose of a collection: playable local set vs. shareable list.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum CollectionKind {
    /// Local modpack: applied to the game (enable/disable members).
    #[default]
    Pack,
    /// Shareable collection à la Steam Workshop collections.
    Collection,
}

impl CollectionKind {
    /// Stable id used by the UI.
    pub fn id(self) -> &'static str {
        match self {
            Self::Pack => "pack",
            Self::Collection => "collection",
        }
    }

    /// Parses the UI id (defaults to pack).
    pub fn parse(id: &str) -> Self {
        match id.trim().to_lowercase().as_str() {
            "collection" => Self::Collection,
            _ => Self::Pack,
        }
    }
}

/// One entry inside a collection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ModCollectionEntry {
    /// Steam Workshop file id (0 for local-only entries).
    #[serde(default)]
    pub published_file_id: u64,
    #[serde(default)]
    pub title: String,
    #[serde(default = "default_source_name")]
    pub source_name: String,
    #[serde(default = "default_entry_mod_type")]
    pub mod_type: String,
    #[serde(default)]
    pub workshop_dependency_ids: Vec<u64>,
    /// Absolute local file for local entries (packs of installed mods).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_path: Option<String>,
    /// Entry participates when the pack is applied.
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

fn default_true() -> bool {
    true
}

impl Default for ModCollectionEntry {
    /// New entries participate (matches the serde default).
    fn default() -> Self {
        Self {
            published_file_id: 0,
            title: String::new(),
            source_name: default_source_name(),
            mod_type: default_entry_mod_type(),
            workshop_dependency_ids: Vec::new(),
            local_path: None,
            enabled: true,
            notes: None,
        }
    }
}

fn default_source_name() -> String {
    "Steam Workshop".to_string()
}
fn default_entry_mod_type() -> String {
    "PlacableObject".to_string()
}

/// A named set of mods with a source marker.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ModCollectionDefinition {
    // NOTE: no `Default` derive on purpose — `Default::default()` would
    // disagree with the serde defaults (`kind: Pack`, `enabled: true`).
    // Construction goes through `ModCollectionService::create`.
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub source_kind: ModCollectionSourceKind,
    #[serde(default = "default_collection_source")]
    pub source_name: String,
    /// Pack (local, applicable) vs. shareable collection.
    #[serde(default)]
    pub kind: CollectionKind,
    /// Author display name (relevant when shared).
    #[serde(default)]
    pub author: String,
    /// Pack participates in apply-all flows.
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workshop_collection_url: Option<String>,
    #[serde(default)]
    pub items: Vec<ModCollectionEntry>,
}

fn default_collection_source() -> String {
    "Local".to_string()
}

/// Root catalog file holding all collections.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CollectionCatalog {
    #[serde(default = "default_schema")]
    pub schema_version: u32,
    #[serde(default)]
    pub collections: Vec<ModCollectionDefinition>,
}

fn default_schema() -> u32 {
    1
}

/// In-memory collection service (CRUD); persistence is plain JSON files.
#[derive(Debug, Default)]
pub struct ModCollectionService {
    catalog: CollectionCatalog,
}

impl ModCollectionService {
    /// Creates an empty service.
    pub fn new() -> Self {
        Self::default()
    }

    /// Loads a catalog from JSON text.
    pub fn load_json(json: &str) -> Result<Self> {
        let catalog: CollectionCatalog = serde_json::from_str(json).map_err(json_err)?;
        Ok(Self { catalog })
    }

    /// Serializes the catalog to JSON text.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(&self.catalog).map_err(json_err)
    }

    /// All collections.
    pub fn collections(&self) -> &[ModCollectionDefinition] {
        &self.catalog.collections
    }

    /// Finds a collection by id.
    pub fn get(&self, id: &str) -> Option<&ModCollectionDefinition> {
        self.catalog.collections.iter().find(|c| c.id == id)
    }

    /// Finds a collection by id (mutable).
    pub fn get_mut(&mut self, id: &str) -> Option<&mut ModCollectionDefinition> {
        self.catalog.collections.iter_mut().find(|c| c.id == id)
    }

    /// Creates a collection and returns its id.
    pub fn create(&mut self, kind: CollectionKind, name: &str, description: &str) -> String {
        let id = uuid_like();
        self.catalog.collections.push(ModCollectionDefinition {
            id: id.clone(),
            name: name.to_string(),
            description: description.to_string(),
            source_kind: ModCollectionSourceKind::Local,
            source_name: "Local".to_string(),
            kind,
            author: String::new(),
            enabled: true,
            workshop_collection_url: None,
            items: Vec::new(),
        });
        id
    }

    /// Lists collections of one kind (packs vs. shareable collections).
    pub fn of_kind(&self, kind: CollectionKind) -> Vec<&ModCollectionDefinition> {
        self.catalog
            .collections
            .iter()
            .filter(|c| c.kind == kind)
            .collect()
    }

    /// Toggles a whole pack/collection. Returns the new state, if known.
    pub fn toggle(&mut self, id: &str) -> Option<bool> {
        match self.get_mut(id) {
            Some(c) => {
                c.enabled = !c.enabled;
                Some(c.enabled)
            }
            None => None,
        }
    }

    /// Toggles one entry by index. Returns the new state, if known.
    pub fn toggle_entry(&mut self, id: &str, index: usize) -> Option<bool> {
        match self.get_mut(id) {
            Some(c) => match c.items.get_mut(index) {
                Some(entry) => {
                    entry.enabled = !entry.enabled;
                    Some(entry.enabled)
                }
                None => None,
            },
            None => None,
        }
    }

    /// Removes one entry by index. Returns false when unknown.
    pub fn remove_entry_at(&mut self, id: &str, index: usize) -> bool {
        match self.get_mut(id) {
            Some(c) if index < c.items.len() => {
                c.items.remove(index);
                true
            }
            _ => false,
        }
    }

    /// Exports one pack/collection as shareable JSON (Steam-like sharing).
    pub fn export_pack(&self, id: &str) -> Result<String> {
        match self.get(id) {
            Some(definition) => serde_json::to_string_pretty(definition).map_err(json_err),
            None => Err(crate::error::CoreError::NotFound(id.to_string())),
        }
    }

    /// Imports one exported pack/collection under a fresh id. Returns the id.
    pub fn import_pack(&mut self, json: &str) -> Result<String> {
        let mut definition: ModCollectionDefinition =
            serde_json::from_str(json).map_err(json_err)?;
        definition.id = uuid_like();
        let id = definition.id.clone();
        self.catalog.collections.push(definition);
        Ok(id)
    }

    /// Renames a collection. Returns false when unknown.
    pub fn rename(&mut self, id: &str, name: &str) -> bool {
        match self.get_mut(id) {
            Some(c) => {
                c.name = name.to_string();
                true
            }
            None => false,
        }
    }

    /// Deletes a collection. Returns false when unknown.
    pub fn delete(&mut self, id: &str) -> bool {
        let before = self.catalog.collections.len();
        self.catalog.collections.retain(|c| c.id != id);
        self.catalog.collections.len() != before
    }

    /// Adds an entry unless it is already present (by Workshop id, or by
    /// local path for local-only entries — id 0 never dedupes).
    pub fn add_item(&mut self, id: &str, entry: ModCollectionEntry) -> bool {
        match self.get_mut(id) {
            Some(c) => {
                if entry.published_file_id != 0
                    && c.items
                        .iter()
                        .any(|e| e.published_file_id == entry.published_file_id)
                {
                    return true;
                }
                if let Some(path) = entry.local_path.as_deref() {
                    if c.items
                        .iter()
                        .any(|e| e.local_path.as_deref() == Some(path))
                    {
                        return true;
                    }
                }
                c.items.push(entry);
                true
            }
            None => false,
        }
    }

    /// Removes an entry by file id.
    pub fn remove_item(&mut self, id: &str, published_file_id: u64) -> bool {
        match self.get_mut(id) {
            Some(c) => {
                let before = c.items.len();
                c.items.retain(|e| e.published_file_id != published_file_id);
                c.items.len() != before
            }
            None => false,
        }
    }
}

/// Pseudo-unique id without external crates (timestamp + counter + pid).
fn uuid_like() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!(
        "{now:032x}-{n:08x}-4c6c-8000-{:012x}",
        std::process::id() as u64
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crud_roundtrip() {
        let mut svc = ModCollectionService::new();
        let id = svc.create(CollectionKind::Pack, "Pack", "desc");
        assert!(svc.rename(&id, "Pack 2"));
        assert!(svc.add_item(
            &id,
            ModCollectionEntry {
                published_file_id: 42,
                title: "Mod".into(),
                ..Default::default()
            }
        ));
        // Duplicate add is idempotent.
        assert!(svc.add_item(
            &id,
            ModCollectionEntry {
                published_file_id: 42,
                ..Default::default()
            }
        ));
        assert_eq!(svc.get(&id).unwrap().items.len(), 1);
        assert!(svc.remove_item(&id, 42));
        assert!(svc.delete(&id));
        assert!(svc.get(&id).is_none());
    }

    #[test]
    fn json_persists() {
        let mut svc = ModCollectionService::new();
        svc.create(CollectionKind::Pack, "Pack", "");
        let json = svc.to_json().unwrap();
        let back = ModCollectionService::load_json(&json).unwrap();
        assert_eq!(back.collections().len(), 1);
    }

    #[test]
    fn kinds_toggle_and_share() {
        let mut svc = ModCollectionService::new();
        let pack = svc.create(CollectionKind::Pack, "Pack", "");
        let coll = svc.create(CollectionKind::Collection, "Shared", "");
        assert_eq!(svc.of_kind(CollectionKind::Pack).len(), 1);
        assert_eq!(svc.of_kind(CollectionKind::Collection).len(), 1);
        assert_eq!(svc.toggle(&pack), Some(false));
        assert!(svc.add_item(
            &coll,
            ModCollectionEntry {
                published_file_id: 7,
                title: "Mod".into(),
                ..Default::default()
            }
        ));
        assert_eq!(svc.toggle_entry(&coll, 0), Some(false));
        // Export/import roundtrip with a fresh id.
        let exported = svc.export_pack(&coll).unwrap();
        let imported = svc.import_pack(&exported).unwrap();
        assert_ne!(imported, coll);
        assert_eq!(svc.get(&imported).unwrap().items.len(), 1);
        assert!(svc.remove_entry_at(&coll, 0));
        assert!(svc.get(&coll).unwrap().items.is_empty());
    }

    #[test]
    fn legacy_files_default_safely() {
        // C# catalogs know neither kind nor enabled flags.
        let legacy = r#"{"schemaVersion":1,"collections":[{"id":"abc","name":"Old","items":[{"publishedFileId":9,"title":"M"}]}]}"#;
        let svc = ModCollectionService::load_json(legacy).unwrap();
        let def = svc.get("abc").unwrap();
        assert_eq!(def.kind, CollectionKind::Pack);
        assert!(def.enabled);
        assert!(def.items[0].enabled);
        assert!(def.items[0].local_path.is_none());
    }

    #[test]
    fn local_entries_dedupe_by_path() {
        let mut svc = ModCollectionService::new();
        let id = svc.create(CollectionKind::Pack, "Pack", "");
        let entry = || ModCollectionEntry {
            title: "a.dll".into(),
            local_path: Some("/game/Mods/a.dll".into()),
            ..Default::default()
        };
        assert!(svc.add_item(&id, entry()));
        // Same path again is idempotent, even with file id 0.
        assert!(svc.add_item(&id, entry()));
        assert_eq!(svc.get(&id).unwrap().items.len(), 1);
    }
}
