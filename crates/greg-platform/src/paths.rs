//! Known paths: app data, logs, workspace fallbacks.

use std::path::PathBuf;

/// `gregModmanager` directory inside the OS local-app-data folder.
pub fn app_data_dir() -> Option<PathBuf> {
    dirs::data_local_dir().map(|d| d.join("gregModmanager"))
}

/// Directory holding `app-YYYYMMDD.log`.
///
/// Prefers `<exe-dir>/logs` when writable (matches the C# behavior),
/// then the app-data dir, then the temp dir.
pub fn log_dir() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let candidate = dir.join("logs");
            if std::fs::create_dir_all(&candidate).is_ok() && is_writable(&candidate) {
                return candidate;
            }
        }
    }
    if let Some(dir) = app_data_dir() {
        let logs = dir.join("logs");
        if std::fs::create_dir_all(&logs).is_ok() {
            return logs;
        }
    }
    std::env::temp_dir().join("gregModmanager-logs")
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

/// Default Steam library guess on Unix.
#[cfg(not(target_os = "windows"))]
pub fn default_game_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| {
        h.join(".steam")
            .join("steam")
            .join("steamapps")
            .join("common")
            .join("Data Center")
    })
}
