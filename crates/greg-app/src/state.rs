//! Application state: services, caches, background channels.
//!
//! Workers never touch the UI directly; a 150 ms Slint timer drains all
//! channels and applies results to properties.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use greg_core::l10n;
use greg_core::models::{ProjectSyncState, WorkshopMetadata};
use greg_core::prefs::Preferences;
use greg_platform::filelog::FileLog;
use greg_steam::backend::{BackendStatus, SteamBackend};
use greg_steam::service::WorkshopService;

use crate::worker::JobHandle;
use crate::MainWindow;

/// Icon ids.
pub const ICON_MODMANAGER: &str = "modmanager";
pub const ICON_WORKSHOP: &str = "workshop";
pub const ICON_MODSTORE: &str = "modstore";
pub const ICON_SETTINGS: &str = "settings";

/// Events from background threads, applied on the UI thread.
pub enum UiEvent {
    /// Steam backend finished connecting (swap into the service).
    BackendReady(Arc<dyn SteamBackend>),
    /// GregApi reachability tick.
    Probe { gregapi_ok: bool },
}

/// Shared application state (behind the Slint callbacks).
pub struct AppState {
    /// UI handle (weak; the strong owner lives in `main`).
    pub weak: slint::Weak<MainWindow>,
    /// UI language code.
    pub lang: String,
    /// Preferences store.
    pub prefs: greg_platform::prefs::JsonFilePreferences,
    /// Workspace handle.
    pub workspace: greg_platform::workspace::Workspace,
    /// Workshop service (swapped when the backend connects).
    pub steam: Arc<WorkshopService>,
    /// Raw backend (swapped on connect).
    pub backend: Arc<dyn SteamBackend>,
    /// Tokio runtime for async Modstore calls.
    pub runtime: tokio::runtime::Runtime,
    /// File log.
    pub file_log: FileLog,
    /// UI log lines.
    pub log_lines: Arc<Mutex<Vec<String>>>,

    /// Project rows cache: (root, file_id).
    pub project_rows: Vec<(PathBuf, u64)>,
    /// Cached sync states: project root -> (published_file_id, state).
    /// Content hashing runs in a background job — never on the UI thread.
    pub project_sync: HashMap<PathBuf, (u64, ProjectSyncState)>,
    /// Background sync-state computation.
    pub sync_job: Option<JobHandle>,

    /// Editor state.
    pub editor_root: Option<PathBuf>,
    pub editor_meta: WorkshopMetadata,
    pub editor_changelog: String,
    pub editor_dep_ids: Vec<u64>,
    pub publish_job: Option<JobHandle>,

    /// Browse caches.
    pub browse_items: Vec<(u64, String)>,
    pub browse_detail_id: Option<u64>,
    pub browse_job: Option<JobHandle>,

    /// Modstore caches.
    pub store_items: Vec<greg_modstore::models::ModStoreCatalogItem>,
    pub store_updates: Vec<greg_modstore::models::ModStoreUpdate>,
    pub store_job: Option<JobHandle>,

    /// Local content cache + pending removal for the confirm dialog.
    pub local_entries: Vec<greg_loader::local_content::LocalContentEntry>,
    pub pending_remove: Option<std::path::PathBuf>,
    /// Last scan signature of the visible local page (change detection).
    pub local_sig: String,

    /// Problem actions cache: (action id, target) parallel to the rows.
    pub problem_actions: Vec<(String, String)>,

    /// Pack catalog (packs + shareable collections) + selection.
    pub pack_service: greg_core::models::ModCollectionService,
    /// Selected pack id.
    pub selected_pack: Option<String>,
    /// Pack ids in row order (resolves list indices).
    pub pack_order: Vec<String>,

    /// Last applied settings language index (watch for changes).
    pub settings_lang_index: i32,

    /// Event channel (background → UI tick).
    pub events_tx: Sender<UiEvent>,
    pub events_rx: Receiver<UiEvent>,

    /// Wakes the probe loop for an immediate live check.
    pub probe_wake: Arc<std::sync::atomic::AtomicBool>,

    /// Tick counter (Steam re-check every ~60 s).
    pub tick_count: u64,
}

impl AppState {
    /// Upgraded UI handle (the window outlives the app).
    pub fn ui(&self) -> MainWindow {
        self.weak.upgrade().expect("window gone")
    }

    /// Creates state + UI defaults (Steam connects in the background).
    pub fn new(weak: slint::Weak<MainWindow>) -> Arc<Mutex<Self>> {
        let ui = weak.upgrade().expect("window gone");
        let prefs = greg_platform::prefs::JsonFilePreferences::open();
        let lang = {
            let saved = prefs.get_string("AppLanguage", "");
            if saved.is_empty() {
                "en".to_string()
            } else {
                saved
            }
        };
        let custom = prefs.get_string(greg_platform::workspace::CUSTOM_WORKSPACE_PATH_KEY, "");
        let workspace = greg_platform::workspace::Workspace::resolve(
            if custom.trim().is_empty() {
                None
            } else {
                Some(custom.as_str())
            },
            None,
        );
        let _ = workspace.ensure_structure();
        let _ = workspace.migrate_legacy_projects();

        let backend: Arc<dyn SteamBackend> =
            Arc::from(greg_steam::backend::UnavailableBackend::new("connecting…"));
        let steam = Arc::new(WorkshopService::new(Arc::clone(&backend), lang.clone()));
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("tokio runtime");
        let file_log = FileLog::shared().clone();
        let (events_tx, events_rx) = mpsc::channel();
        let probe_wake = Arc::new(std::sync::atomic::AtomicBool::new(false));

        let state = Arc::new(Mutex::new(Self {
            weak,
            lang,
            prefs,
            workspace,
            steam,
            backend,
            runtime,
            file_log,
            log_lines: Arc::new(Mutex::new(Vec::new())),
            project_rows: Vec::new(),
            project_sync: HashMap::new(),
            sync_job: None,
            editor_root: None,
            editor_meta: WorkshopMetadata::default(),
            editor_changelog: String::new(),
            editor_dep_ids: Vec::new(),
            publish_job: None,
            browse_items: Vec::new(),
            browse_detail_id: None,
            browse_job: None,
            store_items: Vec::new(),
            store_updates: Vec::new(),
            store_job: None,
            local_entries: Vec::new(),
            pending_remove: None,
            local_sig: String::new(),
            problem_actions: Vec::new(),
            pack_service: crate::actions::load_packs(),
            selected_pack: None,
            pack_order: Vec::new(),
            settings_lang_index: -1,
            events_tx,
            events_rx,
            probe_wake: Arc::clone(&probe_wake),
            tick_count: 0,
        }));

        // Static models.
        ui.set_app_version(env!("CARGO_PKG_VERSION").into());
        ui.set_new_templates(model_of(&[
            "Vanilla object (decoration)",
            "MelonLoader mod",
            "gregCore mod",
            "UXML UI override",
            "Standalone 3D model",
            "Standalone texture",
            "Standalone audio",
        ]));
        ui.set_ed_visibilities(model_of(&["Public", "FriendsOnly", "Private"]));
        ui.set_ed_profiles(model_of(&["decoration", "code"]));
        ui.set_ed_modtypes(model_of(&[
            "PlacableObject",
            "MelonloaderPlugin",
            "Userlib",
            "DataCenterMod",
        ]));
        // One browse page with a list switcher; own uploads live in Projects.
        ui.set_br_lists(model_of(&["Browse", "Subscribed", "Favorited"]));
        ui.set_br_sorts(model_of(&[
            "Updated",
            "Newest",
            "Top rated",
            "Trending",
            "Subscribed",
            "Title A–Z",
        ]));
        ui.set_se_langs(model_of(&[
            "System default",
            "English",
            "Deutsch",
            "Français",
            "Español",
            "Italiano",
            "日本語",
            "Polski",
            "Русский",
            "中文",
        ]));
        ui.set_steam_text("Steam: connecting…".into());
        ui.set_gregapi_text("gregAPI: probing…".into());
        ui.set_login_visible(false);
        ui.set_modstore_available(false);
        ui.set_br_page(1);

        // Initial navigation: projects icon.
        Self::apply_icon_state(&ui, ICON_MODMANAGER, "mymods");

        // Background: Steam connect, then GregApi probes. No AppState
        // crosses threads here; the UI timer (owned by `main`) drains.
        Self::spawn_probe(
            state.lock().expect("state").events_tx.clone(),
            Arc::clone(&probe_wake),
            true,
        );
        {
            let mut guard = state.lock().expect("state");
            crate::actions::refresh_projects(&mut guard);
            crate::actions::refresh_settings(&mut guard);
            // Initial scan so My Mods is populated from the first frame.
            crate::actions::local_refresh(&mut guard);
        }
        state
    }

    /// Switches icon + navigation items (Rust side of `icon-clicked`).
    pub fn apply_icon_state(ui: &MainWindow, icon: &str, page: &str) {
        ui.set_current_icon(icon.into());
        ui.set_current_page(page.into());
        match icon {
            ICON_MODMANAGER => {
                ui.set_nav_visible(true);
                ui.set_nav_title("MODMANAGER".into());
                ui.set_nav_items(model_of_structs(vec![
                    ("mymods", "My Mods"),
                    ("myplugins", "My Plugins"),
                    ("mylibs", "My Libs"),
                    ("modpacks", "Modpacks"),
                    ("collections", "Collections"),
                    ("problems", "Problems"),
                    ("report-bug", "Report a Bug"),
                ]));
            }
            ICON_WORKSHOP => {
                ui.set_nav_visible(true);
                ui.set_nav_title("WORKSHOP UPLOAD".into());
                ui.set_nav_items(model_of_structs(vec![
                    ("projects", "Projects"),
                    ("new", "New Project"),
                    ("browse", "Browse"),
                    ("subscribed", "Subscribed"),
                    ("favorited", "Favorited"),
                ]));
            }
            _ => {
                // Modstore needs no navigation (tabs suffice); settings is one page.
                ui.set_nav_visible(false);
                ui.set_nav_title("".into());
                ui.set_nav_items(model_of_structs(vec![]));
            }
        }
    }

    /// Applies probe results to the shell (gating included).
    pub fn apply_probe(
        ui: &MainWindow,
        steam_ok: bool,
        steam_user: Option<String>,
        steam_hint: String,
        gregapi_ok: bool,
    ) {
        if steam_ok {
            ui.set_steam_ok(true);
            ui.set_steam_text(
                format!(
                    "Steam Connected{}",
                    steam_user.map(|u| format!(" · {u}")).unwrap_or_default()
                )
                .into(),
            );
        } else {
            ui.set_steam_ok(false);
            ui.set_steam_text(if steam_hint.is_empty() {
                "Steam Disconnected".into()
            } else {
                format!("Steam Disconnected · {steam_hint}").into()
            });
        }
        ui.set_gregapi_ok(gregapi_ok);
        ui.set_gregapi_text(if gregapi_ok {
            "gregAPI Online".into()
        } else {
            "gregAPI Offline".into()
        });
        // Gating: GregApi red → login hidden, Modstore unavailable.
        let modstore_enabled = greg_core::models::settings::is_modstore_enabled();
        let available = gregapi_ok && modstore_enabled;
        ui.set_modstore_available(available);
        // Login is only offered when the API is reachable and no session exists.
        // (Session handling arrives with the OAuth callback flow.)
        ui.set_login_visible(gregapi_ok && !ui.get_profile_visible());
        if !available && ui.get_current_page() == "modstore" {
            Self::apply_icon_state(ui, ICON_MODMANAGER, "mymods");
        }
    }

    /// Spawns the connect + probe loop (30 s cadence, immediate on demand
    /// via the shared wake flag).
    fn spawn_probe(
        tx: Sender<UiEvent>,
        wake: Arc<std::sync::atomic::AtomicBool>,
        connect_first: bool,
    ) {
        use std::sync::atomic::Ordering;
        std::thread::spawn(move || {
            if connect_first {
                let backend =
                    greg_steam::backend::connect(greg_core::models::settings::DATA_CENTER_APP_ID);
                let _ = tx.send(UiEvent::BackendReady(Arc::from(backend)));
            }
            loop {
                let _ = tx.send(UiEvent::Probe {
                    gregapi_ok: probe_gregapi(),
                });
                // Sleep in 1 s slices so an on-demand probe wakes us early.
                for _ in 0..30 {
                    if wake.swap(false, Ordering::SeqCst) {
                        break;
                    }
                    std::thread::sleep(Duration::from_secs(1));
                }
            }
        });
    }

    /// Re-reads Steam status into the shell (cheap local IPC).
    pub fn refresh_steam_status(&self) {
        let (ok, user, hint) = match self.backend.status() {
            BackendStatus::Available { user_name } => (true, Some(user_name), String::new()),
            BackendStatus::Unavailable { hint } => (false, None, hint),
        };
        let ui = self.ui();
        // Keep the GregApi side untouched; gating recomputed by caller.
        let gregapi_ok = ui.get_gregapi_ok();
        Self::apply_probe(&ui, ok, user, hint, gregapi_ok);
    }

    /// Requests an immediate live probe (Modstore nav, store refresh).
    pub fn request_probe(state: &Arc<Mutex<Self>>) {
        let flag = state.lock().expect("state").probe_wake.clone();
        flag.store(true, std::sync::atomic::Ordering::SeqCst);
    }

    /// Logs to buffer + file.
    pub fn append_log(state: &Arc<Mutex<Self>>, line: &str) {
        let lines = state.lock().expect("state").log_lines.clone();
        let file_log = state.lock().expect("state").file_log.clone();
        {
            let mut guard = lines.lock().expect("log lock");
            guard.push(line.to_string());
            if guard.len() > 500 {
                guard.remove(0);
            }
        }
        file_log.info(line);
    }

    /// Current log snapshot for the settings page.
    pub fn log_snapshot(&self) -> Vec<String> {
        self.log_lines.lock().expect("log lock").clone()
    }

    /// Localized string with the current language.
    pub fn tr(&self, key: &str) -> String {
        l10n::get(&self.lang, key)
    }
}

/// Live API probe: `GET {api}/api/v1/mods` must answer 2xx + JSON.
/// Redirects, error pages, timeouts and DNS failures all mean OFFLINE —
/// a bare domain answering (e.g. 307 → 503 parking) is not a live API.
fn probe_gregapi() -> bool {
    if !greg_core::models::settings::is_modstore_enabled() {
        return false;
    }
    let base = greg_core::models::settings::modstore_api_url();
    let url = format!("{}/api/v1/mods", base.trim_end_matches('/'));
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .redirect(reqwest::redirect::Policy::none())
        .build();
    let Ok(client) = client else {
        return false;
    };
    match client.get(&url).send() {
        Ok(response) => {
            let status = response.status().as_u16();
            let is_json = response.json::<serde_json::Value>().is_ok();
            api_is_live(status, is_json)
        }
        Err(_) => false,
    }
}

/// Liveness rule, pure and unit-tested: only a successful JSON answer counts.
fn api_is_live(status: u16, body_is_json: bool) -> bool {
    (200..300).contains(&status) && body_is_json
}

/// String list → Slint model.
pub fn model_of(items: &[&str]) -> slint::ModelRc<slint::SharedString> {
    let model = std::rc::Rc::new(slint::VecModel::from(
        items
            .iter()
            .map(|s| slint::SharedString::from(*s))
            .collect::<Vec<_>>(),
    ));
    model.into()
}

/// Nav items → Slint model.
pub fn model_of_structs(items: Vec<(&str, &str)>) -> slint::ModelRc<crate::NavItem> {
    let model = std::rc::Rc::new(slint::VecModel::from(
        items
            .into_iter()
            .map(|(id, label)| crate::NavItem {
                id: id.into(),
                label: label.into(),
            })
            .collect::<Vec<_>>(),
    ));
    model.into()
}

#[cfg(test)]
mod probe_tests {
    use super::api_is_live;

    #[test]
    fn only_successful_json_counts() {
        assert!(api_is_live(200, true));
        assert!(api_is_live(201, true));
        assert!(!api_is_live(200, false));
        assert!(!api_is_live(307, true));
        assert!(!api_is_live(404, false));
        assert!(!api_is_live(503, false));
        assert!(!api_is_live(500, true));
    }
}
