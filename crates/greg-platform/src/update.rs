//! GitHub release discovery and verified update downloads.
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use semver::Version;
use serde::Deserialize;
use sha2::{Digest, Sha256};

const API: &str = "https://api.github.com/repos/mleem97/gregModmanager/releases/latest";
const MAX_DOWNLOAD: u64 = 512 * 1024 * 1024;

#[derive(Debug, Deserialize)]
pub struct Release {
    pub tag_name: String,
    pub assets: Vec<Asset>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Asset {
    pub name: String,
    pub browser_download_url: String,
    pub digest: Option<String>,
    pub size: u64,
}

fn client() -> anyhow::Result<reqwest::blocking::Client> {
    Ok(reqwest::blocking::Client::builder()
        .user_agent(concat!("gregModmanager/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(120))
        .build()?)
}

pub fn latest() -> anyhow::Result<Release> {
    Ok(client()?.get(API).send()?.error_for_status()?.json()?)
}

pub fn newer(tag: &str, current: &str) -> anyhow::Result<bool> {
    Ok(Version::parse(tag.trim_start_matches('v'))? > Version::parse(current)?)
}

/// Package installers write a marker beside the executable. Without it, the
/// installation is portable and must never be overwritten automatically.
pub fn install_kind() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf))
        .and_then(|p| std::fs::read_to_string(p.join(".greg-install-kind")).ok())
        .map(|s| s.trim().to_ascii_lowercase())
        .unwrap_or_else(|| "portable".into())
}

pub fn select_asset<'a>(
    release: &'a Release,
    kind: &str,
    os: &str,
    arch: &str,
) -> Option<&'a Asset> {
    let suffix = match (os, kind) {
        ("windows", "setup") => ".exe",
        ("windows", "msi") => ".msi",
        ("windows", "portable") => ".zip",
        ("linux", "deb") => ".deb",
        ("linux", "rpm") => ".rpm",
        ("linux", "apk") => ".apk",
        ("linux", "arch") => ".pkg.tar.zst",
        ("linux", "portable") => ".tar.gz",
        _ => return None,
    };
    let arch_tag = match arch {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        _ => return None,
    };
    release.assets.iter().find(|a| {
        let name = a.name.to_ascii_lowercase();
        name.contains("greg")
            && name.contains(arch_tag)
            && name.ends_with(suffix)
            && (os != "windows" || name.contains("win"))
            && (os != "linux" || name.contains("linux"))
    })
}

/// Download only assets carrying GitHub's SHA-256 digest, and verify before opening.
pub fn download(asset: &Asset) -> anyhow::Result<PathBuf> {
    let digest = asset
        .digest
        .as_deref()
        .and_then(|s| s.strip_prefix("sha256:"))
        .filter(|s| s.len() == 64 && s.bytes().all(|c| c.is_ascii_hexdigit()))
        .ok_or_else(|| anyhow::anyhow!("Release asset has no valid SHA-256 digest"))?;
    anyhow::ensure!(
        asset.size > 0 && asset.size <= MAX_DOWNLOAD,
        "Invalid release asset size"
    );
    anyhow::ensure!(
        asset.name
            == Path::new(&asset.name)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(""),
        "Invalid asset name"
    );
    anyhow::ensure!(
        asset
            .browser_download_url
            .starts_with("https://github.com/mleem97/gregModmanager/releases/download/"),
        "Invalid asset URL"
    );
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("greg-update-{}-{nonce}", std::process::id()));
    std::fs::create_dir(&dir)?;
    let path = dir.join(&asset.name);
    let tmp = dir.join(format!("{}.part", asset.name));
    let result = (|| -> anyhow::Result<()> {
        let mut response = client()?
            .get(&asset.browser_download_url)
            .send()?
            .error_for_status()?;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp)?;
        let mut hash = Sha256::new();
        let mut total = 0u64;
        let mut buf = [0u8; 65536];
        loop {
            let n = response.read(&mut buf)?;
            if n == 0 {
                break;
            }
            total += n as u64;
            anyhow::ensure!(
                total <= asset.size && total <= MAX_DOWNLOAD,
                "Download exceeds expected size"
            );
            file.write_all(&buf[..n])?;
            hash.update(&buf[..n]);
        }
        anyhow::ensure!(
            total == asset.size && hex::encode(hash.finalize()).eq_ignore_ascii_case(digest),
            "Download checksum mismatch"
        );
        file.sync_all()?;
        std::fs::rename(&tmp, &path)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compares_semver_and_rejects_invalid_versions() {
        assert!(newer("v1.7.8", "1.7.7").unwrap());
        assert!(!newer("v1.7.7", "1.7.7").unwrap());
        assert!(!newer("v1.7.8-rc.1", "1.7.8").unwrap());
        assert!(newer("bad", "1.7.7").is_err());
    }
    #[test]
    fn selects_only_matching_package() {
        let release: Release = serde_json::from_str(r#"{"tag_name":"v1.8.0","assets":[{"name":"greg-linux-x64.deb","browser_download_url":"x","size":1},{"name":"greg-linux-x64.tar.gz","browser_download_url":"x","size":1}]}"#).unwrap();
        assert!(select_asset(&release, "deb", "linux", "x86_64")
            .unwrap()
            .name
            .ends_with(".deb"));
        assert!(select_asset(&release, "rpm", "linux", "x86_64").is_none());
        assert!(select_asset(&release, "portable", "linux", "x86_64")
            .unwrap()
            .name
            .ends_with(".tar.gz"));
    }
    #[test]
    fn rejects_missing_and_invalid_digests_before_download() {
        let mut asset = Asset {
            name: "greg-linux-x64.deb".into(),
            browser_download_url:
                "https://github.com/mleem97/gregModmanager/releases/download/v1/a".into(),
            digest: None,
            size: 1,
        };
        assert!(download(&asset).is_err());
        asset.digest = Some("sha256:not-a-digest".into());
        assert!(download(&asset).is_err());
    }

    #[test]
    fn rejects_wrong_architecture() {
        let release: Release = serde_json::from_str(r#"{"tag_name":"v1.8.0","assets":[{"name":"greg-linux-arm64.deb","browser_download_url":"x","size":1}]}"#).unwrap();
        assert!(select_asset(&release, "deb", "linux", "x86_64").is_none());
        assert!(select_asset(&release, "deb", "linux", "aarch64").is_some());
    }
}
