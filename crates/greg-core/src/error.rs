//! Shared error type for `greg-core`.

/// Errors produced by pure domain logic.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    /// Filesystem failure.
    #[error("I/O error: {0}")]
    Io(String),
    /// JSON (de)serialization failure.
    #[error("JSON error: {0}")]
    Json(String),
    /// Invalid semantic version.
    #[error("invalid version '{0}': {1}")]
    Version(String, String),
    /// Changelog format or lookup failure.
    #[error("changelog error: {0}")]
    Changelog(String),
    /// Missing entity (project, entry, section, ...).
    #[error("not found: {0}")]
    NotFound(String),
    /// Anything else with context.
    #[error("{0}")]
    Other(String),
}

/// Convenience alias.
pub type Result<T> = std::result::Result<T, CoreError>;

/// Maps [`std::io::Error`] into [`CoreError::Io`].
pub fn io_err(e: std::io::Error) -> CoreError {
    CoreError::Io(e.to_string())
}

/// Maps [`serde_json::Error`] into [`CoreError::Json`].
pub fn json_err(e: serde_json::Error) -> CoreError {
    CoreError::Json(e.to_string())
}
