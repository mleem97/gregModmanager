//! Runtime installers: MelonLoader + SteamModfix from GitHub releases.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::error::{LoaderError, Result};

/// MelonLoader install state.
#[derive(Debug, Clone)]
pub struct MelonLoaderInstallState {
    /// Assembly present.
    pub installed: bool,
    /// Detected version, if parseable.
    pub version: Option<String>,
    /// Assembly path.
    pub assembly_path: Option<PathBuf>,
}

/// Outcome of an ensure-install run.
#[derive(Debug, Clone)]
pub struct InstallOutcome {
    /// Success flag.
    pub success: bool,
    /// Changed anything.
    pub changed: bool,
    /// Human-readable message.
    pub message: String,
}

/// Detects the MelonLoader install below `game_root`.
pub fn detect_melon_loader(game_root: &Path) -> MelonLoaderInstallState {
    let assembly = game_root
        .join("MelonLoader")
        .join("net6")
        .join("MelonLoader.dll");
    if !assembly.is_file() {
        // Legacy root-level assembly counts as installed-but-old.
        let legacy = game_root.join("MelonLoader").join("MelonLoader.dll");
        return MelonLoaderInstallState {
            installed: legacy.is_file(),
            version: None,
            assembly_path: legacy.is_file().then_some(legacy),
        };
    }
    MelonLoaderInstallState {
        installed: true,
        version: None,
        assembly_path: Some(assembly),
    }
}

/// Preferred MelonLoader asset names per OS/arch (mirrors the C# order).
pub fn preferred_melon_loader_assets() -> Vec<&'static str> {
    #[cfg(target_os = "windows")]
    {
        vec!["MelonLoader.x64.zip", "MelonLoader.x86.zip"]
    }
    #[cfg(target_os = "macos")]
    {
        #[cfg(target_arch = "aarch64")]
        {
            vec![
                "MelonLoader.MacOS.arm64.zip",
                "MelonLoader.MacOS.x64.zip",
                "MelonLoader.MacOS.zip",
            ]
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            vec![
                "MelonLoader.MacOS.x64.zip",
                "MelonLoader.MacOS.arm64.zip",
                "MelonLoader.MacOS.zip",
            ]
        }
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        #[cfg(target_arch = "aarch64")]
        {
            vec!["MelonLoader.Linux.arm64.zip", "MelonLoader.Linux.x64.zip"]
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            vec!["MelonLoader.Linux.x64.zip", "MelonLoader.Linux.x86_64.zip"]
        }
    }
}

/// GitHub release asset (`name` + `browser_download_url`).
#[derive(Debug, Clone)]
pub struct ReleaseAsset {
    /// File name.
    pub name: String,
    /// Download URL.
    pub url: String,
}

/// Fetches `{api_url}/latest` (GitHub releases API shape) and picks the
/// first asset matching `preferred` names. Returns `(version, url)`.
pub async fn latest_github_asset(
    api_url: &str,
    preferred: &[&str],
    cancel: &AtomicBool,
) -> Result<(String, String)> {
    if cancel.load(Ordering::Relaxed) {
        return Err(LoaderError::Cancelled);
    }
    let text = reqwest::Client::new()
        .get(api_url)
        .header("User-Agent", "gregmodmanager")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| LoaderError::Network(e.to_string()))?
        .error_for_status()
        .map_err(|e| LoaderError::Network(e.to_string()))?
        .text()
        .await
        .map_err(|e| LoaderError::Network(e.to_string()))?;
    let root: serde_json::Value = serde_json::from_str(&text).map_err(crate::error::json_err)?;
    let tag = root
        .get("tag_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| LoaderError::InvalidArtefact("release has no tag_name".into()))?;
    let version = tag.trim_start_matches(['v', 'V']).to_string();
    if semver_parse_lenient(&version).is_none() {
        return Err(LoaderError::InvalidArtefact(format!(
            "invalid release version: {tag}"
        )));
    }
    let assets: Vec<ReleaseAsset> = root
        .get("assets")
        .and_then(|v| v.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|a| {
                    Some(ReleaseAsset {
                        name: a.get("name")?.as_str()?.to_string(),
                        url: a.get("browser_download_url")?.as_str()?.to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    for wanted in preferred {
        if let Some(asset) = assets
            .iter()
            .find(|a| a.name.eq_ignore_ascii_case(wanted))
            .filter(|a| !a.url.trim().is_empty())
        {
            return Ok((version, asset.url.clone()));
        }
    }
    Err(LoaderError::InvalidArtefact(
        "no compatible release asset found".into(),
    ))
}

fn semver_parse_lenient(version: &str) -> Option<(u64, u64, u64)> {
    let core = version.split(['-', '+']).next()?;
    let mut parts = core.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((major, minor, patch))
}

/// Downloads a zip archive and extracts it zip-slip-safe into `destination`.
pub async fn download_and_extract_zip(
    url: &str,
    destination: &Path,
    cancel: &AtomicBool,
) -> Result<()> {
    if cancel.load(Ordering::Relaxed) {
        return Err(LoaderError::Cancelled);
    }
    let bytes = reqwest::Client::new()
        .get(url)
        .header("User-Agent", "gregmodmanager")
        .send()
        .await
        .map_err(|e| LoaderError::Network(e.to_string()))?
        .error_for_status()
        .map_err(|e| LoaderError::Network(e.to_string()))?
        .bytes()
        .await
        .map_err(|e| LoaderError::Network(e.to_string()))?;
    if cancel.load(Ordering::Relaxed) {
        return Err(LoaderError::Cancelled);
    }
    extract_zip_safely(&bytes, destination)
}

/// Zip-slip-safe extraction (rejects absolute paths, `..`, symlinks).
pub fn extract_zip_safely(bytes: &[u8], destination: &Path) -> Result<()> {
    use std::io::Cursor;
    std::fs::create_dir_all(destination).map_err(crate::error::io_err)?;
    let reader = Cursor::new(bytes);
    let mut zip = zip::ZipArchive::new(reader)
        .map_err(|e| LoaderError::InvalidArtefact(format!("unreadable zip: {e}")))?;
    let base = destination
        .canonicalize()
        .unwrap_or_else(|_| destination.to_path_buf());
    for i in 0..zip.len() {
        let mut entry = zip
            .by_index(i)
            .map_err(|e| LoaderError::InvalidArtefact(format!("bad zip entry: {e}")))?;
        let name = entry.name().to_string();
        if name.starts_with('/') || name.starts_with('\\') || name.contains("..") {
            return Err(LoaderError::InvalidArtefact(format!(
                "blocked unsafe zip path: {name}"
            )));
        }
        if entry.is_symlink() {
            return Err(LoaderError::InvalidArtefact(format!(
                "blocked zip symlink: {name}"
            )));
        }
        let target = destination.join(&name);
        if entry.is_dir() {
            std::fs::create_dir_all(&target).map_err(crate::error::io_err)?;
            continue;
        }
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(crate::error::io_err)?;
        }
        let mut out = std::fs::File::create(&target).map_err(crate::error::io_err)?;
        std::io::copy(&mut entry, &mut out).map_err(crate::error::io_err)?;
        drop(out);
        if let Ok(canonical) = target.canonicalize() {
            if !canonical.starts_with(&base) {
                let _ = std::fs::remove_file(&target);
                return Err(LoaderError::InvalidArtefact(format!(
                    "blocked zip escape: {name}"
                )));
            }
        }
    }
    Ok(())
}

/// Ensures the current MelonLoader below `game_root`.
pub async fn ensure_melon_loader(
    game_root: &Path,
    feed_api_url: &str,
    progress: &dyn Fn(&str),
    cancel: &AtomicBool,
) -> InstallOutcome {
    let state = detect_melon_loader(game_root);
    if state.installed {
        return InstallOutcome {
            success: true,
            changed: false,
            message: "MelonLoader is already installed.".into(),
        };
    }
    progress("Fetching latest MelonLoader release...");
    let preferred = preferred_melon_loader_assets();
    let (version, url) = match latest_github_asset(feed_api_url, &preferred, cancel).await {
        Ok(v) => v,
        Err(e) => {
            return InstallOutcome {
                success: false,
                changed: false,
                message: format!("MelonLoader download failed: {e}"),
            }
        }
    };
    progress(&format!("Downloading MelonLoader {version}..."));
    match download_and_extract_zip(&url, game_root, cancel).await {
        Ok(()) => InstallOutcome {
            success: true,
            changed: true,
            message: format!("MelonLoader {version} installed."),
        },
        Err(LoaderError::Cancelled) => InstallOutcome {
            success: false,
            changed: false,
            message: "MelonLoader installation was cancelled.".into(),
        },
        Err(e) => InstallOutcome {
            success: false,
            changed: false,
            message: format!("MelonLoader could not be installed: {e}"),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_missing() {
        let dir = std::env::temp_dir().join(format!(
            "greg-ml-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let state = detect_melon_loader(&dir);
        assert!(!state.installed);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn picks_preferred_asset_names() {
        assert!(!preferred_melon_loader_assets().is_empty());
    }

    #[test]
    fn blocks_zip_escape() {
        use std::io::Write as _;
        let mut buf = Vec::new();
        {
            let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
            zip.start_file("../evil.txt", zip::write::SimpleFileOptions::default())
                .unwrap();
            zip.write_all(b"evil").unwrap();
            zip.finish().unwrap();
        }
        let dir = std::env::temp_dir().join(format!(
            "greg-ziptest-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        assert!(extract_zip_safely(&buf, &dir).is_err());
        std::fs::remove_dir_all(&dir).ok();
    }
}
