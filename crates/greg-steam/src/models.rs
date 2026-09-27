//! Steam Workshop view models (serde).

use serde::{Deserialize, Serialize};

/// Browse sort orders.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorkshopSortMode {
    /// Recently updated first.
    #[default]
    LastUpdated,
    /// Newest first.
    Newest,
    /// Top rated.
    TopRated,
    /// Trending.
    Trending,
    /// Most subscribed.
    MostSubscribed,
    /// Title A–Z.
    TitleAsc,
}

/// One item in a browse/search result page.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BrowseItem {
    #[serde(default)]
    pub published_file_id: u64,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub preview_image_url: String,
    #[serde(default)]
    pub score: f32,
    #[serde(default)]
    pub num_subscriptions: u64,
    #[serde(default)]
    pub updated: u32,
}

/// A page of browse results.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BrowseResult {
    #[serde(default)]
    pub items: Vec<BrowseItem>,
    #[serde(default)]
    pub total_results: u32,
    #[serde(default)]
    pub current_page: u32,
    #[serde(default)]
    pub has_more_pages: bool,
}

/// Full detail of one Workshop item.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ItemDetail {
    #[serde(default)]
    pub published_file_id: u64,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub owner_name: String,
    #[serde(default)]
    pub owner_steam_id: u64,
    #[serde(default)]
    pub preview_image_url: String,
    #[serde(default)]
    pub created: u32,
    #[serde(default)]
    pub updated: u32,
    #[serde(default)]
    pub score: f32,
    #[serde(default)]
    pub size_bytes: u64,
    #[serde(default)]
    pub is_subscribed: bool,
    #[serde(default)]
    pub visibility: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub dependency_hint_compact: String,
    #[serde(default)]
    pub dependency_hint_block: String,
    #[serde(default)]
    pub has_dependency_hints: bool,
}

/// One of the user's own published items (compact).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PublishedItem {
    #[serde(default)]
    pub published_file_id: u64,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub updated: u32,
}

/// Outcome of a publish/update call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishOutcome {
    /// Success flag.
    pub success: bool,
    /// Item id (usable when successful).
    pub published_file_id: u64,
    /// Human-readable message.
    pub message: String,
}

impl PublishOutcome {
    /// Successful publish.
    pub fn ok(id: u64) -> Self {
        Self {
            success: true,
            published_file_id: id,
            message: "Published.".into(),
        }
    }

    /// Failed publish.
    pub fn fail(message: impl Into<String>) -> Self {
        Self {
            success: false,
            published_file_id: 0,
            message: message.into(),
        }
    }
}

/// Outcome of importing a published item into the workspace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportOutcome {
    /// Success flag.
    pub success: bool,
    /// Project root (when successful).
    pub project_root: Option<String>,
    /// Human-readable message.
    pub message: String,
}

impl ImportOutcome {
    /// Successful import.
    pub fn ok(root: impl Into<String>) -> Self {
        Self {
            success: true,
            project_root: Some(root.into()),
            message: "Imported.".into(),
        }
    }

    /// Failed import.
    pub fn fail(message: impl Into<String>) -> Self {
        Self {
            success: false,
            project_root: None,
            message: message.into(),
        }
    }
}

/// Local download result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadResult {
    /// Success flag.
    pub success: bool,
    /// Local directory (when successful).
    pub dir: Option<String>,
    /// Title or message.
    pub message: String,
}

impl DownloadResult {
    /// Successful download.
    pub fn ok(dir: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            success: true,
            dir: Some(dir.into()),
            message: title.into(),
        }
    }

    /// Failed download.
    pub fn fail(message: impl Into<String>) -> Self {
        Self {
            success: false,
            dir: None,
            message: message.into(),
        }
    }
}
