//! Localization across all supported languages (`en`, `de`, `fr`, `es`, `it`, `ja`, `pl`, `ru`, `zh`)
//! with transparent fallback to English (`en`).

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

/// Resolves a language preference (including `""` for System default or OS locale tags
/// such as `"de_DE.UTF-8"`) to one of the supported two-letter language codes
/// (`"en"`, `"de"`, `"fr"`, `"es"`, `"it"`, `"ja"`, `"pl"`, `"ru"`, `"zh"`).
pub fn resolve_lang_code(lang: &str) -> &'static str {
    let trimmed = lang.trim();
    if trimmed.is_empty() {
        let sys = std::env::var("LC_ALL")
            .ok()
            .filter(|v| !v.trim().is_empty())
            .or_else(|| std::env::var("LANG").ok().filter(|v| !v.trim().is_empty()))
            .unwrap_or_default();
        return match_lang_prefix(&sys);
    }
    match_lang_prefix(trimmed)
}

fn match_lang_prefix(raw: &str) -> &'static str {
    let lower = raw.trim().to_ascii_lowercase();
    let code = lower
        .split(['_', '-', '.', '@'])
        .next()
        .unwrap_or(lower.as_str());
    match code {
        "de" => "de",
        "fr" => "fr",
        "es" => "es",
        "it" => "it",
        "ja" => "ja",
        "pl" => "pl",
        "ru" => "ru",
        "zh" => "zh",
        _ => "en",
    }
}

/// Localized string for `key` in `lang` (`de`, `fr`, `es`, `it`, `ja`, `pl`, `ru`, `zh`, `en`,
/// or `""` for system locale detection).
/// Transparently falls back to English when a key is absent in the requested language table,
/// and finally to `key` itself when completely unknown.
pub fn get(lang: &str, key: &str) -> String {
    l10n_strings::get_raw(lang, key).unwrap_or(key).to_string()
}

/// Formats a localized string with `{0}`, `{1}`, … placeholders.
pub fn format(lang: &str, key: &str, args: &[&dyn std::fmt::Display]) -> String {
    let mut out = get(lang, key);
    for (i, arg) in args.iter().enumerate() {
        out = out.replace(&format!("{{{i}}}"), &arg.to_string());
    }
    out
}

/// Number of localized keys in the primary English table.
pub fn key_count() -> usize {
    l10n_strings::key_count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_table_is_strictly_sorted_and_searchable() {
        for (lang, table) in l10n_strings::ALL_TABLES {
            assert!(!table.is_empty(), "table for {lang} must not be empty");
            for pair in table.windows(2) {
                let (k1, _) = pair[0];
                let (k2, _) = pair[1];
                assert!(
                    k1 < k2,
                    "table '{lang}' is not strictly sorted: '{k1}' >= '{k2}'"
                );
            }
            for (k, v) in *table {
                assert_eq!(
                    l10n_strings::get_raw(lang, k),
                    Some(*v),
                    "binary search failed for ({lang}, {k})"
                );
            }
        }
    }

    #[test]
    fn table_is_complete() {
        assert_eq!(key_count(), 582);
        assert_eq!(get("en", "Editor_Description"), "Description");
        assert_eq!(get("de", "Editor_Description"), "Beschreibung");
        assert_eq!(get("es", "Editor_Description"), "Descripción");
        assert_eq!(get("it", "Editor_Description"), "Descrizione");
        assert_eq!(get("ja", "Editor_Description"), "説明");
        assert_eq!(get("pl", "Editor_Description"), "Opis");
        assert_eq!(get("ru", "Editor_Description"), "Описание");
        assert_eq!(get("zh", "Editor_Description"), "描述");
        // UI keys across all 9 languages.
        assert_eq!(get("en", "Ui_Nav_MyMods"), "My Mods");
        assert_eq!(get("de", "Ui_Nav_MyMods"), "Meine Mods");
        assert_eq!(get("fr", "Ui_Nav_MyMods"), "Mes Mods");
        assert_eq!(get("es", "Ui_Nav_MyMods"), "Mis Mods");
        assert_eq!(get("it", "Ui_Nav_MyMods"), "Le mie Mod");
        assert_eq!(get("ja", "Ui_Nav_MyMods"), "マイMod");
        assert_eq!(get("pl", "Ui_Nav_MyMods"), "Moje mody");
        assert_eq!(get("ru", "Ui_Nav_MyMods"), "Мои моды");
        assert_eq!(get("zh", "Ui_Nav_MyMods"), "我的模组");
        // Unknown cultures fall back to English.
        assert_eq!(get("sv", "Editor_Description"), "Description");
        // Unknown keys fall back to the key itself.
        assert_eq!(get("en", "NonExistent_Key_123"), "NonExistent_Key_123");
    }

    #[test]
    fn resolve_lang_code_handles_tags_and_system_default() {
        assert_eq!(resolve_lang_code("de"), "de");
        assert_eq!(resolve_lang_code("de_DE.UTF-8"), "de");
        assert_eq!(resolve_lang_code("fr-FR"), "fr");
        assert_eq!(resolve_lang_code("es_ES"), "es");
        assert_eq!(resolve_lang_code("it_IT"), "it");
        assert_eq!(resolve_lang_code("ja_JP.UTF-8"), "ja");
        assert_eq!(resolve_lang_code("pl_PL"), "pl");
        assert_eq!(resolve_lang_code("ru_RU.UTF-8"), "ru");
        assert_eq!(resolve_lang_code("zh_CN.UTF-8"), "zh");
        assert_eq!(resolve_lang_code("en_US.UTF-8"), "en");
        assert_eq!(resolve_lang_code("unknown"), "en");
        let sys = resolve_lang_code("");
        assert!(
            ["en", "de", "fr", "es", "it", "ja", "pl", "ru", "zh"].contains(&sys),
            "unexpected resolved system lang: {sys}"
        );
    }

    #[test]
    fn formats_placeholders() {
        let s = format("en", "Editor_FileId", &[&12345u64]);
        assert_eq!(s, "Workshop file id: 12345");
        let s_de = format("de", "Ui_Local_StatusSummary", &[&3usize, &1usize, &"Mods"]);
        assert_eq!(s_de, "3 aktiv, 1 deaktiviert in Mods");
    }
}
