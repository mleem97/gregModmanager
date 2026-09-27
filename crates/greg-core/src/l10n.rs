//! Localization: English + German string table (see `l10n_strings.rs`).
//!
//! Other cultures fall back to English, mirroring the C# satellite behavior.

#[path = "l10n_strings.rs"]
mod l10n_strings;

/// Languages offered by the app (code, display name).
pub const SUPPORTED_LANGUAGES: &[(&str, &str)] = &[
    ("", "System default"),
    ("en", "English"),
    ("de", "Deutsch"),
    ("fr", "Français"),
    ("es", "Español"),
    ("it", "Italiano"),
    ("ja", "日本語"),
    ("pl", "Polski"),
    ("ru", "Русский"),
    ("zh", "中文"),
];

/// Raw string for `key` in `lang` (`"de"` → German, anything else → English).
/// Falls back to the key itself when missing.
pub fn get(lang: &str, key: &str) -> String {
    l10n_strings::get_raw(lang, key).unwrap_or(key).to_string()
}

/// Formats a string with `{0}`, `{1}`, … placeholders.
pub fn format(lang: &str, key: &str, args: &[&dyn std::fmt::Display]) -> String {
    let mut out = get(lang, key);
    for (i, arg) in args.iter().enumerate() {
        out = out.replace(&format!("{{{i}}}"), &arg.to_string());
    }
    out
}

/// Number of localized keys.
pub fn key_count() -> usize {
    l10n_strings::key_count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_is_complete() {
        assert_eq!(key_count(), 352);
        assert_eq!(get("en", "Editor_Description"), "Description");
        assert_eq!(get("de", "Editor_Description"), "Beschreibung");
        // Unknown cultures fall back to English.
        assert_eq!(get("fr", "Editor_Description"), "Description");
    }

    #[test]
    fn formats_placeholders() {
        let s = format("en", "Editor_FileId", &[&12345u64]);
        assert_eq!(s, "Workshop file id: 12345");
    }
}
