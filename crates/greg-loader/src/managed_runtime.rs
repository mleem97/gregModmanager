//! Verified deployment descriptor and per-launch runtime acknowledgement.
//!
//! Descriptor hashes prove which reviewed artifacts are installed. Only an
//! actual same-session runtime acknowledgement proves source registration and
//! vanilla-copy prevention succeeded inside the launched game process.

use std::ffi::OsString;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::error::{io_err, LoaderError, Result};

const MAX_DESCRIPTOR_BYTES: u64 = 64 * 1024;
const MAX_READY_BYTES: u64 = 8 * 1024;
const STARTUP_TIMEOUT: Duration = Duration::from_secs(45);

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RuntimeDescriptor {
    schema_version: u32,
    core_path: String,
    companion_path: String,
    core_sha256: String,
    companion_sha256: String,
    startup_hook: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Ready {
    schema_version: u32,
    session: String,
    vanilla_blocked: bool,
    managed_sources: bool,
}

/// Verifies the explicitly deployed runtime before the app releases Steam.
/// Missing/unsupported descriptors, changed DLLs or escaping paths fail closed.
pub fn validate_managed_runtime(game_root: &Path, manager_base: &Path) -> Result<()> {
    // The selected Steam installation can be reached through ~/.steam links.
    // Resolve that trusted root first, then reject links beneath its boundary.
    crate::sync::validate_managed_path(manager_base)?;
    let game_root = game_root.canonicalize().map_err(io_err)?;
    let manager_base = manager_base.canonicalize().map_err(io_err)?;
    if game_root.starts_with(&manager_base) || manager_base.starts_with(&game_root) {
        return Err(LoaderError::InvalidArtefact(
            "game and manager workspace must not overlap".into(),
        ));
    }
    for relative in [
        "GameModding",
        "GameModding/Mods",
        "GameModding/Melons",
        "GameModding/UserLibs",
        "ModConfigs",
        "Configs/runtime",
    ] {
        crate::sync::validate_managed_path(&manager_base.join(relative))?;
    }
    let path = game_root.join("UserData/gregModmanager/runtime-v1.json");
    crate::sync::validate_managed_path(&path)?;
    let data = bounded_file(&path, MAX_DESCRIPTOR_BYTES).map_err(|error| LoaderError::InvalidArtefact(format!("managed runtime is not ready: {error}; install the reviewed gregCore/Companion/startup-hook integration")))?;
    let descriptor: RuntimeDescriptor =
        serde_json::from_slice(&data).map_err(crate::error::json_err)?;
    if descriptor.schema_version != 1 || !descriptor.startup_hook {
        return Err(LoaderError::InvalidArtefact(
            "unsupported managed runtime contract or missing early startup hook".into(),
        ));
    }
    for (relative, expected) in [
        (&descriptor.core_path, &descriptor.core_sha256),
        (&descriptor.companion_path, &descriptor.companion_sha256),
    ] {
        let artifact = checked_artifact(&game_root, relative)?;
        if expected.len() != 64 || !expected.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(LoaderError::InvalidArtefact(
                "invalid runtime SHA256 digest".into(),
            ));
        }
        let actual = hash_file(&artifact)?;
        if !actual.eq_ignore_ascii_case(expected) {
            return Err(LoaderError::InvalidArtefact(format!(
                "managed runtime artifact changed: {}",
                artifact.display()
            )));
        }
    }
    if descriptor.core_path == descriptor.companion_path {
        return Err(LoaderError::InvalidArtefact(
            "core and Companion artifacts must be distinct".into(),
        ));
    }
    Ok(())
}

fn checked_artifact(root: &Path, relative: &str) -> Result<PathBuf> {
    // Apply Windows path rules even on Linux/Proton hosts.
    if relative.is_empty()
        || relative.contains(['\\', ':'])
        || Path::new(relative)
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(LoaderError::InvalidArtefact(
            "runtime artifact path must be a safe relative path using forward slashes".into(),
        ));
    }
    let path = root.join(relative);
    crate::sync::validate_managed_path(&path)?;
    let canonical = path.canonicalize().map_err(io_err)?;
    if !canonical.starts_with(root)
        || !canonical.is_file()
        || !canonical
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("dll"))
    {
        return Err(LoaderError::InvalidArtefact(
            "runtime artifact must be a DLL inside the game root".into(),
        ));
    }
    Ok(canonical)
}

fn hash_file(path: &Path) -> Result<String> {
    let mut file = std::fs::File::open(path).map_err(io_err)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 64 * 1024];
    loop {
        let bytes = file.read(&mut buffer).map_err(io_err)?;
        if bytes == 0 {
            break;
        }
        hasher.update(&buffer[..bytes]);
    }
    Ok(hex::encode(hasher.finalize()))
}

fn bounded_file(path: &Path, limit: u64) -> Result<Vec<u8>> {
    crate::sync::validate_managed_path(path)?;
    let metadata = std::fs::metadata(path).map_err(io_err)?;
    if !metadata.is_file() || metadata.len() > limit {
        return Err(LoaderError::InvalidArtefact(
            "runtime metadata must be a bounded regular file".into(),
        ));
    }
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(io_err)?
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(io_err)?;
    if bytes.len() as u64 > limit {
        return Err(LoaderError::InvalidArtefact(
            "runtime metadata exceeds size limit".into(),
        ));
    }
    Ok(bytes)
}

/// One launch's private environment and readiness deadline.
pub(crate) struct ManagedLaunch {
    pub(crate) base: PathBuf,
    pub(crate) session: String,
    pub(crate) ready: PathBuf,
    started: Instant,
    confirmed: bool,
}

impl ManagedLaunch {
    pub(crate) fn prepare(base: &Path) -> Result<Self> {
        crate::sync::validate_managed_path(base)?;
        let base = base.canonicalize().map_err(io_err)?;
        let directory = base.join("Configs/runtime");
        crate::sync::validate_managed_path(&directory)?;
        std::fs::create_dir_all(&directory).map_err(io_err)?;
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let session = format!(
            "{}-{stamp}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        );
        let ready = directory.join(format!("{session}.ready.json"));
        crate::sync::validate_managed_path(&ready)?;
        if ready.exists() {
            return Err(LoaderError::InvalidArtefact(
                "runtime session path already exists".into(),
            ));
        }
        Ok(Self {
            base,
            session,
            ready,
            started: Instant::now(),
            confirmed: false,
        })
    }

    pub(crate) fn environment(&self, windows_game: bool) -> Result<Vec<(OsString, OsString)>> {
        let child_path = |path: &Path| {
            greg_platform::process::path_for_child(path, windows_game)
                .map_err(|error| LoaderError::Other(error.to_string()))
        };
        Ok(vec![
            ("GREGMODMANAGER_MANAGED".into(), "1".into()),
            ("GREGMODMANAGER_ROOT".into(), child_path(&self.base)?),
            ("GREGMODMANAGER_SESSION".into(), self.session.clone().into()),
            ("GREGMODMANAGER_READY".into(), child_path(&self.ready)?),
        ])
    }

    pub(crate) fn poll(&mut self) -> Result<bool> {
        if self.confirmed {
            return Ok(true);
        }
        crate::sync::validate_managed_path(&self.ready)?;
        if self.ready.exists() {
            let record: Ready =
                serde_json::from_slice(&bounded_file(&self.ready, MAX_READY_BYTES)?)
                    .map_err(crate::error::json_err)?;
            if record.schema_version != 1
                || record.session != self.session
                || !record.vanilla_blocked
                || !record.managed_sources
            {
                return Err(LoaderError::InvalidArtefact(
                    "managed readiness does not match this launch or required protections".into(),
                ));
            }
            self.confirmed = true;
            return Ok(true);
        }
        if self.started.elapsed() >= STARTUP_TIMEOUT {
            return Err(LoaderError::Other(
                "managed runtime startup acknowledgement timed out".into(),
            ));
        }
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "greg-runtime-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        for sub in [
            "game/Mods",
            "game/Plugins",
            "game/UserData/gregModmanager",
            "manager",
        ] {
            std::fs::create_dir_all(path.join(sub)).unwrap();
        }
        std::fs::write(path.join("game/Mods/core.dll"), b"core").unwrap();
        std::fs::write(path.join("game/Plugins/companion.dll"), b"companion").unwrap();
        path.canonicalize().unwrap()
    }
    fn descriptor(root: &Path, core_path: &str) {
        let json = serde_json::json!({"schemaVersion":1,"corePath":core_path,"companionPath":"Plugins/companion.dll","coreSha256":hash_file(&root.join("game/Mods/core.dll")).unwrap(),"companionSha256":hash_file(&root.join("game/Plugins/companion.dll")).unwrap(),"startupHook":true});
        std::fs::write(
            root.join("game/UserData/gregModmanager/runtime-v1.json"),
            serde_json::to_vec(&json).unwrap(),
        )
        .unwrap();
    }
    #[test]
    fn descriptor_requires_matching_hashes_and_safe_artifact_paths() {
        let root = fixture();
        assert!(validate_managed_runtime(&root.join("game"), &root.join("manager")).is_err());
        descriptor(&root, "Mods/core.dll");
        assert!(validate_managed_runtime(&root.join("game"), &root.join("manager")).is_ok());
        std::fs::write(root.join("game/Mods/core.dll"), b"changed").unwrap();
        assert!(validate_managed_runtime(&root.join("game"), &root.join("manager")).is_err());
        for path in [
            "../core.dll",
            "C:/core.dll",
            "Mods/../core.dll",
            "Mods\\core.dll",
        ] {
            descriptor(&root, path);
            assert!(validate_managed_runtime(&root.join("game"), &root.join("manager")).is_err());
        }
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn readiness_requires_fresh_nonce_and_both_runtime_guards() {
        let root = fixture();
        let mut first = ManagedLaunch::prepare(&root.join("manager")).unwrap();
        let second = ManagedLaunch::prepare(&root.join("manager")).unwrap();
        assert_ne!(first.session, second.session);
        assert!(!first.poll().unwrap());
        let mut timed_out = ManagedLaunch::prepare(&root.join("manager")).unwrap();
        timed_out.started = Instant::now() - STARTUP_TIMEOUT - Duration::from_secs(1);
        assert!(timed_out.poll().is_err());
        let stale = serde_json::json!({"schemaVersion":1,"session":second.session,"vanillaBlocked":true,"managedSources":true});
        std::fs::write(&first.ready, serde_json::to_vec(&stale).unwrap()).unwrap();
        assert!(first.poll().is_err());
        let missing_guard = serde_json::json!({"schemaVersion":1,"session":first.session,"vanillaBlocked":false,"managedSources":true});
        std::fs::write(&first.ready, serde_json::to_vec(&missing_guard).unwrap()).unwrap();
        assert!(first.poll().is_err());
        let ready = serde_json::json!({"schemaVersion":1,"session":first.session,"vanillaBlocked":true,"managedSources":true});
        std::fs::write(&first.ready, serde_json::to_vec(&ready).unwrap()).unwrap();
        assert!(first.poll().unwrap());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[cfg(unix)]
    #[test]
    fn rejects_linked_artifacts_and_readiness_paths() {
        let root = fixture();
        descriptor(&root, "Mods/core.dll");
        std::fs::rename(root.join("game/Mods/core.dll"), root.join("outside.dll")).unwrap();
        std::os::unix::fs::symlink(root.join("outside.dll"), root.join("game/Mods/core.dll"))
            .unwrap();
        assert!(validate_managed_runtime(&root.join("game"), &root.join("manager")).is_err());
        // Even a symlinked parent must not escape the managed boundary.
        std::os::unix::fs::symlink(root.join("game"), root.join("manager/GameModding")).unwrap();
        assert!(validate_managed_runtime(&root.join("game"), &root.join("manager")).is_err());
        let mut launch = ManagedLaunch::prepare(&root.join("manager")).unwrap();
        std::os::unix::fs::symlink(root.join("outside.dll"), &launch.ready).unwrap();
        assert!(launch.poll().is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}
