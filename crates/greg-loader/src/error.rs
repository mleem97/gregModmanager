//! Loader error type.

/// Failures from game/mod integration.
#[derive(Debug, thiserror::Error)]
pub enum LoaderError {
    /// Game installation problem.
    #[error("game error: {0}")]
    Game(String),
    /// Filesystem failure.
    #[error("I/O error: {0}")]
    Io(String),
    /// JSON failure.
    #[error("JSON error: {0}")]
    Json(String),
    /// Network failure (release feeds, downloads).
    #[error("network error: {0}")]
    Network(String),
    /// Incompatible or unsafe artefact.
    #[error("invalid artefact: {0}")]
    InvalidArtefact(String),
    /// Cancelled by the user.
    #[error("cancelled")]
    Cancelled,
    /// Anything else with context.
    #[error("{0}")]
    Other(String),
}

/// Convenience alias.
pub type Result<T> = std::result::Result<T, LoaderError>;

/// Maps [`std::io::Error`].
pub fn io_err(e: std::io::Error) -> LoaderError {
    LoaderError::Io(e.to_string())
}

/// Maps [`serde_json::Error`].
pub fn json_err(e: serde_json::Error) -> LoaderError {
    LoaderError::Json(e.to_string())
}
