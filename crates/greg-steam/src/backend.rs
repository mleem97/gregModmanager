//! Steam backend abstraction.
//!
//! All Steamworks calls go through [`SteamBackend`] so UI, CLI and tests run
//! without a Steam client ([`UnavailableBackend`]) or against a fake.

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

use crate::error::{Result, SteamError};
use crate::models::{BrowseItem, ItemDetail, PublishOutcome, WorkshopSortMode};

/// Availability snapshot.
#[derive(Debug, Clone)]
pub enum BackendStatus {
    /// Connected; carries the display name.
    Available { user_name: String },
    /// No client (Steam not running, init failed, ...).
    Unavailable { hint: String },
}

/// One update/create submission.
#[derive(Debug, Clone)]
pub struct UpdateRequest {
    /// Existing item, or `None` to create.
    pub file_id: Option<u64>,
    /// Workshop title.
    pub title: String,
    /// BBCode description.
    pub description: String,
    /// Content folder uploaded as-is.
    pub content_dir: PathBuf,
    /// Preview image file.
    pub preview_file: PathBuf,
    /// Tags.
    pub tags: Vec<String>,
    /// `Public` / `FriendsOnly` / `Private`.
    pub visibility: String,
    /// Steam change note.
    pub change_note: Option<String>,
    /// Gallery screenshots (absolute paths, validated ≤ 1 MiB each).
    /// Uploaded in a second update after the main submit; Steam appends
    /// (no delete/replace API), so re-uploads add, never swap.
    pub screenshots: Vec<PathBuf>,
}

/// Outcome of [`SteamBackend::submit`].
#[derive(Debug, Clone)]
pub struct SubmitOutcome {
    /// Item id to keep.
    pub file_id: u64,
    /// Legal agreement must be accepted first.
    pub needs_agreement: bool,
}

/// A browse/search/list query.
#[derive(Debug, Clone)]
pub struct PageQuery {
    /// What to fetch.
    pub kind: QueryKind,
    /// 1-based page.
    pub page: u32,
    /// Optional tag filter.
    pub tag: Option<String>,
}

/// Query variants.
#[derive(Debug, Clone)]
pub enum QueryKind {
    /// General browse with sort order.
    Browse(WorkshopSortMode),
    /// Text search.
    Search(String),
    /// Subscribed items.
    Subscribed,
    /// Favorited items.
    Favorited,
    /// Own published items.
    Published,
    /// Single item (page ignored).
    Item(u64),
}

/// Page result with total count.
#[derive(Debug, Clone)]
pub struct PageResult {
    /// Mapped items.
    pub items: Vec<BrowseItem>,
    /// Total matches on the server.
    pub total: u32,
}

/// Local install info of a downloaded item.
#[derive(Debug, Clone)]
pub struct InstallDir {
    /// Steam-managed folder.
    pub folder: PathBuf,
    /// Size on disk.
    pub size_on_disk: u64,
}

/// Synchronous Steam facade. Long calls pump callbacks, report progress,
/// honor `cancel`, and time out — mirroring the C# service behavior.
pub trait SteamBackend: Send + Sync {
    /// Availability snapshot.
    fn status(&self) -> BackendStatus;
    /// One result page.
    fn page(&self, query: &PageQuery, cancel: &AtomicBool) -> Result<PageResult>;
    /// Full detail for one item (long description included).
    fn details(&self, id: u64, cancel: &AtomicBool) -> Result<ItemDetail>;
    /// Subscribe to an item.
    fn subscribe(&self, id: u64) -> Result<()>;
    /// Unsubscribe from an item.
    fn unsubscribe(&self, id: u64) -> Result<()>;
    /// (Un)mark a favorite.
    fn favorite(&self, id: u64, fav: bool) -> Result<()>;
    /// Vote up/down.
    fn vote(&self, id: u64, up: bool) -> Result<()>;
    /// All own published item ids.
    fn own_ids(&self) -> Result<Vec<u64>>;
    /// Create or update an item.
    fn submit(
        &self,
        req: &UpdateRequest,
        progress: &dyn Fn(f32),
        cancel: &AtomicBool,
    ) -> Result<SubmitOutcome>;
    /// Attach gallery screenshots to an existing item (second update flow).
    /// Returns the number of attached files.
    fn upload_gallery(
        &self,
        file_id: u64,
        screenshots: &[PathBuf],
        log: &dyn Fn(&str),
        cancel: &AtomicBool,
    ) -> Result<usize>;
    /// Gallery image URLs of an item (additional previews, images only).
    fn additional_preview_urls(&self, file_id: u64, cancel: &AtomicBool) -> Result<Vec<String>>;
    /// Download an item into the Steam-managed folder.
    fn download(&self, id: u64, progress: &dyn Fn(f32), cancel: &AtomicBool) -> Result<PathBuf>;
    /// Install info, if downloaded.
    fn install_info(&self, id: u64) -> Option<InstallDir>;
    /// Open the store page in a browser.
    fn open_in_browser(&self, id: u64);
}

/// Backend used when Steam is unavailable: every call fails with a clear hint.
#[derive(Debug, Clone)]
pub struct UnavailableBackend {
    hint: String,
}

impl UnavailableBackend {
    /// Creates the stub with a hint.
    pub fn new(hint: impl Into<String>) -> Self {
        Self { hint: hint.into() }
    }

    fn err<T>(&self) -> Result<T> {
        Err(SteamError::NotAvailable(self.hint.clone()))
    }
}

impl SteamBackend for UnavailableBackend {
    fn status(&self) -> BackendStatus {
        BackendStatus::Unavailable {
            hint: self.hint.clone(),
        }
    }
    fn page(&self, _q: &PageQuery, _c: &AtomicBool) -> Result<PageResult> {
        self.err()
    }
    fn details(&self, _id: u64, _c: &AtomicBool) -> Result<ItemDetail> {
        self.err()
    }
    fn subscribe(&self, _id: u64) -> Result<()> {
        self.err()
    }
    fn unsubscribe(&self, _id: u64) -> Result<()> {
        self.err()
    }
    fn favorite(&self, _id: u64, _fav: bool) -> Result<()> {
        self.err()
    }
    fn vote(&self, _id: u64, _up: bool) -> Result<()> {
        self.err()
    }
    fn own_ids(&self) -> Result<Vec<u64>> {
        self.err()
    }
    fn submit(
        &self,
        _req: &UpdateRequest,
        _progress: &dyn Fn(f32),
        _cancel: &AtomicBool,
    ) -> Result<SubmitOutcome> {
        self.err()
    }
    fn download(&self, _id: u64, _progress: &dyn Fn(f32), _cancel: &AtomicBool) -> Result<PathBuf> {
        self.err()
    }
    fn upload_gallery(
        &self,
        _file_id: u64,
        _screenshots: &[PathBuf],
        _log: &dyn Fn(&str),
        _cancel: &AtomicBool,
    ) -> Result<usize> {
        self.err()
    }
    fn additional_preview_urls(&self, _file_id: u64, _cancel: &AtomicBool) -> Result<Vec<String>> {
        self.err()
    }
    fn install_info(&self, _id: u64) -> Option<InstallDir> {
        None
    }
    fn open_in_browser(&self, id: u64) {
        let url = format!("https://steamcommunity.com/sharedfiles/filedetails/?id={id}");
        let _ = greg_platform::process::open_url(&url);
    }
}

/// Connects to Steam, falling back to [`UnavailableBackend`] with the reason.
/// Always available (never fails): check [`BackendStatus`] afterwards.
pub fn connect(app_id: u32) -> Box<dyn SteamBackend> {
    #[cfg(feature = "steam")]
    {
        match real::RealBackend::connect(app_id) {
            Ok(backend) => Box::new(backend),
            Err(e) => Box::new(UnavailableBackend::new(e.to_string())),
        }
    }
    #[cfg(not(feature = "steam"))]
    {
        let _ = app_id;
        Box::new(UnavailableBackend::new("built without the `steam` feature"))
    }
}

/// Outcome helper shared by service code.
pub fn outcome_from_submit(outcome: SubmitOutcome) -> PublishOutcome {
    if outcome.needs_agreement {
        PublishOutcome::fail("Workshop legal agreement must be accepted in the Steam client.")
    } else {
        PublishOutcome::ok(outcome.file_id)
    }
}

#[cfg(feature = "steam")]
pub(crate) mod real {
    use super::*;
    use std::sync::atomic::Ordering;
    use std::sync::mpsc;
    use std::time::{Duration, Instant};

    use greg_core::limits::MAX_WORKSHOP_FILE_ID;
    use steamworks::{
        AppIDs, AppId, Client, FileType, PublishedFileId, PublishedFileVisibility, QueryResult,
        QueryResults, UGCQueryType, UGCType, UpdateHandle, UserList, UserListOrder,
    };

    /// Live backend using the `steamworks` crate.
    pub struct RealBackend {
        client: Client,
        app_id: AppId,
    }

    impl RealBackend {
        /// Connects via `Client::init_app`.
        pub fn connect(app_id: u32) -> Result<Self> {
            let client = Client::init_app(AppId(app_id))
                .map_err(|e| SteamError::NotAvailable(e.to_string()))?;
            Ok(Self {
                client,
                app_id: AppId(app_id),
            })
        }

        fn pump_until<T: Send + 'static>(
            &self,
            timeout: Duration,
            cancel: &AtomicBool,
            rx: mpsc::Receiver<Result<T>>,
        ) -> Result<T> {
            let deadline = Instant::now() + timeout;
            loop {
                if cancel.load(Ordering::Relaxed) {
                    return Err(SteamError::Cancelled);
                }
                self.client.run_callbacks();
                match rx.recv_timeout(Duration::from_millis(25)) {
                    Ok(result) => return result,
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        return Err(SteamError::Api("Steam call dropped".into()));
                    }
                }
                if Instant::now() >= deadline {
                    return Err(SteamError::Timeout("no result from Steam".into()));
                }
            }
        }

        fn query_type(sort: WorkshopSortMode) -> UGCQueryType {
            match sort {
                WorkshopSortMode::LastUpdated => UGCQueryType::RankedByLastUpdatedDate,
                WorkshopSortMode::Newest => UGCQueryType::RankedByPublicationDate,
                WorkshopSortMode::TopRated => UGCQueryType::RankedByVote,
                WorkshopSortMode::Trending => UGCQueryType::RankedByTrend,
                WorkshopSortMode::MostSubscribed => UGCQueryType::RankedByTotalUniqueSubscriptions,
                WorkshopSortMode::TitleAsc => UGCQueryType::RankedByLastUpdatedDate,
            }
        }

        fn map_page(results: &QueryResults) -> PageResult {
            let mut items = Vec::new();
            for i in 0..results.returned_results() {
                if let Some(r) = results.get(i) {
                    items.push(BrowseItem {
                        published_file_id: r.published_file_id.0,
                        title: if r.title.trim().is_empty() {
                            format!("Item {}", r.published_file_id.0)
                        } else {
                            r.title.clone()
                        },
                        preview_image_url: results.preview_url(i).unwrap_or_default(),
                        score: r.score,
                        num_subscriptions: subscriptions_of(results, i),
                        updated: r.time_updated,
                    });
                }
            }
            PageResult {
                items,
                total: results.total_results(),
            }
        }

        fn map_detail(client: &Client, results: &QueryResults, detail: &QueryResult) -> ItemDetail {
            let hints = crate::inference::infer("en", &detail.tags, &detail.description);
            ItemDetail {
                published_file_id: detail.published_file_id.0,
                title: detail.title.clone(),
                description: detail.description.clone(),
                tags: detail.tags.clone(),
                owner_name: client.friends().get_friend(detail.owner).name(),
                owner_steam_id: detail.owner.raw(),
                preview_image_url: results.preview_url(0).unwrap_or_default(),
                created: detail.time_created,
                updated: detail.time_updated,
                score: detail.score,
                size_bytes: u64::from(detail.file_size),
                is_subscribed: client
                    .ugc()
                    .subscribed_items(false)
                    .contains(&detail.published_file_id),
                visibility: match detail.visibility {
                    PublishedFileVisibility::Public => "Public".into(),
                    PublishedFileVisibility::FriendsOnly => "FriendsOnly".into(),
                    _ => "Private".into(),
                },
                url: detail.url.clone(),
                dependency_hint_compact: hints.compact_line,
                dependency_hint_block: hints.bullet_block,
                has_dependency_hints: hints.has_any,
            }
        }

        fn apply_fields(
            editor: UpdateHandle,
            req: &UpdateRequest,
            visibility: PublishedFileVisibility,
        ) -> UpdateHandle {
            editor
                .title(&req.title)
                .description(&req.description)
                .content_path(&req.content_dir)
                .preview_path(&req.preview_file)
                .visibility(visibility)
                .tags(req.tags.clone(), false)
        }

        fn fetch_detail(&self, id: u64, cancel: &AtomicBool) -> Result<ItemDetail> {
            let ugc = self.client.ugc();
            let handle = ugc
                .query_item(PublishedFileId(id))
                .map_err(|_| SteamError::Api("query rejected".into()))?;
            let handle = handle.set_return_long_description(true);
            let (tx, rx) = mpsc::channel();
            let client = self.client.clone();
            handle.fetch(move |result| {
                let mapped = result
                    .map(|results| {
                        results
                            .get(0)
                            .map(|detail| Self::map_detail(&client, &results, &detail))
                    })
                    .map_err(|e| SteamError::Api(e.to_string()));
                let _ = tx.send(mapped);
            });
            self.pump_until(Duration::from_secs(45), cancel, rx)?
                .ok_or_else(|| SteamError::Api(format!("item {id} not found")))
        }
    }

    fn subscriptions_of(results: &QueryResults, index: u32) -> u64 {
        use steamworks::UGCStatisticType;
        results
            .statistic(index, UGCStatisticType::Subscriptions)
            .unwrap_or(0)
    }

    impl SteamBackend for RealBackend {
        fn status(&self) -> BackendStatus {
            if !self.client.user().logged_on() {
                return BackendStatus::Available {
                    user_name: "Steam User".into(),
                };
            }
            let name = self
                .client
                .friends()
                .get_friend(self.client.user().steam_id())
                .name();
            BackendStatus::Available {
                user_name: if name.trim().is_empty() {
                    "Steam User".into()
                } else {
                    name
                },
            }
        }

        fn page(&self, query: &PageQuery, cancel: &AtomicBool) -> Result<PageResult> {
            let ugc = self.client.ugc();
            let app_ids = AppIDs::CreatorAppId(self.app_id);
            let mut handle = match &query.kind {
                QueryKind::Browse(sort) => ugc
                    .query_all(Self::query_type(*sort), UGCType::Items, app_ids, query.page)
                    .map_err(|_| SteamError::Api("query rejected".into()))?,
                QueryKind::Search(text) => {
                    let h = ugc
                        .query_all(
                            UGCQueryType::RankedByTextSearch,
                            UGCType::Items,
                            app_ids,
                            query.page,
                        )
                        .map_err(|_| SteamError::Api("query rejected".into()))?;
                    h.set_search_text(text)
                }
                QueryKind::Subscribed => ugc
                    .query_user(
                        self.client.user().steam_id().account_id(),
                        UserList::Subscribed,
                        UGCType::Items,
                        UserListOrder::LastUpdatedDesc,
                        app_ids,
                        query.page,
                    )
                    .map_err(|_| SteamError::Api("query rejected".into()))?,
                QueryKind::Favorited => ugc
                    .query_user(
                        self.client.user().steam_id().account_id(),
                        UserList::Favorited,
                        UGCType::Items,
                        UserListOrder::LastUpdatedDesc,
                        app_ids,
                        query.page,
                    )
                    .map_err(|_| SteamError::Api("query rejected".into()))?,
                QueryKind::Published => ugc
                    .query_user(
                        self.client.user().steam_id().account_id(),
                        UserList::Published,
                        UGCType::Items,
                        UserListOrder::LastUpdatedDesc,
                        app_ids,
                        query.page,
                    )
                    .map_err(|_| SteamError::Api("query rejected".into()))?,
                QueryKind::Item(id) => ugc
                    .query_item(PublishedFileId(*id))
                    .map_err(|_| SteamError::Api("query rejected".into()))?,
            };
            handle = handle.set_return_long_description(true);
            if let Some(tag) = &query.tag {
                handle = handle.add_required_tag(tag);
            }
            let (tx, rx) = mpsc::channel();
            handle.fetch(move |result| {
                let mapped = result
                    .map(|results| Self::map_page(&results))
                    .map_err(|e| SteamError::Api(e.to_string()));
                let _ = tx.send(mapped);
            });
            self.pump_until(Duration::from_secs(45), cancel, rx)
        }

        fn details(&self, id: u64, cancel: &AtomicBool) -> Result<ItemDetail> {
            self.fetch_detail(id, cancel)
        }

        fn subscribe(&self, id: u64) -> Result<()> {
            let (tx, rx) = mpsc::channel();
            self.client
                .ugc()
                .subscribe_item(PublishedFileId(id), move |r| {
                    let _ = tx.send(r.map_err(|e| SteamError::Api(e.to_string())));
                });
            self.pump_until(Duration::from_secs(30), &AtomicBool::new(false), rx)
        }

        fn unsubscribe(&self, id: u64) -> Result<()> {
            let (tx, rx) = mpsc::channel();
            self.client
                .ugc()
                .unsubscribe_item(PublishedFileId(id), move |r| {
                    let _ = tx.send(r.map_err(|e| SteamError::Api(e.to_string())));
                });
            self.pump_until(Duration::from_secs(30), &AtomicBool::new(false), rx)
        }

        fn favorite(&self, _id: u64, _fav: bool) -> Result<()> {
            Err(SteamError::Api(
                "favorites are managed in the Steam client".into(),
            ))
        }

        fn vote(&self, _id: u64, _up: bool) -> Result<()> {
            Err(SteamError::Api(
                "voting is managed in the Steam client".into(),
            ))
        }

        fn own_ids(&self) -> Result<Vec<u64>> {
            let mut ids = Vec::new();
            for page in 1..=5u32 {
                let result = self.page(
                    &PageQuery {
                        kind: QueryKind::Published,
                        page,
                        tag: None,
                    },
                    &AtomicBool::new(false),
                )?;
                if result.items.is_empty() {
                    break;
                }
                ids.extend(result.items.iter().map(|i| i.published_file_id));
                if result.items.len() as u32 >= result.total {
                    break;
                }
            }
            Ok(ids
                .into_iter()
                .filter(|id| *id != 0 && *id <= MAX_WORKSHOP_FILE_ID)
                .collect())
        }

        fn submit(
            &self,
            req: &UpdateRequest,
            progress: &dyn Fn(f32),
            cancel: &AtomicBool,
        ) -> Result<SubmitOutcome> {
            if !req.content_dir.is_dir() {
                return Err(SteamError::PublishFailed(format!(
                    "content folder missing: {}",
                    req.content_dir.display()
                )));
            }
            if !req.preview_file.is_file() {
                return Err(SteamError::PublishFailed(
                    "a preview image is required".into(),
                ));
            }
            let visibility = match req.visibility.as_str() {
                "Private" => PublishedFileVisibility::Private,
                "FriendsOnly" => PublishedFileVisibility::FriendsOnly,
                _ => PublishedFileVisibility::Public,
            };
            let create_id = match req.file_id {
                Some(id) => Some(id),
                None => {
                    let (tx, rx) = mpsc::channel();
                    self.client
                        .ugc()
                        .create_item(self.app_id, FileType::Community, move |r| {
                            let _ = tx.send(
                                r.map(|(id, agreement)| (id.0, agreement))
                                    .map_err(|e| SteamError::Api(e.to_string())),
                            );
                        });
                    let (new_id, agreement) =
                        self.pump_until(Duration::from_secs(60), cancel, rx)?;
                    if agreement {
                        return Err(SteamError::NeedsAgreement);
                    }
                    if new_id == 0 || new_id > MAX_WORKSHOP_FILE_ID {
                        return Err(SteamError::PublishFailed(format!(
                            "Steam returned no usable file id ({new_id})"
                        )));
                    }
                    Some(new_id)
                }
            };
            let editor = Self::apply_fields(
                self.client
                    .ugc()
                    .start_item_update(self.app_id, PublishedFileId(create_id.unwrap_or(0))),
                req,
                visibility,
            );
            let (tx, rx) = mpsc::channel();
            let watch = editor.submit(req.change_note.as_deref(), move |r| {
                let _ = tx.send(
                    r.map(|(id, agreement)| (id.0, agreement))
                        .map_err(|e| SteamError::Api(e.to_string())),
                );
            });
            let deadline = Instant::now() + Duration::from_secs(240);
            loop {
                if cancel.load(Ordering::Relaxed) {
                    return Err(SteamError::Cancelled);
                }
                self.client.run_callbacks();
                let (_status, done, total) = watch.progress();
                if total > 0 {
                    progress((done as f32 / total as f32).clamp(0.0, 1.0));
                }
                match rx.recv_timeout(Duration::from_millis(50)) {
                    Ok(result) => {
                        let (file_id, agreement) = result?;
                        if agreement {
                            return Err(SteamError::NeedsAgreement);
                        }
                        if file_id == 0 || file_id > MAX_WORKSHOP_FILE_ID {
                            return Err(SteamError::PublishFailed(format!(
                                "Steam returned no usable file id ({file_id})"
                            )));
                        }
                        return Ok(SubmitOutcome {
                            file_id,
                            needs_agreement: false,
                        });
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        return Err(SteamError::Api("submit call dropped".into()));
                    }
                }
                if Instant::now() >= deadline {
                    return Err(SteamError::Timeout("submit produced no result".into()));
                }
            }
        }

        /// Gallery screenshots via a second update (`AddItemPreviewFile`).
        ///
        /// The safe `steamworks` 0.13.1 API exposes neither
        /// `AddItemPreviewFile` nor additional-preview query getters, so this
        /// flow drops to `steamworks::sys` FFI for the update calls only.
        /// Completion is polled through
        /// `ManualDispatch_GetAPICallResult` (read side; no manual vtables):
        /// the regular `run_callbacks` pump in the loop keeps dispatching the
        /// pipe, and the deadline bounds the wait — worst case is a timeout
        /// error, never corruption. Interface version must match the one the
        /// `steamworks` crate binds (v021); a version bump fails loudly at
        /// link time, not silently at runtime.
        fn upload_gallery(
            &self,
            file_id: u64,
            screenshots: &[PathBuf],
            log: &dyn Fn(&str),
            cancel: &AtomicBool,
        ) -> Result<usize> {
            use std::ffi::CString;
            use steamworks_sys as sys;

            if screenshots.is_empty() {
                return Ok(0);
            }
            let mut files: Vec<CString> = Vec::new();
            for path in screenshots {
                match CString::new(path.to_string_lossy().as_bytes()) {
                    Ok(c) => files.push(c),
                    Err(_) => log(&format!("Gallery skipped (bad path): {}", path.display())),
                }
            }
            if files.is_empty() {
                return Ok(0);
            }
            let attached = unsafe {
                let ugc = sys::SteamAPI_SteamUGC_v021();
                if ugc.is_null() {
                    return Err(SteamError::Api("Steam UGC interface unavailable".into()));
                }
                let handle = sys::SteamAPI_ISteamUGC_StartItemUpdate(ugc, self.app_id.0, file_id);
                if handle == sys::k_UGCUpdateHandleInvalid {
                    return Err(SteamError::Api("Steam rejected the gallery update".into()));
                }
                let mut attached = 0usize;
                for file in &files {
                    let ok = sys::SteamAPI_ISteamUGC_AddItemPreviewFile(
                        ugc,
                        handle,
                        file.as_ptr(),
                        sys::EItemPreviewType::k_EItemPreviewType_Image,
                    );
                    if ok {
                        attached += 1;
                    } else {
                        log("Gallery attach rejected by Steam for one file.");
                    }
                }
                if attached == 0 {
                    return Err(SteamError::Api("Steam attached no gallery files".into()));
                }
                let note = CString::new("gallery").expect("static cstring");
                let call = sys::SteamAPI_ISteamUGC_SubmitItemUpdate(ugc, handle, note.as_ptr());
                if call == 0 {
                    return Err(SteamError::Api("gallery submit rejected".into()));
                }
                let pipe = sys::SteamAPI_GetHSteamPipe();
                let deadline = Instant::now() + Duration::from_secs(240);
                loop {
                    if cancel.load(Ordering::Relaxed) {
                        return Err(SteamError::Cancelled);
                    }
                    self.client.run_callbacks();
                    let mut result: sys::SubmitItemUpdateResult_t = std::mem::zeroed();
                    let mut failed = false;
                    let ready = sys::SteamAPI_ManualDispatch_GetAPICallResult(
                        pipe,
                        call,
                        &mut result as *mut _ as *mut std::os::raw::c_void,
                        std::mem::size_of::<sys::SubmitItemUpdateResult_t>() as std::os::raw::c_int,
                        sys::SubmitItemUpdateResult_t_k_iCallback as std::os::raw::c_int,
                        &mut failed,
                    );
                    if ready {
                        if failed {
                            return Err(SteamError::Api(
                                "gallery submit failed (transport)".into(),
                            ));
                        }
                        if result.m_eResult != sys::EResult::k_EResultOK {
                            return Err(SteamError::Api(format!(
                                "gallery submit failed: {:?}",
                                result.m_eResult
                            )));
                        }
                        // Packed FFI struct: unaligned read for the u64 field.
                        let returned_id =
                            std::ptr::addr_of!(result.m_nPublishedFileId).read_unaligned();
                        if returned_id != file_id {
                            return Err(SteamError::Api(format!(
                                "gallery submit returned wrong item {returned_id}"
                            )));
                        }
                        break attached;
                    }
                    if Instant::now() >= deadline {
                        return Err(SteamError::Timeout(
                            "gallery submit produced no result".into(),
                        ));
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
            };
            log(&format!("Gallery: {attached} screenshot(s) attached."));
            Ok(attached)
        }

        /// Gallery image URLs via a details query with the additional
        /// previews flag (`GetQueryUGCAdditionalPreview` has no safe
        /// `steamworks` 0.13.1 wrapper, same story as the upload side).
        /// Only image previews are returned; the query handle is always
        /// released before returning.
        fn additional_preview_urls(
            &self,
            file_id: u64,
            cancel: &AtomicBool,
        ) -> Result<Vec<String>> {
            use std::ffi::CStr;
            use std::os::raw::c_char;
            use steamworks_sys as sys;

            unsafe {
                let ugc = sys::SteamAPI_SteamUGC_v021();
                if ugc.is_null() {
                    return Err(SteamError::Api("Steam UGC interface unavailable".into()));
                }
                let mut id = file_id;
                let handle =
                    sys::SteamAPI_ISteamUGC_CreateQueryUGCDetailsRequest(ugc, &mut id as *mut _, 1);
                if handle == sys::k_UGCQueryHandleInvalid {
                    return Err(SteamError::Api("gallery query rejected".into()));
                }
                let outcome: Result<Vec<String>> = (|| {
                    if !sys::SteamAPI_ISteamUGC_SetReturnAdditionalPreviews(ugc, handle, true) {
                        return Err(SteamError::Api("gallery flag rejected".into()));
                    }
                    let call = sys::SteamAPI_ISteamUGC_SendQueryUGCRequest(ugc, handle);
                    if call == 0 {
                        return Err(SteamError::Api("gallery query rejected".into()));
                    }
                    let pipe = sys::SteamAPI_GetHSteamPipe();
                    let deadline = Instant::now() + Duration::from_secs(60);
                    loop {
                        if cancel.load(Ordering::Relaxed) {
                            return Err(SteamError::Cancelled);
                        }
                        self.client.run_callbacks();
                        let mut completed: sys::SteamUGCQueryCompleted_t = std::mem::zeroed();
                        let mut failed = false;
                        let ready = sys::SteamAPI_ManualDispatch_GetAPICallResult(
                            pipe,
                            call,
                            &mut completed as *mut _ as *mut std::os::raw::c_void,
                            std::mem::size_of::<sys::SteamUGCQueryCompleted_t>()
                                as std::os::raw::c_int,
                            sys::SteamUGCQueryCompleted_t_k_iCallback as std::os::raw::c_int,
                            &mut failed,
                        );
                        if ready {
                            if failed {
                                return Err(SteamError::Api(
                                    "gallery query failed (transport)".into(),
                                ));
                            }
                            if completed.m_eResult != sys::EResult::k_EResultOK {
                                return Err(SteamError::Api(format!(
                                    "gallery query failed: {:?}",
                                    completed.m_eResult
                                )));
                            }
                            if completed.m_unNumResultsReturned < 1 {
                                return Err(SteamError::Api(format!("item {file_id} not found")));
                            }
                            break;
                        }
                        if Instant::now() >= deadline {
                            return Err(SteamError::Timeout(
                                "gallery query produced no result".into(),
                            ));
                        }
                        std::thread::sleep(Duration::from_millis(50));
                    }
                    let count =
                        sys::SteamAPI_ISteamUGC_GetQueryUGCNumAdditionalPreviews(ugc, handle, 0);
                    let mut urls = Vec::new();
                    for index in 0..count {
                        let mut url_buf = [0 as c_char; 1024];
                        let mut name_buf = [0 as c_char; 256];
                        let mut preview_type = sys::EItemPreviewType::k_EItemPreviewType_Image;
                        let ok = sys::SteamAPI_ISteamUGC_GetQueryUGCAdditionalPreview(
                            ugc,
                            handle,
                            0,
                            index,
                            url_buf.as_mut_ptr(),
                            url_buf.len() as u32,
                            name_buf.as_mut_ptr(),
                            name_buf.len() as u32,
                            &mut preview_type,
                        );
                        if !ok || preview_type != sys::EItemPreviewType::k_EItemPreviewType_Image {
                            continue;
                        }
                        let url = CStr::from_ptr(url_buf.as_ptr())
                            .to_string_lossy()
                            .into_owned();
                        if !url.trim().is_empty() {
                            urls.push(url);
                        }
                    }
                    Ok(urls)
                })();
                sys::SteamAPI_ISteamUGC_ReleaseQueryUGCRequest(ugc, handle);
                outcome
            }
        }

        fn download(
            &self,
            id: u64,
            progress: &dyn Fn(f32),
            cancel: &AtomicBool,
        ) -> Result<PathBuf> {
            let file_id = PublishedFileId(id);
            if !self.client.ugc().download_item(file_id, true) {
                return Err(SteamError::Api(format!("download of {id} rejected")));
            }
            let deadline = Instant::now() + Duration::from_secs(600);
            loop {
                if cancel.load(Ordering::Relaxed) {
                    return Err(SteamError::Cancelled);
                }
                self.client.run_callbacks();
                if let Some((done, total)) = self.client.ugc().item_download_info(file_id) {
                    if total > 0 {
                        progress((done as f32 / total as f32).clamp(0.0, 1.0));
                    }
                }
                if let Some(info) = self.client.ugc().item_install_info(file_id) {
                    let folder = PathBuf::from(&info.folder);
                    if folder.is_dir() {
                        let done = self
                            .client
                            .ugc()
                            .item_download_info(file_id)
                            .map(|(d, t)| t == 0 || d >= t)
                            .unwrap_or(true);
                        if done {
                            progress(1.0);
                            return Ok(folder);
                        }
                    }
                }
                if Instant::now() >= deadline {
                    return Err(SteamError::Timeout(format!("download of {id} timed out")));
                }
                std::thread::sleep(Duration::from_millis(500));
            }
        }

        fn install_info(&self, id: u64) -> Option<InstallDir> {
            self.client
                .ugc()
                .item_install_info(PublishedFileId(id))
                .map(|info| InstallDir {
                    folder: PathBuf::from(info.folder),
                    size_on_disk: info.size_on_disk,
                })
        }

        fn open_in_browser(&self, id: u64) {
            UnavailableBackend::new("").open_in_browser(id);
        }
    }
}
