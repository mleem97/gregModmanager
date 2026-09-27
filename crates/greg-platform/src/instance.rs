//! Single-instance guard via lockfile (`ProtocolSingleInstance` port, OS part).
//!
//! Protocol-argument forwarding to the running instance is handled by the
//! binaries; this guard only answers "is another instance already running?".

use std::fs::OpenOptions;
use std::path::PathBuf;

/// Holds the lockfile while alive; drop to release.
#[derive(Debug)]
pub struct InstanceGuard {
    _file: std::fs::File,
    path: PathBuf,
}

impl InstanceGuard {
    /// Tries to become the primary instance. `Err` → forward args & exit.
    pub fn acquire(app_name: &str) -> std::io::Result<Self> {
        let path = std::env::temp_dir().join(format!("{app_name}.lock"));
        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .map_err(|e| {
                std::io::Error::new(
                    e.kind(),
                    format!("another instance holds {}", path.display()),
                )
            })?;
        Ok(Self { _file: file, path })
    }

    /// Lockfile path (for diagnostics).
    pub fn path(&self) -> &std::path::Path {
        &self.path
    }
}

impl Drop for InstanceGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
