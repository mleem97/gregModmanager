//! Dependency heuristics from Steam tags/descriptions
//! (`WorkshopDependencyInference` port).

/// Inferred requirement hints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyHints {
    /// One-line summary for lists.
    pub compact_line: String,
    /// Bulleted block for detail views.
    pub bullet_block: String,
    /// True when anything was inferred.
    pub has_any: bool,
}

/// Infers MelonLoader / gregCoreModFramework requirements.
pub fn infer(lang: &str, tags: &[String], description: &str) -> DependencyHints {
    let tag_set: Vec<String> = tags.iter().map(|t| t.trim().to_lowercase()).collect();
    let has_tag = |t: &str| tag_set.iter().any(|x| x == t);
    let desc = description.to_lowercase();

    let greg = has_tag("greg")
        || has_tag("gregcore-mod-framework")
        || desc.contains("gregcoremodframework");
    let melon =
        !greg && (has_tag("melonloader") || has_tag("modded") || desc.contains("melonloader"));

    let mut parts = Vec::new();
    if greg {
        parts.push(greg_core::l10n::get(lang, "Mod_Dep_MelonLoader"));
        parts.push(greg_core::l10n::get(lang, "Mod_Dep_greg"));
    } else if melon {
        parts.push(greg_core::l10n::get(lang, "Mod_Dep_MelonLoader"));
    }
    if parts.is_empty() {
        return DependencyHints {
            compact_line: String::new(),
            bullet_block: String::new(),
            has_any: false,
        };
    }
    let joined = parts.join(" · ");
    DependencyHints {
        compact_line: greg_core::l10n::format(lang, "Mod_StoreDepsLine", &[&joined]),
        bullet_block: parts
            .iter()
            .map(|p| format!("• {p}"))
            .collect::<Vec<_>>()
            .join("\n"),
        has_any: true,
    }
}

/// Extracts numeric Workshop ids from free text (`workshop_dependency` port).
pub fn infer_ids(text: &str) -> Vec<u64> {
    let mut ids = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_digit() {
            let mut j = i;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                j += 1;
            }
            if let Ok(id) = text[i..j].parse::<u64>() {
                if id > 0 && id <= greg_core::limits::MAX_WORKSHOP_FILE_ID {
                    ids.push(id);
                }
            }
            i = j;
        } else {
            i += 1;
        }
    }
    ids.sort_unstable();
    ids.dedup();
    ids
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tags(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn infers_greg_from_tag() {
        let hints = infer("en", &tags(&["greg"]), "");
        assert!(hints.has_any);
        assert!(hints.compact_line.contains('·'));
    }

    #[test]
    fn infers_melon_only() {
        let hints = infer("en", &tags(&["modded"]), "");
        assert!(hints.has_any);
    }

    #[test]
    fn empty_without_hints() {
        let hints = infer("en", &tags(&["vanilla"]), "plain");
        assert!(!hints.has_any);
    }

    #[test]
    fn extracts_ids() {
        assert_eq!(
            infer_ids("needs 1234567890 and 42"),
            vec![42, 1_234_567_890]
        );
        assert!(infer_ids("no numbers here").is_empty());
    }
}
