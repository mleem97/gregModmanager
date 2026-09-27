//! Telemetry payloads (Loki-compatible, opt-in only).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// One Loki stream: labels plus `[timestamp-ns, line]` values.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct LokiStream {
    #[serde(default)]
    pub stream: HashMap<String, String>,
    #[serde(default)]
    pub values: Vec<(String, String)>,
}

/// Loki push request body.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct LokiPushRequest {
    #[serde(default)]
    pub streams: Vec<LokiStream>,
}

/// Startup event tracked on launch.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TelemetryStartupEvent {
    #[serde(default)]
    pub steam_active: bool,
    #[serde(default)]
    pub culture: String,
    #[serde(default)]
    pub os_description: String,
    #[serde(default)]
    pub app_version: String,
    #[serde(default)]
    pub runtime_version: String,
}

/// Collection sync event.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SyncCollectionEvent {
    #[serde(default)]
    pub success: bool,
    #[serde(default)]
    pub item_count: i64,
    #[serde(default)]
    pub duration_ms: i64,
}
