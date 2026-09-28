//! Atomic Modstore installs (`ModStoreUpdateService` install part).
//!
//! Download → size + SHA-256 verify → stage (zip-slip-safe) → atomic swap
//! with backup → manifest update. Any failure rolls back.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt as _;

use crate::error::{ModStoreError, Result};
use crate::models::{
    ModStoreCatalogItem, ModStoreInstallManifest, ModStoreInstallResult, ModStoreInstalledEntry,
    ModStoreRelease, ModStoreUpdate,
};

/// Manifest location: `<gameRoot>/.greg-modmanager/modstore-installed.json`.
pub fn manifest_path(game_root: &Path) -> PathBuf {
    game_root
        .join(".greg-modmanager")
        .join("modstore-installed.json")
}

/// Loads the install manifest (empty when missing/corrupt).
pub fn load_manifest(game_root: &Path) -> ModStoreInstallManifest {
    let path = manifest_path(game_root);
    if !path.is_file() {
        return ModStoreInstallManifest {
            schema_version: 1,
            installed: Vec::new(),
        };
    }
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or(ModStoreInstallManifest {
            schema_version: 1,
            installed: Vec::new(),
        })
}

fn save_manifest_entry(game_root: &Path, entry: ModStoreInstalledEntry) {
    let mut manifest = load_manifest(game_root);
    manifest.installed.retain(|e| e.mod_id != entry.mod_id);
    manifest.installed.push(entry);
    manifest.schema_version = 1;
    if let Some(dir) = manifest_path(game_root).parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Ok(text) = serde_json::to_string_pretty(&manifest) {
        let _ = std::fs::write(manifest_path(game_root), text);
    }
}

/// Installer with trusted base hosts.
pub struct ModStoreInstaller {
    http: reqwest::Client,
    trusted_hosts: Vec<String>,
}

impl ModStoreInstaller {
    /// Creates the installer. `base_urls` define the trusted download hosts.
    pub fn new(base_urls: &[String]) -> Self {
        let trusted_hosts = base_urls
            .iter()
            .filter_map(|u| url::Url::parse(u).ok())
            .filter_map(|u| u.host_str().map(|h| h.to_lowercase()))
            .collect();
        Self {
            http: reqwest::Client::new(),
            trusted_hosts,
        }
    }

    /// Installs a catalog item.
    pub async fn install_item(
        &self,
        item: &ModStoreCatalogItem,
        game_root: &Path,
        progress: &dyn Fn(u8),
        cancel: &AtomicBool,
    ) -> ModStoreInstallResult {
        self.install_release(
            &item.id,
            &item.title,
            &item.release,
            game_root,
            progress,
            cancel,
        )
        .await
    }

    /// Installs an update.
    pub async fn install_update(
        &self,
        update: &ModStoreUpdate,
        game_root: &Path,
        progress: &dyn Fn(u8),
        cancel: &AtomicBool,
    ) -> ModStoreInstallResult {
        self.install_release(
            &update.mod_id,
            &update.title,
            &update.release,
            game_root,
            progress,
            cancel,
        )
        .await
    }

    /// True for `https://` URLs on a trusted host.
    pub fn is_trusted_download_url(&self, value: &str) -> bool {
        let Ok(url) = url::Url::parse(value) else {
            return false;
        };
        if url.scheme() != "https" {
            return false;
        }
        match url.host_str() {
            Some(host) => self.trusted_hosts.iter().any(|h| h == &host.to_lowercase()),
            None => false,
        }
    }

    async fn install_release(
        &self,
        mod_id: &str,
        title: &str,
        release: &ModStoreRelease,
        game_root: &Path,
        progress: &dyn Fn(u8),
        cancel: &AtomicBool,
    ) -> ModStoreInstallResult {
        match self
            .install_release_inner(mod_id, title, release, game_root, progress, cancel)
            .await
        {
            Ok(path) => ModStoreInstallResult::ok(
                format!("{title} was installed at version {}.", release.version),
                path.to_string_lossy(),
            ),
            Err(e) => ModStoreInstallResult::fail(format!("Update installation failed: {e}")),
        }
    }

    async fn install_release_inner(
        &self,
        mod_id: &str,
        title: &str,
        release: &ModStoreRelease,
        game_root: &Path,
        progress: &dyn Fn(u8),
        cancel: &AtomicBool,
    ) -> Result<PathBuf> {
        if !game_root.is_dir() {
            return Err(ModStoreError::InvalidArtefact(
                "The Data Center game folder is not configured.".into(),
            ));
        }
        let checksum = release.checksum_sha256.as_deref().unwrap_or("");
        if checksum.len() != 64 || !checksum.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ModStoreError::InvalidArtefact(
                "The release has no valid SHA-256 checksum and was not installed.".into(),
            ));
        }
        if !self.is_trusted_download_url(&release.download_url) {
            return Err(ModStoreError::InvalidArtefact(
                "The release download URL does not belong to a configured Mod Store host.".into(),
            ));
        }

        let mods_root = game_root.join("Mods").join("GregStore");
        std::fs::create_dir_all(&mods_root)
            .map_err(|e| ModStoreError::Other(format!("cannot create {mods_root:?}: {e}")))?;
        let safe_id: String = mod_id
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        let nonce: u64 = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);
        let destination = mods_root.join(&safe_id);
        let staging = mods_root.join(format!(".{safe_id}-{nonce:x}.staging"));
        let backup = mods_root.join(format!(".{safe_id}-{nonce:x}.backup"));
        let artifact = std::env::temp_dir().join(format!(
            "gregmod-{}-{nonce:x}.download",
            safe_file_stem(&release.id)
        ));

        // 1. Download with progress.
        let response = self
            .http
            .get(&release.download_url)
            .send()
            .await
            .map_err(|e| ModStoreError::Request(e.to_string()))?;
        let mut response = response
            .error_for_status()
            .map_err(|e| ModStoreError::Request(e.to_string()))?;
        let expected = response
            .content_length()
            .or(release.file_size.map(|v| v as u64));
        let mut file = tokio::fs::File::create(&artifact)
            .await
            .map_err(|e| ModStoreError::Other(e.to_string()))?;
        let mut copied: u64 = 0;
        loop {
            if cancel.load(Ordering::Relaxed) {
                let _ = tokio::fs::remove_file(&artifact).await;
                return Err(ModStoreError::Cancelled);
            }
            match response
                .chunk()
                .await
                .map_err(|e| ModStoreError::Request(e.to_string()))?
            {
                Some(chunk) => {
                    file.write_all(&chunk)
                        .await
                        .map_err(|e| ModStoreError::Other(e.to_string()))?;
                    copied += chunk.len() as u64;
                    if let Some(total) = expected {
                        let pct = copied
                            .saturating_mul(90)
                            .checked_div(total)
                            .unwrap_or(0)
                            .min(90) as u8;
                        progress(pct);
                    }
                }
                None => break,
            }
        }
        file.flush()
            .await
            .map_err(|e| ModStoreError::Other(e.to_string()))?;
        drop(file);

        // 2. Size + SHA-256 verify.
        let meta = std::fs::metadata(&artifact)
            .map_err(|e| ModStoreError::Other(format!("download unreadable: {e}")))?;
        if let Some(expected_size) = release.file_size {
            if expected_size > 0 && meta.len() != expected_size as u64 {
                let _ = std::fs::remove_file(&artifact);
                return Err(ModStoreError::InvalidArtefact(
                    "Downloaded file size does not match release metadata.".into(),
                ));
            }
        }
        let data = std::fs::read(&artifact).map_err(|e| ModStoreError::Other(e.to_string()))?;
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let digest = hex::encode(hasher.finalize());
        if !digest.eq_ignore_ascii_case(checksum) {
            let _ = std::fs::remove_file(&artifact);
            return Err(ModStoreError::InvalidArtefact(
                "SHA-256 checksum verification failed.".into(),
            ));
        }

        // 3. Stage (zip-slip-safe) then atomic swap with backup.
        let outcome = stage_and_swap(
            &artifact,
            &release.file_name,
            &staging,
            &destination,
            &backup,
        );
        let _ = std::fs::remove_file(&artifact);
        outcome?;
        progress(95);

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        save_manifest_entry(
            game_root,
            ModStoreInstalledEntry {
                mod_id: mod_id.to_string(),
                title: title.to_string(),
                version: release.version.clone(),
                release_id: release.id.clone(),
                install_path: destination.to_string_lossy().to_string(),
                installed_at: now.to_string(),
            },
        );
        progress(100);
        Ok(destination)
    }
}

fn safe_file_stem(id: &str) -> String {
    id.chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .take(32)
        .collect()
}

/// Stages the artefact and atomically swaps it into `destination`.
fn stage_and_swap(
    artifact: &Path,
    file_name: &str,
    staging: &Path,
    destination: &Path,
    backup: &Path,
) -> Result<()> {
    let _ = std::fs::remove_dir_all(staging);
    std::fs::create_dir_all(staging)
        .map_err(|e| ModStoreError::Other(format!("cannot stage: {e}")))?;
    let result = (|| -> Result<()> {
        if looks_like_zip(artifact)? {
            extract_zip_safely(artifact, staging)?;
        } else {
            let safe = sanitize_file_name(file_name);
            std::fs::copy(artifact, staging.join(safe))
                .map_err(|e| ModStoreError::Other(format!("stage copy failed: {e}")))?;
        }
        if destination.exists() {
            std::fs::rename(destination, backup)
                .map_err(|e| ModStoreError::Other(format!("backup failed: {e}")))?;
        }
        match std::fs::rename(staging, destination) {
            Ok(()) => {
                if backup.exists() {
                    let _ = std::fs::remove_dir_all(backup);
                }
                Ok(())
            }
            Err(e) => {
                // Roll back.
                if backup.exists() && !destination.exists() {
                    let _ = std::fs::rename(backup, destination);
                }
                Err(ModStoreError::Other(format!("swap failed: {e}")))
            }
        }
    })();
    if result.is_err() {
        let _ = std::fs::remove_dir_all(staging);
        if backup.exists() && destination.exists() {
            let _ = std::fs::remove_dir_all(backup);
        }
    }
    result
}

fn looks_like_zip(path: &Path) -> Result<bool> {
    use std::io::Read as _;
    let mut file = std::fs::File::open(path).map_err(|e| ModStoreError::Other(e.to_string()))?;
    let mut magic = [0u8; 4];
    let n = file.read(&mut magic).unwrap_or(0);
    Ok(n >= 4 && magic == [0x50, 0x4B, 0x03, 0x04])
}

/// Extracts a zip while rejecting absolute paths, `..` escapes and symlinks.
fn extract_zip_safely(archive: &Path, destination: &Path) -> Result<()> {
    let file = std::fs::File::open(archive).map_err(|e| ModStoreError::Other(e.to_string()))?;
    let mut zip = zip::ZipArchive::new(file)
        .map_err(|e| ModStoreError::InvalidArtefact(format!("unreadable zip: {e}")))?;
    for i in 0..zip.len() {
        let mut entry = zip
            .by_index(i)
            .map_err(|e| ModStoreError::InvalidArtefact(format!("bad zip entry: {e}")))?;
        let name = entry.name().to_string();
        if name.starts_with('/') || name.starts_with('\\') || name.contains("..") {
            return Err(ModStoreError::InvalidArtefact(format!(
                "blocked unsafe zip path: {name}"
            )));
        }
        if entry.is_symlink() {
            return Err(ModStoreError::InvalidArtefact(format!(
                "blocked zip symlink: {name}"
            )));
        }
        let target = destination.join(&name);
        if entry.is_dir() {
            std::fs::create_dir_all(&target).map_err(|e| ModStoreError::Other(e.to_string()))?;
            continue;
        }
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|e| ModStoreError::Other(e.to_string()))?;
        }
        // Double-check containment after normalization.
        let canonical_base = destination
            .canonicalize()
            .unwrap_or_else(|_| destination.to_path_buf());
        let mut out =
            std::fs::File::create(&target).map_err(|e| ModStoreError::Other(e.to_string()))?;
        std::io::copy(&mut entry, &mut out).map_err(|e| ModStoreError::Other(e.to_string()))?;
        drop(out);
        if let Ok(canonical_target) = target.canonicalize() {
            if !canonical_target.starts_with(&canonical_base) {
                let _ = std::fs::remove_file(&target);
                return Err(ModStoreError::InvalidArtefact(format!(
                    "blocked zip escape: {name}"
                )));
            }
        }
    }
    Ok(())
}

fn sanitize_file_name(value: &str) -> String {
    value
        .chars()
        .map(|c| {
            if matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') || c.is_control() {
                '_'
            } else {
                c
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trusted_hosts() {
        let installer = ModStoreInstaller::new(&["https://datacentermods.com".to_string()]);
        assert!(installer.is_trusted_download_url("https://datacentermods.com/f/x.zip"));
        assert!(!installer.is_trusted_download_url("http://datacentermods.com/f/x.zip"));
        assert!(!installer.is_trusted_download_url("https://evil.com/f/x.zip"));
        assert!(!installer.is_trusted_download_url("not a url"));
    }

    #[test]
    fn manifest_roundtrip() {
        let dir = std::env::temp_dir().join(format!(
            "greg-manifest-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        assert!(load_manifest(&dir).installed.is_empty());
        save_manifest_entry(
            &dir,
            ModStoreInstalledEntry {
                mod_id: "m".into(),
                title: "T".into(),
                version: "1.0.0".into(),
                release_id: "r".into(),
                install_path: "/x".into(),
                installed_at: "1".into(),
            },
        );
        assert_eq!(load_manifest(&dir).installed.len(), 1);
        std::fs::remove_dir_all(&dir).ok();
    }
}
