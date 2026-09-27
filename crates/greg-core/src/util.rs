//! Small pure helpers (byte formatting, folder-name sanitation).

/// Formats a byte count like `1.5 MB` (mirrors `WorkspaceService.FormatBytes`).
pub fn format_bytes(bytes: i64) -> String {
    let bytes = bytes.max(0) as f64;
    let suffixes = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes;
    let mut index = 0;
    while value >= 1024.0 && index < suffixes.len() - 1 {
        value /= 1024.0;
        index += 1;
    }
    if (value - value.round()).abs() < f64::EPSILON {
        format!("{} {}", value as i64, suffixes[index])
    } else {
        format!("{value:.2} {}", suffixes[index])
    }
}

/// Today's date as `YYYY-MM-DD` (civil calendar, no external crates).
pub fn today_ymd() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    days_to_ymd(secs / 86_400)
}

/// Converts days since the Unix epoch to `YYYY-MM-DD`.
pub fn days_to_ymd(days: u64) -> String {
    // Howard Hinnant's civil-from-days algorithm.
    let z = days as i64 + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    format!("{:04}-{:02}-{:02}", if m <= 2 { y + 1 } else { y }, m, d)
}

/// Sanitizes a single path segment for a project folder.
pub fn sanitize_folder_name(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let mut s: String = trimmed
        .chars()
        .map(|c| {
            if matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') || c.is_control() {
                '_'
            } else {
                c
            }
        })
        .collect();
    while s.contains("..") {
        s = s.replace("..", "_");
    }
    let s = s.trim_matches(|c| c == '.' || c == ' ');
    let mut out: String = s.chars().take(80).collect();
    out = out.trim_end().to_string();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(2048), "2 KB");
        assert!(format_bytes(1_572_864).contains("MB"));
    }

    #[test]
    fn sanitizes_names() {
        assert_eq!(sanitize_folder_name(""), "");
        assert_eq!(sanitize_folder_name("my/mod:name"), "my_mod_name");
        // Faithful to C#: separators become `_`, then `..` collapses to `_`.
        assert_eq!(sanitize_folder_name("a/../b"), "a___b");
        assert_eq!(sanitize_folder_name("  .dots.  "), "dots");
    }
}
