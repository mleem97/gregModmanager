//! Small shared data types (stats, crash reports, automation status).

use serde::{Deserialize, Serialize};

/// Total size of `content/` plus largest direct children.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ContentStats {
    #[serde(default)]
    pub exists: bool,
    #[serde(default)]
    pub total_bytes: i64,
    #[serde(default)]
    pub file_count: i64,
}

/// Crash report payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CrashReport {
    #[serde(default)]
    pub process_id: u32,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub stack_trace: String,
    #[serde(default)]
    pub timestamp: String,
}

/// Debug session-log event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DebugLogPayload {
    #[serde(default)]
    pub hypothesis_id: String,
    #[serde(default)]
    pub location: String,
    #[serde(default)]
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    #[serde(default)]
    pub timestamp: String,
}

/// Automation status written beside a project (`.ralph/tasks/status.json`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RalphTaskStatus {
    #[serde(default)]
    pub last_command: String,
    #[serde(default)]
    pub ok: bool,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub timestamp: String,
}
