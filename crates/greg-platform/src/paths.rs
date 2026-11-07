//! Known paths: app data, logs, workspace fallbacks.

use std::path::PathBuf;

/// `gregModmanager` directory inside the OS local-app-data folder.
pub fn app_data_dir() -> Option<PathBuf> {
    dirs::data_local_dir().map(|d| d.join("gregModmanager"))
}

/// Default manager base: `$HOME/GregModmanager` on every desktop platform.
pub fn default_manager_base_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join("GregModmanager"))
}

/// Resolve the manager base without opening the settings database.
/// The OS app-data locator is intentionally tiny: settings live in the base.
pub fn manager_base_dir() -> PathBuf {
    if let Ok(value) = std::env::var("GREG_WORKSPACE") {
        if !value.trim().is_empty() {
            return PathBuf::from(value.trim());
        }
    }
    if let Some(data) = app_data_dir() {
        if let Ok(value) = std::fs::read_to_string(data.join("workspace.path")) {
            if !value.trim().is_empty() {
                return PathBuf::from(value.trim());
            }
        }
        // Read the previous JSON store only as a bootstrap/import source.
        if let Ok(text) = std::fs::read_to_string(data.join("preferences.json")) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                if let Some(value) = json.get("CustomWorkspacePath").and_then(|v| v.as_str()) {
                    if !value.trim().is_empty() {
                        return PathBuf::from(value.trim());
                    }
                }
            }
        }
    }
    default_manager_base_dir().unwrap_or_else(|| std::env::temp_dir().join("GregModmanager"))
}

/// Directory holding daily logs inside the resolved manager base.
pub fn log_dir() -> PathBuf {
    manager_base_dir().join("Logs")
}

/// True when a probe file can be created and removed.
pub fn is_writable(dir: &std::path::Path) -> bool {
    let probe = dir.join(format!(".write-test-{}.tmp", std::process::id()));
    std::fs::write(&probe, b"ok")
        .and_then(|()| std::fs::remove_file(&probe))
        .is_ok()
}

/// Legacy workspace fallback: `~/DataCenterWS`.
pub fn legacy_fallback_workspace() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join("DataCenterWS"))
}

/// User NuGet global-packages cache (`~/.nuget/packages`).
pub fn nuget_packages_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".nuget").join("packages"))
}

/// Default Steam library guess (`<x86>/Steam/steamapps/common/Data Center`).
/// Real resolution happens via the Steam client in `greg-steam`.
#[cfg(target_os = "windows")]
pub fn default_game_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|d| {
        d.join("Steam")
            .join("steamapps")
            .join("common")
            .join("Data Center")
    })
}

/// Default Steam library guess on Linux.
#[cfg(target_os = "linux")]
pub fn default_game_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| {
        h.join(".steam")
            .join("steam")
            .join("steamapps")
            .join("common")
            .join("Data Center")
    })
}

/// Default Steam library guess on macOS.
///
/// Steam installs its standard library below the user's Application Support
/// directory on macOS, rather than below `~/.steam` as on Linux.
#[cfg(target_os = "macos")]
pub fn default_game_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| {
        h.join("Library")
            .join("Application Support")
            .join("Steam")
            .join("steamapps")
            .join("common")
            .join("Data Center")
    })
}
