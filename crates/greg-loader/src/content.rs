//! Content types, native configs and plugin packages.

use serde::{Deserialize, Serialize};

/// Where a synced item belongs (`ModContentType` port).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModContentType {
    /// Native shop/static content.
    #[default]
    PlacableObject,
    /// MelonLoader plugin DLL.
    MelonloaderPlugin,
    /// Shared user library.
    Userlib,
    /// MelonLoader mod (Data Center).
    DataCenterMod,
}

impl ModContentType {
    /// Parses with the C# aliases (`MelonLoaderPlugin`, `DataCenterMods`).
    pub fn parse(value: &str) -> Self {
        match value.trim() {
            "MelonloaderPlugin" | "MelonLoaderPlugin" => Self::MelonloaderPlugin,
            "Userlib" => Self::Userlib,
            "DataCenterMod" | "DataCenterMods" => Self::DataCenterMod,
            _ => Self::PlacableObject,
        }
    }

    /// Canonical name.
    pub fn name(self) -> &'static str {
        match self {
            Self::PlacableObject => "PlacableObject",
            Self::MelonloaderPlugin => "MelonloaderPlugin",
            Self::Userlib => "Userlib",
            Self::DataCenterMod => "DataCenterMod",
        }
    }
}

/// New-project template kinds.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkshopTemplateKind {
    /// Native decoration (`config.json`).
    #[default]
    VanillaObjectDecoration,
    /// MelonLoader mod.
    ModdedMelonLoader,
    /// gregCore framework mod.
    ModdedGregCore,
    /// UXML UI override.
    UxmlUiOverride,
    /// Standalone 3D model asset.
    Standalone3DModel,
    /// Standalone texture asset.
    StandaloneTexture,
    /// Standalone audio asset.
    StandaloneAudio,
}

/// Asset-store metadata (`modstore.meta.json`).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct AssetModMetadata {
    pub asset_type: String,
    pub tags: Vec<String>,
    pub is_standalone: bool,
}

/// Optional runtime options (`content/modconfig.json`).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct ModOptionsConfigFile {
    pub schema_version: u32,
    pub mod_kind: String,
    pub notes: String,
    pub settings: std::collections::HashMap<String, serde_json::Value>,
}

/// Native game mod definition (`content/config.json`).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct NativeModConfig {
    pub mod_name: String,
    pub shop_items: Vec<NativeShopItem>,
    pub static_items: Vec<NativeStaticItem>,
    pub dlls: Vec<NativeDllRef>,
}

/// Purchasable native object.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NativeShopItem {
    #[serde(default)]
    pub item_name: String,
    #[serde(default)]
    pub price: i64,
    #[serde(default)]
    pub xp_to_unlock: i64,
    #[serde(default)]
    pub size_in_u: i64,
    #[serde(default)]
    pub mass: f64,
    #[serde(default = "default_scale")]
    pub model_scale: f64,
    #[serde(default = "default_collider_size")]
    pub collider_size: [f64; 3],
    #[serde(default)]
    pub collider_center: [f64; 3],
    #[serde(default)]
    pub model_file: String,
    #[serde(default)]
    pub texture_file: String,
    #[serde(default)]
    pub icon_file: String,
    #[serde(default)]
    pub object_type: i64,
}

fn default_scale() -> f64 {
    1.0
}
fn default_collider_size() -> [f64; 3] {
    [0.5, 0.5, 0.5]
}

/// Statically placed native object.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct NativeStaticItem {
    pub item_name: String,
    pub model_file: String,
    pub texture_file: String,
    pub model_scale: f64,
    pub collider_size: [f64; 3],
    pub collider_center: [f64; 3],
    pub position: [f64; 3],
    pub rotation: [f64; 3],
}

/// Native plugin DLL reference (experimental game path).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct NativeDllRef {
    pub file_name: String,
    pub entry_class: String,
}

/// Distributable greg plugin artefact.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PluginPackageInfo {
    /// Channel (`stable`, `github`, `beta`).
    pub channel: String,
    /// Plugin id / file stem.
    pub name: String,
    /// Version string.
    pub version: String,
    /// Download URL (when remote).
    pub download_url: String,
    /// Target directory override.
    pub target_dir: String,
}

/// Health-check outcome (`DependencyCheckResult` port).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyCheck {
    /// Component label.
    pub label: String,
    /// Status.
    pub status: DependencyStatus,
    /// Detail or path.
    pub detail: String,
}

/// Health states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyStatus {
    /// Present and fine.
    Ok,
    /// Present with caveats.
    Warning,
    /// Missing.
    Missing,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_with_aliases() {
        assert_eq!(
            ModContentType::parse("MelonLoaderPlugin"),
            ModContentType::MelonloaderPlugin
        );
        assert_eq!(
            ModContentType::parse("DataCenterMods"),
            ModContentType::DataCenterMod
        );
        assert_eq!(ModContentType::parse("Userlib"), ModContentType::Userlib);
        assert_eq!(ModContentType::parse("???"), ModContentType::PlacableObject);
    }

    #[test]
    fn native_config_defaults() {
        let cfg: NativeModConfig = serde_json::from_str(r#"{"shopItems": [{}]}"#).unwrap();
        assert_eq!(cfg.shop_items[0].model_scale, 1.0);
        assert_eq!(cfg.shop_items[0].collider_size, [0.5, 0.5, 0.5]);
    }
}
