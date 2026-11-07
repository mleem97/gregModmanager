//! SQLite-backed settings with typed in-memory reads and one-row writes.
//!
//! The previous JSON store is retained as a one-time import source, never deleted.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use greg_core::prefs::Preferences;
use rusqlite::{params, Connection};
use serde_json::Value;

use crate::error::{io_err, json_err, PlatformError, Result};

/// Cached SQLite preferences. Clones share one cache and serialized connection.
#[derive(Debug, Clone)]
pub struct SqlitePreferences {
    inner: Arc<Mutex<PrefsInner>>,
}

/// Source-compatible name for callers migrating from the old JSON store.
pub type JsonFilePreferences = SqlitePreferences;

#[derive(Debug)]
struct PrefsInner {
    path: PathBuf,
    connection: Connection,
    data: HashMap<String, Value>,
    last_error: Option<String>,
}

fn db_err(error: rusqlite::Error) -> PlatformError {
    PlatformError::Other(format!("settings database: {error}"))
}

fn read_json(path: &Path) -> Result<HashMap<String, Value>> {
    if !path.is_file() {
        return Ok(HashMap::new());
    }
    let text = std::fs::read_to_string(path).map_err(io_err)?;
    let value = serde_json::from_str::<Value>(&text).map_err(json_err)?;
    let Value::Object(map) = value else {
        return Err(PlatformError::Other(
            "preferences import must be a JSON object".into(),
        ));
    };
    Ok(map.into_iter().collect())
}

fn configure(connection: &Connection) -> Result<()> {
    connection
        .busy_timeout(Duration::from_millis(250))
        .map_err(db_err)?;
    let version: i64 = connection
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(db_err)?;
    if version > 1 {
        return Err(PlatformError::Other(format!(
            "settings database version {version} is newer than supported version 1"
        )));
    }
    connection
        .execute_batch(
            "PRAGMA journal_mode=WAL;
         PRAGMA synchronous=NORMAL;
         CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
         CREATE TABLE IF NOT EXISTS imports (source TEXT PRIMARY KEY);
         PRAGMA user_version=1;",
        )
        .map_err(db_err)
}

fn import_json(connection: &mut Connection, path: &Path) -> Result<()> {
    if !path.is_file() {
        return Ok(());
    }
    let source = path.to_string_lossy();
    let imported: bool = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM imports WHERE source=?1)",
            [source.as_ref()],
            |row| row.get(0),
        )
        .map_err(db_err)?;
    if imported {
        return Ok(());
    }
    let values = read_json(path)?;
    let tx = connection.transaction().map_err(db_err)?;
    for (key, value) in values {
        tx.execute(
            "INSERT OR IGNORE INTO settings(key,value) VALUES (?1,?2)",
            params![key, serde_json::to_string(&value).map_err(json_err)?],
        )
        .map_err(db_err)?;
    }
    tx.execute("INSERT INTO imports(source) VALUES (?1)", [source.as_ref()])
        .map_err(db_err)?;
    tx.commit().map_err(db_err)
}

fn load_cache(connection: &Connection) -> Result<HashMap<String, Value>> {
    let mut stmt = connection
        .prepare("SELECT key,value FROM settings")
        .map_err(db_err)?;
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(db_err)?;
    let mut values = HashMap::new();
    for row in rows {
        let (key, text) = row.map_err(db_err)?;
        values.insert(key, serde_json::from_str(&text).map_err(json_err)?);
    }
    Ok(values)
}

fn private_dir(path: &Path) -> Result<()> {
    std::fs::create_dir_all(path).map_err(io_err)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).map_err(io_err)?;
    }
    Ok(())
}

fn open_inner(base: &Path, imports: &[PathBuf]) -> Result<PrefsInner> {
    let configs = base.join("Configs");
    private_dir(&configs)?;
    let path = configs.join("settings.sqlite3");
    // Create with owner-only permissions before SQLite opens journals alongside it.
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
        options.mode(0o600);
        let file = options.open(&path).map_err(io_err)?;
        file.set_permissions(std::fs::Permissions::from_mode(0o600))
            .map_err(io_err)?;
    }
    #[cfg(not(unix))]
    options.open(&path).map_err(io_err)?;
    let mut connection = Connection::open(&path).map_err(db_err)?;
    configure(&connection)?;
    let mut last_error = None;
    for source in imports {
        if let Err(error) = import_json(&mut connection, source) {
            // A damaged old JSON file must not hide valid database settings.
            // Leave it unmarked so a repaired source can be imported later.
            last_error = Some(error.to_string());
        }
    }
    let data = load_cache(&connection)?;
    Ok(PrefsInner {
        path,
        connection,
        data,
        last_error,
    })
}

impl SqlitePreferences {
    /// Open the resolved manager base and import the former OS app-data store.
    /// If storage is unavailable, preserve usable imported values in memory and
    /// expose the failure through `last_error()` rather than crashing startup.
    pub fn open() -> Self {
        let base = crate::paths::manager_base_dir();
        let mut imports = vec![base.join("Configs/preferences.json")];
        if let Some(data) = crate::paths::app_data_dir() {
            imports.push(data.join("preferences.json"));
        }
        Self::open_with_imports(&base, &imports)
    }

    /// Best-effort explicit-base constructor; does not read the user's home.
    pub fn open_at(base: impl AsRef<Path>) -> Self {
        let base = base.as_ref();
        Self::open_with_imports(base, &[base.join("Configs/preferences.json")])
    }

    /// Fallible explicit-base constructor for composition roots and tests.
    pub fn try_open_at(base: impl AsRef<Path>) -> Result<Self> {
        let base = base.as_ref();
        let inner = open_inner(base, &[base.join("Configs/preferences.json")])?;
        Ok(Self {
            inner: Arc::new(Mutex::new(inner)),
        })
    }

    fn open_with_imports(base: &Path, imports: &[PathBuf]) -> Self {
        let inner = match open_inner(base, imports) {
            Ok(inner) => inner,
            Err(error) => {
                let connection =
                    Connection::open_in_memory().expect("open in-memory settings fallback");
                configure(&connection).expect("initialize in-memory settings fallback");
                let mut data = HashMap::new();
                for source in imports {
                    if let Ok(values) = read_json(source) {
                        data.extend(values);
                    }
                }
                PrefsInner {
                    path: base.join("Configs/settings.sqlite3"),
                    connection,
                    data,
                    last_error: Some(error.to_string()),
                }
            }
        };
        Self {
            inner: Arc::new(Mutex::new(inner)),
        }
    }

    /// SQLite path for diagnostics. A startup failure is reported separately.
    pub fn path(&self) -> PathBuf {
        self.inner.lock().expect("prefs lock").path.clone()
    }

    /// Whether the live connection has a durable database file. Import
    /// diagnostics do not make an otherwise usable SQLite database transient.
    pub fn is_persistent(&self) -> bool {
        self.inner
            .lock()
            .expect("prefs lock")
            .connection
            .path()
            .is_some_and(|path| !path.is_empty())
    }

    /// Last failed storage operation, including memory-only startup fallback.
    pub fn last_error(&self) -> Option<String> {
        self.inner.lock().expect("prefs lock").last_error.clone()
    }

    /// Consume an already-reported diagnostic so later successful operations
    /// are not mistaken for the previous failed operation.
    pub fn take_error(&self) -> Option<String> {
        self.inner.lock().expect("prefs lock").last_error.take()
    }

    /// Copy current settings to a new base and then atomically update the base
    /// locator. The previous source database is kept intact. The source snapshot
    /// replaces destination settings; one prior destination snapshot is retained
    /// in `settings_before_relocation` inside the same private database.
    pub fn relocate(&self, base: impl AsRef<Path>) -> Result<()> {
        let data = crate::paths::app_data_dir()
            .ok_or_else(|| PlatformError::Other("app-data directory unavailable".into()))?;
        self.relocate_with_locator(base.as_ref(), &data)
    }

    fn relocate_with_locator(&self, base: &Path, locator_dir: &Path) -> Result<()> {
        let mut inner = self.inner.lock().expect("prefs lock");
        if inner.path == base.join("Configs/settings.sqlite3") {
            return Ok(());
        }
        let imported_sources: Vec<String> = inner
            .connection
            .prepare("SELECT source FROM imports")
            .map_err(db_err)?
            .query_map([], |row| row.get(0))
            .map_err(db_err)?
            .collect::<std::result::Result<_, _>>()
            .map_err(db_err)?;
        let mut destination = open_inner(base, &[])?;
        let tx = destination.connection.transaction().map_err(db_err)?;
        // Preserve exactly one recovery snapshot, then replace the live table.
        // Merging destination-only keys could resurrect a logged-out session.
        tx.execute_batch(
            "CREATE TABLE IF NOT EXISTS settings_before_relocation (key TEXT PRIMARY KEY, value TEXT NOT NULL);
             DELETE FROM settings_before_relocation;
             INSERT INTO settings_before_relocation SELECT key,value FROM settings;
             DELETE FROM settings;"
        ).map_err(db_err)?;
        for source in imported_sources {
            tx.execute(
                "INSERT OR IGNORE INTO imports(source) VALUES (?1)",
                [source],
            )
            .map_err(db_err)?;
        }
        // The new source-authoritative snapshot must not be merged with old
        // JSON on the next startup, even when those files were never imported.
        let mut suppressed_imports = vec![base.join("Configs/preferences.json")];
        if let Some(data) = crate::paths::app_data_dir() {
            suppressed_imports.push(data.join("preferences.json"));
        }
        for source in suppressed_imports {
            tx.execute(
                "INSERT OR IGNORE INTO imports(source) VALUES (?1)",
                [source.to_string_lossy().as_ref()],
            )
            .map_err(db_err)?;
        }
        for (key, value) in &inner.data {
            tx.execute("INSERT INTO settings(key,value) VALUES (?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
                params![key, serde_json::to_string(value).map_err(json_err)?]).map_err(db_err)?;
        }
        tx.commit().map_err(db_err)?;
        destination.data = load_cache(&destination.connection)?;
        write_base_locator_at(base, locator_dir)?;
        *inner = destination;
        Ok(())
    }

    fn set_value(&self, key: &str, value: Value) {
        let mut inner = self.inner.lock().expect("prefs lock");
        if inner.data.get(key) == Some(&value) {
            return;
        }
        let text = value.to_string();
        let result = inner.connection.execute(
            "INSERT INTO settings(key,value) VALUES (?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            params![key, text]);
        match result {
            Ok(_) => {
                inner.data.insert(key.into(), value);
            }
            Err(error) => {
                inner.last_error = Some(error.to_string());
            }
        }
    }
}

/// Persist only the selected base outside that base, allowing it to be found
/// on next startup. No OAuth/session credentials are written to the locator.
pub fn write_base_locator(base: &Path) -> Result<()> {
    let data = crate::paths::app_data_dir()
        .ok_or_else(|| PlatformError::Other("app-data directory unavailable".into()))?;
    write_base_locator_at(base, &data)
}

fn write_base_locator_at(base: &Path, data: &Path) -> Result<()> {
    private_dir(data)?;
    let path = data.join("workspace.path");
    let temp = data.join(format!("workspace.path.{}.tmp", std::process::id()));
    std::fs::write(&temp, base.to_string_lossy().as_bytes()).map_err(io_err)?;
    std::fs::rename(temp, path).map_err(io_err)
}

impl Preferences for SqlitePreferences {
    fn get_string(&self, key: &str, default: &str) -> String {
        self.inner
            .lock()
            .expect("prefs lock")
            .data
            .get(key)
            .and_then(Value::as_str)
            .unwrap_or(default)
            .to_string()
    }
    fn set_string(&mut self, key: &str, value: &str) {
        self.set_value(key, Value::String(value.into()));
    }
    fn get_bool(&self, key: &str, default: bool) -> bool {
        self.inner
            .lock()
            .expect("prefs lock")
            .data
            .get(key)
            .and_then(Value::as_bool)
            .unwrap_or(default)
    }
    fn set_bool(&mut self, key: &str, value: bool) {
        self.set_value(key, Value::Bool(value));
    }
    fn get_int(&self, key: &str, default: i64) -> i64 {
        self.inner
            .lock()
            .expect("prefs lock")
            .data
            .get(key)
            .and_then(Value::as_i64)
            .unwrap_or(default)
    }
    fn set_int(&mut self, key: &str, value: i64) {
        self.set_value(key, Value::Number(value.into()));
    }
    fn remove(&mut self, key: &str) {
        let mut inner = self.inner.lock().expect("prefs lock");
        match inner
            .connection
            .execute("DELETE FROM settings WHERE key=?1", [key])
        {
            Ok(_) => {
                inner.data.remove(key);
            }
            Err(error) => {
                inner.last_error = Some(error.to_string());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn temp_base(name: &str) -> PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("greg-prefs-{}-{nonce}-{name}", std::process::id()))
    }

    #[test]
    fn typed_values_persist_without_coercion_and_removals_stay_removed() {
        let base = temp_base("roundtrip");
        {
            let mut p = SqlitePreferences::try_open_at(&base).unwrap();
            p.set_string("k", "v");
            p.set_bool("b", true);
            p.set_int("n", 3);
            p.set_string("s", "true");
            assert!(!p.get_bool("s", false));
            assert_eq!(p.get_int("k", 7), 7);
            p.remove("k");
        }
        let p = SqlitePreferences::try_open_at(&base).unwrap();
        assert_eq!(p.get_string("k", "gone"), "gone");
        assert!(p.get_bool("b", false));
        assert_eq!(p.get_int("n", 0), 3);
        assert!(p.last_error().is_none());
        drop(p);
        std::fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn json_import_happens_once_and_keeps_source_untouched() {
        let base = temp_base("import");
        std::fs::create_dir_all(base.join("Configs")).unwrap();
        let path = base.join("Configs/preferences.json");
        std::fs::write(&path, r#"{"theme":"dark","optin":true,"count":4}"#).unwrap();
        let mut p = SqlitePreferences::try_open_at(&base).unwrap();
        assert_eq!(p.get_string("theme", ""), "dark");
        assert!(p.get_bool("optin", false));
        assert_eq!(p.get_int("count", 0), 4);
        p.remove("theme");
        drop(p);
        let p = SqlitePreferences::try_open_at(&base).unwrap();
        assert_eq!(p.get_string("theme", "gone"), "gone");
        assert!(path.is_file());
        drop(p);
        std::fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn relocation_copies_settings_and_retains_previous_database() {
        let source = temp_base("source");
        let destination = temp_base("destination");
        let locator = temp_base("locator");
        let mut p = SqlitePreferences::try_open_at(&source).unwrap();
        p.set_string("theme", "dark");
        let old_path = p.path();
        let mut existing = SqlitePreferences::try_open_at(&destination).unwrap();
        existing.set_bool("destination-only", true);
        drop(existing);
        p.relocate_with_locator(&destination, &locator).unwrap();
        assert_eq!(p.path(), destination.join("Configs/settings.sqlite3"));
        assert_eq!(p.get_string("theme", ""), "dark");
        assert!(!p.get_bool("destination-only", false));
        let old_destination_value: String = p
            .inner
            .lock()
            .unwrap()
            .connection
            .query_row(
                "SELECT value FROM settings_before_relocation WHERE key='destination-only'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(old_destination_value, "true");
        assert_eq!(
            std::fs::read_to_string(locator.join("workspace.path")).unwrap(),
            destination.to_string_lossy()
        );
        p.set_string("theme", "light");
        let old = SqlitePreferences::try_open_at(&source).unwrap();
        assert_eq!(old.get_string("theme", ""), "dark");
        assert!(old_path.is_file());
        drop(old);
        drop(p);
        for dir in [source, destination, locator] {
            std::fs::remove_dir_all(dir).unwrap();
        }
    }

    #[test]
    fn failed_relocation_keeps_previous_connection_and_values() {
        let source = temp_base("relocate-error");
        let dest = temp_base("blocked");
        let locator = temp_base("unused-locator");
        std::fs::write(&dest, "not a directory").unwrap();
        let mut p = SqlitePreferences::try_open_at(&source).unwrap();
        p.set_string("language", "de");
        assert!(p.relocate_with_locator(&dest, &locator).is_err());
        assert_eq!(p.path(), source.join("Configs/settings.sqlite3"));
        assert_eq!(p.get_string("language", ""), "de");
        assert!(!locator.exists());
        drop(p);
        std::fs::remove_dir_all(source).unwrap();
        std::fs::remove_file(dest).unwrap();
    }

    #[test]
    fn corrupt_import_does_not_hide_existing_database_settings() {
        let base = temp_base("damaged-import");
        let mut p = SqlitePreferences::try_open_at(&base).unwrap();
        p.set_string("language", "de");
        drop(p);
        std::fs::write(base.join("Configs/preferences.json"), "{broken").unwrap();
        let p = SqlitePreferences::try_open_at(&base).unwrap();
        assert_eq!(p.get_string("language", ""), "de");
        assert!(p.last_error().is_some());
        assert!(
            p.is_persistent(),
            "a damaged import must not hide durable settings"
        );
        drop(p);
        std::fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn unavailable_base_reports_transient_fallback() {
        let path = temp_base("unavailable-storage");
        std::fs::write(&path, "not a directory").unwrap();
        let p = SqlitePreferences::open_at(&path);
        assert!(!p.is_persistent());
        assert!(p.last_error().is_some());
        drop(p);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn relocation_retains_import_history_so_removed_tokens_do_not_reappear() {
        let source = temp_base("import-source");
        let destination = temp_base("import-dest");
        let locator = temp_base("import-locator");
        std::fs::create_dir_all(source.join("Configs")).unwrap();
        let legacy = source.join("Configs/preferences.json");
        std::fs::write(&legacy, r#"{"old-token":"removed"}"#).unwrap();
        let mut p = SqlitePreferences::try_open_at(&source).unwrap();
        p.remove("old-token");
        p.relocate_with_locator(&destination, &locator).unwrap();
        drop(p);
        let p = SqlitePreferences::open_with_imports(&destination, &[legacy]);
        assert_eq!(p.get_string("old-token", "gone"), "gone");
        drop(p);
        for dir in [source, destination, locator] {
            std::fs::remove_dir_all(dir).unwrap();
        }
    }

    #[test]
    fn relocation_never_resurrects_destination_login_or_legacy_json() {
        let source = temp_base("logged-out");
        let destination = temp_base("old-session");
        let locator = temp_base("session-locator");
        let mut old = SqlitePreferences::try_open_at(&destination).unwrap();
        old.set_string("greg_access_token", "old-token");
        old.set_string("greg_session_id", "old-session");
        drop(old);
        let legacy = destination.join("Configs/preferences.json");
        std::fs::write(&legacy, r#"{"greg_access_token":"legacy-token"}"#).unwrap();
        // Source has never logged in, so there are no credentials to remove.
        let p = SqlitePreferences::try_open_at(&source).unwrap();
        p.relocate_with_locator(&destination, &locator).unwrap();
        assert_eq!(p.get_string("greg_access_token", ""), "");
        assert_eq!(p.get_string("greg_session_id", ""), "");
        drop(p);
        let p = SqlitePreferences::open_with_imports(&destination, &[legacy]);
        assert_eq!(p.get_string("greg_access_token", ""), "");
        assert_eq!(p.get_string("greg_session_id", ""), "");
        let backup_token: String = p
            .inner
            .lock()
            .unwrap()
            .connection
            .query_row(
                "SELECT value FROM settings_before_relocation WHERE key='greg_access_token'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(backup_token, r#""old-token""#);
        drop(p);
        for dir in [source, destination, locator] {
            std::fs::remove_dir_all(dir).unwrap();
        }
    }

    #[test]
    fn diagnostics_can_be_consumed_once_without_affecting_persistence() {
        let base = temp_base("diagnostics");
        std::fs::create_dir_all(base.join("Configs")).unwrap();
        std::fs::write(base.join("Configs/preferences.json"), "{broken").unwrap();
        let p = SqlitePreferences::try_open_at(&base).unwrap();
        assert!(p.is_persistent());
        assert!(p.take_error().is_some());
        assert!(p.take_error().is_none());
        assert!(p.is_persistent());
        drop(p);
        std::fs::remove_dir_all(base).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn store_and_parent_protect_session_tokens() {
        use std::os::unix::fs::PermissionsExt;
        let base = temp_base("permissions");
        let mut p = SqlitePreferences::try_open_at(&base).unwrap();
        p.set_string("access-token", "test-only");
        assert_eq!(
            std::fs::metadata(p.path()).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert_eq!(
            std::fs::metadata(base.join("Configs"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
        drop(p);
        std::fs::remove_dir_all(base).unwrap();
    }
}
