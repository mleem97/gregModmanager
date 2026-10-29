//! Workshop orchestration (`SteamWorkshopService` port).
//!
//! Owns the publish flow (rate limit → adopt → submit → hash), browsing,
//! subscriptions and workspace import. Steam access goes through
//! [`SteamBackend`] only.

use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use greg_core::docs::project_docs;
use greg_core::limits::{self, MAX_DESCRIPTION_LENGTH, MAX_TITLE_LENGTH};
use greg_core::models::WorkshopMetadata;
use greg_platform::workspace;

use crate::backend::{
    outcome_from_submit, BackendStatus, PageQuery, QueryKind, SteamBackend, UpdateRequest,
};
use crate::error::{Result, SteamError};
pub use crate::models::{
    BrowseResult, DecodedPreviewImage, ImportOutcome, ItemDetail, PublishOutcome, PublishedItem,
    WorkshopItemMetadata, WorkshopSortMode, EXAMPLE_WORKSHOP_ITEM_ID,
};
use crate::rate_limit::PublishRateLimiter;
pub use crate::web_api::{
    download_and_cache_preview_image, download_and_cache_preview_image_blocking,
    download_preview_image, download_preview_image_blocking, fetch_item_detail_web_api,
    fetch_item_detail_web_api_blocking, parse_get_details_response, parse_persona_name_response,
    DEFAULT_STEAM_WEB_API_BASE_URL, STEAM_WEB_API_KEY_ENV,
};

/// Marker placed in workshop content for install routing.
#[derive(serde::Serialize, serde::Deserialize)]
struct ModTypeMarker {
    #[serde(rename = "modType")]
    mod_type: String,
}

/// Central Workshop facade.
pub struct WorkshopService {
    backend: Arc<dyn SteamBackend>,
    limiter: PublishRateLimiter,
    lang: String,
    web_api_key: RwLock<Option<String>>,
    web_api_base_url: RwLock<String>,
}

impl WorkshopService {
    /// Creates the service over a connected (or unavailable) backend.
    pub fn new(backend: Arc<dyn SteamBackend>, lang: impl Into<String>) -> Self {
        Self {
            backend,
            limiter: PublishRateLimiter::shared(),
            lang: lang.into(),
            web_api_key: RwLock::new(None),
            web_api_base_url: RwLock::new(DEFAULT_STEAM_WEB_API_BASE_URL.to_string()),
        }
    }

    /// Builder-style configuration for an optional Steam Web API key (Tier 2 fallback).
    pub fn with_web_api_key(self, key: impl Into<String>) -> Self {
        self.set_web_api_key(Some(key.into()));
        self
    }

    /// Sets or clears the optional Steam Web API key used for Tier 2 fallback when
    /// local Steamworks (Tier 1) is unavailable.
    pub fn set_web_api_key(&self, key: Option<String>) {
        let normalized = key.map(|k| k.trim().to_string()).filter(|k| !k.is_empty());
        if let Ok(mut guard) = self.web_api_key.write() {
            *guard = normalized;
        }
    }

    /// Returns the effective Steam Web API key (configured key or `STEAM_WEB_API_KEY` env var).
    pub fn web_api_key(&self) -> Option<String> {
        if let Ok(guard) = self.web_api_key.read() {
            if let Some(key) = guard.as_ref().filter(|k| !k.trim().is_empty()) {
                return Some(key.clone());
            }
        }
        std::env::var(STEAM_WEB_API_KEY_ENV)
            .ok()
            .map(|k| k.trim().to_string())
            .filter(|k| !k.is_empty())
    }

    /// Overrides the Steam Web API base URL (used by unit tests with a local mock server).
    pub fn set_web_api_base_url(&self, base_url: impl Into<String>) {
        let url = base_url.into();
        let trimmed = url.trim();
        if let Ok(mut guard) = self.web_api_base_url.write() {
            *guard = if trimmed.is_empty() {
                DEFAULT_STEAM_WEB_API_BASE_URL.to_string()
            } else {
                trimmed.to_string()
            };
        }
    }

    /// Current Steam Web API base URL.
    pub fn web_api_base_url(&self) -> String {
        self.web_api_base_url
            .read()
            .map(|g| g.clone())
            .unwrap_or_else(|_| DEFAULT_STEAM_WEB_API_BASE_URL.to_string())
    }

    /// Availability snapshot.
    pub fn status(&self) -> BackendStatus {
        self.backend.status()
    }

    /// Opens the store page in a browser.
    pub fn open_in_browser(&self, id: u64) {
        self.backend.open_in_browser(id);
    }

    /// Exact description sent to Steam: README-resolved + requirement notices.
    /// Single source of truth for upload AND publish hash.
    pub fn build_upload_description(&self, project_root: &Path, meta: &WorkshopMetadata) -> String {
        project_docs::build_effective_steam_description(&self.lang, project_root, meta)
    }

    /// Effective change note: manual text wins, otherwise the CHANGELOG.md
    /// section for the metadata version (or `[Unreleased]`).
    pub fn resolve_changelog(
        &self,
        project_root: &Path,
        meta: &WorkshopMetadata,
        manual: Option<&str>,
    ) -> String {
        let resolved = project_docs::resolve_steam_changelog(project_root, &meta.version, manual);
        if resolved.text.trim().is_empty() {
            return String::new();
        }
        project_docs::format_steam_change_note(&meta.version, &resolved.text)
    }

    /// Publishes a project: new item or update of `meta.published_file_id`.
    /// Records the id + publish hash in `metadata.json` on success.
    #[allow(clippy::too_many_arguments)]
    pub fn publish(
        &self,
        project_root: &Path,
        meta: &mut WorkshopMetadata,
        manual_changelog: Option<&str>,
        progress: &dyn Fn(f32),
        log: &dyn Fn(&str),
        cancel: &AtomicBool,
    ) -> PublishOutcome {
        match self.publish_inner(project_root, meta, manual_changelog, progress, log, cancel) {
            Ok(id) => PublishOutcome::ok(id),
            Err(e) => PublishOutcome::fail(e.to_string()),
        }
    }

    fn publish_inner(
        &self,
        project_root: &Path,
        meta: &mut WorkshopMetadata,
        manual_changelog: Option<&str>,
        progress: &dyn Fn(f32),
        log: &dyn Fn(&str),
        cancel: &AtomicBool,
    ) -> Result<u64> {
        if !matches!(self.backend.status(), BackendStatus::Available { .. }) {
            return Err(SteamError::NotAvailable(
                "Steam is not available (is Steam running?)".into(),
            ));
        }
        let content = project_root.join("content");
        if !content.is_dir() {
            return Err(SteamError::PublishFailed(format!(
                "content folder missing: {}",
                content.display()
            )));
        }
        let title = meta.title.trim().to_string();
        let description = self.build_upload_description(project_root, meta);
        if title.is_empty() {
            return Err(SteamError::PublishFailed("Title is required.".into()));
        }
        if title.chars().count() > MAX_TITLE_LENGTH
            || description.chars().count() > MAX_DESCRIPTION_LENGTH
        {
            return Err(SteamError::PublishFailed(
                "Title or description exceeds Steam limits.".into(),
            ));
        }
        let preview = project_root.join(&meta.preview_image_relative_path);
        if !preview.is_file() {
            return Err(SteamError::PublishFailed(
                "A preview image is required for every Workshop upload and update.".into(),
            ));
        }

        // Marker so subscribers know where to install this item.
        let marker = ModTypeMarker {
            mod_type: meta.mod_type.clone(),
        };
        if let Ok(text) = serde_json::to_string(&marker) {
            let _ = std::fs::write(content.join("greg-modmanager.meta.json"), text);
        }

        if let Err(retry_after) = self.limiter.try_acquire() {
            return Err(SteamError::PublishFailed(format!(
                "Steam publish cooldown active. Please wait {}s before trying again.",
                retry_after.as_secs().max(1)
            )));
        }

        // Adopt instead of duplicate: reuse an own item with the same title.
        if !limits::is_usable_workshop_id(meta.published_file_id) {
            match self.find_own_by_title(&title, cancel) {
                Ok(Some(id)) => {
                    meta.published_file_id = id;
                    let _ = workspace::save_metadata(project_root, meta);
                    log(&format!(
                        "Adopted existing workshop item {id} (same title) instead of creating a duplicate."
                    ));
                }
                Ok(None) => log("No own item with this title — creating a new one."),
                Err(e) => log(&format!(
                    "Own-item lookup skipped: {e} — creating a new one."
                )),
            }
        }

        let change_note = self.resolve_changelog(project_root, meta, manual_changelog);
        let screenshots = resolve_screenshots(project_root, &meta.additional_previews, log);
        let req = UpdateRequest {
            file_id: limits::is_usable_workshop_id(meta.published_file_id)
                .then_some(meta.published_file_id),
            title,
            description: description.clone(),
            content_dir: content,
            preview_file: preview,
            tags: meta
                .tags
                .iter()
                .filter(|t| !t.trim().is_empty())
                .map(|t| t.trim().to_string())
                .collect(),
            visibility: meta.visibility.clone(),
            change_note: if change_note.trim().is_empty() {
                None
            } else {
                Some(change_note)
            },
            screenshots,
        };

        // Submit, with one replication-wait retry on backend "not found" errors.
        let outcome = match self.backend.submit(&req, progress, cancel) {
            Ok(o) => o,
            Err(e) => {
                let msg = e.to_string().to_lowercase();
                if msg.contains("not found") && req.file_id.is_some() {
                    log("Backend does not know the item yet — waiting for replication...");
                    self.wait_for_replication(req.file_id.unwrap_or(0), log, cancel);
                    self.backend.submit(&req, progress, cancel)?
                } else {
                    return Err(e);
                }
            }
        };
        let outcome = outcome_from_submit(outcome);
        if !outcome.success {
            return Err(SteamError::PublishFailed(outcome.message));
        }
        // Gallery second phase (mirrors the C# UploadAdditionalPreviewsAsync):
        // failures here never fail the publish itself, they are reported.
        if !req.screenshots.is_empty() {
            match self.backend.upload_gallery(
                outcome.published_file_id,
                &req.screenshots,
                log,
                cancel,
            ) {
                Ok(attached) => {
                    if attached < req.screenshots.len() {
                        log(&format!(
                            "Gallery partially attached ({attached}/{}).",
                            req.screenshots.len()
                        ));
                    }
                }
                Err(e) => log(&format!("Gallery upload skipped: {e}")),
            }
        }
        meta.published_file_id = outcome.published_file_id;
        meta.last_published_hash =
            workspace::compute_publish_hash(project_root, meta, &description);
        workspace::save_metadata(project_root, meta)
            .map_err(|e| SteamError::PublishFailed(e.to_string()))?;
        Ok(outcome.published_file_id)
    }

    fn wait_for_replication(&self, file_id: u64, log: &dyn Fn(&str), cancel: &AtomicBool) {
        if file_id == 0 {
            return;
        }
        let deadline = Instant::now() + Duration::from_secs(120);
        while Instant::now() < deadline {
            if cancel.load(std::sync::atomic::Ordering::Relaxed) {
                return;
            }
            if self.backend.details(file_id, cancel).is_ok() {
                log(&format!(
                    "Workshop backend knows item {file_id} — submitting..."
                ));
                return;
            }
            log(&format!(
                "Waiting for Workshop backend to replicate item {file_id}..."
            ));
            std::thread::sleep(Duration::from_secs(5));
        }
    }

    fn find_own_by_title(&self, title: &str, cancel: &AtomicBool) -> Result<Option<u64>> {
        let wanted = title.trim().to_lowercase();
        // Only the first page — own items are sorted by last updated desc, so
        // a recently updated item is almost certainly on page 1. Scanning up to
        // 5 pages caused up to 5 blocking Steam round-trips (each with a
        // 45 s callback timeout) before the actual upload started.
        let result = self.backend.page(
            &PageQuery {
                kind: QueryKind::Published,
                page: 1,
                tag: None,
            },
            cancel,
        )?;
        for item in &result.items {
            if item.title.trim().to_lowercase() == wanted {
                return Ok(Some(item.published_file_id));
            }
        }
        Ok(None)
    }

    /// Browse one result page.
    pub fn browse(
        &self,
        page: u32,
        sort: WorkshopSortMode,
        tag: Option<&str>,
        cancel: &AtomicBool,
    ) -> BrowseResult {
        self.to_browse_result(
            page,
            self.backend.page(
                &PageQuery {
                    kind: QueryKind::Browse(sort),
                    page,
                    tag: tag.map(str::to_string),
                },
                cancel,
            ),
        )
    }

    /// Text search, one page.
    pub fn search(&self, text: &str, page: u32, cancel: &AtomicBool) -> BrowseResult {
        self.to_browse_result(
            page,
            self.backend.page(
                &PageQuery {
                    kind: QueryKind::Search(text.to_string()),
                    page,
                    tag: None,
                },
                cancel,
            ),
        )
    }

    /// Subscribed items, one page.
    pub fn subscribed(&self, page: u32, cancel: &AtomicBool) -> BrowseResult {
        self.to_browse_result(
            page,
            self.backend.page(
                &PageQuery {
                    kind: QueryKind::Subscribed,
                    page,
                    tag: None,
                },
                cancel,
            ),
        )
    }

    /// Favorited items, one page.
    pub fn favorited(&self, page: u32, cancel: &AtomicBool) -> BrowseResult {
        self.to_browse_result(
            page,
            self.backend.page(
                &PageQuery {
                    kind: QueryKind::Favorited,
                    page,
                    tag: None,
                },
                cancel,
            ),
        )
    }

    /// Own published items, one page.
    pub fn my_published(&self, page: u32, cancel: &AtomicBool) -> BrowseResult {
        self.to_browse_result(
            page,
            self.backend.page(
                &PageQuery {
                    kind: QueryKind::Published,
                    page,
                    tag: None,
                },
                cancel,
            ),
        )
    }

    /// Full item detail using the strict Two-Tier Fallback Principle:
    /// 1. **Tier 1 (Primary — Local Steamworks):** Queries `self.backend.details(id, cancel)`.
    ///    Requires no Web API key; Steam only needs to be running locally.
    /// 2. **Tier 2 (Secondary — Steam Web API Fallback):** If Tier 1 fails, checks for an
    ///    optional Steam Web API key (`self.web_api_key()`). If none is configured, returns
    ///    [`SteamError::WebApiKeyMissing`] with a clear message. If configured, fetches
    ///    the item metadata from the Steam Web API.
    pub fn item_details(&self, id: u64, cancel: &AtomicBool) -> Result<ItemDetail> {
        match self.backend.details(id, cancel) {
            Ok(detail) => Ok(self.enrich_tier1_detail(id, detail)),
            Err(SteamError::Cancelled) => Err(SteamError::Cancelled),
            Err(tier1_err) => {
                let Some(api_key) = self.web_api_key() else {
                    return Err(SteamError::WebApiKeyMissing(tier1_err.to_string()));
                };
                let base_url = self.web_api_base_url();
                let mut detail = fetch_item_detail_web_api_blocking(
                    &base_url, &api_key, id, &self.lang, cancel,
                )?;
                self.enrich_tier2_detail(id, &mut detail);
                Ok(detail)
            }
        }
    }

    /// Async version of [`Self::item_details`] implementing the Two-Tier Fallback Principle.
    pub async fn item_details_async(&self, id: u64, cancel: &AtomicBool) -> Result<ItemDetail> {
        match self.backend.details(id, cancel) {
            Ok(detail) => Ok(self.enrich_tier1_detail(id, detail)),
            Err(SteamError::Cancelled) => Err(SteamError::Cancelled),
            Err(tier1_err) => {
                let Some(api_key) = self.web_api_key() else {
                    return Err(SteamError::WebApiKeyMissing(tier1_err.to_string()));
                };
                let base_url = self.web_api_base_url();
                let mut detail =
                    fetch_item_detail_web_api(&base_url, &api_key, id, &self.lang, cancel).await?;
                self.enrich_tier2_detail(id, &mut detail);
                Ok(detail)
            }
        }
    }

    /// Fetches Workshop item metadata (Tier 1 -> Tier 2 fallback) and downloads/validates
    /// its `preview_url` image into `cache_dir` for Slint UI rendering.
    pub async fn fetch_item_metadata_and_preview(
        &self,
        id: u64,
        cache_dir: &Path,
        cancel: &AtomicBool,
    ) -> Result<(ItemDetail, Option<PathBuf>)> {
        let detail = self.item_details_async(id, cancel).await?;
        if detail.preview_image_url.trim().is_empty() {
            return Ok((detail, None));
        }
        let cached =
            download_and_cache_preview_image(&detail.preview_image_url, id, cache_dir, cancel)
                .await
                .ok()
                .and_then(|decoded| decoded.cached_path);
        Ok((detail, cached))
    }

    /// Blocking counterpart of [`Self::fetch_item_metadata_and_preview`].
    pub fn fetch_item_metadata_and_preview_blocking(
        &self,
        id: u64,
        cache_dir: &Path,
        cancel: &AtomicBool,
    ) -> Result<(ItemDetail, Option<PathBuf>)> {
        let detail = self.item_details(id, cancel)?;
        if detail.preview_image_url.trim().is_empty() {
            return Ok((detail, None));
        }
        let cached = download_and_cache_preview_image_blocking(
            &detail.preview_image_url,
            id,
            cache_dir,
            cancel,
        )
        .ok()
        .and_then(|decoded| decoded.cached_path);
        Ok((detail, cached))
    }

    fn enrich_tier1_detail(&self, id: u64, mut detail: ItemDetail) -> ItemDetail {
        if detail.published_file_id == 0 {
            detail.published_file_id = id;
        }
        if detail.source.trim().is_empty() {
            detail.source = "steamworks".to_string();
        }
        if detail.url.trim().is_empty() {
            detail.url = format!(
                "https://steamcommunity.com/sharedfiles/filedetails/?id={}",
                detail.published_file_id
            );
        }
        if let Some(install) = self.backend.install_info(detail.published_file_id) {
            if detail.size_bytes == 0 && install.size_on_disk > 0 {
                detail.size_bytes = install.size_on_disk;
            }
            if detail.status_state.trim().is_empty() {
                detail.status_state = if detail.is_subscribed {
                    "Subscribed · Installed".to_string()
                } else {
                    "Installed".to_string()
                };
            }
        } else if detail.status_state.trim().is_empty() {
            detail.status_state = if detail.is_subscribed {
                "Subscribed · Download pending".to_string()
            } else {
                "Available (Steamworks)".to_string()
            };
        }
        if !detail.has_dependency_hints
            && detail.dependency_hint_compact.is_empty()
            && (!detail.tags.is_empty() || !detail.description.is_empty())
        {
            let hints = crate::inference::infer(&self.lang, &detail.tags, &detail.description);
            detail.dependency_hint_compact = hints.compact_line;
            detail.dependency_hint_block = hints.bullet_block;
            detail.has_dependency_hints = hints.has_any;
        }
        detail
    }

    fn enrich_tier2_detail(&self, id: u64, detail: &mut ItemDetail) {
        if detail.published_file_id == 0 {
            detail.published_file_id = id;
        }
        if detail.source.trim().is_empty() {
            detail.source = "web_api".to_string();
        }
        if let Some(install) = self.backend.install_info(detail.published_file_id) {
            if detail.size_bytes == 0 && install.size_on_disk > 0 {
                detail.size_bytes = install.size_on_disk;
            }
            detail.status_state = "Installed · Web API".to_string();
        } else if detail.status_state.trim().is_empty() {
            detail.status_state = "Available (Web API)".to_string();
        }
    }

    /// Subscribe / unsubscribe / favorite / vote passthroughs.
    pub fn subscribe(&self, id: u64) -> Result<()> {
        self.backend.subscribe(id)
    }
    /// Unsubscribe passthrough.
    pub fn unsubscribe(&self, id: u64) -> Result<()> {
        self.backend.unsubscribe(id)
    }
    /// Favorite passthrough.
    pub fn favorite(&self, id: u64, fav: bool) -> Result<()> {
        self.backend.favorite(id, fav)
    }
    /// Vote passthrough.
    pub fn vote(&self, id: u64, up: bool) -> Result<()> {
        self.backend.vote(id, up)
    }

    /// Own published items (first page, compact).
    pub fn list_my_published(&self, cancel: &AtomicBool) -> Vec<PublishedItem> {
        self.my_published(1, cancel)
            .items
            .into_iter()
            .map(|i| PublishedItem {
                published_file_id: i.published_file_id,
                title: i.title,
                updated: i.updated,
            })
            .collect()
    }

    /// Downloads a published item into a new workspace project for editing.
    /// Reuses the existing project when the item was imported before.
    /// Downloads an item's gallery into `screenshots/` and returns the
    /// relative paths for `additional_previews` (mirrors the C#
    /// `DownloadGalleryImagesAsync`). Non-fatal by design: failures are logged
    /// and skipped. Metadata persistence stays with the caller.
    pub fn download_gallery(
        &self,
        project_root: &Path,
        file_id: u64,
        log: &dyn Fn(&str),
        cancel: &AtomicBool,
    ) -> Vec<String> {
        let urls = match self.backend.additional_preview_urls(file_id, cancel) {
            Ok(urls) => urls,
            Err(e) => {
                log(&format!("Gallery sync skipped: {e}"));
                return Vec::new();
            }
        };
        if urls.is_empty() {
            return Vec::new();
        }
        let dir = project_root.join("screenshots");
        if let Err(e) = std::fs::create_dir_all(&dir) {
            log(&format!("Gallery sync skipped: {e}"));
            return Vec::new();
        }
        let mut synced = Vec::new();
        for (index, url) in urls.iter().enumerate() {
            if cancel.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }
            let bytes = match reqwest::blocking::get(url).and_then(|r| r.bytes()) {
                Ok(b) => b.to_vec(),
                Err(_) => {
                    log(&format!("Gallery download failed: {url}"));
                    continue;
                }
            };
            let name = format!("screenshot_{index}{}", image_ext(&bytes));
            match std::fs::write(dir.join(&name), &bytes) {
                Ok(()) => synced.push(format!("screenshots/{name}")),
                Err(e) => log(&format!("Gallery save failed ({name}): {e}")),
            }
        }
        if !synced.is_empty() {
            log(&format!("Gallery synced: {} screenshot(s).", synced.len()));
        }
        synced
    }

    pub fn import_to_workspace(
        &self,
        ws: &greg_platform::workspace::Workspace,
        published_file_id: u64,
        folder_name: Option<&str>,
        log: &dyn Fn(&str),
        progress: &dyn Fn(f32),
        cancel: &AtomicBool,
    ) -> ImportOutcome {
        if !matches!(self.backend.status(), BackendStatus::Available { .. }) {
            return ImportOutcome::fail("Steam is not available (is Steam running?)");
        }
        if let Ok(Some(existing)) = ws.find_project_by_published_file_id(published_file_id) {
            log(&format!(
                "Existing project reused: {}",
                existing.root_path.display()
            ));
            return ImportOutcome::ok(existing.root_path.to_string_lossy());
        }
        let detail = match self.backend.details(published_file_id, cancel) {
            Ok(d) => d,
            Err(e) => return ImportOutcome::fail(format!("Could not load workshop item: {e}")),
        };
        log(&format!("Downloading workshop file {published_file_id}..."));
        let src_dir = match self.backend.download(published_file_id, progress, cancel) {
            Ok(d) => d,
            Err(e) => return ImportOutcome::fail(format!("Download failed: {e}")),
        };
        let base = folder_name
            .filter(|s| !s.trim().is_empty())
            .map(greg_core::util::sanitize_folder_name)
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| default_import_folder_name(&detail.title, published_file_id));
        let dest_root = available_project_root(ws.root(), &base);
        if let Err(e) = std::fs::create_dir_all(dest_root.join("content")) {
            return ImportOutcome::fail(format!("Cannot create project: {e}"));
        }
        if let Err(e) = workspace::copy_dir_recursive(&src_dir, &dest_root.join("content")) {
            let _ = std::fs::remove_dir_all(&dest_root);
            return ImportOutcome::fail(format!("Copy failed: {e}"));
        }
        let mut meta = WorkshopMetadata {
            published_file_id,
            title: if detail.title.trim().is_empty() {
                base.clone()
            } else {
                detail.title.trim().to_string()
            },
            description: detail.description.clone(),
            visibility: detail.visibility.clone(),
            tags: detail.tags.clone(),
            ..Default::default()
        };
        meta.preview_image_relative_path =
            fetch_preview_image(&detail.preview_image_url, &dest_root, log);
        meta.additional_previews =
            self.download_gallery(&dest_root, published_file_id, log, cancel);
        if let Err(e) = workspace::save_metadata(&dest_root, &meta) {
            return ImportOutcome::fail(format!("Cannot save metadata: {e}"));
        }
        log(&format!("Imported to {}", dest_root.display()));
        ImportOutcome::ok(dest_root.to_string_lossy())
    }

    /// Copies live Steam data into `target`, preserving app-local fields
    /// from `local` (flags, preview paths, dependency ids).
    pub fn apply_steam_to_metadata(
        steam: &ItemDetail,
        target: &mut WorkshopMetadata,
        local: &WorkshopMetadata,
    ) {
        target.published_file_id = steam.published_file_id;
        target.title = steam.title.clone();
        if !steam.description.trim().is_empty() {
            target.description = steam.description.clone();
        }
        target.visibility = match steam.visibility.as_str() {
            "Private" | "FriendsOnly" | "Public" => steam.visibility.clone(),
            _ => "Public".into(),
        };
        target.tags = steam
            .tags
            .iter()
            .filter(|t| !t.trim().is_empty())
            .map(|t| t.trim().to_string())
            .take(20)
            .collect();
        target.needsgreg = local.needsgreg;
        target.needs_melon_loader = local.needs_melon_loader;
        target.native_config_profile = if local.native_config_profile.trim().is_empty() {
            "decoration".into()
        } else {
            local.native_config_profile.clone()
        };
        target.mod_type = if local.mod_type.trim().is_empty() {
            "PlacableObject".into()
        } else {
            local.mod_type.clone()
        };
        target.preview_image_relative_path = local.preview_image_relative_path.clone();
        target.additional_previews = local.additional_previews.clone();
        target.workshop_dependency_ids = local.workshop_dependency_ids.clone();
    }

    fn to_browse_result(
        &self,
        page: u32,
        result: Result<crate::backend::PageResult>,
    ) -> BrowseResult {
        match result {
            Ok(r) => BrowseResult {
                current_page: page,
                has_more_pages: r.total > page * 50,
                total_results: r.total,
                items: r.items,
            },
            Err(_) => BrowseResult {
                current_page: page,
                ..Default::default()
            },
        }
    }
}

fn default_import_folder_name(title: &str, published_file_id: u64) -> String {
    let base = greg_core::util::sanitize_folder_name(title);
    let short: String = base.chars().take(48).collect();
    if short.trim().is_empty() {
        format!("ws_{published_file_id}")
    } else {
        format!("{short}_{published_file_id}")
    }
}

fn available_project_root(workspace_root: &Path, base: &str) -> PathBuf {
    let mut candidate = workspace_root.join(base);
    if !candidate.exists() {
        return candidate;
    }
    for suffix in 2..=10_000u32 {
        candidate = workspace_root.join(format!("{base}_{suffix}"));
        if !candidate.exists() {
            return candidate;
        }
    }
    candidate
}

/// Downloads the preview image with magic-byte extension sniffing.
/// Returns the relative path used (defaults to `preview.png`).
fn fetch_preview_image(url: &str, dest_root: &Path, log: &dyn Fn(&str)) -> String {
    if url.trim().is_empty() {
        return "preview.png".into();
    }
    let bytes = match reqwest::blocking::get(url).and_then(|r| r.bytes()) {
        Ok(b) => b.to_vec(),
        Err(_) => return "preview.png".into(),
    };
    let name = format!("preview{}", image_ext(&bytes));
    match std::fs::write(dest_root.join(&name), &bytes) {
        Ok(()) => {
            log(&format!("Downloaded preview image as {name}"));
            name
        }
        Err(_) => "preview.png".into(),
    }
}

/// Steam gallery limit per preview file (mirrors the C# `SteamConstants`).
/// Image extension sniffed from magic bytes (shared with preview fetch).
pub fn image_ext(bytes: &[u8]) -> &'static str {
    if bytes.len() >= 3 && bytes[0] == 0xFF && bytes[1] == 0xD8 {
        ".jpg"
    } else if bytes.len() >= 8 && bytes[0..4] == [0x89, 0x50, 0x4E, 0x47] {
        ".png"
    } else if bytes.len() >= 4 && bytes[0..3] == [0x47, 0x49, 0x46] {
        ".gif"
    } else if bytes.len() >= 4 && bytes[0..4] == [0x52, 0x49, 0x46, 0x46] {
        ".webp"
    } else {
        ".png"
    }
}

pub const MAX_PREVIEW_FILE_BYTES: u64 = 1024 * 1024;

/// Resolves gallery screenshots to absolute, attachable paths.
///
/// Skips missing files and anything over [`MAX_PREVIEW_FILE_BYTES`] with a
/// log line (Steam rejects those server-side; failing loudly per file keeps
/// one bad screenshot from blocking the gallery).
fn resolve_screenshots(
    project_root: &Path,
    additional_previews: &[String],
    log: &dyn Fn(&str),
) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for rel in additional_previews {
        let rel = rel.trim();
        if rel.is_empty() {
            continue;
        }
        let path = project_root.join(rel);
        match std::fs::metadata(&path) {
            Ok(meta) if meta.is_file() => {
                if meta.len() > MAX_PREVIEW_FILE_BYTES {
                    log(&format!(
                        "Gallery skipped (over 1 MiB): {rel} ({}).",
                        greg_core::util::format_bytes(meta.len() as i64)
                    ));
                    continue;
                }
                out.push(path);
            }
            _ => log(&format!("Gallery skipped (not found): {rel}.")),
        }
    }
    out
}

#[cfg(test)]
mod gallery_tests {
    use super::resolve_screenshots;
    use std::path::PathBuf;

    fn project_with(files: &[(&str, usize)]) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "greg-gallery-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("tmp");
        for (name, size) in files {
            std::fs::write(dir.join(name), vec![1u8; *size]).expect("file");
        }
        dir
    }

    #[test]
    fn keeps_small_files_skips_missing_and_oversize() {
        use std::cell::RefCell;
        let dir = project_with(&[("a.png", 100), ("big.png", 2 * 1024 * 1024)]);
        let logged = RefCell::new(Vec::new());
        let out = resolve_screenshots(
            &dir,
            &[
                "a.png".to_string(),
                "missing.png".to_string(),
                "big.png".to_string(),
                "  ".to_string(),
            ],
            &|line| logged.borrow_mut().push(line.to_string()),
        );
        assert_eq!(out, vec![dir.join("a.png")]);
        assert_eq!(logged.borrow().len(), 2, "missing + oversize logged");
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[cfg(test)]
mod download_gallery_tests {
    use super::WorkshopService;
    use crate::backend::{
        BackendStatus, PageQuery, PageResult, SteamBackend, SubmitOutcome, UpdateRequest,
    };
    use crate::error::{Result, SteamError};
    use crate::models::ItemDetail;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::AtomicBool;

    /// Canned gallery URLs; everything else unavailable.
    struct FakeBackend {
        gallery_urls: Vec<String>,
    }

    impl SteamBackend for FakeBackend {
        fn status(&self) -> BackendStatus {
            BackendStatus::Available {
                user_name: "Test".into(),
            }
        }
        fn page(&self, _q: &PageQuery, _c: &AtomicBool) -> Result<PageResult> {
            Err(SteamError::NotAvailable("fake".into()))
        }
        fn details(&self, _id: u64, _c: &AtomicBool) -> Result<ItemDetail> {
            Err(SteamError::NotAvailable("fake".into()))
        }
        fn subscribe(&self, _id: u64) -> Result<()> {
            Err(SteamError::NotAvailable("fake".into()))
        }
        fn unsubscribe(&self, _id: u64) -> Result<()> {
            Err(SteamError::NotAvailable("fake".into()))
        }
        fn favorite(&self, _id: u64, _fav: bool) -> Result<()> {
            Err(SteamError::NotAvailable("fake".into()))
        }
        fn vote(&self, _id: u64, _up: bool) -> Result<()> {
            Err(SteamError::NotAvailable("fake".into()))
        }
        fn own_ids(&self) -> Result<Vec<u64>> {
            Ok(Vec::new())
        }
        fn submit(
            &self,
            _req: &UpdateRequest,
            _progress: &dyn Fn(f32),
            _cancel: &AtomicBool,
        ) -> Result<SubmitOutcome> {
            Err(SteamError::NotAvailable("fake".into()))
        }
        fn upload_gallery(
            &self,
            _file_id: u64,
            _screenshots: &[PathBuf],
            _log: &dyn Fn(&str),
            _cancel: &AtomicBool,
        ) -> Result<usize> {
            Ok(0)
        }
        fn additional_preview_urls(
            &self,
            _file_id: u64,
            _cancel: &AtomicBool,
        ) -> Result<Vec<String>> {
            Ok(self.gallery_urls.clone())
        }
        fn download(
            &self,
            _id: u64,
            _progress: &dyn Fn(f32),
            _cancel: &AtomicBool,
        ) -> Result<PathBuf> {
            Err(SteamError::NotAvailable("fake".into()))
        }
        fn install_info(&self, _id: u64) -> Option<crate::backend::InstallDir> {
            None
        }
        fn open_in_browser(&self, _id: u64) {}
    }

    /// Minimal PNG payload (magic bytes only — extension sniffing).
    const PNG: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00];

    /// Serves `hits` PNG responses on localhost, returns the base URL.
    fn serve_png(hits: usize) -> String {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("localhost listener");
        let addr = listener.local_addr().expect("addr");
        std::thread::spawn(move || {
            for stream in listener.incoming().take(hits) {
                let mut stream = match stream {
                    Ok(s) => s,
                    Err(_) => break,
                };
                use std::io::{Read, Write};
                let mut buf = [0u8; 1024];
                let _ = stream.read(&mut buf);
                let head = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    PNG.len()
                );
                let _ = stream.write_all(head.as_bytes());
                let _ = stream.write_all(PNG);
            }
        });
        format!("http://{addr}")
    }

    #[test]
    fn downloads_gallery_into_screenshots() {
        let base = serve_png(2);
        let backend = FakeBackend {
            gallery_urls: vec![format!("{base}/a.png"), format!("{base}/b.png")],
        };
        let service = WorkshopService::new(std::sync::Arc::new(backend), "en");
        let dir = std::env::temp_dir().join(format!(
            "greg-gallery-dl-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("tmp");
        let cancel = AtomicBool::new(false);
        let out = service.download_gallery(&dir, 1, &|_| {}, &cancel);
        assert_eq!(
            out,
            vec![
                "screenshots/screenshot_0.png".to_string(),
                "screenshots/screenshot_1.png".to_string()
            ]
        );
        assert!(Path::new(&dir.join("screenshots/screenshot_0.png")).is_file());
        assert!(Path::new(&dir.join("screenshots/screenshot_1.png")).is_file());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn empty_gallery_downloads_nothing() {
        let backend = FakeBackend {
            gallery_urls: Vec::new(),
        };
        let service = WorkshopService::new(std::sync::Arc::new(backend), "en");
        let dir = std::env::temp_dir().join("greg-gallery-empty");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("tmp");
        let cancel = AtomicBool::new(false);
        let out = service.download_gallery(&dir, 1, &|_| {}, &cancel);
        assert!(out.is_empty());
        assert!(!dir.join("screenshots").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[cfg(test)]
mod workshop_metadata_and_preview_tests {
    use super::*;
    use crate::backend::{
        BackendStatus, InstallDir, PageQuery, PageResult, SteamBackend, SubmitOutcome,
        UnavailableBackend, UpdateRequest,
    };
    use std::path::PathBuf;
    use std::sync::atomic::AtomicBool;
    use std::sync::Arc;

    /// Valid 2x2 PNG header (8-byte magic + 13-byte IHDR chunk for 2x2 image).
    const VALID_2X2_PNG: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        0x00, 0x00, 0x00, 0x0D, // IHDR length = 13
        b'I', b'H', b'D', b'R', // "IHDR"
        0x00, 0x00, 0x00, 0x02, // width = 2
        0x00, 0x00, 0x00, 0x02, // height = 2
        0x08, 0x06, 0x00, 0x00, 0x00, // bit depth 8, RGBA
    ];

    struct Tier1AvailableBackend {
        installed: bool,
    }

    impl SteamBackend for Tier1AvailableBackend {
        fn status(&self) -> BackendStatus {
            BackendStatus::Available {
                user_name: "GregLocalUser".into(),
            }
        }
        fn page(&self, _q: &PageQuery, _c: &AtomicBool) -> Result<PageResult> {
            Err(SteamError::NotAvailable("unused".into()))
        }
        fn details(&self, id: u64, _c: &AtomicBool) -> Result<ItemDetail> {
            assert_eq!(id, EXAMPLE_WORKSHOP_ITEM_ID);
            Ok(ItemDetail {
                published_file_id: id,
                title: "Greg Data Center Rack Pack".into(),
                description: "Local Steamworks description for item 3741628037.".into(),
                tags: vec!["greg".into(), "decoration".into()],
                owner_name: "TeamGreg".into(),
                owner_steam_id: 76561198012345678,
                preview_image_url:
                    "https://steamuserimages-a.akamaihd.net/ugc/3741628037/preview.png".into(),
                created: 1_730_000_000,
                updated: 1_730_500_000,
                score: 4.8,
                size_bytes: 2_048_000,
                is_subscribed: true,
                visibility: "Public".into(),
                ..Default::default()
            })
        }
        fn subscribe(&self, _id: u64) -> Result<()> {
            Ok(())
        }
        fn unsubscribe(&self, _id: u64) -> Result<()> {
            Ok(())
        }
        fn favorite(&self, _id: u64, _fav: bool) -> Result<()> {
            Ok(())
        }
        fn vote(&self, _id: u64, _up: bool) -> Result<()> {
            Ok(())
        }
        fn own_ids(&self) -> Result<Vec<u64>> {
            Ok(vec![EXAMPLE_WORKSHOP_ITEM_ID])
        }
        fn submit(
            &self,
            _req: &UpdateRequest,
            _progress: &dyn Fn(f32),
            _cancel: &AtomicBool,
        ) -> Result<SubmitOutcome> {
            Err(SteamError::NotAvailable("unused".into()))
        }
        fn upload_gallery(
            &self,
            _file_id: u64,
            _screenshots: &[PathBuf],
            _log: &dyn Fn(&str),
            _cancel: &AtomicBool,
        ) -> Result<usize> {
            Ok(0)
        }
        fn additional_preview_urls(
            &self,
            _file_id: u64,
            _cancel: &AtomicBool,
        ) -> Result<Vec<String>> {
            Ok(Vec::new())
        }
        fn download(
            &self,
            _id: u64,
            _progress: &dyn Fn(f32),
            _cancel: &AtomicBool,
        ) -> Result<PathBuf> {
            Err(SteamError::NotAvailable("unused".into()))
        }
        fn install_info(&self, id: u64) -> Option<InstallDir> {
            if self.installed && id == EXAMPLE_WORKSHOP_ITEM_ID {
                Some(InstallDir {
                    folder: PathBuf::from("/tmp/steamapps/workshop/content/3022670/3741628037"),
                    size_on_disk: 2_048_000,
                })
            } else {
                None
            }
        }
        fn open_in_browser(&self, _id: u64) {}
    }

    #[test]
    fn tier1_steamworks_succeeds_without_web_api_key_for_3741628037() {
        let backend = Arc::new(Tier1AvailableBackend { installed: true });
        let service = WorkshopService::new(backend, "en");
        service.set_web_api_key(None);

        let cancel = AtomicBool::new(false);
        let detail = service
            .item_details(EXAMPLE_WORKSHOP_ITEM_ID, &cancel)
            .expect("Tier 1 local Steamworks should succeed without a Web API key");

        assert_eq!(detail.published_file_id, 3_741_628_037);
        assert_eq!(detail.title, "Greg Data Center Rack Pack");
        assert_eq!(
            detail.description,
            "Local Steamworks description for item 3741628037."
        );
        assert_eq!(detail.owner_name, "TeamGreg");
        assert_eq!(detail.status_state, "Subscribed · Installed");
        assert_eq!(detail.source, "steamworks");
        assert_eq!(
            detail.preview_image_url,
            "https://steamuserimages-a.akamaihd.net/ugc/3741628037/preview.png"
        );
        assert!(detail.has_dependency_hints);
    }

    #[test]
    fn tier1_failure_without_web_api_key_returns_clear_error_for_3741628037() {
        let backend = Arc::new(UnavailableBackend::new("Steam client is not running"));
        let service = WorkshopService::new(backend, "en");
        service.set_web_api_key(None);
        // Ensure env var does not interfere if set in test environment.
        std::env::remove_var(STEAM_WEB_API_KEY_ENV);

        let cancel = AtomicBool::new(false);
        let err = service
            .item_details(EXAMPLE_WORKSHOP_ITEM_ID, &cancel)
            .expect_err("Should fail when Tier 1 is unavailable and no Web API key is set");

        assert!(
            matches!(err, SteamError::WebApiKeyMissing(_)),
            "Expected SteamError::WebApiKeyMissing, got {err:?}"
        );
        let msg = err.to_string();
        assert!(
            msg.contains("Steam is not available locally")
                && msg.contains("Steam Web API key")
                && msg.contains("Settings"),
            "Unexpected error message: {msg}"
        );
    }

    #[test]
    fn parses_steam_web_api_get_details_json_for_3741628037() {
        let sample_json = r#"{
            "response": {
                "result": 1,
                "resultcount": 1,
                "publishedfiledetails": [
                    {
                        "result": 1,
                        "publishedfileid": "3741628037",
                        "creator": "76561198012345678",
                        "file_size": "4194304",
                        "preview_url": "https://steamuserimages-a.akamaihd.net/ugc/3741628037/preview_3741628037.png",
                        "title": "High-Density Blade Server 3741628037",
                        "file_description": "Adds a modular blade server rack to Data Center. Requires gregCoreModFramework.",
                        "time_created": 1731000000,
                        "time_updated": 1731500000,
                        "visibility": 0,
                        "banned": 0,
                        "vote_data": {
                            "score": 0.96,
                            "votes_up": 96,
                            "votes_down": 4
                        },
                        "tags": [
                            { "tag": "greg", "display_name": "greg" },
                            { "tag": "server", "display_name": "Server" }
                        ]
                    }
                ]
            }
        }"#;

        let mut detail = parse_get_details_response(sample_json, "en")
            .expect("valid Web API JSON for 3741628037");
        assert_eq!(detail.published_file_id, 3_741_628_037);
        assert_eq!(detail.title, "High-Density Blade Server 3741628037");
        assert!(detail
            .description
            .contains("Adds a modular blade server rack"));
        assert_eq!(detail.owner_steam_id, 76561198012345678);
        assert_eq!(detail.owner_name, "76561198012345678");
        assert_eq!(detail.status_state, "Available (Web API)");
        assert_eq!(detail.source, "web_api");
        assert_eq!(
            detail.preview_image_url,
            "https://steamuserimages-a.akamaihd.net/ugc/3741628037/preview_3741628037.png"
        );
        assert_eq!(detail.size_bytes, 4_194_304);
        assert_eq!(detail.visibility, "Public");
        assert!(detail.has_dependency_hints);

        let player_json = r#"{
            "response": {
                "players": [
                    {
                        "steamid": "76561198012345678",
                        "personaname": "MarvinGregDev"
                    }
                ]
            }
        }"#;
        let persona =
            parse_persona_name_response(player_json, detail.owner_steam_id).expect("persona");
        detail.owner_name = persona;
        assert_eq!(detail.owner_name, "MarvinGregDev");
    }

    #[test]
    fn tier2_web_api_fallback_and_preview_download_for_3741628037() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind mock server");
        let addr = listener.local_addr().expect("local addr");
        let base_url = format!("http://{addr}");
        let preview_url = format!("{base_url}/ugc/3741628037/preview.png");
        let preview_url_for_server = preview_url.clone();

        // Serve 3 requests: GetDetails, GetPlayerSummaries, and the preview PNG.
        std::thread::spawn(move || {
            for stream in listener.incoming().take(3) {
                let Ok(mut stream) = stream else { break };
                use std::io::{Read, Write};
                let mut req_buf = [0u8; 2048];
                let n = stream.read(&mut req_buf).unwrap_or(0);
                let req_line = String::from_utf8_lossy(&req_buf[..n]);
                if req_line.contains("GetDetails") {
                    let body = format!(
                        r#"{{
                            "response": {{
                                "publishedfiledetails": [{{
                                    "result": 1,
                                    "publishedfileid": "3741628037",
                                    "creator": "76561198012345678",
                                    "title": "Workshop Item 3741628037 via Web API",
                                    "file_description": "Fallback metadata fetched via Steam Web API.",
                                    "preview_url": "{preview_url_for_server}",
                                    "file_size": "512000",
                                    "time_created": 1730000000,
                                    "time_updated": 1730600000,
                                    "visibility": 0,
                                    "tags": [{{"tag": "decoration"}}]
                                }}]
                            }}
                        }}"#
                    );
                    let resp = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = stream.write_all(resp.as_bytes());
                } else if req_line.contains("GetPlayerSummaries") {
                    let body = r#"{"response":{"players":[{"steamid":"76561198012345678","personaname":"GregWorkshopAuthor"}]}}"#;
                    let resp = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = stream.write_all(resp.as_bytes());
                } else {
                    let head = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: image/png\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        VALID_2X2_PNG.len()
                    );
                    let _ = stream.write_all(head.as_bytes());
                    let _ = stream.write_all(VALID_2X2_PNG);
                }
            }
        });

        let backend = Arc::new(UnavailableBackend::new("Steam client offline"));
        let service =
            WorkshopService::new(backend, "en").with_web_api_key("TEST_STEAM_WEB_API_KEY_123");
        service.set_web_api_base_url(&base_url);

        let cache_dir = std::env::temp_dir().join(format!(
            "greg-preview-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let cancel = AtomicBool::new(false);

        let (detail, cached_path) = service
            .fetch_item_metadata_and_preview_blocking(EXAMPLE_WORKSHOP_ITEM_ID, &cache_dir, &cancel)
            .expect("Tier 2 fallback + preview download should succeed");

        assert_eq!(detail.published_file_id, 3_741_628_037);
        assert_eq!(detail.title, "Workshop Item 3741628037 via Web API");
        assert_eq!(
            detail.description,
            "Fallback metadata fetched via Steam Web API."
        );
        assert_eq!(detail.owner_name, "GregWorkshopAuthor");
        assert_eq!(detail.status_state, "Available (Web API)");
        assert_eq!(detail.source, "web_api");
        assert_eq!(detail.preview_image_url, preview_url);

        let path = cached_path.expect("preview image should be cached on disk");
        assert!(path.is_file(), "cached preview file must exist");

        let decoded = crate::web_api::decode_and_cache_preview_bytes(
            VALID_2X2_PNG.to_vec(),
            &detail.preview_image_url,
            EXAMPLE_WORKSHOP_ITEM_ID,
            None,
        )
        .expect("decode 2x2 PNG");
        assert_eq!(decoded.width, 2);
        assert_eq!(decoded.height, 2);
        assert_eq!(decoded.rgba8.len(), 2 * 2 * 4);
        assert_eq!(decoded.extension, ".png");

        let _ = std::fs::remove_dir_all(&cache_dir);
    }
}
