//! Universal Report Assistant: Report data model, categories, validation,
//! Markdown generation and secret redaction.
//!
//! Pure domain logic, no UI dependencies, no network. Consumed by the Slint
//! report flow (`greg-app`) later; kept free of `slint` so it stays unit-testable
//! without the UI stack.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{CoreError, Result};

/// Version of the shared, versioned report schema.
///
/// Bumped only on breaking changes; additive fields stay on the same version
/// (consumers must ignore unknown fields).
pub const REPORT_SCHEMA_VERSION: u32 = 1;

/// Source product that produced a report (shared schema across products).
pub const SOURCE_MODMANAGER: &str = "modmanager";
pub const SOURCE_GREG_LINT: &str = "greglint";
pub const SOURCE_VSIX: &str = "vsix";
pub const SOURCE_DATACENTERMODS: &str = "datacentermods";

/// Transmission modes a report may be delivered with.
pub const MODE_USER_DIRECT: &str = "user_direct";
pub const MODE_OPERATOR_SERVICE: &str = "operator_service";
pub const MODE_LOCAL_EXPORT: &str = "local_export";

/// Maximum length of the report title (mirrors the Workshop title limit).
pub const MAX_REPORT_TITLE_LENGTH: usize = 128;
/// Maximum length of a single log / diagnostic text block that may enter a
/// generated report body (defense against unbounded inputs).
pub const MAX_REPORT_TEXT_CHARS: usize = 4000;

/// Known report categories.
pub const CATEGORY_BUG: &str = "bug";
pub const CATEGORY_CRASH: &str = "crash";
pub const CATEGORY_FEATURE: &str = "feature";
pub const CATEGORY_IMPROVEMENT: &str = "improvement";
pub const CATEGORY_PERFORMANCE: &str = "performance";
pub const CATEGORY_COMPATIBILITY: &str = "compatibility";
pub const CATEGORY_UI_UX: &str = "ui_ux";
pub const CATEGORY_DOCUMENTATION: &str = "documentation";
pub const CATEGORY_INSTALLATION: &str = "installation";
pub const CATEGORY_CONFIGURATION: &str = "configuration";
pub const CATEGORY_SECURITY: &str = "security";
pub const CATEGORY_OTHER: &str = "other";

/// Report categories accepted by the assistant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReportCategory {
    Bug,
    Crash,
    Feature,
    Improvement,
    Performance,
    Compatibility,
    UiUx,
    Documentation,
    Installation,
    Configuration,
    Security,
    Other,
}

impl ReportCategory {
    /// Canonical category identifier.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Bug => CATEGORY_BUG,
            Self::Crash => CATEGORY_CRASH,
            Self::Feature => CATEGORY_FEATURE,
            Self::Improvement => CATEGORY_IMPROVEMENT,
            Self::Performance => CATEGORY_PERFORMANCE,
            Self::Compatibility => CATEGORY_COMPATIBILITY,
            Self::UiUx => CATEGORY_UI_UX,
            Self::Documentation => CATEGORY_DOCUMENTATION,
            Self::Installation => CATEGORY_INSTALLATION,
            Self::Configuration => CATEGORY_CONFIGURATION,
            Self::Security => CATEGORY_SECURITY,
            Self::Other => CATEGORY_OTHER,
        }
    }

    /// Parses a category identifier (tolerant of surrounding whitespace).
    pub fn parse(value: &str) -> Option<Self> {
        let normalized = value.trim().to_ascii_lowercase();
        Some(match normalized.as_str() {
            CATEGORY_BUG => Self::Bug,
            CATEGORY_CRASH => Self::Crash,
            CATEGORY_FEATURE => Self::Feature,
            CATEGORY_IMPROVEMENT => Self::Improvement,
            CATEGORY_PERFORMANCE => Self::Performance,
            CATEGORY_COMPATIBILITY => Self::Compatibility,
            CATEGORY_UI_UX => Self::UiUx,
            CATEGORY_DOCUMENTATION => Self::Documentation,
            CATEGORY_INSTALLATION => Self::Installation,
            CATEGORY_CONFIGURATION => Self::Configuration,
            CATEGORY_SECURITY => Self::Security,
            CATEGORY_OTHER => Self::Other,
            _ => return None,
        })
    }

    /// All known category identifiers.
    pub fn all() -> Vec<Self> {
        vec![
            Self::Bug,
            Self::Crash,
            Self::Feature,
            Self::Improvement,
            Self::Performance,
            Self::Compatibility,
            Self::UiUx,
            Self::Documentation,
            Self::Installation,
            Self::Configuration,
            Self::Security,
            Self::Other,
        ]
    }
}

/// Source product that produced a report (shared schema, cross-product flow).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReportSource {
    Modmanager,
    GregLint,
    Vsix,
    Datacentermods,
    Other,
}

impl ReportSource {
    /// Canonical source identifier.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Modmanager => SOURCE_MODMANAGER,
            Self::GregLint => SOURCE_GREG_LINT,
            Self::Vsix => SOURCE_VSIX,
            Self::Datacentermods => SOURCE_DATACENTERMODS,
            Self::Other => "other",
        }
    }

    /// Parses a source identifier (tolerant of surrounding whitespace).
    pub fn parse(value: &str) -> Option<Self> {
        let normalized = value.trim().to_ascii_lowercase();
        Some(match normalized.as_str() {
            SOURCE_MODMANAGER => Self::Modmanager,
            SOURCE_GREG_LINT => Self::GregLint,
            SOURCE_VSIX => Self::Vsix,
            SOURCE_DATACENTERMODS => Self::Datacentermods,
            _ => Self::Other,
        })
    }

    /// All known source identifiers.
    pub fn all() -> Vec<Self> {
        vec![
            Self::Modmanager,
            Self::GregLint,
            Self::Vsix,
            Self::Datacentermods,
            Self::Other,
        ]
    }
}

/// How a report may be delivered to the target tracker / operator service.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReportTransmissionMode {
    UserDirect,
    OperatorService,
    LocalExport,
}

impl ReportTransmissionMode {
    /// Canonical mode identifier.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UserDirect => MODE_USER_DIRECT,
            Self::OperatorService => MODE_OPERATOR_SERVICE,
            Self::LocalExport => MODE_LOCAL_EXPORT,
        }
    }

    /// Parses a transmission mode identifier (tolerant of case, separators, whitespace).
    pub fn parse(value: &str) -> Option<Self> {
        let normalized = value.trim().to_ascii_lowercase().replace('-', "_");
        Some(match normalized.as_str() {
            MODE_USER_DIRECT => Self::UserDirect,
            MODE_OPERATOR_SERVICE => Self::OperatorService,
            MODE_LOCAL_EXPORT => Self::LocalExport,
            _ => return None,
        })
    }

    /// All known transmission modes.
    pub fn all() -> Vec<Self> {
        vec![Self::UserDirect, Self::OperatorService, Self::LocalExport]
    }
}

/// Lifecycle state of a report submission attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReportSubmissionState {
    Draft,
    InProgress,
    Submitted,
    Failed,
    Cancelled,
}

impl ReportSubmissionState {
    /// Canonical state identifier.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::InProgress => "in_progress",
            Self::Submitted => "submitted",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    /// Parses a submission state identifier (tolerant of case/whitespace).
    pub fn parse(value: &str) -> Option<Self> {
        let normalized = value.trim().to_ascii_lowercase();
        Some(match normalized.as_str() {
            "draft" => Self::Draft,
            "in_progress" | "inprogress" | "in-progress" => Self::InProgress,
            "submitted" => Self::Submitted,
            "failed" => Self::Failed,
            "cancelled" | "canceled" => Self::Cancelled,
            _ => return None,
        })
    }

    /// All known submission states.
    pub fn all() -> Vec<Self> {
        vec![
            Self::Draft,
            Self::InProgress,
            Self::Submitted,
            Self::Failed,
            Self::Cancelled,
        ]
    }

    /// True while the report may still be delivered.
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Submitted | Self::Failed | Self::Cancelled)
    }
}

/// Severity of a single validation finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReportIssueSeverity {
    Ok,
    Warning,
    Error,
}

/// One validation finding for a report draft.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportValidationIssue {
    /// Affected field / section (e.g. `title`, `description`).
    pub field: String,
    /// Human-readable finding.
    pub message: String,
    /// Severity of the finding.
    pub severity: ReportIssueSeverity,
}

impl ReportValidationIssue {
    /// Creates an error finding.
    pub fn error(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
            severity: ReportIssueSeverity::Error,
        }
    }

    /// Creates a warning finding.
    pub fn warning(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
            severity: ReportIssueSeverity::Warning,
        }
    }

    /// True for blocking findings.
    pub fn is_error(&self) -> bool {
        self.severity == ReportIssueSeverity::Error
    }
}

/// Operating / runtime context attached to a report draft.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportEnvironment {
    /// Operating system identifier (e.g. `Windows`, `Linux`, `macOS`).
    pub os: String,
    /// Application version the user ran.
    pub app_version: String,
    /// Game / title the user was running (when known).
    pub game: String,
    /// Mod loader state (MelonLoader / BepInEx, when known).
    pub loader: String,
}

impl ReportEnvironment {
    /// Creates an environment snapshot from the collected context.
    pub fn new(
        os: impl Into<String>,
        app_version: impl Into<String>,
        game: impl Into<String>,
        loader: impl Into<String>,
    ) -> Self {
        Self {
            os: os.into(),
            app_version: app_version.into(),
            game: game.into(),
            loader: loader.into(),
        }
    }

    /// Human-readable environment summary (empty parts are skipped).
    pub fn describe(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        let os = self.os.trim();
        if !os.is_empty() {
            parts.push(format!("OS: {os}"));
        }
        let version = self.app_version.trim();
        if !version.is_empty() {
            parts.push(format!("App version: {version}"));
        }
        let game = self.game.trim();
        if !game.is_empty() {
            parts.push(format!("Game: {game}"));
        }
        let loader = self.loader.trim();
        if !loader.is_empty() {
            parts.push(format!("Loader: {loader}"));
        }
        parts.join("; ")
    }
}

/// Report payload collected by the assistant before submission.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportData {
    /// Report title.
    #[serde(default)]
    pub title: String,
    /// Report category identifier (see [`ReportCategory`]).
    #[serde(default)]
    pub category: String,
    /// Free-form user description / problem statement.
    #[serde(default)]
    pub description: String,
    /// Reproduction steps.
    #[serde(default)]
    pub steps_to_reproduce: String,
    /// Expected behavior.
    #[serde(default)]
    pub expected_behavior: String,
    /// Actual behavior.
    #[serde(default)]
    pub actual_behavior: String,
    /// Log / diagnostic text supplied by the user or the collector.
    #[serde(default)]
    pub logs: String,
    /// Environment summary (OS, app version, …).
    #[serde(default)]
    pub environment: String,
    /// Optional project identifier.
    #[serde(default)]
    pub project_id: Option<String>,
    /// Optional mod identifier.
    #[serde(default)]
    pub mod_id: Option<String>,
    /// Selected attachment file names.
    #[serde(default)]
    pub attachments: Vec<String>,
    /// Unique report id within the shared schema.
    #[serde(default)]
    pub id: String,
    /// Version of the shared report schema this payload follows.
    #[serde(default)]
    pub schema_version: u32,
    /// Creation timestamp (unix seconds, UTC).
    #[serde(default)]
    pub created_at: String,
    /// Source product that produced the report.
    #[serde(default)]
    pub source: String,
    /// Transmission mode planned for the submission.
    #[serde(default)]
    pub transmission_mode: String,
    /// Current submission lifecycle state.
    #[serde(default)]
    pub submission_state: String,
    /// External issue reference (issue number or local export path).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_issue_ref: Option<String>,
    /// Optional free-text context appended to the generated issue body.
    #[serde(default)]
    pub additional_context: String,
}

impl ReportData {
    /// Creates a new report draft stamped with the shared schema version.
    pub fn new(source: &str, transmission_mode: &str) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        Self {
            id: format!("rpt-{now}"),
            schema_version: REPORT_SCHEMA_VERSION,
            created_at: now.to_string(),
            source: source.trim().to_ascii_lowercase(),
            transmission_mode: transmission_mode.trim().to_ascii_lowercase(),
            submission_state: ReportSubmissionState::Draft.as_str().to_string(),
            external_issue_ref: None,
            ..Self::default()
        }
    }

    /// True when the report carries a usable title.
    pub fn has_title(&self) -> bool {
        !self.title.trim().is_empty()
    }

    /// True when the report carries a usable description.
    pub fn has_description(&self) -> bool {
        !self.description.trim().is_empty()
    }

    /// Returns the effective category, falling back to [`ReportCategory::Other`].
    pub fn effective_category(&self) -> ReportCategory {
        ReportCategory::parse(&self.category).unwrap_or(ReportCategory::Other)
    }

    /// Returns the effective transmission mode, falling back to local export.
    pub fn effective_transmission_mode(&self) -> ReportTransmissionMode {
        ReportTransmissionMode::parse(&self.transmission_mode)
            .unwrap_or(ReportTransmissionMode::LocalExport)
    }

    /// Returns the effective submission state, falling back to draft.
    pub fn effective_submission_state(&self) -> ReportSubmissionState {
        ReportSubmissionState::parse(&self.submission_state).unwrap_or(ReportSubmissionState::Draft)
    }

    /// Marks the submission lifecycle state.
    pub fn set_submission_state(&mut self, state: ReportSubmissionState) {
        self.submission_state = state.as_str().to_string();
    }

    /// Marks the transmission mode.
    pub fn set_transmission_mode(&mut self, mode: ReportTransmissionMode) {
        self.transmission_mode = mode.as_str().to_string();
    }
}

/// Validates a report draft and returns blocking/non-blocking findings.
///
/// Validation is intentionally explicit and never invents missing information;
/// it only reports what is present or absent.
pub fn validate(data: &ReportData) -> Vec<ReportValidationIssue> {
    let mut issues = Vec::new();

    if !data.has_title() {
        issues.push(ReportValidationIssue::error("title", "Title is required."));
    } else if data.title.trim().chars().count() > MAX_REPORT_TITLE_LENGTH {
        issues.push(ReportValidationIssue::error(
            "title",
            format!(
                "Title exceeds {MAX_REPORT_TITLE_LENGTH} characters ({}).",
                data.title.trim().chars().count()
            ),
        ));
    }

    let category = data.effective_category();
    let description_satisfied = data.has_description() || !data.logs.trim().is_empty();
    if category == ReportCategory::Crash {
        // Crash reports require either a description or a stack trace / log.
        if !description_satisfied {
            issues.push(ReportValidationIssue::error(
                "description",
                "Crash reports require either a description or a stack trace / log.",
            ));
        }
    } else if !matches!(
        category,
        ReportCategory::Improvement | ReportCategory::Documentation | ReportCategory::Other
    ) && !description_satisfied
    {
        issues.push(ReportValidationIssue::error(
            "description",
            "A description of the problem or request is required.",
        ));
    }

    if category == ReportCategory::Bug && data.steps_to_reproduce.trim().is_empty() {
        issues.push(ReportValidationIssue::warning(
            "stepsToReproduce",
            "Reproduction steps are missing — they help maintainers reproduce the issue.",
        ));
    }

    if category == ReportCategory::Performance && data.logs.trim().is_empty() {
        issues.push(ReportValidationIssue::warning(
            "logs",
            "Measurements or logs are missing — performance issues benefit from concrete data.",
        ));
    }

    if category == ReportCategory::Security && data.environment.trim().is_empty() {
        issues.push(ReportValidationIssue::warning(
            "environment",
            "Security reports benefit from version and environment context.",
        ));
    }

    issues
}

/// Redacts known secret material from free text before it enters a report body.
///
/// Conservative marker-based redaction: over-redaction is preferable to leaking
/// credentials. Detection is substring based (no regex dependency added).
pub fn redact(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let markers: &[&str] = &[
        "-----BEGIN",
        "PRIVATE KEY-----",
        "AKIA",
        "ghp_",
        "github_pat_",
        "sk_live_",
        "sk-ant-",
        "xoxb-",
        "xoxp-",
        "api_key=",
        "apikey=",
        "api-key=",
        "client_secret=",
        "clientSecret=",
        "access_token=",
        "accessToken=",
        "refresh_token=",
        "refreshToken=",
        "password=",
        "passwd=",
        "secret=",
        "token=",
        "Bearer ",
    ];
    let mut i = 0usize;
    while i < bytes.len() {
        let matched = markers
            .iter()
            .find(|m| text[i..].starts_with(*m) && i + m.len() <= bytes.len())
            .copied();
        match matched {
            Some(marker) => {
                out.push_str("[REDACTED]");
                i += marker.len();
                // Skip the contiguous credential-like run after the marker
                // (e.g. the token body following `ghp_`, `sk_live_`, …) so the
                // secret material itself never reaches the output.
                while i < bytes.len() && is_secret_char(bytes[i]) {
                    i += 1;
                }
            }
            None => {
                // Safety for multi-byte UTF-8: copy one byte at a time when the
                // next char is not an ASCII marker boundary.
                let ch = text[i..].chars().next().unwrap_or('\0');
                out.push(ch);
                i += ch.len_utf8();
            }
        }
    }
    out
}

/// True for bytes that can belong to credential-like runs (`A-Z`, `a-z`, `0-9`,
/// `_`, `-`, `.`, `~`).
fn is_secret_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'-' || b == b'.' || b == b'~'
}

/// Redacts and truncates free text to the maximum report text length.
pub fn sanitize(text: &str) -> String {
    let redacted = redact(text);
    if redacted.chars().count() > MAX_REPORT_TEXT_CHARS {
        let truncated: String = redacted.chars().take(MAX_REPORT_TEXT_CHARS).collect();
        format!("{truncated}\n… [truncated]")
    } else {
        redacted
    }
}

/// Builds the GitHub-ready Markdown issue body from a report.
///
/// The body is sanitized first; secrets never reach the generated issue text.
pub fn generate_markdown(data: &ReportData) -> String {
    let description = sanitize(&data.description);
    let steps = sanitize(&data.steps_to_reproduce);
    let expected = sanitize(&data.expected_behavior);
    let actual = sanitize(&data.actual_behavior);
    let logs = sanitize(&data.logs);
    let environment = sanitize(&data.environment);
    let category = data.effective_category().as_str();

    let mut md = String::new();
    md.push_str("## Summary\n\n");
    md.push_str(&description);
    md.push_str("\n\n## Report Type\n\n");
    md.push_str(category);
    md.push_str("\n\n");

    if !steps.trim().is_empty() {
        md.push_str("## Steps to Reproduce\n\n");
        md.push_str(&steps);
        md.push_str("\n\n");
    }
    if !expected.trim().is_empty() {
        md.push_str("## Expected Behavior\n\n");
        md.push_str(&expected);
        md.push_str("\n\n");
    }
    if !actual.trim().is_empty() {
        md.push_str("## Actual Behavior\n\n");
        md.push_str(&actual);
        md.push_str("\n\n");
    }
    if !logs.trim().is_empty() {
        md.push_str("## Logs and Diagnostics\n\n");
        md.push_str(&logs);
        md.push_str("\n\n");
    }
    if !environment.trim().is_empty() {
        md.push_str("## Environment\n\n");
        md.push_str(&environment);
        md.push_str("\n\n");
    }
    if !data.attachments.is_empty() {
        md.push_str("## Attachments\n\n");
        for attachment in &data.attachments {
            let name = attachment.trim();
            if name.is_empty() {
                continue;
            }
            md.push_str(&format!("- {name}\n"));
        }
        md.push('\n');
    }
    if !data.additional_context.trim().is_empty() {
        md.push_str("## Additional Context\n\n");
        md.push_str(&sanitize(&data.additional_context));
        md.push_str("\n\n");
    }
    md
}

/// Exports the generated Markdown issue body to a local file.
///
/// The written content is sanitized (secrets redacted, size capped) — raw
/// credentials must never leave the machine through an export either.
pub fn export_markdown(data: &ReportData, path: &Path) -> Result<()> {
    let md = generate_markdown(data);
    std::fs::write(path, format!("{md}\n")).map_err(|e| CoreError::Io(e.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report() -> ReportData {
        let mut data = ReportData::new(SOURCE_MODMANAGER, MODE_LOCAL_EXPORT);
        data.title = "Crash on launch".to_string();
        data.category = CATEGORY_BUG.to_string();
        data.description = "App crashes when starting the game.".to_string();
        data.steps_to_reproduce = "1. Start app\n2. Launch".to_string();
        data.expected_behavior = "App starts cleanly.".to_string();
        data.actual_behavior = "App closes immediately.".to_string();
        data.logs = String::new();
        data.environment = "Windows 11, app v1.7.5".to_string();
        data.attachments = vec!["app.log".to_string()];
        data
    }

    #[test]
    fn category_roundtrip() {
        assert_eq!(ReportCategory::parse("Bug").unwrap(), ReportCategory::Bug);
        assert_eq!(
            ReportCategory::parse("  PERFORMANCE ").unwrap(),
            ReportCategory::Performance
        );
        assert!(ReportCategory::parse("does-not-exist").is_none());
    }

    #[test]
    fn markdown_contains_required_sections_and_skips_empty_optional() {
        let md = generate_markdown(&report());
        assert!(md.starts_with("## Summary\n\n"));
        assert!(md.contains("## Report Type\n\nbug"));
        assert!(md.contains("## Steps to Reproduce"));
        assert!(md.contains("## Expected Behavior"));
        assert!(md.contains("## Actual Behavior"));
        assert!(md.contains("## Attachments"));
        assert!(md.contains("- app.log"));
        assert!(md.contains("## Environment"));
    }

    #[test]
    fn markdown_uses_redacted_content() {
        let mut r = report();
        r.description = "token=ghp_abcdefghijklmnopqrstuvwxyz leaked".to_string();
        let md = generate_markdown(&r);
        assert!(md.contains("[REDACTED]"));
        assert!(!md.contains("ghp_"));
    }

    #[test]
    fn validation_flags_missing_title_and_bug_repro_warning() {
        let mut r = report();
        r.title = "   ".to_string();
        r.steps_to_reproduce = String::new();
        let issues = validate(&r);
        assert!(issues.iter().any(|i| i.field == "title" && i.is_error()));
        assert!(issues
            .iter()
            .any(|i| i.field == "stepsToReproduce" && !i.is_error()));
    }

    #[test]
    fn crash_requires_description_or_logs() {
        let mut r = report();
        r.category = CATEGORY_CRASH.to_string();
        r.description = String::new();
        r.logs = String::new();
        assert!(validate(&r)
            .iter()
            .any(|i| i.field == "description" && i.is_error()));
        r.logs = "panic: boom".to_string();
        assert!(!validate(&r)
            .iter()
            .any(|i| i.field == "description" && i.is_error()));
    }

    #[test]
    fn redact_removes_secret_markers() {
        let out = redact("Authorization: Bearer ghp_abcdefghijklmnopqrstuvwxyz1234567890");
        assert!(!out.contains("ghp_"));
        assert!(out.contains("[REDACTED]"));
        assert!(!out.contains("abcdefghijklmnopqrstuvwxyz"));
    }

    #[test]
    fn sanitize_truncates_oversized_logs() {
        let long = "x".repeat(MAX_REPORT_TEXT_CHARS + 100);
        let sanitized = sanitize(&long);
        assert!(sanitized.chars().count() > MAX_REPORT_TEXT_CHARS);
        assert!(sanitized.ends_with("\n… [truncated]"));
    }

    #[test]
    fn schema_stamps_new_reports_with_version_and_source() {
        let data = ReportData::new(SOURCE_GREG_LINT, MODE_OPERATOR_SERVICE);
        assert_eq!(data.schema_version, REPORT_SCHEMA_VERSION);
        assert_eq!(data.source, SOURCE_GREG_LINT);
        assert_eq!(data.transmission_mode, MODE_OPERATOR_SERVICE);
        assert_eq!(
            data.effective_submission_state(),
            ReportSubmissionState::Draft
        );
        assert_eq!(
            data.effective_transmission_mode(),
            ReportTransmissionMode::OperatorService
        );
        assert_eq!(data.id, format!("rpt-{}", data.created_at));
    }

    #[test]
    fn submission_state_transitions_parse() {
        assert_eq!(
            ReportSubmissionState::parse("In_Progress").unwrap(),
            ReportSubmissionState::InProgress
        );
        assert_eq!(
            ReportSubmissionState::parse("submitted").unwrap(),
            ReportSubmissionState::Submitted
        );
        assert!(ReportSubmissionState::parse("bogus").is_none());
        assert!(!ReportSubmissionState::Draft.is_terminal());
        assert!(ReportSubmissionState::Submitted.is_terminal());
    }

    #[test]
    fn transmission_modes_parse_and_fall_back() {
        assert_eq!(
            ReportTransmissionMode::parse(" User_Direct ").unwrap(),
            ReportTransmissionMode::UserDirect
        );
        assert_eq!(
            ReportTransmissionMode::parse("operator-service").unwrap(),
            ReportTransmissionMode::OperatorService
        );
        let mut data = ReportData::new(SOURCE_MODMANAGER, MODE_LOCAL_EXPORT);
        assert_eq!(
            data.effective_transmission_mode(),
            ReportTransmissionMode::LocalExport
        );
        data.transmission_mode = "not-a-mode".to_string();
        assert_eq!(
            data.effective_transmission_mode(),
            ReportTransmissionMode::LocalExport
        );
    }

    #[test]
    fn environment_describe_skips_empty_parts() {
        let env = ReportEnvironment::new("Windows", "1.7.5", "", "");
        assert_eq!(env.describe(), "OS: Windows; App version: 1.7.5");
        assert_eq!(ReportEnvironment::default().describe(), "");
    }

    #[test]
    fn markdown_includes_additional_context_and_skips_empty_optional() {
        let mut r = report();
        assert!(!generate_markdown(&r).contains("## Additional Context"));
        r.additional_context = "User supplied extra context.".to_string();
        let md = generate_markdown(&r);
        assert!(md.contains("## Additional Context"));
        assert!(md.contains("User supplied extra context."));
    }

    #[test]
    fn markdown_omits_absent_environment_section() {
        let mut r = report();
        r.environment = String::new();
        let md = generate_markdown(&r);
        assert!(!md.contains("## Environment"));
    }

    #[test]
    fn export_markdown_writes_sanitized_body() {
        let dir = std::env::temp_dir().join(format!("greg-report-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("report.md");
        let mut r = report();
        r.additional_context = "token=ghp_abcdefghijklmnopqrstuvwxyz1234567890".to_string();
        export_markdown(&r, &path).unwrap();
        let written = std::fs::read_to_string(&path).unwrap();
        assert!(written.contains("[REDACTED]"));
        assert!(!written.contains("ghp_"));
        assert!(written.ends_with('\n'));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
