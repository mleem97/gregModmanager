//! Shared workshop metadata (`metadata.json`).
//!
//! Field names are wire-compatible with the C# `WorkshopMetadata`
//! (`camelCase`, incl. `workshop_dependency`).

use serde::{Deserialize, Serialize};

/// Known mod types (free-form strings stay accepted for compatibility).
pub const MOD_TYPE_PLACABLE_OBJECT: &str = "PlacableObject";
pub const MOD_TYPE_MELONLOADER_PLUGIN: &str = "MelonloaderPlugin";
pub const MOD_TYPE_USERLIB: &str = "Userlib";
pub const MOD_TYPE_DATACENTER_MOD: &str = "DataCenterMod";

/// Known visibilities.
pub const VISIBILITY_PUBLIC: &str = "Public";
pub const VISIBILITY_FRIENDS_ONLY: &str = "FriendsOnly";
pub const VISIBILITY_PRIVATE: &str = "Private";

/// Native config profiles.
pub const NATIVE_PROFILE_DECORATION: &str = "decoration";
pub const NATIVE_PROFILE_CODE: &str = "code";

/// Project metadata persisted as `metadata.json` next to `content/`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkshopMetadata {
    #[serde(default)]
    pub published_file_id: u64,
    #[serde(default)]
    pub title: String,
    /// Local semantic version shown in the Steam change note.
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub description: String,
    /// `Public`, `FriendsOnly` or `Private`.
    #[serde(default = "default_visibility")]
    pub visibility: String,
    #[serde(default = "default_preview")]
    pub preview_image_relative_path: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub needsgreg: bool,
    /// Appends a MelonLoader requirement notice to the Steam description.
    #[serde(default, rename = "needsMelonLoader")]
    pub needs_melon_loader: bool,
    /// `decoration` (native shop/static `config.json`) or `code`.
    #[serde(default = "default_native_profile")]
    pub native_config_profile: String,
    #[serde(default)]
    pub additional_previews: Vec<String>,
    /// Other Steam Workshop file ids this item depends on.
    #[serde(default, rename = "workshop_dependency")]
    pub workshop_dependency_ids: Vec<u64>,
    /// `PlacableObject`, `MelonloaderPlugin`, `Userlib` or `DataCenterMod`.
    #[serde(default = "default_mod_type")]
    pub mod_type: String,
    /// SHA-256 over the last successful publish. Local-only, never uploaded.
    #[serde(default)]
    pub last_published_hash: String,
}

fn default_version() -> String {
    "1.0.0".to_string()
}
fn default_visibility() -> String {
    VISIBILITY_PUBLIC.to_string()
}
fn default_preview() -> String {
    "preview.png".to_string()
}
fn default_native_profile() -> String {
    NATIVE_PROFILE_DECORATION.to_string()
}
fn default_mod_type() -> String {
    MOD_TYPE_PLACABLE_OBJECT.to_string()
}

impl Default for WorkshopMetadata {
    /// Mirrors the C# property initializers.
    fn default() -> Self {
        Self {
            published_file_id: 0,
            title: String::new(),
            version: default_version(),
            description: String::new(),
            visibility: default_visibility(),
            preview_image_relative_path: default_preview(),
            tags: Vec::new(),
            needsgreg: false,
            needs_melon_loader: false,
            native_config_profile: default_native_profile(),
            additional_previews: Vec::new(),
            workshop_dependency_ids: Vec::new(),
            mod_type: default_mod_type(),
            last_published_hash: String::new(),
        }
    }
}

impl WorkshopMetadata {
    /// Fills missing values with defaults (after deserializing partial files).
    pub fn normalize(&mut self) {
        if self.version.trim().is_empty() {
            self.version = default_version();
        }
        if self.visibility.trim().is_empty() {
            self.visibility = default_visibility();
        }
        if self.preview_image_relative_path.trim().is_empty() {
            self.preview_image_relative_path = default_preview();
        }
        if self.native_config_profile.trim().is_empty() {
            self.native_config_profile = default_native_profile();
        }
        if self.mod_type.trim().is_empty() {
            self.mod_type = default_mod_type();
        }
        self.workshop_dependency_ids.retain(|id| *id > 0);
        self.workshop_dependency_ids.sort_unstable();
        self.workshop_dependency_ids.dedup();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_names_match_csharp() {
        let meta = WorkshopMetadata {
            title: "My Mod".into(),
            version: "1.0.0".into(),
            needs_melon_loader: true,
            workshop_dependency_ids: vec![1, 2],
            ..Default::default()
        };
        let json = serde_json::to_string(&meta).unwrap();
        assert!(json.contains("\"publishedFileId\":0"));
        assert!(json.contains("\"needsMelonLoader\":true"));
        assert!(json.contains("\"workshop_dependency\":[1,2]"));
        assert!(json.contains("\"previewImageRelativePath\":\"preview.png\""));

        let back: WorkshopMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(back, meta);
    }

    #[test]
    fn normalize_repairs_partial_files() {
        let mut meta: WorkshopMetadata = serde_json::from_str(r#"{"title":"T"}"#).unwrap();
        meta.normalize();
        assert_eq!(meta.version, "1.0.0");
        assert_eq!(meta.visibility, "Public");
        assert_eq!(meta.mod_type, "PlacableObject");
    }
}
