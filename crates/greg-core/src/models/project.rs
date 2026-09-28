//! Local workspace project handles.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// One project folder in the workspace.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkshopProject {
    /// Folder name.
    pub name: String,
    /// Project root (holds `metadata.json`, `content/`).
    pub root_path: PathBuf,
    /// `<root>/content`.
    pub content_path: PathBuf,
    /// `<root>/metadata.json`.
    pub metadata_path: PathBuf,
    /// True when `content/` exists.
    pub is_valid_layout: bool,
}

/// Local-vs-published sync state (hash comparison, no network).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectSyncState {
    /// No published id — nothing to compare.
    Unpublished,
    /// Has an id but no hash recorded yet (legacy project).
    Unknown,
    /// Local state matches the last successful publish.
    Synced,
    /// Local content/metadata changed since the last publish.
    Modified,
}
