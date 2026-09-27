//! Telemetry secrets + Loki sender (opt-in only).

use greg_core::models::LokiPushRequest;

use crate::error::{PlatformError, Result};

/// Credentials from the environment (empty → telemetry inactive).
#[derive(Debug, Clone, Default)]
pub struct TelemetrySecrets {
    /// Loki push URL.
    pub url: String,
    /// Basic-auth user.
    pub user: String,
    /// Basic-auth password / token.
    pub pass: String,
    /// Tenant / org id header.
    pub tenant: String,
}

impl TelemetrySecrets {
    /// Reads `GREG_LOKI_URL/USER/PASS/TENANT`.
    pub fn from_env() -> Self {
        Self {
            url: std::env::var("GREG_LOKI_URL").unwrap_or_default(),
            user: std::env::var("GREG_LOKI_USER").unwrap_or_default(),
            pass: std::env::var("GREG_LOKI_PASS").unwrap_or_default(),
            tenant: std::env::var("GREG_LOKI_TENANT").unwrap_or_default(),
        }
    }

    /// Local test endpoint override (mirrors the C# local-build behavior).
    pub fn push_url(&self) -> String {
        if std::env::var("GREG_LOCAL_BUILD").as_deref() == Ok("1") {
            return "http://localhost:3100/loki/api/v1/push".to_string();
        }
        self.url.clone()
    }

    /// True when a push URL is configured.
    pub fn is_configured(&self) -> bool {
        !self.push_url().is_empty()
    }
}

/// Pushes a Loki request. No-op when unconfigured.
pub async fn push_loki(secrets: &TelemetrySecrets, payload: &LokiPushRequest) -> Result<()> {
    let url = secrets.push_url();
    if url.is_empty() {
        return Ok(());
    }
    let client = reqwest::Client::new();
    let mut req = client.post(&url).json(payload);
    if !secrets.user.is_empty() {
        req = req.basic_auth(&secrets.user, Some(&secrets.pass));
    }
    if !secrets.tenant.is_empty() {
        req = req.header("X-Scope-OrgID", &secrets.tenant);
    }
    req.send()
        .await
        .map_err(|e| PlatformError::Network(e.to_string()))?
        .error_for_status()
        .map_err(|e| PlatformError::Network(e.to_string()))?;
    Ok(())
}
