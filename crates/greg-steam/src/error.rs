//! Steam error type.

/// Failures from Steam Workshop integration.
#[derive(Debug, thiserror::Error)]
pub enum SteamError {
    /// Steam client/API unavailable (not running, no user, ...).
    #[error("Steam is not available: {0}")]
    NotAvailable(String),
    /// Native/API call failed.
    #[error("Steam API error: {0}")]
    Api(String),
    /// Workshop legal agreement must be accepted in the Steam client.
    #[error("Workshop legal agreement must be accepted in the Steam client.")]
    NeedsAgreement,
    /// Publish rejected with detail.
    #[error("Steam publish failed: {0}")]
    PublishFailed(String),
    /// Operation timed out waiting for Steam.
    #[error("Steam operation timed out: {0}")]
    Timeout(String),
    /// Cancelled by the user.
    #[error("cancelled")]
    Cancelled,
    /// Anything else with context.
    #[error("{0}")]
    Other(String),
}

/// Convenience alias.
pub type Result<T> = std::result::Result<T, SteamError>;
