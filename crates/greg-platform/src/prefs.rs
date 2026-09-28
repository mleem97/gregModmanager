//! JSON-file settings store (`JsonFilePreferences` port).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use greg_core::prefs::Preferences;
use serde_json::Value;

use crate::error::{io_err, json_err, Result};
use crate::paths::app_data_dir;

/// `preferences.json` in the app-data dir, mirroring the C# behavior
/// (corrupt files are ignored, values keep their JSON types).
#[derive(Debug, Clone)]
pub struct JsonFilePreferences {
    inner: Arc<Mutex<PrefsInner>>,
}

#[derive(Debug)]
struct PrefsInner {
    path: PathBuf,
    data: HashMap<String, Value>,
}

impl JsonFilePreferences {
    /// Opens (or creates) the store. Best-effort: never fails.
    pub fn open() -> Self {
        let path = app_data_dir()
            .map(|d| d.join("preferences.json"))
            .unwrap_or_else(|| std::env::temp_dir().join("gregmodmanager-preferences.json"));
        let mut inner = PrefsInner {
            path,
            data: HashMap::new(),
        };
        if let Ok(text) = std::fs::read_to_string(&inner.path) {
            if let Ok(Value::Object(map)) = serde_json::from_str::<Value>(&text) {
                inner.data = map.into_iter().collect();
            }
        }
        Self {
            inner: Arc::new(Mutex::new(inner)),
        }
    }

    /// Storage path (for diagnostics).
    pub fn path(&self) -> PathBuf {
        self.inner.lock().expect("prefs lock").path.clone()
    }

    fn save_locked(inner: &PrefsInner) -> Result<()> {
        if let Some(dir) = inner.path.parent() {
            std::fs::create_dir_all(dir).map_err(io_err)?;
        }
        let text = serde_json::to_string_pretty(&inner.data).map_err(json_err)?;
        std::fs::write(&inner.path, text).map_err(io_err)?;
        // The store holds the OAuth access token: owner-only permissions.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&inner.path, std::fs::Permissions::from_mode(0o600));
        }
        Ok(())
    }
}

impl Preferences for JsonFilePreferences {
    fn get_string(&self, key: &str, default: &str) -> String {
        let inner = self.inner.lock().expect("prefs lock");
        inner
            .data
            .get(key)
            .and_then(|v| v.as_str())
            .unwrap_or(default)
            .to_string()
    }

    fn set_string(&mut self, key: &str, value: &str) {
        let mut inner = self.inner.lock().expect("prefs lock");
        inner.data.insert(key.into(), Value::String(value.into()));
        let _ = Self::save_locked(&inner);
    }

    fn get_bool(&self, key: &str, default: bool) -> bool {
        let inner = self.inner.lock().expect("prefs lock");
        match inner.data.get(key) {
            Some(Value::Bool(b)) => *b,
            _ => default,
        }
    }

    fn set_bool(&mut self, key: &str, value: bool) {
        let mut inner = self.inner.lock().expect("prefs lock");
        inner.data.insert(key.into(), Value::Bool(value));
        let _ = Self::save_locked(&inner);
    }

    fn get_int(&self, key: &str, default: i64) -> i64 {
        let inner = self.inner.lock().expect("prefs lock");
        match inner.data.get(key) {
            Some(Value::Number(n)) => n.as_i64().unwrap_or(default),
            _ => default,
        }
    }

    fn set_int(&mut self, key: &str, value: i64) {
        let mut inner = self.inner.lock().expect("prefs lock");
        inner.data.insert(key.into(), Value::Number(value.into()));
        let _ = Self::save_locked(&inner);
    }

    fn remove(&mut self, key: &str) {
        let mut inner = self.inner.lock().expect("prefs lock");
        inner.data.remove(key);
        let _ = Self::save_locked(&inner);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_roundtrip() {
        let mut p = JsonFilePreferences::open();
        p.set_string("k", "v");
        assert_eq!(p.get_string("k", ""), "v");
        p.set_bool("b", true);
        assert!(p.get_bool("b", false));
        p.set_int("n", 3);
        assert_eq!(p.get_int("n", 0), 3);
        // String "1"/"true" must NOT count as bool (typed JSON, like C#).
        p.set_string("s", "true");
        assert!(!p.get_bool("s", false));
    }

    /// Token store must be owner-only (OAuth access token lives here).
    #[cfg(unix)]
    #[test]
    fn store_is_owner_only() {
        use std::os::unix::fs::PermissionsExt;
        let mut p = JsonFilePreferences::open();
        p.set_string("perm-probe", "x");
        p.remove("perm-probe");
        let mode = std::fs::metadata(p.path())
            .expect("prefs file")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600, "preferences.json must be 0600, got {mode:o}");
    }
}
