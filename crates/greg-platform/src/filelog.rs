//! Session file log (`AppFileLog` port).
//!
//! Logging must never crash the app: every method is best-effort.

use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

use greg_core::models::CrashReport;

use crate::paths::log_dir;

fn timestamp_file() -> String {
    greg_core::util::today_ymd().replace('-', "")
}

fn timestamp_line() -> String {
    let ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("{}.{:03}", ms / 1000, ms % 1000)
}

/// File-backed session log.
#[derive(Debug, Clone)]
pub struct FileLog {
    inner: Arc<Mutex<FileLogInner>>,
}

#[derive(Debug)]
struct FileLogInner {
    path: PathBuf,
    mirror_to_console: bool,
    session_started: bool,
}

impl FileLog {
    /// Resolves the log path (override > exe-dir > app-data > temp).
    pub fn open(path_override: Option<PathBuf>) -> Self {
        let path = match path_override {
            Some(p) => p,
            None => log_dir().join(format!("app-{}.log", timestamp_file())),
        };
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        Self {
            inner: Arc::new(Mutex::new(FileLogInner {
                path,
                mirror_to_console: false,
                session_started: false,
            })),
        }
    }

    /// Global shared instance (log dir default).
    pub fn shared() -> &'static Self {
        static INSTANCE: OnceLock<FileLog> = OnceLock::new();
        INSTANCE.get_or_init(|| FileLog::open(None))
    }

    /// Log file path.
    pub fn path(&self) -> PathBuf {
        self.inner.lock().expect("log lock").path.clone()
    }

    /// Mirror every line to stdout (CLI `--verbose`).
    pub fn set_mirror_to_console(&self, mirror: bool) {
        self.inner.lock().expect("log lock").mirror_to_console = mirror;
    }

    /// Directory for crash reports beside the log.
    pub fn reports_dir(&self) -> PathBuf {
        let dir = self
            .path()
            .parent()
            .map(|p| p.join("reports"))
            .unwrap_or_else(std::env::temp_dir);
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    /// Session marker path (crash detection across runs).
    fn marker_path(&self) -> PathBuf {
        self.path()
            .parent()
            .map(|p| p.join("session-active.marker"))
            .unwrap_or_else(std::env::temp_dir)
    }

    /// Marks session start; warns when the previous run crashed.
    ///
    /// Locking discipline: never hold the guard across `path()` /
    /// `marker_path()` / `reports_dir()` calls (`std::sync::Mutex` is not
    /// reentrant — nesting them deadlocks, unlike C# `lock`).
    pub fn start_session(&self) {
        let already = {
            let mut inner = self.inner.lock().expect("log lock");
            std::mem::replace(&mut inner.session_started, true)
        };
        if already {
            return;
        }
        let marker = self.marker_path();
        if marker.is_file() {
            let prev = std::fs::read_to_string(&marker).unwrap_or_default();
            self.warn(&format!("Previous run ended unexpectedly. Marker: {prev}"));
        }
        let payload = format!("pid={};started={}", std::process::id(), timestamp_line());
        let _ = std::fs::write(&marker, payload);
        self.info("Session started.");
    }

    /// Marks a clean shutdown (removes the marker).
    pub fn end_session(&self) {
        self.info("Session ended cleanly.");
        let _ = std::fs::remove_file(self.marker_path());
    }

    /// Records a crash marker plus a JSON crash report.
    pub fn mark_crash(&self, source: &str, message: Option<&str>) {
        let msg = message.unwrap_or("");
        self.error(&format!("Crash marker from {source}: {msg}"));
        let payload = format!("pid={};source={source};message={msg}", std::process::id());
        let _ = std::fs::write(self.marker_path(), payload);
        let report = CrashReport {
            process_id: std::process::id(),
            message: msg.to_string(),
            stack_trace: String::new(),
            timestamp: timestamp_line(),
        };
        if let Ok(json) = serde_json::to_string_pretty(&report) {
            let path = self
                .reports_dir()
                .join(format!("crash-{}.json", timestamp_file()));
            let _ = std::fs::write(path, json);
        }
    }

    /// Logs an informational line.
    pub fn info(&self, message: &str) {
        let inner = self.inner.lock().expect("log lock");
        Self::write_locked(&inner, "INFO", message);
    }

    /// Logs a warning line.
    pub fn warn(&self, message: &str) {
        let inner = self.inner.lock().expect("log lock");
        Self::write_locked(&inner, "WARN", message);
    }

    /// Logs an error line.
    pub fn error(&self, message: &str) {
        let inner = self.inner.lock().expect("log lock");
        Self::write_locked(&inner, "ERROR", message);
    }

    fn write_locked(inner: &FileLogInner, level: &str, message: &str) {
        let line = format!(
            "{} [{level}] [pid:{}] {message}\n",
            timestamp_line(),
            std::process::id()
        );
        use std::io::Write as _;
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&inner.path)
        {
            let _ = file.write_all(line.as_bytes());
        }
        if inner.mirror_to_console {
            let _ = std::io::stdout().write_all(line.as_bytes());
        }
    }
}
