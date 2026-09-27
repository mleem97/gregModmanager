//! SteamCMD `workshop_build_item.vdf` generator (`VdfGeneratorService` port).

use greg_core::models::settings::DATA_CENTER_APP_ID;

/// Builds a minimal SteamCMD `workshopitem` VDF for manual uploads.
pub fn build_workshop_item_vdf(
    content_folder: &str,
    preview_file: Option<&str>,
    published_file_id: Option<u64>,
) -> String {
    let id = match published_file_id {
        Some(id) if id != 0 => format!("\"{id}\""),
        _ => "\"0\"".to_string(),
    };
    let preview = match preview_file {
        Some(p) if !p.trim().is_empty() => format!("\"{}\"", escape(p)),
        _ => "\"\"".to_string(),
    };
    format!(
        "\"workshopitem\"\n{{\n\t\"appid\" \"{DATA_CENTER_APP_ID}\"\n\t\"publishedfileid\" {id}\n\t\"content\" \"{content}\"\n\t\"preview\" {preview}\n}}",
        content = escape(content_folder),
    )
}

fn escape(path: &str) -> String {
    path.replace('\\', "/").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_new_item_vdf() {
        let vdf = build_workshop_item_vdf("C:\\mods\\x", Some("preview.png"), None);
        assert!(vdf.contains("\"appid\" \"4170200\""));
        assert!(vdf.contains("\"publishedfileid\" \"0\""));
        assert!(vdf.contains("\"content\" \"C:/mods/x\""));
    }
}
