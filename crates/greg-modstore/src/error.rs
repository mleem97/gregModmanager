//! Modstore error type.

/// Failures from the Modstore client.
#[derive(Debug, thiserror::Error)]
pub enum ModStoreError {
    /// Disabled or not configured.
    #[error("Modstore is not available: {0}")]
    NotAvailable(String),
    /// HTTP failure on all endpoints.
    #[error("request failed: {0}")]
    Request(String),
    /// Untrusted or malformed artefact.
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
pub type Result<T> = std::result::Result<T, ModStoreError>;
