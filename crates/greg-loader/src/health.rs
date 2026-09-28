//! Installation health checks (`ModDependencyService` port, read-only).

use std::path::{Path, PathBuf};

use crate::content::{DependencyCheck, DependencyStatus};
use crate::error::Result;
use crate::game::GameAdapterRegistry;

/// Dependency probe over a game root.
pub struct ModDependencyService {
    game_root: Option<PathBuf>,
}

impl ModDependencyService {
    /// Creates the probe (root resolved lazily via adapters).
    pub fn new(game_root: Option<PathBuf>) -> Self {
        Self { game_root }
    }

    /// Explicit root (tests, CLI overrides).
    pub fn with_root(root: PathBuf) -> Self {
        Self {
            game_root: Some(root),
        }
    }

    /// Resolved game root, if any.
    pub fn game_root(&self) -> Option<PathBuf> {
        if let Some(root) = &self.game_root {
            if root.is_dir() {
                return Some(root.clone());
            }
        }
        GameAdapterRegistry::with_defaults()
            .detect(None)
            .map(|(_, installation)| installation.root_path)
    }

    /// MelonLoader base dir.
    pub fn melon_loader_dir(root: &Path) -> PathBuf {
        root.join("MelonLoader")
    }

    /// MelonLoader .NET 6 dir.
    pub fn melon_loader_net6_dir(root: &Path) -> PathBuf {
        Self::melon_loader_dir(root).join("net6")
    }

    /// Il2Cpp interop assemblies dir.
    pub fn il2cpp_assemblies_dir(root: &Path) -> PathBuf {
        Self::melon_loader_dir(root).join("Il2CppAssemblies")
    }

    /// greg plugins dir (`{root}/greg/Plugins`).
    pub fn greg_plugins_dir(root: &Path) -> PathBuf {
        root.join("greg").join("Plugins")
    }

    /// Mod config dir (`{root}/UserData/ModCfg`).
    pub fn mod_cfg_dir(root: &Path) -> PathBuf {
        root.join("UserData").join("ModCfg")
    }

    /// Runs all read-only checks, ordered.
    pub fn run_checks(&self) -> Result<Vec<DependencyCheck>> {
        let mut results = Vec::new();
        let Some(root) = self.game_root() else {
            results.push(DependencyCheck {
                label: "Data Center".into(),
                status: DependencyStatus::Missing,
                detail: "Game folder not found. Start Steam and install Data Center.".into(),
            });
            return Ok(results);
        };
        results.push(DependencyCheck {
            label: "Data Center".into(),
            status: DependencyStatus::Ok,
            detail: root.to_string_lossy().to_string(),
        });
        check_melon_loader(&root, &mut results);
        check_il2cpp(&root, &mut results);
        check_greg_framework(&root, &mut results);
        check_greg_plugins(&root, &mut results);
        check_mod_cfg(&root, &mut results);
        let sources = crate::discovery::discover(&root);
        results.push(DependencyCheck {
            label: "MelonLoader sources".into(),
            status: if sources.is_empty() {
                DependencyStatus::Warning
            } else {
                DependencyStatus::Ok
            },
            detail: if sources.is_empty() {
                "No external MelonLoader sources detected.".into()
            } else {
                format!("{} source folder(s) detected.", sources.len())
            },
        });
        Ok(results)
    }

    /// Creates the ModCfg dir when missing.
    pub fn ensure_mod_cfg(&self) -> Result<()> {
        if let Some(root) = self.game_root() {
            std::fs::create_dir_all(Self::mod_cfg_dir(&root)).map_err(crate::error::io_err)?;
        }
        Ok(())
    }

    /// Creates the greg plugins dir when missing.
    pub fn ensure_greg_plugins(&self) -> Result<()> {
        if let Some(root) = self.game_root() {
            std::fs::create_dir_all(Self::greg_plugins_dir(&root)).map_err(crate::error::io_err)?;
        }
        Ok(())
    }
}

fn check_melon_loader(root: &Path, results: &mut Vec<DependencyCheck>) {
    let dll = ModDependencyService::melon_loader_net6_dir(root).join("MelonLoader.dll");
    results.push(if dll.is_file() {
        DependencyCheck {
            label: "MelonLoader".into(),
            status: DependencyStatus::Ok,
            detail: ModDependencyService::melon_loader_net6_dir(root)
                .to_string_lossy()
                .to_string(),
        }
    } else {
        DependencyCheck {
            label: "MelonLoader".into(),
            status: DependencyStatus::Missing,
            detail: "MelonLoader is not installed.".into(),
        }
    });
}

fn check_il2cpp(root: &Path, results: &mut Vec<DependencyCheck>) {
    let asm = ModDependencyService::il2cpp_assemblies_dir(root).join("Assembly-CSharp.dll");
    results.push(if asm.is_file() {
        DependencyCheck {
            label: "Il2Cpp Interop".into(),
            status: DependencyStatus::Ok,
            detail: ModDependencyService::il2cpp_assemblies_dir(root)
                .to_string_lossy()
                .to_string(),
        }
    } else if ModDependencyService::melon_loader_net6_dir(root).is_dir() {
        DependencyCheck {
            label: "Il2Cpp Interop".into(),
            status: DependencyStatus::Warning,
            detail: "Assembly-CSharp.dll is missing. Start the game once with MelonLoader.".into(),
        }
    } else {
        DependencyCheck {
            label: "Il2Cpp Interop".into(),
            status: DependencyStatus::Missing,
            detail: "Install MelonLoader first, then start the game once.".into(),
        }
    });
}

fn check_greg_framework(root: &Path, results: &mut Vec<DependencyCheck>) {
    let dll = root.join("Mods").join("gregCore.dll");
    results.push(if dll.is_file() {
        DependencyCheck {
            label: "gregCoreModFramework".into(),
            status: DependencyStatus::Ok,
            detail: dll.to_string_lossy().to_string(),
        }
    } else {
        DependencyCheck {
            label: "gregCoreModFramework".into(),
            status: DependencyStatus::Missing,
            detail: "gregCore.dll is missing under Mods/.".into(),
        }
    });
}

fn check_greg_plugins(root: &Path, results: &mut Vec<DependencyCheck>) {
    let dir = ModDependencyService::greg_plugins_dir(root);
    results.push(if dir.is_dir() {
        let count = std::fs::read_dir(&dir)
            .map(|entries| {
                entries
                    .flatten()
                    .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("dll"))
                    .count()
            })
            .unwrap_or(0);
        DependencyCheck {
            label: "greg Plugins".into(),
            status: if count > 0 {
                DependencyStatus::Ok
            } else {
                DependencyStatus::Warning
            },
            detail: if count > 0 {
                format!("{count} plugin DLL(s) under greg/Plugins/")
            } else {
                "greg/Plugins/ exists but holds no DLLs.".into()
            },
        }
    } else {
        DependencyCheck {
            label: "greg Plugins".into(),
            status: DependencyStatus::Warning,
            detail: "greg/Plugins/ does not exist yet.".into(),
        }
    });
}

fn check_mod_cfg(root: &Path, results: &mut Vec<DependencyCheck>) {
    let dir = ModDependencyService::mod_cfg_dir(root);
    results.push(if dir.is_dir() {
        DependencyCheck {
            label: "Mod configuration".into(),
            status: DependencyStatus::Ok,
            detail: dir.to_string_lossy().to_string(),
        }
    } else {
        DependencyCheck {
            label: "Mod configuration".into(),
            status: DependencyStatus::Warning,
            detail: "UserData/ModCfg/ is missing (created on first mod start).".into(),
        }
    });
}
