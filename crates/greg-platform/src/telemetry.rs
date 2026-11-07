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
    let payload = anonymize_payload(payload);
    let mut req = client.post(&url).json(&payload);
    if !secrets.user.is_empty() {
        req = req.basic_auth(&secrets.user, Some(&secrets.pass));
    }
    if !secrets.tenant.is_empty() {
        req = req.header("X-Scope-OrgID", &secrets.tenant);
    }
    req.send()
        .await
        .map_err(|_| PlatformError::Network("Telemetry request failed.".into()))?
        .error_for_status()
        .map_err(|_| PlatformError::Network("Telemetry endpoint returned an error.".into()))?;
    Ok(())
}

/// Retains only coarse, non-identifying telemetry dimensions. Free-form log
/// values and labels can contain mod titles, account names, IDs, or paths, so
/// values are reduced to a small status vocabulary before any network send.
fn anonymize_payload(payload: &LokiPushRequest) -> LokiPushRequest {
    let allowed = [
        "app",
        "event",
        "os",
        "version",
        "status",
        "result",
        "operation",
    ];
    let streams = payload
        .streams
        .iter()
        .map(|stream| {
            let labels = stream
                .stream
                .iter()
                .filter(|(key, _)| allowed.contains(&key.to_ascii_lowercase().as_str()))
                .map(|(key, value)| (key.clone(), greg_core::report::redact(value)))
                .collect();
            let values = stream
                .values
                .iter()
                .map(|(timestamp, _)| (timestamp.clone(), "diagnostic event".to_string()))
                .collect();
            greg_core::models::LokiStream {
                stream: labels,
                values,
            }
        })
        .collect();
    LokiPushRequest { streams }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn telemetry_drops_identifying_dimensions_and_free_text() {
        let request = LokiPushRequest {
            streams: vec![greg_core::models::LokiStream {
                stream: HashMap::from([
                    ("app".into(), "gregModmanager".into()),
                    ("steamId".into(), "76561198000000000".into()),
                    ("account".into(), "MLeeM".into()),
                    ("status".into(), "ok".into()),
                ]),
                values: vec![("1".into(), "mod title /home/user/Game".into())],
            }],
        };
        let safe = anonymize_payload(&request);
        let json = serde_json::to_string(&safe).unwrap();
        assert!(!json.contains("76561198000000000"));
        assert!(!json.contains("MLeeM"));
        assert!(!json.contains("mod title"));
        assert!(json.contains("diagnostic event"));
    }
}
