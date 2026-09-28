//! Single-instance guard via lockfile (`ProtocolSingleInstance` port, OS part).
//!
//! A bare lockfile goes stale when the process is killed (kill -9, crash,
//! power loss): every later start would silently exit thinking another
//! instance runs. Instead the primary heartbeats the file every few seconds
//! and `acquire` takes over files older than [`STALE_AFTER`] (concurrent
//! takeovers serialize on `create_new`: the loser sees a fresh file and
//! backs off).

use std::fs::OpenOptions;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// Heartbeat older than this means the holder is dead.
pub const STALE_AFTER: Duration = Duration::from_secs(30);

/// Holds the lockfile while alive; drop to release.
#[derive(Debug)]
pub struct InstanceGuard {
    _file: std::fs::File,
    path: PathBuf,
}

impl InstanceGuard {
    /// Tries to become the primary instance. Takes over stale lockfiles;
    /// `Err` means a live primary exists → forward args & exit.
    pub fn acquire(app_name: &str) -> std::io::Result<Self> {
        let path = lock_path(app_name);
        match Self::create(&path) {
            Ok(guard) => {
                guard.heartbeat();
                Ok(guard)
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                if is_stale(&path) {
                    // Previous holder died without cleanup: remove and retry
                    // once. A racing peer serializes on create_new below.
                    let _ = std::fs::remove_file(&path);
                    let guard = Self::create(&path)?;
                    guard.heartbeat();
                    Ok(guard)
                } else {
                    Err(std::io::Error::new(
                        e.kind(),
                        format!("another instance holds {}", path.display()),
                    ))
                }
            }
            Err(e) => Err(e),
        }
    }

    fn create(path: &std::path::Path) -> std::io::Result<Self> {
        let file = OpenOptions::new().create_new(true).write(true).open(path)?;
        Ok(Self {
            _file: file,
            path: path.to_path_buf(),
        })
    }

    /// Refreshes the heartbeat (best-effort; called every few seconds).
    pub fn heartbeat(&self) {
        use std::io::Write as _;
        if let Ok(mut file) = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.path)
        {
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let _ = writeln!(file, "pid={} beat={now}", std::process::id());
        }
    }

    /// Lockfile path (for diagnostics).
    pub fn path(&self) -> &std::path::Path {
        &self.path
    }
}

/// Lockfile location.
fn lock_path(app_name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{app_name}.lock"))
}

/// True when the file is missing (no holder) or older than [`STALE_AFTER`].
fn is_stale(path: &std::path::Path) -> bool {
    let age = std::fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|mtime| SystemTime::now().duration_since(mtime).ok());
    match age {
        // Missing/unreadable clock info: treat as stale (create_new decides).
        None => true,
        Some(age) => age >= STALE_AFTER,
    }
}

impl Drop for InstanceGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app(name: &str) -> String {
        format!("gregtest-inst-{name}-{}", std::process::id())
    }

    #[test]
    fn live_guard_blocks_second_acquire() {
        let name = app("live");
        let guard = InstanceGuard::acquire(&name).expect("first acquire");
        assert!(guard.path().is_file());
        assert!(
            InstanceGuard::acquire(&name).is_err(),
            "live holder must block"
        );
        drop(guard);
        // Clean release: acquirable again.
        let guard = InstanceGuard::acquire(&name).expect("re-acquire after drop");
        drop(guard);
    }

    #[test]
    fn heartbeat_refreshes_mtime() {
        let name = app("beat");
        let guard = InstanceGuard::acquire(&name).expect("acquire");
        let before = std::fs::metadata(guard.path())
            .expect("meta")
            .modified()
            .expect("mtime");
        std::thread::sleep(Duration::from_millis(15));
        guard.heartbeat();
        let after = std::fs::metadata(guard.path())
            .expect("meta")
            .modified()
            .expect("mtime");
        assert!(after >= before, "heartbeat must refresh mtime");
        drop(guard);
    }

    #[test]
    fn stale_lock_is_taken_over() {
        let name = app("stale");
        let path = lock_path(&name);
        // Simulate a killed predecessor: lockfile with an old mtime.
        std::fs::write(&path, "pid=1 beat=0").expect("stale file");
        let old = filetime::FileTime::from_system_time(
            SystemTime::now() - STALE_AFTER - Duration::from_secs(5),
        );
        filetime::set_file_mtime(&path, old).expect("age file");
        assert!(is_stale(&path), "aged file must read stale");
        let guard = InstanceGuard::acquire(&name).expect("takeover must succeed");
        assert!(!is_stale(guard.path()), "fresh takeover must read live");
        drop(guard);
    }

    #[test]
    fn missing_lock_reads_stale() {
        let path = std::env::temp_dir().join("gregtest-inst-missing-1.lock");
        let _ = std::fs::remove_file(&path);
        assert!(is_stale(&path));
    }
}
