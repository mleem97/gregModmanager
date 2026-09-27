//! Game adapters (`IGameAdapter`, Data Center, registry port).

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::Result;

/// What an adapter supports.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct GameAdapterCapabilities {
    pub supports_local_mods: bool,
    pub supports_workshop: bool,
    pub supports_load_order: bool,
}

/// A detected installation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GameInstallation {
    /// Adapter id.
    pub adapter_id: String,
    /// Absolute root path.
    pub root_path: PathBuf,
    /// Executable path, if found.
    pub executable_path: Option<PathBuf>,
}

/// Well-known folders of an installation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GamePathSet {
    /// MelonLoader mods (`{root}/Mods`).
    pub mods: PathBuf,
    /// MelonLoader plugins (`{root}/Plugins`).
    pub plugins: PathBuf,
    /// Shared libraries (`{root}/Plugins/Dependencies`).
    pub user_libraries: PathBuf,
    /// Mod configs (`{root}/UserData/ModCfg`).
    pub mod_cfg: PathBuf,
    /// Savegames (`{root}/UserData/Saves`).
    pub saves: PathBuf,
    /// Native workshop delivery folder.
    pub workshop: PathBuf,
}

/// Compatibility verdict.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GameCompatibility {
    /// Compatible flag.
    pub compatible: bool,
    /// Human-readable reasons.
    pub reasons: Vec<String>,
}

/// One file copy into the game tree.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GameFileInstallRequest {
    /// Absolute source file.
    pub source_path: PathBuf,
    /// Game-relative target (forward slashes, no `..`).
    pub relative_target: String,
}

/// Planned install operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GameInstallPlan {
    /// Adapter id.
    pub adapter_id: String,
    /// Normalized game root.
    pub game_root: PathBuf,
    /// Normalized file requests.
    pub files: Vec<GameFileInstallRequest>,
    /// Warnings (missing sources, ...).
    pub warnings: Vec<String>,
}

/// Planned generic operation (uninstall, launch, ...).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GameOperationPlan {
    /// Adapter id.
    pub adapter_id: String,
    /// Operation name.
    pub operation: String,
    /// Normalized game root.
    pub game_root: PathBuf,
    /// Executable for launches.
    pub executable: Option<PathBuf>,
}

/// Game contract.
pub trait GameAdapter: Send + Sync {
    /// Stable id (`datacenter`).
    fn id(&self) -> &'static str;
    /// Display name.
    fn display_name(&self) -> &'static str;
    /// Steam AppID.
    fn steam_app_id(&self) -> u32;
    /// Capabilities.
    fn capabilities(&self) -> GameAdapterCapabilities;
    /// Detects an installation (explicit root or auto-detect).
    fn detect(&self, candidate_root: Option<&Path>) -> Option<GameInstallation>;
    /// Well-known folders.
    fn paths(&self, game_root: &Path) -> GamePathSet;
    /// Compatibility verdict.
    fn check_compatibility(&self, game_root: &Path) -> GameCompatibility;
    /// Plans file installs (validates targets, warns on missing sources).
    fn plan_install(&self, game_root: &Path, files: Vec<GameFileInstallRequest>)
        -> GameInstallPlan;
    /// Plans removal of owned relative paths.
    fn plan_uninstall(
        &self,
        game_root: &Path,
        owned_relative_paths: Vec<String>,
    ) -> GameOperationPlan;
    /// Plans a game launch.
    fn plan_launch(&self, game_root: &Path, arguments: Vec<String>) -> GameOperationPlan;
}

/// Registry of game adapters.
#[derive(Default)]
pub struct GameAdapterRegistry {
    adapters: Vec<Box<dyn GameAdapter>>,
}

impl GameAdapterRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registry with the built-in adapters.
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register(Box::new(DataCenterGameAdapter));
        registry
    }

    /// Registers an adapter.
    pub fn register(&mut self, adapter: Box<dyn GameAdapter>) {
        self.adapters.push(adapter);
    }

    /// Resolves by adapter id.
    pub fn resolve(&self, id: &str) -> Option<&dyn GameAdapter> {
        self.adapters
            .iter()
            .find(|a| a.id() == id)
            .map(|a| a.as_ref())
    }

    /// Resolves by detecting any adapter at `game_root`.
    pub fn detect(&self, game_root: Option<&Path>) -> Option<(&dyn GameAdapter, GameInstallation)> {
        for adapter in &self.adapters {
            if let Some(installation) = adapter.detect(game_root) {
                return Some((adapter.as_ref(), installation));
            }
        }
        None
    }

    /// All registered adapters.
    pub fn all(&self) -> Vec<&dyn GameAdapter> {
        self.adapters.iter().map(|a| a.as_ref()).collect()
    }
}

/// Data Center adapter (`datacenter`, AppID from settings).
pub struct DataCenterGameAdapter;

impl DataCenterGameAdapter {
    fn unity_data_dir(root: &Path) -> PathBuf {
        root.join("Data Center_Data")
    }

    fn find_executable(root: &Path) -> Option<PathBuf> {
        #[cfg(target_os = "windows")]
        let candidates = ["Data Center.exe", "DataCenter.exe"];
        #[cfg(not(target_os = "windows"))]
        let candidates = ["Data Center.x86_64", "DataCenter.x86_64"];
        // Probe both naming styles on every OS (installs vary).
        let all = [
            "Data Center.exe",
            "DataCenter.exe",
            "Data Center.x86_64",
            "DataCenter.x86_64",
        ];
        let _ = candidates;
        all.iter().map(|name| root.join(name)).find(|p| p.is_file())
    }
}

impl GameAdapter for DataCenterGameAdapter {
    fn id(&self) -> &'static str {
        "datacenter"
    }

    fn display_name(&self) -> &'static str {
        "Data Center"
    }

    fn steam_app_id(&self) -> u32 {
        greg_core::models::settings::DATA_CENTER_APP_ID
    }

    fn capabilities(&self) -> GameAdapterCapabilities {
        GameAdapterCapabilities {
            supports_local_mods: true,
            supports_workshop: true,
            supports_load_order: false,
        }
    }

    fn detect(&self, candidate_root: Option<&Path>) -> Option<GameInstallation> {
        let roots: Vec<PathBuf> = match candidate_root {
            Some(root) => vec![root.to_path_buf()],
            None => {
                let mut roots = Vec::new();
                if let Ok(exe_dir) = std::env::current_exe() {
                    if let Some(dir) = exe_dir.parent() {
                        roots.push(dir.to_path_buf());
                    }
                }
                if let Some(guess) = greg_platform::paths::default_game_dir() {
                    roots.push(guess);
                }
                if let Ok(env) = std::env::var("GREG_GAME_ROOT") {
                    roots.push(PathBuf::from(env));
                }
                roots
            }
        };
        for root in roots {
            if !root.is_dir() {
                continue;
            }
            let looks_like_game = root.join("Mods").is_dir()
                || root.join("Plugins").is_dir()
                || Self::find_executable(&root).is_some();
            if looks_like_game {
                return Some(GameInstallation {
                    adapter_id: self.id().to_string(),
                    executable_path: Self::find_executable(&root),
                    root_path: root,
                });
            }
        }
        None
    }

    fn paths(&self, game_root: &Path) -> GamePathSet {
        GamePathSet {
            mods: game_root.join("Mods"),
            plugins: game_root.join("Plugins"),
            user_libraries: game_root.join("Plugins").join("Dependencies"),
            mod_cfg: game_root.join("UserData").join("ModCfg"),
            saves: game_root.join("UserData").join("Saves"),
            workshop: Self::unity_data_dir(game_root)
                .join("StreamingAssets")
                .join("Mods"),
        }
    }

    fn check_compatibility(&self, game_root: &Path) -> GameCompatibility {
        let mut reasons = Vec::new();
        if Self::find_executable(game_root).is_none() {
            reasons.push("Game executable not found.".to_string());
        }
        GameCompatibility {
            compatible: reasons.is_empty(),
            reasons,
        }
    }

    fn plan_install(
        &self,
        game_root: &Path,
        files: Vec<GameFileInstallRequest>,
    ) -> GameInstallPlan {
        let mut normalized = Vec::new();
        let mut warnings = Vec::new();
        for file in files {
            let target = file.relative_target.replace('\\', "/");
            if target
                .split('/')
                .any(|part| part == ".." || part == "." || part.contains('\0') || part.is_empty())
            {
                warnings.push(format!("Blocked unsafe target: {}", file.relative_target));
                continue;
            }
            if !file.source_path.is_file() {
                warnings.push(format!("Source missing: {}", file.source_path.display()));
            }
            normalized.push(GameFileInstallRequest {
                source_path: file.source_path,
                relative_target: target,
            });
        }
        GameInstallPlan {
            adapter_id: self.id().to_string(),
            game_root: normalize_root(game_root),
            files: normalized,
            warnings,
        }
    }

    fn plan_uninstall(
        &self,
        game_root: &Path,
        owned_relative_paths: Vec<String>,
    ) -> GameOperationPlan {
        let _ = owned_relative_paths;
        GameOperationPlan {
            adapter_id: self.id().to_string(),
            operation: "uninstall".to_string(),
            game_root: normalize_root(game_root),
            executable: None,
        }
    }

    fn plan_launch(&self, game_root: &Path, arguments: Vec<String>) -> GameOperationPlan {
        let _ = arguments;
        GameOperationPlan {
            adapter_id: self.id().to_string(),
            operation: "launch".to_string(),
            game_root: normalize_root(game_root),
            executable: Self::find_executable(game_root),
        }
    }
}

fn normalize_root(root: &Path) -> PathBuf {
    root.to_path_buf()
}

/// Applies an install plan (copies files). Returns the installed targets.
pub fn apply_install_plan(plan: &GameInstallPlan) -> Result<Vec<PathBuf>> {
    let mut installed = Vec::new();
    for file in &plan.files {
        let target = plan.game_root.join(&file.relative_target);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(crate::error::io_err)?;
        }
        std::fs::copy(&file.source_path, &target).map_err(crate::error::io_err)?;
        installed.push(target);
    }
    Ok(installed)
}
