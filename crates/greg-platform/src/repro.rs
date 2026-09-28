//! Support reproduction bundle (`ReproBundleService` port).
//!
//! Zips the current log, crash reports and a manifest into one file.
//! Never includes tokens or credentials.

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

    // Manifest first (redacted by the caller).
    zip.start_file("manifest.json", options)
        .map_err(|e| crate::error::PlatformError::Other(e.to_string()))?;
    let text = serde_json::to_string_pretty(manifest).map_err(crate::error::json_err)?;
    zip.write_all(text.as_bytes())
        .map_err(|e| crate::error::PlatformError::Other(e.to_string()))?;

    // Current log (tail: last 2000 lines max).
    if log_path.is_file() {
        let content = std::fs::read_to_string(log_path).map_err(io_err)?;
        let lines: Vec<&str> = content.lines().collect();
        let start = lines.len().saturating_sub(2000);
        zip.start_file("app.log", options)
            .map_err(|e| crate::error::PlatformError::Other(e.to_string()))?;
        zip.write_all(lines[start..].join("\n").as_bytes())
            .map_err(|e| crate::error::PlatformError::Other(e.to_string()))?;
    }

    // Crash reports.
    if reports_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(reports_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("json") {
                    if let Ok(data) = std::fs::read(&path) {
                        let name = format!(
                            "reports/{}",
                            path.file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("report.json")
                        );
                        zip.start_file(name, options)
                            .map_err(|e| crate::error::PlatformError::Other(e.to_string()))?;
                        zip.write_all(&data)
                            .map_err(|e| crate::error::PlatformError::Other(e.to_string()))?;
                    }
                }
            }
        }
    }

    zip.finish()
        .map_err(|e| crate::error::PlatformError::Other(e.to_string()))?;
    Ok(bundle)
}
