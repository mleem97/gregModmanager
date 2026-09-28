//! Key-value settings contract (`IPreferences` port).

/// Minimal settings store used by UI, CLI and services.
pub trait Preferences: Send + Sync {
    /// Reads a string value.
    fn get_string(&self, key: &str, default: &str) -> String;
    /// Writes a string value.
    fn set_string(&mut self, key: &str, value: &str);
    /// Reads a boolean value (`"1"`/`"true"` → true).
    fn get_bool(&self, key: &str, default: bool) -> bool {
        match self.get_string(key, "").as_str() {
            "" => default,
            v => v == "1" || v.eq_ignore_ascii_case("true"),
        }
    }
    /// Writes a boolean value.
    fn set_bool(&mut self, key: &str, value: bool) {
        self.set_string(key, if value { "1" } else { "0" });
    }
    /// Reads an integer value.
    fn get_int(&self, key: &str, default: i64) -> i64 {
        let raw = self.get_string(key, "");
        if raw.is_empty() {
            return default;
        }
        raw.parse().unwrap_or(default)
    }
    /// Writes an integer value.
    fn set_int(&mut self, key: &str, value: i64) {
        self.set_string(key, &value.to_string());
    }
    /// Removes a key.
    fn remove(&mut self, key: &str);
}

/// In-memory implementation (tests, headless defaults).
#[derive(Debug, Default)]
pub struct MemoryPreferences {
    map: std::collections::HashMap<String, String>,
}

impl MemoryPreferences {
    /// Creates an empty store.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Preferences for MemoryPreferences {
    fn get_string(&self, key: &str, default: &str) -> String {
        self.map.get(key).cloned().unwrap_or_else(|| default.into())
    }
    fn set_string(&mut self, key: &str, value: &str) {
        self.map.insert(key.into(), value.into());
    }
    fn remove(&mut self, key: &str) {
        self.map.remove(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_helpers() {
        let mut p = MemoryPreferences::new();
        assert!(p.get_bool("x", true));
        p.set_bool("x", false);
        assert!(!p.get_bool("x", true));
        assert_eq!(p.get_int("n", 7), 7);
        p.set_int("n", 42);
        assert_eq!(p.get_int("n", 7), 42);
        p.remove("n");
        assert_eq!(p.get_int("n", 7), 7);
    }
}
