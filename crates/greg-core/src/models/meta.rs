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

/// Reserved provider-independent classification tag. The Modstore API already
/// accepts tags, so this preserves its wire contract without inventing columns.
pub const CONTENT_KIND_TAG_PREFIX: &str = "greg-mod-type:";

/// Shared upload/install classification for Steam and the Modstore.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectContentKind {
    PlacableObject,
    MelonloaderPlugin,
    Userlib,
    DataCenterMod,
}

impl ProjectContentKind {
    /// Accepted canonical names and historical aliases. Unknown explicit types
    /// are rejected by publishers/installers instead of silently routing files.
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "placableobject" => Some(Self::PlacableObject),
            "melonloaderplugin" => Some(Self::MelonloaderPlugin),
            "userlib" => Some(Self::Userlib),
            "datacentermod" | "datacentermods" => Some(Self::DataCenterMod),
            _ => None,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::PlacableObject => MOD_TYPE_PLACABLE_OBJECT,
            Self::MelonloaderPlugin => MOD_TYPE_MELONLOADER_PLUGIN,
            Self::Userlib => MOD_TYPE_USERLIB,
            Self::DataCenterMod => MOD_TYPE_DATACENTER_MOD,
        }
    }
    /// Destination directly under GameModding, never under the game install.
    pub fn installation_dir(self) -> &'static str {
        match self {
            Self::MelonloaderPlugin => "Melons",
            Self::Userlib => "UserLibs",
            _ => "Mods",
        }
    }
    /// Decode reserved tags. No tag means a legacy unclassified mod; malformed
    /// or conflicting explicit tags must fail rather than select an arbitrary path.
    pub fn from_publication_tags(tags: &[String]) -> Result<Option<Self>, String> {
        let mut result = None;
        for tag in tags {
            let tag = tag.trim();
            let Some((prefix, value)) = tag.split_once(':') else {
                continue;
            };
            if !prefix.eq_ignore_ascii_case("greg-mod-type") {
                continue;
            }
            let kind = Self::parse(value)
                .ok_or_else(|| format!("Unknown project content type: {value}"))?;
            if result.is_some_and(|existing| existing != kind) {
                return Err("Conflicting project content type tags".into());
            }
            result = Some(kind);
        }
        Ok(result)
    }
}

/// Optional build check configuration shared by both upload destinations.
/// `project` is a relative csproj/sln/Cargo.toml path, not a shell command.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectBuildConfig {
    pub project: String,
    #[serde(default = "default_build_configuration")]
    pub configuration: String,
}
fn default_build_configuration() -> String {
    "Release".into()
}

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
    /// Shared optional catalog topic; independent of the runtime content type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub installation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub support_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_game_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recommended_game_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build: Option<ProjectBuildConfig>,
    /// SHA-256 over the last successful publish. Local-only, never uploaded.
    #[serde(default)]
    pub last_published_hash: String,
    /// Modstore mod id from the last successful Modstore publish.
    /// Local-only routing hint, never uploaded.
    #[serde(default)]
    pub modstore_id: String,
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
            category: None,
            license: None,
            installation: None,
            support_url: None,
            min_game_version: None,
            recommended_game_version: None,
            build: None,
            last_published_hash: String::new(),
            modstore_id: String::new(),
        }
    }
}

impl WorkshopMetadata {
    /// Shared runtime kind; explicit unknown values remain distinguishable.
    pub fn content_kind(&self) -> Option<ProjectContentKind> {
        ProjectContentKind::parse(&self.mod_type)
    }

    /// Identical classification tags for both providers, with stale type tags
    /// removed so changing the project type cannot emit conflicting metadata.
    pub fn publication_tags(&self) -> Vec<String> {
        let mut tags: Vec<String> = self
            .tags
            .iter()
            .filter(|tag| {
                !tag.trim()
                    .split_once(':')
                    .is_some_and(|(prefix, _)| prefix.eq_ignore_ascii_case("greg-mod-type"))
            })
            .cloned()
            .collect();
        if let Some(kind) = self.content_kind() {
            tags.push(format!("{CONTENT_KIND_TAG_PREFIX}{}", kind.name()));
        }
        tags
    }

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
        if let Some(kind) = self.content_kind() {
            self.mod_type = kind.name().into();
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
    fn shared_types_roundtrip_and_replace_stale_provider_tags() {
        for (name, directory) in [
            ("Userlib", "UserLibs"),
            ("MelonLoaderPlugin", "Melons"),
            ("DataCenterMods", "Mods"),
            ("PlacableObject", "Mods"),
        ] {
            let meta: WorkshopMetadata = serde_json::from_value(
                serde_json::json!({"modType":name,"tags":["custom","greg-mod-type:Userlib"]}),
            )
            .unwrap();
            let kind = meta.content_kind().unwrap();
            assert_eq!(kind.installation_dir(), directory);
            let tags = meta.publication_tags();
            assert_eq!(
                ProjectContentKind::from_publication_tags(&tags).unwrap(),
                Some(kind)
            );
            assert!(tags.contains(&"custom".into()));
            assert_eq!(
                tags.iter()
                    .filter(|tag| tag.starts_with(CONTENT_KIND_TAG_PREFIX))
                    .count(),
                1
            );
        }
        assert!(ProjectContentKind::parse("arbitrary-folder").is_none());
        assert!(
            ProjectContentKind::from_publication_tags(&["greg-mod-type:unknown".into()]).is_err()
        );
        assert!(ProjectContentKind::from_publication_tags(&[
            "greg-mod-type:Userlib".into(),
            "greg-mod-type:DataCenterMod".into()
        ])
        .is_err());
    }

    #[test]
    fn legacy_metadata_has_no_build_and_common_build_defaults_to_release() {
        let old: WorkshopMetadata = serde_json::from_str(r#"{"title":"legacy"}"#).unwrap();
        assert!(old.build.is_none());
        assert!(!serde_json::to_value(old)
            .unwrap()
            .as_object()
            .unwrap()
            .contains_key("build"));
        let meta: WorkshopMetadata = serde_json::from_str(
            r#"{"build":{"project":"src/Mod.csproj"},"category":"Automation"}"#,
        )
        .unwrap();
        assert_eq!(meta.build.unwrap().configuration, "Release");
        assert_eq!(meta.category.as_deref(), Some("Automation"));
    }

    #[test]
    fn normalize_repairs_partial_files() {
        let mut meta: WorkshopMetadata = serde_json::from_str(r#"{"title":"T"}"#).unwrap();
        meta.normalize();
        assert_eq!(meta.version, "1.0.0");
        assert_eq!(meta.visibility, "Public");
        assert_eq!(meta.mod_type, "PlacableObject");
    }
    #[test]
    fn normalize_canonicalizes_legacy_type_without_hiding_unknown_values() {
        let mut meta = WorkshopMetadata {
            mod_type: "DataCenterMods".into(),
            ..Default::default()
        };
        meta.normalize();
        assert_eq!(meta.mod_type, "DataCenterMod");
        meta.mod_type = "UnsupportedFutureType".into();
        meta.normalize();
        assert_eq!(meta.mod_type, "UnsupportedFutureType");
        assert!(meta.content_kind().is_none());
    }
}
