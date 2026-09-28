//! Upload-readiness checks shared by UI, CLI and services.

use serde::{Deserialize, Serialize};

/// Severity of a single check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UploadCheckSeverity {
    /// All good.
    Ok,
    /// Should be fixed, but upload may proceed.
    Warning,
    /// Blocks the upload.
    Error,
}

/// One labelled check result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UploadCheckResult {
    /// Short label (`Title`, `Changelog`, …).
    pub label: String,
    /// Outcome.
    pub severity: UploadCheckSeverity,
    /// Human-readable detail.
    pub detail: String,
}

impl UploadCheckResult {
    /// Creates an `Ok` result.
    pub fn ok(label: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            severity: UploadCheckSeverity::Ok,
            detail: detail.into(),
        }
    }

    /// Creates a warning result.
    pub fn warning(label: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            severity: UploadCheckSeverity::Warning,
            detail: detail.into(),
        }
    }

    /// Creates an error result.
    pub fn error(label: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            severity: UploadCheckSeverity::Error,
            detail: detail.into(),
        }
    }
}

/// True when no check failed with an error.
pub fn is_ready_to_upload(results: &[UploadCheckResult]) -> bool {
    results
        .iter()
        .all(|r| r.severity != UploadCheckSeverity::Error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readiness_gate() {
        assert!(is_ready_to_upload(&[]));
        assert!(is_ready_to_upload(&[UploadCheckResult::warning("T", "d")]));
        assert!(!is_ready_to_upload(&[UploadCheckResult::error("T", "d")]));
    }
}
