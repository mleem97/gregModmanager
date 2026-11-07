//! Anonymized support reproduction bundle.
//!
//! Raw crash report details are excluded because arbitrary stack traces may
//! contain account names, mod titles, and custom paths that cannot be inferred.

use std::io::Write as _;
use std::path::{Path, PathBuf};

use crate::error::{io_err, Result};

/// Creates `repro-<timestamp>.zip` in `out_dir` (or the temp dir).
/// Returns the bundle path.
pub fn create_bundle(
    log_path: &Path,
    reports_dir: &Path,
    manifest: &serde_json::Value,
    out_dir: Option<&Path>,
) -> Result<PathBuf> {
    let dir = out_dir
        .map(Path::to_path_buf)
        .unwrap_or_else(std::env::temp_dir);
    std::fs::create_dir_all(&dir).map_err(io_err)?;
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let bundle = dir.join(format!("repro-{stamp}.zip"));
    let file = std::fs::File::create(&bundle).map_err(io_err)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    zip.start_file("manifest.json", options)
        .map_err(|e| crate::error::PlatformError::Other(e.to_string()))?;
    let text =
        serde_json::to_string_pretty(&sanitize_json(manifest)).map_err(crate::error::json_err)?;
    zip.write_all(text.as_bytes())
        .map_err(|e| crate::error::PlatformError::Other(e.to_string()))?;

    if log_path.is_file() {
        let content = std::fs::read_to_string(log_path).map_err(io_err)?;
        let lines: Vec<&str> = content.lines().collect();
        let start = lines.len().saturating_sub(2000);
        let safe = crate::filelog::redact_diagnostic_text(&lines[start..].join("\n"));
        zip.start_file("app.log", options)
            .map_err(|e| crate::error::PlatformError::Other(e.to_string()))?;
        zip.write_all(safe.as_bytes())
            .map_err(|e| crate::error::PlatformError::Other(e.to_string()))?;
    }

    // Provide only a count; do not copy arbitrary crash messages or traces.
    if reports_dir.is_dir() {
        let count = std::fs::read_dir(reports_dir)
            .ok()
            .into_iter()
            .flatten()
            .flatten()
            .filter(|entry| entry.path().extension().and_then(|e| e.to_str()) == Some("json"))
            .count();
        zip.start_file("reports/summary.json", options)
            .map_err(|e| crate::error::PlatformError::Other(e.to_string()))?;
        let summary = serde_json::json!({
            "redactedReportCount": count,
            "details": "Crash report contents omitted for privacy"
        });
        zip.write_all(
            serde_json::to_string(&summary)
                .map_err(crate::error::json_err)?
                .as_bytes(),
        )
        .map_err(|e| crate::error::PlatformError::Other(e.to_string()))?;
    }

    zip.finish()
        .map_err(|e| crate::error::PlatformError::Other(e.to_string()))?;
    Ok(bundle)
}

fn sanitize_json(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::String(text) => {
            serde_json::Value::String(crate::filelog::redact_diagnostic_text(text))
        }
        serde_json::Value::Array(values) => {
            serde_json::Value::Array(values.iter().map(sanitize_json).collect())
        }
        serde_json::Value::Object(fields) => {
            let mut clean = serde_json::Map::new();
            for (key, value) in fields {
                let normalized = key.to_ascii_lowercase().replace(['_', '-'], "");
                let identifying = [
                    "id", "userid", "account", "username", "email", "path", "token", "secret",
                    "identity", "session", "title",
                ]
                .iter()
                .any(|marker| normalized.contains(marker));
                if !identifying {
                    clean.insert(key.clone(), sanitize_json(value));
                }
            }
            serde_json::Value::Object(clean)
        }
        _ => value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read as _;

    #[test]
    fn support_bundle_redacts_logs_and_omits_crash_details() {
        let base = std::env::temp_dir().join(format!(
            "greg-repro-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let logs = base.join("Logs");
        let reports = logs.join("reports");
        std::fs::create_dir_all(&reports).unwrap();
        let log_path = logs.join("app.log");
        std::fs::write(
            &log_path,
            "Login failed for marvin@example.test at /home/marvin/private/Game; SteamID 76561198000000000\n",
        )
        .unwrap();
        std::fs::write(
            reports.join("crash.json"),
            r#"{"processId":1234,"message":"Marvin private mod title"}"#,
        )
        .unwrap();
        let bundle = create_bundle(
            &log_path,
            &reports,
            &serde_json::json!({
                "os":"linux",
                "accountName":"MLeeM",
                "workspacePath":"/home/marvin/private"
            }),
            Some(&base),
        )
        .unwrap();
        let file = std::fs::File::open(&bundle).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        let mut log = String::new();
        archive
            .by_name("app.log")
            .unwrap()
            .read_to_string(&mut log)
            .unwrap();
        assert!(!log.contains("marvin@example.test"));
        assert!(!log.contains("/home/marvin"));
        assert!(!log.contains("76561198000000000"));
        let mut manifest = String::new();
        archive
            .by_name("manifest.json")
            .unwrap()
            .read_to_string(&mut manifest)
            .unwrap();
        assert!(!manifest.contains("MLeeM"));
        assert!(!manifest.contains("workspacePath"));
        let mut summary = String::new();
        archive
            .by_name("reports/summary.json")
            .unwrap()
            .read_to_string(&mut summary)
            .unwrap();
        assert!(!summary.contains("Marvin"));
        assert!(summary.contains("redactedReportCount"));
        assert!(archive.by_name("reports/crash.json").is_err());
        drop(archive);
        std::fs::remove_dir_all(base).unwrap();
    }
}
