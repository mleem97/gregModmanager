//! Modstore wire models (`ModStoreModels` port).
//!
//! Field names are `camelCase` with defaults, matching the web API.

use serde::{Deserialize, Serialize};

/// One published file of a mod.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct ModStoreRelease {
    pub id: String,
    pub version: String,
    pub changelog: String,
    pub checksum_sha256: Option<String>,
    pub file_size: Option<i64>,
    pub file_name: String,
    pub published_at: Option<String>,
    pub download_url: String,
}

/// One catalog entry.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct ModStoreCatalogItem {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub author: String,
    pub category: String,
    pub tags: Vec<String>,
    pub image_url: Option<String>,
    pub release: ModStoreRelease,
}

impl ModStoreCatalogItem {
    /// Shared type stored in existing provider tags. Category remains a topic;
    /// it must never decide whether arbitrary DLLs are plugins or libraries.
    pub fn content_kind(&self) -> Result<greg_core::models::ProjectContentKind, String> {
        Ok(
            greg_core::models::ProjectContentKind::from_publication_tags(&self.tags)?
                .unwrap_or(greg_core::models::ProjectContentKind::DataCenterMod),
        )
    }
}

/// Catalog response.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct ModStoreCatalogResponse {
    pub schema_version: u32,
    pub mods: Vec<ModStoreCatalogItem>,
}

/// One locally installed mod.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct ModStoreInstalledEntry {
    pub mod_id: String,
    pub title: String,
    pub version: String,
    pub release_id: String,
    pub install_path: String,
    /// Canonical shared type; absent in legacy manifests.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub mod_type: String,
    pub installed_at: String,
}

impl ModStoreInstalledEntry {
    /// Preserve the original routing on updates. Legacy manifests were all mods.
    pub fn content_kind(&self) -> Result<greg_core::models::ProjectContentKind, String> {
        if self.mod_type.trim().is_empty() {
            return Ok(greg_core::models::ProjectContentKind::DataCenterMod);
        }
        greg_core::models::ProjectContentKind::parse(&self.mod_type)
            .ok_or_else(|| format!("Unknown installed project content type: {}", self.mod_type))
    }
}

/// Local install manifest (`.greg-modmanager/modstore-installed.json`).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct ModStoreInstallManifest {
    pub schema_version: u32,
    pub installed: Vec<ModStoreInstalledEntry>,
}

/// Available update for an installed mod.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct ModStoreUpdate {
    pub mod_id: String,
    pub title: String,
    pub installed_version: String,
    pub release: ModStoreRelease,
}

impl ModStoreUpdate {
    /// Update responses contain release data only, so use the local typed
    /// manifest instead of guessing a destination from title/file extension.
    pub fn content_kind(
        &self,
        manifest: &ModStoreInstallManifest,
    ) -> Result<greg_core::models::ProjectContentKind, String> {
        manifest
            .installed
            .iter()
            .find(|entry| entry.mod_id == self.mod_id)
            .map(ModStoreInstalledEntry::content_kind)
            .unwrap_or(Ok(greg_core::models::ProjectContentKind::DataCenterMod))
    }
}

/// Update-check response.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct ModStoreUpdateResponse {
    pub schema_version: u32,
    pub updates: Vec<ModStoreUpdate>,
}

/// Update-check request.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct ModStoreUpdateCheckRequest {
    pub installed: Vec<ModStoreInstalledVersion>,
}

/// One installed version for the check request.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct ModStoreInstalledVersion {
    pub mod_id: String,
    pub version: String,
}

/// Install outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModStoreInstallResult {
    /// Success flag.
    pub success: bool,
    /// Human-readable message.
    pub message: String,
    /// Install path (when successful).
    pub install_path: Option<String>,
}

impl ModStoreInstallResult {
    /// Successful install.
    pub fn ok(message: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            install_path: Some(path.into()),
        }
    }

    /// Failed install.
    pub fn fail(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            install_path: None,
        }
    }
}

/// Marker file identifying Modstore-managed content (`ModStoreMarker` port).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct ModStoreMarker {
    pub mod_id: String,
    pub version: String,
}

#[cfg(test)]
mod classification_tests {
    use super::*;
    use greg_core::models::ProjectContentKind;
    #[test]
    fn catalog_routes_explicit_types_without_using_topic_categories() {
        let mut item = ModStoreCatalogItem {
            category: "Userlib".into(),
            ..Default::default()
        };
        assert_eq!(
            item.content_kind().unwrap(),
            ProjectContentKind::DataCenterMod
        );
        item.tags = vec!["greg-mod-type:MelonloaderPlugin".into()];
        assert_eq!(
            item.content_kind().unwrap(),
            ProjectContentKind::MelonloaderPlugin
        );
        item.tags = vec!["greg-mod-type:Userlib".into()];
        assert_eq!(item.content_kind().unwrap(), ProjectContentKind::Userlib);
        item.tags = vec!["greg-mod-type:garbage".into()];
        assert!(item.content_kind().is_err());
    }
}
