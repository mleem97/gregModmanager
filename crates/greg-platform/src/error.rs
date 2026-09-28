//! Platform error type.

/// Errors from OS/filesystem integration.
#[derive(Debug, thiserror::Error)]
pub enum PlatformError {
    /// Filesystem failure.
    #[error("I/O error: {0}")]
    Io(String),
    /// JSON failure.
    #[error("JSON error: {0}")]
    Json(String),
    /// Invalid project layout or name.
    #[error("invalid project: {0}")]
    Project(String),
    /// Network failure (telemetry, downloads of helpers).
    #[error("network error: {0}")]
    Network(String),
    /// Anything else with context.
    #[error("{0}")]
    Other(String),
}

/// Convenience alias.
pub type Result<T> = std::result::Result<T, PlatformError>;

/// Maps [`std::io::Error`].
pub fn io_err(e: std::io::Error) -> PlatformError {
    PlatformError::Io(e.to_string())
}

/// Maps [`serde_json::Error`].
pub fn json_err(e: serde_json::Error) -> PlatformError {
    PlatformError::Json(e.to_string())
}

impl From<greg_core::CoreError> for PlatformError {
    fn from(e: greg_core::CoreError) -> Self {
        PlatformError::Other(e.to_string())
    }
}
