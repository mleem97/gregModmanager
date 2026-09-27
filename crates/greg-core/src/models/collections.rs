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

/// One entry inside a collection.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ModCollectionEntry {
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
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
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub source_kind: ModCollectionSourceKind,
    #[serde(default = "default_collection_source")]
    pub source_name: String,
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
    pub fn create(&mut self, name: &str, description: &str) -> String {
        let id = uuid_like();
        self.catalog.collections.push(ModCollectionDefinition {
            id: id.clone(),
            name: name.to_string(),
            description: description.to_string(),
            source_kind: ModCollectionSourceKind::Local,
            source_name: "Local".to_string(),
            workshop_collection_url: None,
            items: Vec::new(),
        });
        id
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

    /// Adds an entry unless the file id is already present.
    pub fn add_item(&mut self, id: &str, entry: ModCollectionEntry) -> bool {
        match self.get_mut(id) {
            Some(c) => {
                if c.items
                    .iter()
                    .any(|e| e.published_file_id == entry.published_file_id)
                {
                    return true;
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
        let id = svc.create("Pack", "desc");
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
        svc.create("Pack", "");
        let json = svc.to_json().unwrap();
        let back = ModCollectionService::load_json(&json).unwrap();
        assert_eq!(back.collections().len(), 1);
    }
}
