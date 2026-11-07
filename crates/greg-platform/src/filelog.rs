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

static SENSITIVE_PATHS: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

/// Registers a user-specific filesystem root for redaction from local logs and
/// exported diagnostics. The raw path is held only in process memory.
pub fn register_sensitive_path(path: &std::path::Path) {
    let mut values = Vec::new();
    values.push(path.to_string_lossy().into_owned());
    if let Ok(canonical) = path.canonicalize() {
        values.push(canonical.to_string_lossy().into_owned());
    }
    let roots = SENSITIVE_PATHS.get_or_init(|| Mutex::new(Vec::new()));
    if let Ok(mut roots) = roots.lock() {
        for value in values {
            if !value.is_empty() && !roots.iter().any(|known| known == &value) {
                roots.push(value);
            }
        }
        roots.sort_by_key(|value| std::cmp::Reverse(value.len()));
    }
}

/// Redacts PII-like values and all known local roots before they enter logs,
/// reports, or telemetry. Arbitrary free-form names cannot be inferred safely;
/// callers must not pass them into diagnostics.
pub fn redact_diagnostic_text(text: &str) -> String {
    let mut redacted = text.to_string();
    let mut roots = Vec::new();
    if let Some(home) = dirs::home_dir() {
        roots.push(home.to_string_lossy().into_owned());
    }
    for env_name in ["HOME", "USERPROFILE"] {
        if let Ok(value) = std::env::var(env_name) {
            if !value.is_empty() {
                roots.push(value);
            }
        }
    }
    if let Some(known) = SENSITIVE_PATHS.get() {
        if let Ok(known) = known.lock() {
            roots.extend(known.iter().cloned());
        }
    }
    roots.sort_by_key(|value| std::cmp::Reverse(value.len()));
    roots.dedup();
    for root in roots {
        if root.is_empty() {
            continue;
        }
        redacted = redacted.replace(&root, "[PATH REDACTED]");
        #[cfg(windows)]
        {
            redacted = replace_ascii_case_insensitive(&redacted, &root, "[PATH REDACTED]");
        }
    }
    greg_core::report::redact(&redacted)
}

#[cfg(windows)]
fn replace_ascii_case_insensitive(text: &str, needle: &str, replacement: &str) -> String {
    let lower = text.to_ascii_lowercase();
    let needle_lower = needle.to_ascii_lowercase();
    let mut output = String::with_capacity(text.len());
    let mut cursor = 0;
    while let Some(offset) = lower[cursor..].find(&needle_lower) {
        let start = cursor + offset;
        let end = start + needle.len();
        output.push_str(&text[cursor..start]);
        output.push_str(replacement);
        cursor = end;
    }
    output.push_str(&text[cursor..]);
    output
}

/// File-backed session log.
#[derive(Debug, Clone)]
pub struct FileLog {
    inner: Arc<Mutex<FileLogInner>>,
}

#[derive(Debug)]
struct FileLogInner {
    path: PathBuf,
    file: Option<std::fs::File>,
    mirror_to_console: bool,
    session_started: bool,
}

impl FileLog {
    /// Sanitizes a diagnostic string using the current process's redaction
    /// registry. Exposed so the in-memory log/UI cannot retain raw input that
    /// would already be scrubbed before disk persistence.
    pub fn redact_diagnostic_text(text: &str) -> String {
        redact_diagnostic_text(text)
    }

    /// Resolves an explicit log file or the manager base Logs directory.
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
                file: std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&path)
                    .ok(),
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

    /// Switch future session logs to the selected manager base. Existing logs
    /// remain untouched; the live session marker follows the new log location.
    pub fn relocate(&self, base: &std::path::Path) -> crate::Result<()> {
        let logs = base.join("Logs");
        std::fs::create_dir_all(&logs).map_err(crate::error::io_err)?;
        let path = logs.join(format!("app-{}.log", timestamp_file()));
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(crate::error::io_err)?;
        let mut inner = self.inner.lock().expect("log lock");
        if inner.path == path {
            return Ok(());
        }
        if inner.session_started {
            if let Some(previous) = inner.path.parent() {
                let _ = std::fs::remove_file(previous.join("session-active.marker"));
            }
            let payload = format!("pid={};started={}", std::process::id(), timestamp_line());
            let _ = std::fs::write(logs.join("session-active.marker"), payload);
        }
        inner.path = path;
        inner.file = Some(file);
        Ok(())
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

    /// Scrubs existing app logs in this manager directory after known game and
    /// workspace roots have been registered. Old crash-report bodies are never
    /// rewritten here; support bundles export only a redacted count for them.
    pub fn scrub_history(&self, logs_dir: &std::path::Path) {
        let mut paths = Vec::new();
        if let Ok(entries) = std::fs::read_dir(logs_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with("app-") && name.ends_with(".log"))
                {
                    paths.push(path);
                }
            }
        }
        for path in paths {
            let Ok(content) = std::fs::read_to_string(&path) else {
                continue;
            };
            let sanitized = redact_diagnostic_text(&content);
            if sanitized == content {
                continue;
            }
            let temporary = path.with_extension("log.redacted.tmp");
            if std::fs::write(&temporary, sanitized).is_ok()
                && std::fs::rename(&temporary, &path).is_ok()
                && path == self.path()
            {
                if let Ok(file) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&path)
                {
                    self.inner.lock().expect("log lock").file = Some(file);
                }
            } else {
                let _ = std::fs::remove_file(temporary);
            }
        }
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
        let msg = redact_diagnostic_text(message.unwrap_or(""));
        let source = redact_diagnostic_text(source);
        self.error(&format!("Crash marker from {source}: {msg}"));
        let payload = format!("source={source};message={msg}");
        let _ = std::fs::write(self.marker_path(), payload);
        let report = CrashReport {
            process_id: 0,
            message: msg,
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
        let mut inner = self.inner.lock().expect("log lock");
        Self::write_locked(&mut inner, "INFO", message);
    }

    /// Logs a warning line.
    pub fn warn(&self, message: &str) {
        let mut inner = self.inner.lock().expect("log lock");
        Self::write_locked(&mut inner, "WARN", message);
    }

    /// Logs an error line.
    pub fn error(&self, message: &str) {
        let mut inner = self.inner.lock().expect("log lock");
        Self::write_locked(&mut inner, "ERROR", message);
    }

    fn write_locked(inner: &mut FileLogInner, level: &str, message: &str) {
        let message = redact_diagnostic_text(message);
        let line = format!("{} [{level}] {message}\n", timestamp_line());
        use std::io::Write as _;
        if inner.file.is_none() {
            inner.file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&inner.path)
                .ok();
        }
        if let Some(file) = &mut inner.file {
            let _ = file.write_all(line.as_bytes());
        }
        if inner.mirror_to_console {
            let _ = std::io::stdout().write_all(line.as_bytes());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn central_log_sink_redacts_registered_paths_and_identity_markers() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let base = std::env::temp_dir().join(format!("greg-redact-{nonce}"));
        let private_root = base.join("private user game");
        register_sensitive_path(&private_root);
        let path = base.join("app.log");
        let log = FileLog::open(Some(path.clone()));
        log.info(&format!(
            "account_name=MLeeM at {} email=mlee@example.test",
            private_root.display()
        ));
        let contents = std::fs::read_to_string(path).unwrap();
        assert!(!contents.contains("MLeeM"));
        assert!(!contents.contains("private user game"));
        assert!(!contents.contains("mlee@example.test"));
        std::fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn relocating_log_moves_live_marker_without_moving_history() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let base =
            std::env::temp_dir().join(format!("greg-filelog-{}-{nonce}", std::process::id()));
        let old_path = base.join("old/session.log");
        let log = FileLog::open(Some(old_path.clone()));
        log.start_session();
        log.info("before relocation");
        let destination = base.join("new");
        log.relocate(&destination).unwrap();
        log.info("after relocation");
        assert!(!base.join("old/session-active.marker").exists());
        assert!(destination.join("Logs/session-active.marker").is_file());
        assert!(std::fs::read_to_string(&old_path)
            .unwrap()
            .contains("before relocation"));
        let current = std::fs::read_to_string(log.path()).unwrap();
        assert!(current.contains("after relocation"));
        assert!(!current.contains("before relocation"));
        log.end_session();
        assert!(!destination.join("Logs/session-active.marker").exists());
        drop(log);
        std::fs::remove_dir_all(base).unwrap();
    }
}
