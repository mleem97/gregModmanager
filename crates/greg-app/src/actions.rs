//! Slint callback implementations (one section per shell area).

use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use greg_core::l10n;
use greg_core::models::{ProjectSyncState, WorkshopMetadata};
use greg_core::prefs::Preferences;
use slint::{ComponentHandle as _, Model, ModelRc, SharedString, VecModel};

use crate::state::{
    AppState, UiEvent, ICON_MODMANAGER, ICON_MODSTORE, ICON_SETTINGS, ICON_WORKSHOP,
};
use crate::worker::{JobHandle, JobMessage, JobOutcome};
use crate::{
    BrowseRow, CheckRow, HealthRow, LocalRow, PackEntryRow, PackRow, ProblemRow, ProjectRow,
    StoreRow, UpdateRow,
};

// ---------------------------------------------------------------------------
// Tick: drain events + background jobs (runs on the UI thread).
// ---------------------------------------------------------------------------

/// Drains all channels and applies results to properties.
pub fn tick(state: &mut AppState) {
    // Background events (connect, probe).
    let events: Vec<UiEvent> = {
        let mut out = Vec::new();
        while let Ok(event) = state.events_rx.try_recv() {
            out.push(event);
        }
        out
    };
    for event in events {
        match event {
            UiEvent::BackendReady(backend) => {
                state.backend = Arc::clone(&backend);
                state.steam = Arc::new(greg_steam::service::WorkshopService::new(
                    backend,
                    state.lang.clone(),
                ));
                state.refresh_steam_status();
                refresh_projects(state);
            }
            UiEvent::Probe { gregapi_ok } => {
                let ui = state.ui();
                // Re-read Steam status locally (cheap IPC), keep GregApi side fresh.
                let (ok, user, hint) = match state.backend.status() {
                    greg_steam::backend::BackendStatus::Available { user_name } => {
                        (true, Some(user_name), String::new())
                    }
                    greg_steam::backend::BackendStatus::Unavailable { hint } => (false, None, hint),
                };
                AppState::apply_probe(&ui, ok, user, hint, gregapi_ok);
            }
            UiEvent::AuthSessionEstablished(session) => {
                apply_auth_session(state, session);
            }
            UiEvent::AuthSessionFailed(message) => {
                state.push_log(&format!("Login failed: {message}"));
                let ui = state.ui();
                ui.set_ed_status(format!("Login failed: {message}").into());
            }
        }
    }
    // Steam re-check roughly every 60 s (400 × 150 ms).
    state.tick_count += 1;
    if state.tick_count.is_multiple_of(400) {
        state.refresh_steam_status();
    }
    // Local auto-scan roughly every 10 s, only while a local page is
    // visible and only when the folder actually changed (no flicker).
    if state.tick_count.is_multiple_of(66) {
        auto_refresh_local(state);
        refresh_stale_game_ui(state);
    }
    // Game watchdog roughly every 1 s (cheap non-blocking poll).
    if state.tick_count.is_multiple_of(7) {
        poll_game_state(state);
    }
    drain_publish(state);
    drain_sync(state);
    drain_thumb(state);
    drain_browse(state);
    drain_store(state);
    // Language switch watch (ComboBox has no change callback wired).
    let lang_index = state.ui().get_se_lang();
    if state.settings_lang_index != lang_index {
        state.settings_lang_index = lang_index;
        settings_apply_lang_index(state, lang_index);
    }
}

/// Applies a settings language index (shared by watch; no extra callback).
fn settings_apply_lang_index(state: &mut AppState, index: i32) {
    let codes = ["", "en", "de", "fr", "es", "it", "ja", "pl", "ru", "zh"];
    let code = codes.get(index.max(0) as usize).copied().unwrap_or("en");
    state.lang = if code.is_empty() {
        "en".into()
    } else {
        code.into()
    };
    state.prefs.set_string("AppLanguage", code);
    let backend = Arc::clone(&state.backend);
    state.steam = Arc::new(greg_steam::service::WorkshopService::new(
        backend,
        state.lang.clone(),
    ));
}

// ---------------------------------------------------------------------------
// Shell: icons, navigation, header.
// ---------------------------------------------------------------------------

/// Handles icon clicks (navigation mapping per icon).
pub fn on_icon(state: &Arc<std::sync::Mutex<AppState>>, icon: &str) {
    let mut guard = state.lock().expect("state");
    if icon == ICON_MODSTORE && !guard.ui().get_modstore_available() {
        return;
    }
    match icon {
        ICON_MODMANAGER => {
            AppState::apply_icon_state(&guard.ui(), ICON_MODMANAGER, "mymods");
            drop(guard);
            local_refresh(&mut state.lock().expect("state"));
        }
        ICON_WORKSHOP => {
            AppState::apply_icon_state(&guard.ui(), ICON_WORKSHOP, "projects");
            guard.ui().set_br_list(0);
        }
        ICON_MODSTORE => {
            AppState::apply_icon_state(&guard.ui(), ICON_MODSTORE, "modstore");
            drop(guard);
            // Fresh live check the moment the user heads for the Modstore.
            AppState::request_probe(state);
        }
        ICON_SETTINGS => {
            AppState::apply_icon_state(&guard.ui(), ICON_SETTINGS, "settings");
            refresh_settings(&mut guard);
        }
        _ => {}
    }
}

/// Bug-report platform: the teamGreg org (all Greg mods & programs) plus
/// this app's tracker. Alternative to Discord/Steam threads.
pub const BUG_REPORT_URL: &str =
    "https://git.datacentermods.com/teamGreg/gregModmanager/issues/new";
/// Org landing: lists every Greg mod/program repository with its tracker.
pub const BUG_REPORT_ORG_URL: &str = "https://git.datacentermods.com/teamGreg";

/// Handles navigation clicks (page ids + the report action).
pub fn on_nav(state: &Arc<std::sync::Mutex<AppState>>, id: &str) {
    if id == "report-bug" {
        // Action row, not a page: open the chooser dialog, stay where we are.
        let guard = state.lock().expect("state");
        guard.ui().set_show_bug_dialog(true);
        return;
    }
    if id == "problems" {
        {
            let guard = state.lock().expect("state");
            guard.ui().set_current_page("problems".into());
        }
        refresh_problems(state);
        return;
    }
    if id == "modpacks" || id == "collections" {
        {
            let guard = state.lock().expect("state");
            guard.ui().set_current_page(id.into());
        }
        packs_refresh(state);
        return;
    }
    if matches!(id, "mymods" | "myplugins" | "mylibs") {
        {
            let guard = state.lock().expect("state");
            guard.ui().set_current_page(id.into());
        }
        local_refresh(&mut state.lock().expect("state"));
        return;
    }
    // One browse page for all lists; entering via nav auto-loads it so the
    // page never sits empty. "My Uploads" is gone: own uploads live in
    // Projects (sync state + Steam link per row).
    let fetch = matches!(id, "browse" | "subscribed" | "favorited");
    {
        let guard = state.lock().expect("state");
        match id {
            "projects" | "new" => guard.ui().set_current_page(id.into()),
            "browse" => {
                guard.ui().set_br_list(0);
                guard.ui().set_current_page("browse".into());
            }
            "subscribed" => {
                guard.ui().set_br_list(1);
                guard.ui().set_current_page("browse".into());
            }
            "favorited" => {
                guard.ui().set_br_list(2);
                guard.ui().set_current_page("browse".into());
            }
            _ => {}
        }
    }
    if fetch {
        browse_go(state, None);
    }
}

/// Launches the game as a supervised direct-exe child.
///
/// Modes: `steam` (quick start, mods as installed), `mods` (ensures the
/// parked `Mods/` folder is restored first), `vanilla` (parks `Mods/`
/// away for this session, restored on exit). The Steamworks session is
/// released before the start — otherwise Steam keeps seeing the game as
/// running through the manager and the next launch fails until
/// everything is killed (Windows zombie case).
pub fn start_game(state: &Arc<std::sync::Mutex<AppState>>, mode: &str) {
    poll_game_state(&mut state.lock().expect("state"));
    let launch_mode = match mode {
        "vanilla" => greg_loader::launch::LaunchMode::Vanilla,
        _ => greg_loader::launch::LaunchMode::Mods,
    };
    // Refuse while our own session runs.
    if let Some(pid) = tracked_game_pid(&mut state.lock().expect("state")) {
        let guard = state.lock().expect("state");
        AppState::append_log(
            state,
            &l10n::format(&guard.lang, "Game_AlreadyRunning", &[&pid]),
        );
        return;
    }
    // Refuse while a stale game process lives (would disagree with Steam).
    let stale = greg_loader::launch::stale_game_pids();
    if let Some(pid) = stale.first() {
        let guard = state.lock().expect("state");
        AppState::append_log(
            state,
            &l10n::format(&guard.lang, "Game_StaleRunning", &[pid]),
        );
        let mut guard = state.lock().expect("state");
        set_game_ui(&mut guard, false, None);
        return;
    }
    // Detect the installation (explicit root or auto-detect).
    let (game_root, exe) = {
        let guard = state.lock().expect("state");
        let root = game_root_of(&guard);
        if root.as_os_str().is_empty() {
            AppState::append_log(state, "Launch: game folder not found.");
            return;
        }
        let registry = greg_loader::game::GameAdapterRegistry::with_defaults();
        match registry.detect(Some(&root)) {
            Some((adapter, installation)) => {
                let plan = adapter.plan_launch(&installation.root_path, vec![]);
                (installation.root_path, plan.executable)
            }
            None => {
                AppState::append_log(state, "Launch: game installation not found.");
                return;
            }
        }
    };
    let Some(exe) = exe else {
        AppState::append_log(state, "Launch: game executable not found.");
        return;
    };
    // Free the Steam session first so the play session owns the app state.
    release_steam_for_game(&mut state.lock().expect("state"));
    match greg_loader::launch::GameSession::start(&game_root, &exe, launch_mode) {
        Ok(session) => {
            let pid = session.pid();
            let mut guard = state.lock().expect("state");
            AppState::append_log(
                state,
                &l10n::format(&guard.lang, "Game_Started", &[&launch_mode.label(), &pid]),
            );
            guard.game_session = Some(session);
            set_game_ui(&mut guard, true, Some(pid));
        }
        Err(e) => {
            AppState::append_log(state, &format!("Launch failed: {e}"));
            let mut guard = state.lock().expect("state");
            reconnect_steam(&mut guard);
        }
    }
}

/// Stops the tracked game session, or kills stale game processes when no
/// session exists (the Windows zombie escape hatch). Restores a parked
/// `Mods/` folder and reconnects Steamworks afterwards.
pub fn stop_game(state: &Arc<std::sync::Mutex<AppState>>) {
    // Tracked session first.
    let stopped = {
        let mut guard = state.lock().expect("state");
        let running = match guard.game_session.as_mut() {
            Some(session) => session.is_running(),
            None => false,
        };
        if !running {
            false
        } else {
            match guard.game_session.as_mut() {
                Some(session) => match session.stop() {
                    Ok(_) => {
                        AppState::append_log(state, &l10n::get(&guard.lang, "Game_Stopped"));
                        true
                    }
                    Err(e) => {
                        AppState::append_log(state, &format!("Stop failed: {e}"));
                        false
                    }
                },
                None => false,
            }
        }
    };
    if stopped {
        let mut guard = state.lock().expect("state");
        guard.game_session = None;
        reconnect_steam(&mut guard);
        set_game_ui(&mut guard, false, None);
        local_refresh(&mut guard);
        return;
    }
    // No live session: kill stale processes by exe name.
    let stale = greg_loader::launch::stale_game_pids();
    if stale.is_empty() {
        AppState::append_log(state, "Stop: no game process running.");
        return;
    }
    for pid in &stale {
        match greg_platform::process::kill_tree(*pid) {
            Ok(()) => {
                AppState::append_log(state, &format!("Stop: killed stale game process {pid}."))
            }
            Err(e) => AppState::append_log(state, &format!("Stop failed for pid {pid}: {e}")),
        }
    }
    // A stale kill may have freed a vanilla parking: restore when idle.
    {
        let mut guard = state.lock().expect("state");
        recover_parked_mods(&mut guard);
        reconnect_steam(&mut guard);
        set_game_ui(&mut guard, false, None);
        local_refresh(&mut guard);
    }
}

/// Polls the tracked session (cheap non-blocking wait). On exit the
/// vanilla state is already unwound by the session; Steamworks reconnects
/// and the UI flips back. Call regularly from the UI tick.
pub fn poll_game_state(state: &mut AppState) {
    let event = match state.game_session.as_mut() {
        Some(session) => session.poll(),
        None => return,
    };
    match event {
        greg_loader::launch::SessionEvent::Running => {
            let pid = state.game_session.as_ref().map(|s| s.pid()).unwrap_or(0);
            let mode = state
                .game_session
                .as_ref()
                .map(|s| s.mode().label().to_string())
                .unwrap_or_default();
            let text = l10n::format(&state.lang, "Game_StatusRunning", &[&mode, &pid]);
            state.ui().set_game_text(text.into());
        }
        greg_loader::launch::SessionEvent::Exited { code } => {
            let elapsed = match state.game_session.as_ref() {
                Some(s) => format_duration(s.elapsed()),
                None => String::new(),
            };
            let lang = state.lang.clone();
            state.game_session = None;
            reconnect_steam(state);
            set_game_ui(state, false, None);
            let code_text = code.map(|c| c.to_string()).unwrap_or_else(|| "?".into());
            state.push_log(&l10n::format(&lang, "Game_Exited", &[&code_text, &elapsed]));
            local_refresh(state);
        }
    }
}

/// Refreshes the game status UI for stale processes (no tracked session).
/// Runs on the slow tick so the Stop button appears even when the game
/// was started outside the manager (Windows zombie case).
fn refresh_stale_game_ui(state: &mut AppState) {
    let tracked_alive = match state.game_session.as_mut() {
        Some(session) => session.is_running(),
        None => false,
    };
    if tracked_alive {
        return;
    }
    // Drop a dead session object the watchdog has not reaped yet.
    if state.game_session.is_some() {
        poll_game_state(state);
        if state.game_session.is_some() {
            return;
        }
    }
    let stale = greg_loader::launch::stale_game_pids();
    let ui = state.ui();
    ui.set_game_running(!stale.is_empty());
    if let Some(pid) = stale.first() {
        ui.set_game_text(l10n::format(&state.lang, "Game_StaleRunning", &[pid]).into());
    } else {
        ui.set_game_text(l10n::get(&state.lang, "Game_StatusStopped").into());
    }
}

/// True while a tracked session or any stale game process is alive.
/// Mod changes are refused then (they would corrupt a running game).
fn game_running(state: &Arc<std::sync::Mutex<AppState>>) -> bool {
    let mut guard = state.lock().expect("state");
    if let Some(session) = guard.game_session.as_mut() {
        if session.is_running() {
            return true;
        }
    }
    !greg_loader::launch::stale_game_pids().is_empty()
}

/// Logs the stop-first guard message. Returns `true` when blocked.
fn guard_game_running(state: &Arc<std::sync::Mutex<AppState>>) -> bool {
    if game_running(state) {
        let guard = state.lock().expect("state");
        AppState::append_log(state, &l10n::get(&guard.lang, "Game_BlockedWhileRunning"));
        return true;
    }
    false
}
/// restores a parked `Mods/` when the game is gone. A still-running game
/// is intentionally left alive (with its vanilla parking, if any).
pub fn finalize_game_state(state: &Arc<std::sync::Mutex<AppState>>) {
    let mut guard = state.lock().expect("state");
    // NLL-friendly: the session borrow ends before the log call.
    let running_pid = match guard.game_session.as_mut() {
        Some(session) => {
            if session.is_running() {
                Some(session.pid())
            } else {
                None
            }
        }
        None => None,
    };
    if let Some(pid) = running_pid {
        let msg = format!(
            "manager exiting while the game still runs (pid {pid}) — vanilla parking restores at the next start"
        );
        guard.file_log.info(&msg);
        return;
    }
    guard.game_session = None;
    recover_parked_mods(&mut guard);
}

/// Restores a parked `Mods/` folder left by a killed vanilla session.
/// Call at startup and after stale kills. Refreshes the local listing
/// when something was actually restored.
pub fn recover_parked_mods(state: &mut AppState) {
    let root = game_root_of(state);
    if root.as_os_str().is_empty() {
        return;
    }
    if greg_loader::launch::ensure_mods_restored(
        &root,
        greg_loader::launch::any_game_process_running(),
    ) {
        state.push_log("Recovered the parked Mods folder from an unclean vanilla session.");
        local_refresh(state);
    }
}

/// Releases the Steamworks session so Steam stops seeing the game as
/// running through the manager. In-flight jobs are cancelled first; the
/// native shutdown lands when their handles drop.
fn release_steam_for_game(state: &mut AppState) {
    for job in [
        state.store_job.as_ref(),
        state.browse_job.as_ref(),
        state.publish_job.as_ref(),
    ]
    .into_iter()
    .flatten()
    {
        job.cancel();
    }
    let backend: Arc<dyn greg_steam::backend::SteamBackend> = Arc::from(
        greg_steam::backend::UnavailableBackend::new("released for game launch"),
    );
    state.backend = Arc::clone(&backend);
    state.steam = Arc::new(greg_steam::service::WorkshopService::new(
        backend,
        state.lang.clone(),
    ));
    state.steam_released = true;
    state.refresh_steam_status();
    state.push_log("Steam session released for the game launch.");
}

/// Reconnects Steamworks after the play session via the existing
/// `BackendReady` event flow (no-op unless a release happened).
fn reconnect_steam(state: &mut AppState) {
    if !state.steam_released {
        return;
    }
    state.steam_released = false;
    let tx = state.events_tx.clone();
    std::thread::spawn(move || {
        let backend = greg_steam::backend::connect(greg_core::models::settings::DATA_CENTER_APP_ID);
        let _ = tx.send(UiEvent::BackendReady(Arc::from(backend)));
    });
    state.push_log("Reconnecting Steam...");
}

/// Tracked game pid when our own session is alive.
fn tracked_game_pid(state: &mut AppState) -> Option<u32> {
    let running = match state.game_session.as_mut() {
        Some(session) => session.is_running(),
        None => false,
    };
    if running {
        state.game_session.as_ref().map(|s| s.pid())
    } else {
        None
    }
}

/// Formats a play duration as `H:MM:SS`.
fn format_duration(d: std::time::Duration) -> String {
    let s = d.as_secs();
    format!("{}:{:02}:{:02}", s / 3600, (s % 3600) / 60, s % 60)
}

/// Applies the game status UI (header Stop button + status text).
fn set_game_ui(state: &mut AppState, running: bool, pid: Option<u32>) {
    let ui = state.ui();
    ui.set_game_running(running);
    if running {
        let mode = state
            .game_session
            .as_ref()
            .map(|s| s.mode().label().to_string())
            .unwrap_or_default();
        ui.set_game_text(
            l10n::format(
                &state.lang,
                "Game_StatusRunning",
                &[&mode, &pid.unwrap_or(0)],
            )
            .into(),
        );
    } else {
        ui.set_game_text(l10n::get(&state.lang, "Game_StatusStopped").into());
    }
}

fn login_url_formats() -> Vec<String> {
    greg_core::models::settings::auth_endpoint_candidates()
        .into_iter()
        .map(|base| {
            format!(
                "{}/login?client_id=greg_desktop&response_type=code&redirect_uri={{0}}&requestId={{1}}&state=desktop_flow&nonce=mock_nonce",
                base.trim_end_matches('/')
            )
        })
        .collect()
}

fn percent_encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for b in value.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// Generates a cryptographically random desktop auth request id.
///
/// ASVS 3.11 (session ids must be random and unpredictable): the previous
/// timestamp+PID construction was guessable, letting an attacker pre-compute
/// a victim's next `requestId`. 128 bits from the OS CSPRNG, hex-encoded
/// (32 chars, inside the server's `^[a-zA-Z0-9_-]{16,128}$` shape).
fn new_request_id() -> String {
    let mut bytes = [0u8; 16];
    if getrandom::fill(&mut bytes).is_ok() {
        return bytes.iter().map(|b| format!("{b:02x}")).collect();
    }
    // CSPRNG unavailable (should not happen): fall back to a
    // time+pid+counter mix rather than failing the login outright.
    use std::sync::atomic::{AtomicU64, Ordering};
    static FALLBACK_COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let count = FALLBACK_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("fb{nanos:x}{:x}{count:x}", std::process::id())
}

/// Opens the login URL in a browser (session completes via `greg://` later).
pub fn login(state: &Arc<std::sync::Mutex<AppState>>) {
    let request_id = new_request_id();
    let formats = login_url_formats();
    let redirect = percent_encode(greg_modstore::auth_client::AUTH_CALLBACK_REDIRECT_URI);
    let candidate = formats.first().map(|f| {
        f.replacen("{0}", &redirect, 1)
            .replacen("{1}", &percent_encode(&request_id), 1)
    });
    match candidate {
        Some(url) => {
            state.lock().expect("state").pending_request_id = Some(request_id);
            AppState::append_log(state, &format!("Opening login: {url}"));
            let _ = greg_platform::process::open_url(&url);
        }
        None => AppState::append_log(state, "Login: no auth endpoint configured."),
    }
}

/// Profile card click toggles the session menu (logout arrives with OAuth).
pub fn profile_clicked(state: &Arc<std::sync::Mutex<AppState>>) {
    let guard = state.lock().expect("state");
    let ui = guard.ui();
    ui.set_show_profile_menu(!ui.get_show_profile_menu());
}

/// Session menu selection: navigation mirrors the icon bar, logout ends it.
pub fn profile_menu(state: &Arc<std::sync::Mutex<AppState>>, id: &str) {
    {
        let guard = state.lock().expect("state");
        guard.ui().set_show_profile_menu(false);
    }
    match id {
        "logout" => logout(state),
        "mymods" => on_icon(state, ICON_MODMANAGER),
        "upload" => on_icon(state, ICON_WORKSHOP),
        "settings" => on_icon(state, ICON_SETTINGS),
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// OAuth session flow (`greg://auth/callback` handling).
// ---------------------------------------------------------------------------

/// Persisted session keys (refresh via verify on boot; no refresh endpoint
/// in the ported API, so the access token itself is stored).
const PREF_ACCESS_TOKEN: &str = "greg_access_token";
const PREF_SESSION_ID: &str = "greg_session_id";
const PREF_DISPLAY_NAME: &str = "greg_display_name";
const PREF_EMAIL: &str = "greg_email";
const PREF_AVATAR_URL: &str = "greg_avatar_url";

/// Parsed `greg://` URL.
enum ProtocolUrl {
    /// OAuth callback parameters.
    AuthCallback {
        request_id: String,
        code: String,
        state: String,
        nonce: String,
        signature: String,
    },
    /// Anything else (install intents, mod links): logged, not handled.
    Other(String),
}

/// Parses a `greg://` URL. Accepts both `greg://auth/callback` and the
/// legacy C# `greg://v1/auth/callback` path.
fn parse_protocol_url(url: &str) -> Option<ProtocolUrl> {
    let url = url.trim();
    if !url.starts_with("greg://") {
        return None;
    }
    let (path, query) = match url.split_once('?') {
        Some((p, q)) => (p, q),
        None => (url, ""),
    };
    let path = path.to_lowercase();
    if path == "greg://auth/callback" || path == "greg://v1/auth/callback" {
        let get = |key: &str| {
            query.split('&').find_map(|pair| {
                let (k, v) = pair.split_once('=')?;
                (k == key).then(|| percent_decode(v))
            })
        };
        let oauth = get("requestId").or_else(|| get("request_id"));
        return Some(ProtocolUrl::AuthCallback {
            request_id: oauth.unwrap_or_default(),
            code: get("code").unwrap_or_default(),
            state: get("state").unwrap_or_default(),
            nonce: get("nonce").unwrap_or_default(),
            signature: get("sig").or_else(|| get("signature")).unwrap_or_default(),
        });
    }
    Some(ProtocolUrl::Other(url.to_string()))
}

fn percent_decode(value: &str) -> String {
    let mut out = Vec::with_capacity(value.len());
    let bytes = value.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(h), Some(l)) = (hex(bytes[i + 1]), hex(bytes[i + 2])) {
                out.push(h << 4 | l);
                i += 3;
                continue;
            }
        }
        out.push(if bytes[i] == b'+' { b' ' } else { bytes[i] });
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// Handles one `greg://` URL (own argv or second-instance handoff).
pub fn handle_protocol_url(state: &Arc<std::sync::Mutex<AppState>>, url: &str) {
    let parsed = match parse_protocol_url(url) {
        Some(parsed) => parsed,
        None => return,
    };
    match parsed {
        ProtocolUrl::Other(other) => {
            AppState::append_log(state, &format!("Protocol URL ignored: {other}"));
        }
        ProtocolUrl::AuthCallback {
            request_id,
            code,
            state: oauth_state,
            nonce,
            signature,
        } => {
            if code.trim().is_empty() {
                AppState::append_log(state, "Login callback without code — ignored.");
                return;
            }
            let (expected, formats, bases, tx, runtime_handle) = {
                let guard = state.lock().expect("state");
                (
                    guard.pending_request_id.clone(),
                    login_url_formats(),
                    store_base_urls(&guard),
                    guard.events_tx.clone(),
                    guard.runtime.handle().clone(),
                )
            };
            if let Some(expected) = expected {
                if !request_id.is_empty() && request_id != expected {
                    AppState::append_log(state, "Login callback request mismatch — ignored.");
                    return;
                }
            } else {
                AppState::append_log(
                    state,
                    "Login callback without pending request (restarted mid-flow?) — continuing.",
                );
            }
            let request = greg_core::models::TokenExchangeRequest {
                request_id,
                code,
                state: oauth_state,
                nonce,
                signature,
                redirect_uri: None,
            };
            runtime_handle.spawn(async move {
                let client = greg_modstore::auth_client::AuthApiClient::new(formats, bases);
                let event = match client.exchange_callback_code(request).await {
                    Ok(Some(session)) => UiEvent::AuthSessionEstablished(session),
                    Ok(None) => UiEvent::AuthSessionFailed("empty session response".into()),
                    Err(e) => UiEvent::AuthSessionFailed(e.to_string()),
                };
                let _ = tx.send(event);
            });
        }
    }
}

/// Applies an established session: state, persisted token, profile UI.
pub fn apply_auth_session(state: &mut AppState, session: greg_core::models::ActiveSession) {
    state.pending_request_id = None;
    state.session = Some(session.clone());
    state
        .prefs
        .set_string(PREF_ACCESS_TOKEN, &session.access_token);
    state.prefs.set_string(PREF_SESSION_ID, &session.session_id);
    state
        .prefs
        .set_string(PREF_DISPLAY_NAME, &session.identity.display_name);
    state.prefs.set_string(PREF_EMAIL, &session.identity.email);
    state.prefs.set_string(
        PREF_AVATAR_URL,
        session.identity.avatar_url.as_deref().unwrap_or(""),
    );
    let ui = state.ui();
    ui.set_profile_visible(true);
    ui.set_profile_name(session.identity.display_name.clone().into());
    ui.set_profile_hint(if session.identity.email.is_empty() {
        "Modstore session".into()
    } else {
        session.identity.email.clone().into()
    });
    // Login is only offered when no session exists.
    ui.set_login_visible(false);
    state.push_log(&format!("Logged in as {}.", session.identity.display_name));
}

/// Restores a persisted session on boot (verify, then rebuild identity).
pub fn restore_session(state: &Arc<std::sync::Mutex<AppState>>) {
    let (token, session_id, display_name, email, avatar_url, bases, tx, runtime_handle) = {
        let guard = state.lock().expect("state");
        (
            guard.prefs.get_string(PREF_ACCESS_TOKEN, ""),
            guard.prefs.get_string(PREF_SESSION_ID, ""),
            guard.prefs.get_string(PREF_DISPLAY_NAME, ""),
            guard.prefs.get_string(PREF_EMAIL, ""),
            guard.prefs.get_string(PREF_AVATAR_URL, ""),
            store_base_urls(&guard),
            guard.events_tx.clone(),
            guard.runtime.handle().clone(),
        )
    };
    if token.trim().is_empty() {
        return;
    }
    runtime_handle.spawn(async move {
        let client = greg_modstore::auth_client::BetterAuthClient::new(bases);
        if !client.verify_session(&token).await {
            let _ = tx.send(UiEvent::AuthSessionFailed("stored session expired".into()));
            return;
        }
        let _ = tx.send(UiEvent::AuthSessionEstablished(
            greg_core::models::ActiveSession {
                access_token: token,
                session_id,
                identity: greg_core::models::AccountIdentity {
                    subject_id: String::new(),
                    email,
                    display_name: if display_name.is_empty() {
                        "User".to_string()
                    } else {
                        display_name
                    },
                    avatar_url: if avatar_url.is_empty() {
                        None
                    } else {
                        Some(avatar_url)
                    },
                    roles: vec!["user".to_string()],
                    tenant: String::new(),
                },
            },
        ));
    });
}

/// Ends the session (server call in background, local state cleared now).
pub fn logout(state: &Arc<std::sync::Mutex<AppState>>) {
    let (token, bases, runtime_handle) = {
        let mut guard = state.lock().expect("state");
        let token = guard.session.as_ref().map(|s| s.access_token.clone());
        let bases = store_base_urls(&guard);
        let handle = guard.runtime.handle().clone();
        guard.session = None;
        guard.pending_request_id = None;
        for key in [
            PREF_ACCESS_TOKEN,
            PREF_SESSION_ID,
            PREF_DISPLAY_NAME,
            PREF_EMAIL,
            PREF_AVATAR_URL,
        ] {
            guard.prefs.remove(key);
        }
        let ui = guard.ui();
        ui.set_profile_visible(false);
        ui.set_profile_name("".into());
        ui.set_profile_hint("".into());
        ui.set_show_profile_menu(false);
        ui.set_login_visible(ui.get_modstore_available());
        (token, bases, handle)
    };
    AppState::append_log(state, "Logged out.");
    if let Some(token) = token {
        runtime_handle.spawn(async move {
            let client = greg_modstore::auth_client::AuthApiClient::new(Vec::new(), bases);
            client.end_session(&token).await;
        });
    }
}

/// Handoff file for `greg://` URLs from second instances.
fn handoff_path() -> std::path::PathBuf {
    std::env::temp_dir().join("gregmodmanager-protocol")
}

/// Forwards one protocol URL to the running primary instance.
pub fn forward_to_primary(url: &str) {
    use std::fmt::Write as _;
    let mut content = String::new();
    if let Ok(existing) = std::fs::read_to_string(handoff_path()) {
        content.push_str(&existing);
        if !content.ends_with('\n') {
            content.push('\n');
        }
    }
    let _ = writeln!(content, "{url}");
    let _ = std::fs::write(handoff_path(), content);
    // May carry single-use OAuth codes: owner-only permissions.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(handoff_path(), std::fs::Permissions::from_mode(0o600));
    }
}

/// Picks up second-instance handoffs (called from the main timer with the
/// shared state — handling a URL may spawn work and lock separately).
pub fn drain_handoff(state: &Arc<std::sync::Mutex<AppState>>) {
    let path = handoff_path();
    let content = match std::fs::read_to_string(&path) {
        Ok(content) => content,
        Err(_) => return,
    };
    let _ = std::fs::remove_file(&path);
    for url in content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        AppState::append_log(state, &format!("Protocol handoff: {url}"));
        handle_protocol_url(state, url);
    }
}

// ---------------------------------------------------------------------------
// Custom window chrome (no native frame).
//
// Dragging uses the native `WindowMoveArea` element and resizing the native
// border handling (`resize-border-width`) — both compositor-side, hence
// Wayland-safe. Only min/max/close need Rust.
// ---------------------------------------------------------------------------

/// Minimize the window.
pub fn window_minimize(state: &Arc<std::sync::Mutex<AppState>>) {
    let guard = state.lock().expect("state");
    let ui = guard.ui();
    ui.window().set_minimized(true);
}

/// Toggles maximized state (builtin `maximized` property mirrors it).
pub fn window_maximize(state: &Arc<std::sync::Mutex<AppState>>) {
    let guard = state.lock().expect("state");
    let ui = guard.ui();
    let win = ui.window();
    win.set_maximized(!win.is_maximized());
}

/// Closes the window (ends the session, then quits).
pub fn window_close(state: &Arc<std::sync::Mutex<AppState>>) {
    let guard = state.lock().expect("state");
    guard.file_log.end_session();
    std::process::exit(0);
}

// ---------------------------------------------------------------------------
// Projects.
// ---------------------------------------------------------------------------

/// Refreshes list rows (metadata + sync states).
///
/// Sync-state hashing reads every content byte, so it runs in a background
/// job: this function paints cached states instantly ("?" while unknown)
/// and spawns the hasher only for projects that actually need it.
pub fn refresh_projects(state: &mut AppState) {
    let projects = state.workspace.scan_projects().unwrap_or_default();
    let mut rows: Vec<ProjectRow> = Vec::new();
    let mut cache: Vec<(PathBuf, u64)> = Vec::new();
    let mut pending: Vec<(PathBuf, WorkshopMetadata, String)> = Vec::new();
    for project in &projects {
        let meta = greg_platform::workspace::load_metadata(&project.root_path).unwrap_or_default();
        let desc = state
            .steam
            .build_upload_description(&project.root_path, &meta);
        let sync = match state.project_sync.get(&project.root_path) {
            Some((file_id, cached)) if *file_id == meta.published_file_id => *cached,
            _ => {
                if !greg_core::limits::is_usable_workshop_id(meta.published_file_id) {
                    ProjectSyncState::Unpublished
                } else if meta.last_published_hash.is_empty() {
                    ProjectSyncState::Unknown
                } else {
                    // Expensive (hashes content/) — background job fills the cache.
                    pending.push((project.root_path.clone(), meta.clone(), desc.clone()));
                    ProjectSyncState::Unknown
                }
            }
        };
        let (state_text, state_ok) = match sync {
            ProjectSyncState::Unpublished => ("NEW", false),
            ProjectSyncState::Unknown => ("?", false),
            ProjectSyncState::Synced => ("SYNCED", true),
            ProjectSyncState::Modified => ("UPDATED!", false),
        };
        let title = if meta.title.trim().is_empty() {
            project.name.clone()
        } else {
            format!("{} ({})", meta.title, project.name)
        };
        rows.push(ProjectRow {
            name: project.name.clone().into(),
            title: title.into(),
            meta: format!(
                "v{} · {} · {}",
                meta.version, meta.mod_type, meta.visibility
            )
            .into(),
            state: state_text.into(),
            state_ok,
            root: project.root_path.to_string_lossy().to_string().into(),
            fileid: if meta.published_file_id == 0 {
                SharedString::new()
            } else {
                meta.published_file_id.to_string().into()
            },
            thumb: if meta.preview_image_relative_path.trim().is_empty() {
                slint::Image::default()
            } else {
                thumb_of(Some(
                    &project.root_path.join(&meta.preview_image_relative_path),
                ))
            },
        });
        cache.push((project.root_path.clone(), meta.published_file_id));
    }
    // Search filter lives in the UI; Rust keeps the full set.
    state
        .ui()
        .set_projects(ModelRc::from(Rc::new(VecModel::from(
            rows.into_iter()
                .filter(|r| {
                    let q: String = state.ui().get_project_search().to_string().to_lowercase();
                    q.is_empty()
                        || r.name.to_string().to_lowercase().contains(&q)
                        || r.title.to_string().to_lowercase().contains(&q)
                })
                .collect::<Vec<_>>(),
        ))));
    state.project_rows = cache;
    if !pending.is_empty() && state.sync_job.is_none() {
        spawn_sync_job(state, pending);
    }
}

/// Hashes project content off the UI thread (one record per line:
/// `path \x1f file_id \x1f state` joined by `\x1e`).
fn spawn_sync_job(state: &mut AppState, pending: Vec<(PathBuf, WorkshopMetadata, String)>) {
    let workspace = state.workspace.clone();
    let job = JobHandle::spawn(move |sink| {
        let mut out = Vec::with_capacity(pending.len());
        for (root, meta, desc) in &pending {
            if sink.cancelled().load(std::sync::atomic::Ordering::Relaxed) {
                return JobOutcome {
                    success: false,
                    message: "cancelled".into(),
                };
            }
            let sync = workspace
                .sync_state(root, desc)
                .unwrap_or(ProjectSyncState::Unknown);
            out.push(format!(
                "{}\x1f{}\x1f{}",
                root.display(),
                meta.published_file_id,
                sync_code(sync)
            ));
        }
        JobOutcome {
            success: true,
            message: out.join("\x1e"),
        }
    });
    state.sync_job = Some(job);
}

fn sync_code(sync: ProjectSyncState) -> &'static str {
    match sync {
        ProjectSyncState::Unpublished => "new",
        ProjectSyncState::Unknown => "unknown",
        ProjectSyncState::Synced => "synced",
        ProjectSyncState::Modified => "modified",
    }
}

fn sync_from_code(code: &str) -> Option<ProjectSyncState> {
    match code {
        "new" => Some(ProjectSyncState::Unpublished),
        "unknown" => Some(ProjectSyncState::Unknown),
        "synced" => Some(ProjectSyncState::Synced),
        "modified" => Some(ProjectSyncState::Modified),
        _ => None,
    }
}

/// Applies finished sync hashes to the cache and repaints the rows.
fn drain_sync(state: &mut AppState) {
    let messages = match state.sync_job.as_mut() {
        Some(job) => job.drain(),
        None => return,
    };
    let finished = state.sync_job.as_ref().is_some_and(|job| job.is_finished());
    let mut done: Option<JobOutcome> = None;
    for msg in messages {
        if let JobMessage::Done(outcome) = msg {
            done = Some(outcome);
        }
    }
    let Some(outcome) = done else {
        if finished {
            state.sync_job = None;
        }
        return;
    };
    state.sync_job = None;
    if !outcome.success {
        return;
    }
    for record in outcome.message.split('\x1e') {
        let mut parts = record.splitn(3, '\x1f');
        let (Some(root), Some(file_id), Some(code)) = (parts.next(), parts.next(), parts.next())
        else {
            continue;
        };
        let (Ok(file_id), Some(sync)) = (file_id.parse::<u64>(), sync_from_code(code)) else {
            continue;
        };
        state
            .project_sync
            .insert(PathBuf::from(root), (file_id, sync));
    }
    refresh_projects(state);
}

/// Opens a project in the editor.
pub fn open_project(state: &Arc<std::sync::Mutex<AppState>>, root: &str) {
    let root = PathBuf::from(root);
    let mut guard = state.lock().expect("state");
    let meta = greg_platform::workspace::load_metadata(&root).unwrap_or_default();
    if guard.editor_root.as_ref() != Some(&root) || guard.editor_changelog.trim().is_empty() {
        let resolved = greg_core::docs::resolve_steam_changelog(&root, &meta.version, None);
        if !resolved.text.trim().is_empty() {
            guard.editor_changelog = resolved.text;
            guard
                .ui()
                .set_ed_changelog(guard.editor_changelog.clone().into());
        }
    }
    guard.editor_root = Some(root);
    guard.editor_meta = meta;
    guard.editor_dep_ids = guard.editor_meta.workshop_dependency_ids.clone();
    push_editor(&mut guard);
    run_editor_checks(&mut guard);
    AppState::apply_icon_state(&guard.ui(), ICON_WORKSHOP, "editor");
}

/// Pushes editor state into properties.
fn push_editor(state: &mut AppState) {
    let ui = &state.ui();
    let preview = match state.editor_root.clone() {
        Some(root)
            if !state
                .editor_meta
                .preview_image_relative_path
                .trim()
                .is_empty() =>
        {
            thumb_of(Some(
                &root.join(&state.editor_meta.preview_image_relative_path),
            ))
        }
        _ => slint::Image::default(),
    };
    ui.set_ed_preview_image(preview);
    let meta = &state.editor_meta;
    ui.set_ed_title(meta.title.clone().into());
    ui.set_ed_version(meta.version.clone().into());
    ui.set_ed_desc(meta.description.clone().into());
    ui.set_ed_tags(meta.tags.join(", ").into());
    ui.set_ed_visibility(match meta.visibility.as_str() {
        "FriendsOnly" => 1,
        "Private" => 2,
        _ => 0,
    });
    ui.set_ed_profile(i32::from(meta.native_config_profile == "code"));
    ui.set_ed_modtype(match meta.mod_type.as_str() {
        "MelonloaderPlugin" => 1,
        "Userlib" => 2,
        "DataCenterMod" => 3,
        _ => 0,
    });
    ui.set_ed_needsgreg(meta.needsgreg);
    ui.set_ed_needsml(meta.needs_melon_loader);
    ui.set_ed_changelog(state.editor_changelog.clone().into());
    ui.set_ed_fileid_text(
        if meta.published_file_id == 0 {
            state.tr("Editor_NotPublished")
        } else {
            format!("file id: {}", meta.published_file_id)
        }
        .into(),
    );
    ui.set_ed_project_name(
        state
            .editor_root
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("Workshop item")
            .to_string()
            .into(),
    );
    ui.set_ed_preview_text(meta.preview_image_relative_path.clone().into());
    ui.set_ed_shots(model_of_strings(&meta.additional_previews));
    ui.set_ed_deps(model_of_strings(
        &meta
            .workshop_dependency_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>(),
    ));
    ui.set_ed_dep_add_text("".into());
    update_doc_hints(state);
}

/// Pulls editable properties back into metadata.
fn pull_editor(state: &mut AppState) {
    let ui = &state.ui();
    let meta = &mut state.editor_meta;
    meta.title = ui.get_ed_title().to_string();
    meta.version = ui.get_ed_version().trim().to_string();
    if meta.version.is_empty() {
        meta.version = "1.0.0".to_string();
    }
    meta.description = ui.get_ed_desc().to_string();
    meta.tags = ui
        .get_ed_tags()
        .split(',')
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .take(20)
        .collect();
    meta.visibility = match ui.get_ed_visibility() {
        1 => "FriendsOnly".into(),
        2 => "Private".into(),
        _ => "Public".into(),
    };
    meta.native_config_profile = if ui.get_ed_profile() == 1 {
        "code".into()
    } else {
        "decoration".into()
    };
    meta.mod_type = match ui.get_ed_modtype() {
        1 => "MelonloaderPlugin".into(),
        2 => "Userlib".into(),
        3 => "DataCenterMod".into(),
        _ => "PlacableObject".into(),
    };
    meta.needsgreg = ui.get_ed_needsgreg();
    meta.needs_melon_loader = ui.get_ed_needsml();
    meta.workshop_dependency_ids = state.editor_dep_ids.clone();
    state.editor_changelog = ui.get_ed_changelog().to_string();
}

/// Updates README/CHANGELOG hint labels.
fn update_doc_hints(state: &mut AppState) {
    let (desc_hint, change_hint) = match state.editor_root.clone() {
        Some(root) => {
            let readme = greg_core::docs::find_readme(&root).is_some();
            let desc_hint = if readme {
                let converted = greg_core::docs::resolve_steam_description(&root, "").text;
                l10n::format(
                    &state.lang,
                    "Editor_ReadmeSteamHint",
                    &[
                        &state.editor_meta.description.chars().count().to_string(),
                        &greg_core::limits::MAX_DESCRIPTION_LENGTH.to_string(),
                        &converted.chars().count().to_string(),
                    ],
                )
            } else {
                format!(
                    "{}/{}",
                    state.editor_meta.description.chars().count(),
                    greg_core::limits::MAX_DESCRIPTION_LENGTH
                )
            };
            let manual = if state.editor_changelog.trim().is_empty() {
                None
            } else {
                Some(state.editor_changelog.as_str())
            };
            let resolved =
                greg_core::docs::resolve_steam_changelog(&root, &state.editor_meta.version, manual);
            let change_hint = match resolved.source {
                greg_core::docs::ChangelogSource::FileVersion
                | greg_core::docs::ChangelogSource::FileUnreleased => {
                    let from = resolved.entry_version.as_deref().unwrap_or("?");
                    let from = if from == "Unreleased" {
                        "[Unreleased]".to_string()
                    } else {
                        format!("[{from}]")
                    };
                    l10n::format(
                        &state.lang,
                        "Editor_ChangelogFileHint",
                        &[
                            &from,
                            &greg_core::docs::normalize_version(&state.editor_meta.version)
                                .unwrap_or_default(),
                        ],
                    )
                }
                _ => l10n::get(&state.lang, "Editor_ChangeNotesHint"),
            };
            (desc_hint, change_hint)
        }
        None => (String::new(), String::new()),
    };
    state.ui().set_ed_desc_hint(desc_hint.into());
    state.ui().set_ed_changelog_hint(change_hint.into());
}

/// Runs upload checks into the UI.
fn run_editor_checks(state: &mut AppState) {
    let (checks, ready) = match state.editor_root.clone() {
        Some(root) => {
            let manual = if state.editor_changelog.trim().is_empty() {
                None
            } else {
                Some(state.editor_changelog.as_str())
            };
            let checks = greg_core::upload::check(&state.lang, &root, &state.editor_meta, manual);
            let ready = greg_core::models::is_ready_to_upload(&checks);
            (checks, ready)
        }
        None => (Vec::new(), false),
    };
    let rows: Vec<CheckRow> = checks
        .into_iter()
        .map(|c| CheckRow {
            label: c.label.into(),
            detail: c.detail.into(),
            ok: c.severity != greg_core::models::UploadCheckSeverity::Error,
        })
        .collect();
    state.ui().set_ed_checks(model_of_rows(rows));
    state.ui().set_ed_ready(ready);
    state.ui().set_ed_readiness_text(
        if ready {
            state.tr("Editor_ReadyToUpload")
        } else {
            state.tr("Editor_IssuesFound")
        }
        .into(),
    );
    update_doc_hints(state);
}

/// Editor field changed (pull + re-check).
pub fn editor_changed(state: &Arc<std::sync::Mutex<AppState>>) {
    let mut guard = state.lock().expect("state");
    // Refresh an untouched changelog when the version changed.
    let version: String = guard.ui().get_ed_version().to_string();
    if guard.editor_changelog.trim().is_empty() {
        if let Some(root) = guard.editor_root.clone() {
            let resolved = greg_core::docs::resolve_steam_changelog(&root, version.trim(), None);
            if !resolved.text.trim().is_empty() {
                guard.editor_changelog = resolved.text.clone();
                guard.ui().set_ed_changelog(resolved.text.into());
            }
        }
    }
    pull_editor(&mut guard);
    run_editor_checks(&mut guard);
}

/// Saves editor metadata.
pub fn editor_save(state: &Arc<std::sync::Mutex<AppState>>) {
    let mut guard = state.lock().expect("state");
    pull_editor(&mut guard);
    if let Some(root) = guard.editor_root.clone() {
        match greg_platform::workspace::save_metadata(&root, &guard.editor_meta) {
            Ok(()) => AppState::append_log(state, &format!("Saved {}", root.display())),
            Err(e) => AppState::append_log(state, &format!("Save failed: {e}")),
        }
    }
    run_editor_checks(&mut guard);
    refresh_projects(&mut guard);
}

/// Starts the background publish.
pub fn editor_publish(state: &Arc<std::sync::Mutex<AppState>>) {
    // Save first (separate lock scope).
    editor_save(state);
    let mut guard = state.lock().expect("state");
    let ui = guard.ui();
    let Some(root) = guard.editor_root.clone() else {
        ui.set_ed_status("No project open.".into());
        return;
    };
    if guard.publish_job.is_some() {
        ui.set_ed_status("Upload already running…".into());
        return;
    }
    if !ui.get_ed_ready() {
        // Never die silently: list the blocking checks in the status line.
        let blocking: Vec<String> = ui
            .get_ed_checks()
            .iter()
            .filter(|row| !row.ok)
            .map(|row| format!("{} — {}", row.label, row.detail))
            .collect();
        ui.set_ed_status(if blocking.is_empty() {
            "Not ready to upload.".into()
        } else {
            format!("Fix errors before uploading: {}", blocking.join(" · ")).into()
        });
        return;
    }
    let mut meta = guard.editor_meta.clone();
    let manual = if guard.editor_changelog.trim().is_empty() {
        None
    } else {
        Some(guard.editor_changelog.clone())
    };
    let steam = Arc::clone(&guard.steam);
    guard.ui().set_ed_publishing(true);
    guard.ui().set_ed_progress(0.0);
    guard.ui().set_ed_status("Uploading…".into());
    AppState::append_log(state, &format!("Upload started: {}", root.display()));
    let job = JobHandle::spawn(move |sink| {
        let outcome = steam.publish(
            &root,
            &mut meta,
            manual.as_deref(),
            &|p| sink.progress(p),
            &|s| {
                sink.status(s.to_string());
                sink.log(s.to_string());
            },
            sink.cancelled(),
        );
        // File-log the lifecycle (the UI log alone is lost on a hang/kill).
        if outcome.success {
            let _ = greg_platform::workspace::save_metadata(&root, &meta);
            greg_platform::filelog::FileLog::shared().info(&format!(
                "Upload finished: {} -> file id {}",
                root.display(),
                outcome.published_file_id
            ));
        } else {
            greg_platform::filelog::FileLog::shared().info(&format!(
                "Upload failed: {}: {}",
                root.display(),
                outcome.message
            ));
        }
        crate::worker::JobOutcome {
            success: outcome.success,
            message: if outcome.success {
                format!("Published file id {}", outcome.published_file_id)
            } else {
                outcome.message
            },
        }
    });
    guard.publish_job = Some(job);
}

/// Cancels the running publish.
pub fn editor_cancel_publish(state: &Arc<std::sync::Mutex<AppState>>) {
    let guard = state.lock().expect("state");
    if let Some(job) = guard.publish_job.as_ref() {
        job.cancel();
    }
}

fn drain_publish(state: &mut AppState) {
    let mut done = false;
    let mut finished = false;
    let messages = match state.publish_job.as_mut() {
        Some(job) => {
            let messages = job.drain();
            finished = job.is_finished();
            messages
        }
        None => Vec::new(),
    };
    let mut log_lines: Vec<String> = state
        .ui()
        .get_ed_log()
        .iter()
        .map(|s| s.to_string())
        .collect();
    for msg in messages {
        match msg {
            JobMessage::Progress(p) => state.ui().set_ed_progress(p),
            JobMessage::Status(s) => state.ui().set_ed_status(s.into()),
            JobMessage::Log(line) => {
                log_lines.push(line);
                if log_lines.len() > 300 {
                    log_lines.remove(0);
                }
            }
            JobMessage::Done(outcome) => {
                done = outcome.success;
                state.ui().set_ed_status(outcome.message.into());
            }
        }
    }
    state.ui().set_ed_log(model_of_strings(&log_lines));
    if done || finished {
        state.publish_job = None;
        state.ui().set_ed_publishing(false);
        if done {
            state.ui().set_ed_progress(1.0);
        }
        // Reload metadata + projects + checks (worker persisted id/hash).
        if let Some(root) = state.editor_root.clone() {
            state.editor_meta = greg_platform::workspace::load_metadata(&root).unwrap_or_default();
            state.editor_dep_ids = state.editor_meta.workshop_dependency_ids.clone();
            push_editor(state);
        }
        refresh_projects(state);
        run_editor_checks(state);
    }
}

/// Creates a project from the form, then opens it.
pub fn create_project(state: &Arc<std::sync::Mutex<AppState>>) {
    let (folder, template) = {
        let guard = state.lock().expect("state");
        (
            guard.ui().get_new_folder().to_string(),
            guard.ui().get_new_template(),
        )
    };
    if folder.trim().is_empty() {
        return;
    }
    let kind = match template {
        1 => greg_loader::content::WorkshopTemplateKind::ModdedMelonLoader,
        2 => greg_loader::content::WorkshopTemplateKind::ModdedGregCore,
        3 => greg_loader::content::WorkshopTemplateKind::UxmlUiOverride,
        4 => greg_loader::content::WorkshopTemplateKind::Standalone3DModel,
        5 => greg_loader::content::WorkshopTemplateKind::StandaloneTexture,
        6 => greg_loader::content::WorkshopTemplateKind::StandaloneAudio,
        _ => greg_loader::content::WorkshopTemplateKind::VanillaObjectDecoration,
    };
    let guard = state.lock().expect("state");
    let root = guard.workspace.root().to_path_buf();
    match greg_loader::templates::scaffold(&root, folder.trim(), kind, folder.trim()) {
        Ok((project_root, meta)) => {
            if greg_platform::workspace::save_metadata(&project_root, &meta).is_ok() {
                guard.ui().set_new_folder("".into());
                drop(guard);
                refresh_projects_locked(state);
                open_project(state, &project_root.to_string_lossy());
            }
        }
        Err(e) => AppState::append_log(state, &format!("Create failed: {e}")),
    }
}

fn refresh_projects_locked(state: &Arc<std::sync::Mutex<AppState>>) {
    if let Ok(mut guard) = state.lock() {
        refresh_projects(&mut guard);
    }
}

/// Opens a Steam store page for a file id string.
pub fn view_on_steam(state: &Arc<std::sync::Mutex<AppState>>, file_id: &str) {
    if let Ok(id) = file_id.parse::<u64>() {
        if id != 0 {
            state.lock().expect("state").steam.open_in_browser(id);
        }
    }
}

// ---------------------------------------------------------------------------
// Browse.
// ---------------------------------------------------------------------------

fn browse_sort(index: i32) -> greg_steam::models::WorkshopSortMode {
    use greg_steam::models::WorkshopSortMode as S;
    match index {
        1 => S::Newest,
        2 => S::TopRated,
        3 => S::Trending,
        4 => S::MostSubscribed,
        5 => S::TitleAsc,
        _ => S::LastUpdated,
    }
}

/// Starts a page/detail fetch job.
pub fn browse_go(state: &Arc<std::sync::Mutex<AppState>>, detail_id: Option<u64>) {
    let mut guard = state.lock().expect("state");
    if guard.browse_job.is_some() {
        return;
    }
    let steam = Arc::clone(&guard.steam);
    let list = guard.ui().get_br_list();
    let sort = browse_sort(guard.ui().get_br_sort());
    let tag = {
        let t = guard.ui().get_br_tag().to_string();
        t.trim().to_string()
    };
    let tag = if tag.is_empty() { None } else { Some(tag) };
    let search = guard.ui().get_br_search().to_string();
    let mut page = guard.ui().get_br_page();
    if page < 1 {
        page = 1;
        guard.ui().set_br_page(1);
    }
    guard.ui().set_br_busy(true);
    guard.ui().set_br_status("Loading…".into());
    let job = JobHandle::spawn(move |sink| {
        let cancel = sink.cancelled().clone();
        let outcome = if let Some(id) = detail_id {
            match steam.item_details(id, &cancel) {
                Ok(detail) => {
                    let payload = serde_json::to_string(&detail).unwrap_or_default();
                    JobOutcome {
                        success: true,
                        message: format!("FULL:{payload}"),
                    }
                }
                Err(e) => JobOutcome {
                    success: false,
                    message: e.to_string(),
                },
            }
        } else {
            let result = if !search.trim().is_empty() {
                steam.search(search.trim(), page as u32, &cancel)
            } else {
                match list {
                    1 => steam.subscribed(page as u32, &cancel),
                    2 => steam.favorited(page as u32, &cancel),
                    _ => steam.browse(page as u32, sort, tag.as_deref(), &cancel),
                }
            };
            // Stash items through the tick via message payload.
            let payload = serde_json::to_string(&result.items).unwrap_or_default();
            JobOutcome {
                success: true,
                message: format!("PAGE:{}:{payload}", result.total_results),
            }
        };
        let _ = sink;
        outcome
    });
    guard.browse_job = Some(job);
}

/// Re-fetches page one after a list/sort change.
pub fn browse_filter_changed(state: &Arc<std::sync::Mutex<AppState>>) {
    {
        let guard = state.lock().expect("state");
        guard.ui().set_br_page(1);
    }
    browse_go(state, None);
}

/// Cancels the browse job.
pub fn browse_cancel(state: &Arc<std::sync::Mutex<AppState>>) {
    let guard = state.lock().expect("state");
    if let Some(job) = guard.browse_job.as_ref() {
        job.cancel();
    }
}

/// Selects an item (detail fetch).
pub fn browse_select(state: &Arc<std::sync::Mutex<AppState>>, id: &str) {
    if let Ok(parsed) = id.parse::<u64>() {
        if parsed != 0 {
            browse_go(state, Some(parsed));
        }
    }
}

/// Page navigation.
pub fn browse_page(state: &Arc<std::sync::Mutex<AppState>>, delta: i32) {
    let page = {
        let guard = state.lock().expect("state");
        (guard.ui().get_br_page() + delta).max(1)
    };
    {
        let guard = state.lock().expect("state");
        guard.ui().set_br_page(page);
    }
    browse_go(state, None);
}

/// Toggles subscription for the open detail.
pub fn browse_subscribe_toggle(state: &Arc<std::sync::Mutex<AppState>>) {
    let (id, subscribed) = {
        let guard = state.lock().expect("state");
        (guard.browse_detail_id, guard.ui().get_br_d_subscribed())
    };
    let Some(id) = id else { return };
    let guard = state.lock().expect("state");
    let result = if subscribed {
        guard.steam.unsubscribe(id)
    } else {
        guard.steam.subscribe(id)
    };
    match result {
        Ok(()) => {
            AppState::append_log(state, &format!("toggled subscription {id}"));
            drop(guard);
            browse_go(state, Some(id));
        }
        Err(e) => AppState::append_log(state, &format!("subscription failed: {e}")),
    }
}

/// Imports the open detail into the workspace.
pub fn browse_import(state: &Arc<std::sync::Mutex<AppState>>) {
    let (id, ws, steam) = {
        let guard = state.lock().expect("state");
        match guard.browse_detail_id {
            Some(id) => (id, guard.workspace.clone(), Arc::clone(&guard.steam)),
            None => return,
        }
    };
    {
        let guard = state.lock().expect("state");
        guard.ui().set_br_status("Importing…".into());
    }
    let mut guard = state.lock().expect("state");
    if guard.browse_job.is_some() {
        return;
    }
    let job = JobHandle::spawn(move |sink| {
        let cancel = sink.cancelled().clone();
        let outcome = steam.import_to_workspace(
            &ws,
            id,
            None,
            &|s| sink.log(s.to_string()),
            &|p| sink.progress(p),
            &cancel,
        );
        JobOutcome {
            success: outcome.success,
            message: format!("IMPORT:{}", outcome.message),
        }
    });
    guard.browse_job = Some(job);
}

fn drain_browse(state: &mut AppState) {
    let mut done: Option<JobOutcome> = None;
    let mut finished = false;
    let messages = match state.browse_job.as_mut() {
        Some(job) => {
            let messages = job.drain();
            finished = job.is_finished();
            messages
        }
        None => Vec::new(),
    };
    for msg in messages {
        if let JobMessage::Done(outcome) = msg {
            done = Some(outcome);
        }
    }
    let Some(outcome) = done else {
        if finished {
            state.browse_job = None;
            state.ui().set_br_busy(false);
        }
        return;
    };
    state.browse_job = None;
    state.ui().set_br_busy(false);
    if !outcome.success {
        state.ui().set_br_status(outcome.message.into());
        return;
    }
    if let Some(payload) = outcome.message.strip_prefix("FULL:") {
        match serde_json::from_str::<greg_steam::models::ItemDetail>(payload) {
            Ok(detail) => {
                state.browse_detail_id = Some(detail.published_file_id);
                state.ui().set_br_has_detail(true);
                state.ui().set_br_d_title(detail.title.into());
                state.ui().set_br_d_meta(
                    format!(
                        "by {} · ★ {:.1} · {} bytes",
                        detail.owner_name, detail.score, detail.size_bytes
                    )
                    .into(),
                );
                state.ui().set_br_d_tags(detail.tags.join(", ").into());
                state.ui().set_br_d_desc(detail.description.into());
                state
                    .ui()
                    .set_br_d_deps(detail.dependency_hint_compact.into());
                state.ui().set_br_d_subscribed(detail.is_subscribed);
                state.ui().set_br_status("".into());
            }
            Err(e) => state
                .ui()
                .set_br_status(format!("bad detail payload: {e}").into()),
        }
        return;
    }
    if let Some(rest) = outcome.message.strip_prefix("PAGE:") {
        if let Some((total, payload)) = rest.split_once(':') {
            if let Ok(total) = total.parse::<i32>() {
                state.ui().set_br_total(total);
            }
            if let Ok(items) = serde_json::from_str::<Vec<greg_steam::models::BrowseItem>>(payload)
            {
                state.browse_items = items
                    .iter()
                    .map(|i| (i.published_file_id, i.title.clone()))
                    .collect();
                queue_thumbs(
                    state,
                    items
                        .iter()
                        .map(|i| (i.published_file_id.to_string(), i.preview_image_url.clone()))
                        .collect(),
                );
                let rows: Vec<BrowseRow> = items
                    .into_iter()
                    .map(|i| BrowseRow {
                        id: i.published_file_id.to_string().into(),
                        title: i.title.into(),
                        score: format!("★ {:.1}", i.score).into(),
                        thumb: slint::Image::default(),
                    })
                    .collect();
                state
                    .ui()
                    .set_br_items(ModelRc::from(Rc::new(VecModel::from(rows))));
                state.ui().set_br_status("".into());
            }
        }
        return;
    }
    if let Some(message) = outcome.message.strip_prefix("IMPORT:") {
        state.ui().set_br_status(message.into());
        refresh_projects(state);
    }
}

// ---------------------------------------------------------------------------
// Modstore.
// ---------------------------------------------------------------------------

/// Refreshes the catalog.
pub fn store_refresh(state: &Arc<std::sync::Mutex<AppState>>) {
    AppState::request_probe(state);
    let mut guard = state.lock().expect("state");
    if guard.store_job.is_some() || !guard.ui().get_modstore_available() {
        return;
    }
    let urls = store_base_urls(&guard);
    let token = guard.session.as_ref().map(|s| s.access_token.clone());
    let rt = guard.runtime.handle().clone();
    guard.ui().set_br_busy(false);
    guard.ui().set_st_busy(true);
    guard.ui().set_st_status("Loading catalog…".into());
    let job = JobHandle::spawn(move |sink| {
        let outcome =
            match greg_modstore::client::ModStoreClient::new(urls).map(|client| match token {
                Some(token) => client.with_token(token),
                None => client,
            }) {
                Ok(client) => match rt.block_on(client.catalog()) {
                    Ok(response) => {
                        let payload = serde_json::to_string(&response.mods).unwrap_or_default();
                        JobOutcome {
                            success: true,
                            message: format!("CATALOG:{payload}"),
                        }
                    }
                    Err(e) => {
                        sink.log(e.to_string());
                        JobOutcome {
                            success: false,
                            message: e.to_string(),
                        }
                    }
                },
                Err(e) => JobOutcome {
                    success: false,
                    message: e.to_string(),
                },
            };
        let _ = sink;
        outcome
    });
    guard.store_job = Some(job);
}

/// Checks for updates.
pub fn store_check_updates(state: &Arc<std::sync::Mutex<AppState>>) {
    let mut guard = state.lock().expect("state");
    if guard.store_job.is_some() || !guard.ui().get_modstore_available() {
        return;
    }
    let urls = store_base_urls(&guard);
    let token = guard.session.as_ref().map(|s| s.access_token.clone());
    let rt = guard.runtime.handle().clone();
    let game_root = game_root_of(&guard);
    guard.ui().set_st_busy(true);
    guard.ui().set_st_status("Checking for updates…".into());
    let job = JobHandle::spawn(move |sink| {
        let manifest = greg_modstore::install::load_manifest(&game_root);
        let installed = manifest
            .installed
            .into_iter()
            .map(|e| greg_modstore::models::ModStoreInstalledVersion {
                mod_id: e.mod_id,
                version: e.version,
            })
            .collect();
        let outcome =
            match greg_modstore::client::ModStoreClient::new(urls).map(|client| match token {
                Some(token) => client.with_token(token),
                None => client,
            }) {
                Ok(client) => match rt.block_on(client.check_updates(installed)) {
                    Ok(response) => {
                        let payload = serde_json::to_string(&response.updates).unwrap_or_default();
                        JobOutcome {
                            success: true,
                            message: format!("UPDATES:{payload}"),
                        }
                    }
                    Err(e) => {
                        sink.log(e.to_string());
                        JobOutcome {
                            success: false,
                            message: e.to_string(),
                        }
                    }
                },
                Err(e) => JobOutcome {
                    success: false,
                    message: e.to_string(),
                },
            };
        let _ = sink;
        outcome
    });
    guard.store_job = Some(job);
}

/// Cancels the store job.
pub fn store_cancel(state: &Arc<std::sync::Mutex<AppState>>) {
    let guard = state.lock().expect("state");
    if let Some(job) = guard.store_job.as_ref() {
        job.cancel();
    }
}

/// Installs a catalog item / update by list index.
pub fn store_install(state: &Arc<std::sync::Mutex<AppState>>, index: i32, is_update: bool) {
    let mut guard = state.lock().expect("state");
    if guard.store_job.is_some() || !guard.ui().get_modstore_available() {
        return;
    }
    let urls = store_base_urls(&guard);
    let rt = guard.runtime.handle().clone();
    let game_root = game_root_of(&guard);
    let target = if is_update {
        guard
            .store_updates
            .get(index as usize)
            .map(|u| StoreTarget::Update(u.clone()))
    } else {
        guard
            .store_items
            .get(index as usize)
            .map(|i| StoreTarget::Item(i.clone()))
    };
    let Some(target) = target else { return };
    guard.ui().set_st_busy(true);
    guard.ui().set_st_progress(0.0);
    guard.ui().set_st_status("Installing…".into());
    let job = JobHandle::spawn(move |sink| {
        let installer = greg_modstore::install::ModStoreInstaller::new(&urls);
        let cancelled = sink.cancelled().clone();
        let progress_sink = sink.clone();
        let progress = move |p: u8| progress_sink.progress(f32::from(p) / 100.0);
        let result = rt.block_on(async move {
            match target {
                StoreTarget::Item(item) => {
                    installer
                        .install_item(&item, &game_root, &progress, &cancelled)
                        .await
                }
                StoreTarget::Update(update) => {
                    installer
                        .install_update(&update, &game_root, &progress, &cancelled)
                        .await
                }
            }
        });
        JobOutcome {
            success: result.success,
            message: format!("INSTALLED:{}", result.message),
        }
    });
    guard.store_job = Some(job);
}

#[derive(Debug, Clone)]
enum StoreTarget {
    Item(greg_modstore::models::ModStoreCatalogItem),
    Update(greg_modstore::models::ModStoreUpdate),
}

fn store_base_urls(state: &AppState) -> Vec<String> {
    let configured = state.prefs.get_string("ModStoreBaseUrl", "");
    if !configured.trim().is_empty() {
        return vec![configured.trim().to_string()];
    }
    vec![greg_core::models::settings::modstore_api_url()]
}

fn game_root_of(state: &AppState) -> PathBuf {
    let configured = state.prefs.get_string("GameRootPath", "");
    if !configured.trim().is_empty() {
        return PathBuf::from(configured);
    }
    greg_loader::health::ModDependencyService::new(None)
        .game_root()
        .unwrap_or_default()
}

fn drain_store(state: &mut AppState) {
    let mut done: Option<JobOutcome> = None;
    let mut finished = false;
    let mut progress = None;
    let mut status_lines: Vec<String> = Vec::new();
    let messages = match state.store_job.as_mut() {
        Some(job) => {
            let messages = job.drain();
            finished = job.is_finished();
            messages
        }
        None => Vec::new(),
    };
    for msg in messages {
        match msg {
            JobMessage::Progress(p) => progress = Some(p),
            JobMessage::Status(s) => status_lines.push(s),
            JobMessage::Log(line) => AppState::append_log_ref(state, &line),
            JobMessage::Done(outcome) => done = Some(outcome),
        }
    }
    if let Some(p) = progress {
        state.ui().set_st_progress(p);
    }
    if let Some(last) = status_lines.last() {
        state.ui().set_st_status(last.clone().into());
    }
    let Some(outcome) = done else {
        if finished {
            state.store_job = None;
            state.ui().set_st_busy(false);
            state.ui().set_st_progress(0.0);
        }
        return;
    };
    state.store_job = None;
    state.ui().set_st_busy(false);
    state.ui().set_st_progress(0.0);
    if !outcome.success {
        state.ui().set_st_status(outcome.message.into());
        return;
    }
    if let Some(payload) = outcome.message.strip_prefix("CATALOG:") {
        if let Ok(items) =
            serde_json::from_str::<Vec<greg_modstore::models::ModStoreCatalogItem>>(payload)
        {
            state.store_items = items.clone();
            queue_thumbs(
                state,
                items
                    .iter()
                    .map(|i| (i.slug.clone(), i.image_url.clone().unwrap_or_default()))
                    .collect(),
            );
            let rows: Vec<StoreRow> = items
                .into_iter()
                .map(|i| StoreRow {
                    title: i.title.into(),
                    meta: format!("by {} · v{} · {}", i.author, i.release.version, i.category)
                        .into(),
                    slug: i.slug.into(),
                    thumb: slint::Image::default(),
                })
                .collect();
            // Search filter applies live in tick via the stored query.
            let q = state.ui().get_st_search().to_string().to_lowercase();
            let rows: Vec<StoreRow> = rows
                .into_iter()
                .filter(|r| q.is_empty() || r.title.to_string().to_lowercase().contains(&q))
                .collect();
            state
                .ui()
                .set_st_items(ModelRc::from(Rc::new(VecModel::from(rows))));
            state
                .ui()
                .set_st_status(format!("Catalog: {} mods", state.store_items.len()).into());
        }
        return;
    }
    if let Some(payload) = outcome.message.strip_prefix("UPDATES:") {
        if let Ok(updates) =
            serde_json::from_str::<Vec<greg_modstore::models::ModStoreUpdate>>(payload)
        {
            state.store_updates = updates.clone();
            let rows: Vec<UpdateRow> = updates
                .into_iter()
                .map(|u| UpdateRow {
                    title: u.title.into(),
                    versions: format!("{} → {}", u.installed_version, u.release.version).into(),
                })
                .collect();
            state
                .ui()
                .set_st_updates(ModelRc::from(Rc::new(VecModel::from(rows))));
            state
                .ui()
                .set_st_status(format!("Updates: {}", state.store_updates.len()).into());
        }
        return;
    }
    if let Some(message) = outcome.message.strip_prefix("INSTALLED:") {
        state.ui().set_st_status(message.into());
        refresh_projects(state);
        // Installs land in the game folders — rescan local content too.
        local_refresh(state);
    }
}

// ---------------------------------------------------------------------------
// Local content (My Mods / My Plugins / My Libs).
// ---------------------------------------------------------------------------

fn local_kind_of(state: &AppState) -> greg_loader::local_content::LocalContentKind {
    use greg_loader::local_content::LocalContentKind as K;
    match state.ui().get_current_page().as_str() {
        "myplugins" => K::Plugins,
        "mylibs" => K::Libs,
        _ => K::Mods,
    }
}

/// Refreshes the local list for the active page + records its signature.
pub fn local_refresh(state: &mut AppState) {
    let kind = local_kind_of(state);
    let root = game_root_of(state);
    state.local_sig = if root.is_dir() {
        greg_loader::local_content::signature(&root, kind)
    } else {
        String::new()
    };
    state.ui().set_local_title(kind.title().into());
    if !root.is_dir() {
        state.local_entries.clear();
        state
            .ui()
            .set_local_rows(model_of_rows(Vec::<LocalRow>::new()));
        state
            .ui()
            .set_local_status("Game folder not found — set the game root in Settings.".into());
        return;
    }
    let entries = greg_loader::local_content::scan(&root, kind);
    let q = state.ui().get_local_search().to_string().to_lowercase();
    let rows: Vec<LocalRow> = entries
        .iter()
        .enumerate()
        .filter(|(_, e)| q.is_empty() || e.name.to_lowercase().contains(&q))
        .map(|(i, e)| LocalRow {
            name: e.name.clone().into(),
            detail: e.detail.clone().into(),
            enabled: e.enabled,
            index: i as i32,
            thumb: thumb_of(greg_loader::local_content::sibling_preview(&e.path).as_deref()),
        })
        .collect();
    // NOTE: row indices address the full cache; toggle/remove resolve
    // through `local_filtered`, so both stay consistent.
    state.local_entries = entries;
    state.ui().set_local_rows(model_of_rows(rows));
    state.ui().set_local_status(
        format!(
            "{} entries in {}",
            state.local_entries.len(),
            root.display()
        )
        .into(),
    );
}

/// True on My Mods / My Plugins / My Libs.
fn is_local_page(state: &AppState) -> bool {
    matches!(
        state.ui().get_current_page().as_str(),
        "mymods" | "myplugins" | "mylibs"
    )
}

/// Rescans the visible local page when the folder changed underneath us
/// (manual drops, external installers, toggles outside the app).
fn auto_refresh_local(state: &mut AppState) {
    if !is_local_page(state) {
        return;
    }
    let kind = local_kind_of(state);
    let root = game_root_of(state);
    if !root.is_dir() {
        return;
    }
    let signature = greg_loader::local_content::signature(&root, kind);
    if signature != state.local_sig {
        local_refresh(state);
    }
}

/// Filtered cache indices for the current search text.
fn local_filtered(state: &AppState) -> Vec<usize> {
    let q = state.ui().get_local_search().to_string().to_lowercase();
    state
        .local_entries
        .iter()
        .enumerate()
        .filter(|(_, e)| q.is_empty() || e.name.to_lowercase().contains(&q))
        .map(|(i, _)| i)
        .collect()
}

/// Toggles an entry by list index.
pub fn local_toggle(state: &Arc<std::sync::Mutex<AppState>>, index: i32) {
    if guard_game_running(state) {
        return;
    }
    let entry = {
        let guard = state.lock().expect("state");
        let filtered = local_filtered(&guard);
        match filtered.get(index as usize) {
            Some(&i) => guard.local_entries.get(i).cloned(),
            None => None,
        }
    };
    let Some(entry) = entry else { return };
    match greg_loader::local_content::set_enabled(&entry, !entry.enabled) {
        Ok(_) => {
            AppState::append_log(
                state,
                &format!(
                    "{} {}",
                    if entry.enabled { "Disabled" } else { "Enabled" },
                    entry.name
                ),
            );
            local_refresh(&mut state.lock().expect("state"));
        }
        Err(e) => AppState::append_log(state, &format!("Toggle failed: {e}")),
    }
}

/// Opens the confirm dialog for a removal.
pub fn local_remove(state: &Arc<std::sync::Mutex<AppState>>, index: i32) {
    if guard_game_running(state) {
        return;
    }
    let mut guard = state.lock().expect("state");
    let filtered = local_filtered(&guard);
    let entry = filtered
        .get(index as usize)
        .and_then(|&i| guard.local_entries.get(i).cloned());
    match entry {
        Some(entry) => {
            guard.pending_remove = Some(entry.path.clone());
            guard
                .ui()
                .set_confirm_text(format!("Delete {}?", entry.name).into());
            guard.ui().set_show_confirm(true);
        }
        None => guard.ui().set_show_confirm(false),
    }
}

/// Opens the scanned folder of the active kind.
pub fn local_open_folder(state: &Arc<std::sync::Mutex<AppState>>) {
    let guard = state.lock().expect("state");
    let root = game_root_of(&guard);
    let kind = local_kind_of(&guard);
    let dir = match kind {
        greg_loader::local_content::LocalContentKind::Mods => root.join("Mods"),
        greg_loader::local_content::LocalContentKind::Plugins => root.join("Plugins"),
        greg_loader::local_content::LocalContentKind::Libs => {
            root.join("Plugins").join("Dependencies")
        }
    };
    let target = if dir.is_dir() { dir } else { root };
    let _ = greg_platform::process::open_folder(&target);
}

/// Confirm dialog: delete the pending file.
pub fn confirm_yes(state: &Arc<std::sync::Mutex<AppState>>) {
    if guard_game_running(state) {
        let mut guard = state.lock().expect("state");
        guard.ui().set_show_confirm(false);
        guard.pending_remove = None;
        return;
    }
    let pending = {
        let mut guard = state.lock().expect("state");
        guard.ui().set_show_confirm(false);
        guard.pending_remove.take()
    };
    if let Some(path) = pending {
        let entry = greg_loader::local_content::LocalContentEntry {
            name: path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?")
                .to_string(),
            detail: String::new(),
            enabled: true,
            path,
        };
        match greg_loader::local_content::remove(&entry) {
            Ok(()) => AppState::append_log(state, &format!("Removed {}", entry.name)),
            Err(e) => AppState::append_log(state, &format!("Remove failed: {e}")),
        }
        local_refresh(&mut state.lock().expect("state"));
    }
}

/// Confirm dialog: dismiss.
pub fn confirm_no(state: &Arc<std::sync::Mutex<AppState>>) {
    let mut guard = state.lock().expect("state");
    guard.pending_remove = None;
    guard.ui().set_show_confirm(false);
}

/// Bug dialog: report against this app (prefilled tracker link).
pub fn bug_open_manager(state: &Arc<std::sync::Mutex<AppState>>) {
    let title = format!("Bug report (gregModmanager v{})", env!("CARGO_PKG_VERSION"));
    let url = format!("{}?title={}", BUG_REPORT_URL, percent_encode_bug(&title));
    {
        let guard = state.lock().expect("state");
        guard.ui().set_show_bug_dialog(false);
    }
    if greg_platform::process::open_url(&url).is_err() {
        AppState::append_log(state, "Report a Bug: could not open the browser.");
    } else {
        AppState::append_log(state, "Report a Bug: opened the app tracker.");
    }
}

/// Bug dialog: all other Greg mods & programs (org trackers).
pub fn bug_open_org(state: &Arc<std::sync::Mutex<AppState>>) {
    {
        let guard = state.lock().expect("state");
        guard.ui().set_show_bug_dialog(false);
    }
    if greg_platform::process::open_url(BUG_REPORT_ORG_URL).is_err() {
        AppState::append_log(state, "Report a Bug: could not open the browser.");
    } else {
        AppState::append_log(state, "Report a Bug: opened the Greg org page.");
    }
}

/// Bug dialog: dismiss.
pub fn bug_close(state: &Arc<std::sync::Mutex<AppState>>) {
    let guard = state.lock().expect("state");
    guard.ui().set_show_bug_dialog(false);
}

fn percent_encode_bug(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for b in value.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~' | b'(' | b')') {
            out.push(b as char);
        } else if b == b' ' {
            out.push('+');
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Problems inbox.
// ---------------------------------------------------------------------------

/// One aggregated problem with an optional jump action.
struct Problem {
    severity: &'static str,
    source: String,
    message: String,
    action: Option<(String, String, String)>,
}

/// Recomputes the problems inbox for the current state.
pub fn refresh_problems(state: &Arc<std::sync::Mutex<AppState>>) {
    let mut guard = state.lock().expect("state");
    let mut problems: Vec<Problem> = Vec::new();

    // Game root.
    let game_root = game_root_of(&guard);
    if !game_root.is_dir() {
        problems.push(Problem {
            severity: "error",
            source: "Game".into(),
            message: "Game folder not found — set the game root in Settings.".into(),
            action: Some(("settings".into(), String::new(), "Open Settings".into())),
        });
    }

    // Steam + GregApi status.
    let steam_hint = match guard.backend.status() {
        greg_steam::backend::BackendStatus::Available { .. } => None,
        greg_steam::backend::BackendStatus::Unavailable { hint } => Some(hint),
    };
    if let Some(hint) = steam_hint {
        problems.push(Problem {
            severity: "warning",
            source: "Steam".into(),
            message: if hint.is_empty() {
                "Steam is disconnected — Workshop features are limited.".into()
            } else {
                format!("Steam is disconnected ({hint}) — Workshop features are limited.")
            },
            action: None,
        });
    }
    if !guard.ui().get_gregapi_ok() {
        problems.push(Problem {
            severity: "warning",
            source: "GregApi".into(),
            message: "gregAPI is offline — Modstore and login are unavailable.".into(),
            action: None,
        });
    }

    // Tracked game session.
    {
        let mut guard = state.lock().expect("state");
        if let Some(session) = guard.game_session.as_mut() {
            if session.is_running() {
                let pid = session.pid();
                let mode = session.mode().label().to_string();
                problems.push(Problem {
                    severity: "info",
                    source: "Game".into(),
                    message: l10n::format(&guard.lang, "Game_StatusRunning", &[&mode, &pid]),
                    action: Some(("stop-game".into(), String::new(), "Stop game".into())),
                });
            }
        }
    }
    // Stale game processes (started outside the manager, Windows zombies).
    if state.lock().expect("state").game_session.is_none() {
        let stale = greg_loader::launch::stale_game_pids();
        if let Some(pid) = stale.first() {
            let guard = state.lock().expect("state");
            problems.push(Problem {
                severity: "warning",
                source: "Game".into(),
                message: l10n::format(&guard.lang, "Game_StaleRunning", &[pid]),
                action: Some(("stop-game".into(), String::new(), "Stop game".into())),
            });
        }
    }
    // Parked Mods/ from an unclean vanilla session.
    if game_root.is_dir()
        && greg_loader::launch::mods_state(&game_root)
            == greg_loader::launch::ModsState::DisabledByManager
        && !greg_loader::launch::any_game_process_running()
    {
        problems.push(Problem {
            severity: "warning",
            source: "Game".into(),
            message: "Mods folder is parked (vanilla) — restore it to play with mods.".into(),
            action: Some(("restore-mods".into(), String::new(), "Restore mods".into())),
        });
    }
    // Health checks (only with a game root; without one the game error above covers it).
    if game_root.is_dir() {
        let service = greg_loader::health::ModDependencyService::new(Some(game_root.clone()));
        if let Ok(checks) = service.run_checks() {
            for check in checks {
                match check.status {
                    greg_loader::content::DependencyStatus::Ok => {}
                    greg_loader::content::DependencyStatus::Warning => problems.push(Problem {
                        severity: "warning",
                        source: check.label,
                        message: check.detail,
                        action: None,
                    }),
                    greg_loader::content::DependencyStatus::Missing => problems.push(Problem {
                        severity: "error",
                        source: check.label,
                        message: check.detail,
                        action: None,
                    }),
                }
            }
        }
    }

    // Projects with unpublished changes.
    if let Ok(projects) = guard.workspace.scan_projects() {
        for project in projects {
            let meta =
                greg_platform::workspace::load_metadata(&project.root_path).unwrap_or_default();
            let desc = guard
                .steam
                .build_upload_description(&project.root_path, &meta);
            let sync = guard
                .workspace
                .sync_state(&project.root_path, &desc)
                .unwrap_or(ProjectSyncState::Unknown);
            let name = if meta.title.trim().is_empty() {
                project.name.clone()
            } else {
                meta.title.clone()
            };
            match sync {
                ProjectSyncState::Modified => problems.push(Problem {
                    severity: "warning",
                    source: "Projects".into(),
                    message: format!("{name}: local changes not published yet."),
                    action: Some((
                        "editor".into(),
                        project.root_path.to_string_lossy().to_string(),
                        "Open editor".into(),
                    )),
                }),
                ProjectSyncState::Unpublished => problems.push(Problem {
                    severity: "info",
                    source: "Projects".into(),
                    message: format!("{name}: never published."),
                    action: Some((
                        "editor".into(),
                        project.root_path.to_string_lossy().to_string(),
                        "Open editor".into(),
                    )),
                }),
                ProjectSyncState::Synced | ProjectSyncState::Unknown => {}
            }
        }
    }

    let status = if problems.is_empty() {
        "All clear — no problems found.".to_string()
    } else {
        let (errors, warnings) = problems.iter().fold((0, 0), |(e, w), p| {
            (
                e + usize::from(p.severity == "error"),
                w + usize::from(p.severity == "warning"),
            )
        });
        format!("{errors} errors, {warnings} warnings")
    };
    let rows: Vec<ProblemRow> = problems
        .iter()
        .enumerate()
        .map(|(i, p)| ProblemRow {
            severity: p.severity.into(),
            source: p.source.clone().into(),
            message: p.message.clone().into(),
            has_action: p.action.is_some(),
            action_label: p
                .action
                .as_ref()
                .map(|(_, _, label)| label.clone().into())
                .unwrap_or_default(),
            index: i as i32,
        })
        .collect();
    guard.problem_actions = problems
        .into_iter()
        .map(|p| {
            p.action
                .map(|(id, target, _)| (id, target))
                .unwrap_or_default()
        })
        .collect();
    guard.ui().set_problem_rows(model_of_rows(rows));
    guard.ui().set_problem_status(status.into());
}

/// Executes a problem-row action by list index.
pub fn problems_act(state: &Arc<std::sync::Mutex<AppState>>, index: i32) {
    let action = {
        let guard = state.lock().expect("state");
        guard
            .problem_actions
            .get(index as usize)
            .cloned()
            .unwrap_or_default()
    };
    match action.0.as_str() {
        "settings" => on_icon(state, crate::state::ICON_SETTINGS),
        "editor" if !action.1.is_empty() => open_project(state, &action.1),
        "stop-game" => stop_game(state),
        "restore-mods" => {
            {
                let mut guard = state.lock().expect("state");
                recover_parked_mods(&mut guard);
            }
            refresh_problems(state);
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Modpacks + shareable collections.
// ---------------------------------------------------------------------------

/// Catalog file: app-data `collections.json` (C# used `{root}/collections.json`).
pub fn packs_path() -> std::path::PathBuf {
    greg_platform::paths::app_data_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("collections.json")
}

/// Loads the catalog (empty when missing/corrupt).
pub fn load_packs() -> greg_core::models::ModCollectionService {
    let path = packs_path();
    if !path.is_file() {
        return greg_core::models::ModCollectionService::new();
    }
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|text| greg_core::models::ModCollectionService::load_json(&text).ok())
        .unwrap_or_default()
}

/// Persists the catalog (best-effort).
pub fn save_packs(svc: &greg_core::models::ModCollectionService) {
    if let Ok(json) = svc.to_json() {
        let path = packs_path();
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(path, json);
    }
}

fn packs_kind_of(state: &AppState) -> greg_core::models::CollectionKind {
    use greg_core::models::CollectionKind as K;
    match state.ui().get_current_page().as_str() {
        "collections" => K::Collection,
        _ => K::Pack,
    }
}

/// Refreshes packs list + selected entries for the active kind.
pub fn packs_refresh(state: &Arc<std::sync::Mutex<AppState>>) {
    let mut guard = state.lock().expect("state");
    let kind = packs_kind_of(&guard);
    guard.ui().set_pack_kind(kind.id().into());
    guard
        .ui()
        .set_pack_title(if kind == greg_core::models::CollectionKind::Collection {
            "COLLECTIONS".into()
        } else {
            "MODPACKS".into()
        });
    let ids: Vec<String> = guard
        .pack_service
        .of_kind(kind)
        .iter()
        .map(|c| c.id.clone())
        .collect();
    // Drop selections that vanished (or belong to the other kind).
    if guard
        .selected_pack
        .as_ref()
        .is_none_or(|id| !ids.contains(id))
    {
        guard.selected_pack = ids.first().cloned();
    }
    guard.pack_order = ids;
    let rows: Vec<PackRow> = guard
        .pack_order
        .iter()
        .enumerate()
        .filter_map(|(i, id)| {
            guard.pack_service.get(id).map(|c| PackRow {
                name: if c.name.trim().is_empty() {
                    "(unnamed)".into()
                } else {
                    c.name.clone().into()
                },
                detail: format!(
                    "{} entries{}",
                    c.items.len(),
                    if c.enabled { "" } else { " · off" }
                )
                .into(),
                enabled: c.enabled,
                index: i as i32,
            })
        })
        .collect();
    let selected_index = guard
        .selected_pack
        .as_ref()
        .and_then(|id| guard.pack_order.iter().position(|x| x == id))
        .map(|i| i as i32)
        .unwrap_or(-1);
    guard.ui().set_pack_packs(model_of_rows(rows));
    guard.ui().set_pack_selected(selected_index);
    refresh_pack_entries(&mut guard);
    if guard.ui().get_pack_status().to_string().is_empty() {
        guard.ui().set_pack_status(
            "Create packs from your installed mods, apply them, or share collections as files."
                .into(),
        );
    }
}

fn refresh_pack_entries(guard: &mut AppState) {
    let entries: Vec<PackEntryRow> = guard
        .selected_pack
        .as_ref()
        .and_then(|id| guard.pack_service.get(id))
        .map(|c| {
            c.items
                .iter()
                .enumerate()
                .map(|(i, e)| {
                    let detail = if e.published_file_id != 0 {
                        format!("Workshop {}", e.published_file_id)
                    } else if let Some(path) = e.local_path.as_deref() {
                        let size = std::fs::metadata(path)
                            .map(|m| greg_core::util::format_bytes(m.len() as i64))
                            .unwrap_or_default();
                        let folder = std::path::Path::new(path)
                            .parent()
                            .and_then(|p| p.file_name())
                            .and_then(|n| n.to_str())
                            .unwrap_or("?");
                        format!("{size} · {folder}")
                    } else {
                        "unresolved entry".to_string()
                    };
                    PackEntryRow {
                        title: if e.title.trim().is_empty() {
                            "(untitled)".into()
                        } else {
                            e.title.clone().into()
                        },
                        detail: detail.into(),
                        enabled: e.enabled,
                        index: i as i32,
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    guard.ui().set_pack_entries(model_of_rows(entries));
}

/// Creates a pack/collection of the active kind.
pub fn packs_create(state: &Arc<std::sync::Mutex<AppState>>) {
    let mut guard = state.lock().expect("state");
    let name = guard.ui().get_pack_new_name().to_string();
    if name.trim().is_empty() {
        return;
    }
    let kind = packs_kind_of(&guard);
    let id = guard.pack_service.create(kind, name.trim(), "");
    guard.selected_pack = Some(id);
    guard.ui().set_pack_new_name("".into());
    save_packs(&guard.pack_service);
    drop(guard);
    packs_refresh(state);
}

/// Selects a pack by row index.
pub fn packs_select(state: &Arc<std::sync::Mutex<AppState>>, index: i32) {
    let mut guard = state.lock().expect("state");
    let id = guard.pack_order.get(index as usize).cloned();
    guard.selected_pack = id;
    guard.ui().set_pack_selected(index);
    refresh_pack_entries(&mut guard);
}

/// Toggles a whole pack by row index.
pub fn packs_toggle(state: &Arc<std::sync::Mutex<AppState>>, index: i32) {
    let mut guard = state.lock().expect("state");
    if let Some(id) = guard.pack_order.get(index as usize).cloned() {
        guard.pack_service.toggle(&id);
        save_packs(&guard.pack_service);
    }
    drop(guard);
    packs_refresh(state);
}

/// Deletes a pack by row index.
pub fn packs_delete(state: &Arc<std::sync::Mutex<AppState>>, index: i32) {
    let mut guard = state.lock().expect("state");
    if let Some(id) = guard.pack_order.get(index as usize).cloned() {
        guard.pack_service.delete(&id);
        if guard.selected_pack.as_deref() == Some(&id) {
            guard.selected_pack = None;
        }
        save_packs(&guard.pack_service);
    }
    drop(guard);
    packs_refresh(state);
}

/// Toggles one entry of the selected pack.
pub fn packs_toggle_entry(state: &Arc<std::sync::Mutex<AppState>>, index: i32) {
    let mut guard = state.lock().expect("state");
    if let Some(id) = guard.selected_pack.clone() {
        guard.pack_service.toggle_entry(&id, index as usize);
        save_packs(&guard.pack_service);
        refresh_pack_entries(&mut guard);
    }
}

/// Removes one entry of the selected pack.
pub fn packs_remove_entry(state: &Arc<std::sync::Mutex<AppState>>, index: i32) {
    let mut guard = state.lock().expect("state");
    if let Some(id) = guard.selected_pack.clone() {
        guard.pack_service.remove_entry_at(&id, index as usize);
        save_packs(&guard.pack_service);
        refresh_pack_entries(&mut guard);
        drop(guard);
        packs_refresh(state);
    }
}

/// Adds a local file (DLL) to the selected pack.
pub fn packs_add_file(state: &Arc<std::sync::Mutex<AppState>>) {
    let selected = {
        let guard = state.lock().expect("state");
        guard.selected_pack.clone()
    };
    let Some(pack_id) = selected else { return };
    let Some(path) = rfd::FileDialog::new()
        .add_filter("Mod files", &["dll", "zip"])
        .pick_file()
    else {
        return;
    };
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("?")
        .to_string();
    let mod_type = if path.to_string_lossy().to_lowercase().contains("plugin") {
        "MelonloaderPlugin"
    } else {
        "DataCenterMod"
    };
    let mut guard = state.lock().expect("state");
    guard.pack_service.add_item(
        &pack_id,
        greg_core::models::ModCollectionEntry {
            title: name,
            source_name: "Local file".into(),
            mod_type: mod_type.into(),
            local_path: Some(path.to_string_lossy().to_string()),
            ..Default::default()
        },
    );
    save_packs(&guard.pack_service);
    refresh_pack_entries(&mut guard);
}

/// Exports the selected pack to a shareable file.
pub fn packs_export(state: &Arc<std::sync::Mutex<AppState>>) {
    let (id, name) = {
        let guard = state.lock().expect("state");
        match guard.selected_pack.clone() {
            Some(id) => {
                let name = guard
                    .pack_service
                    .get(&id)
                    .map(|c| {
                        if c.name.trim().is_empty() {
                            "pack".to_string()
                        } else {
                            c.name.clone()
                        }
                    })
                    .unwrap_or_else(|| "pack".to_string());
                (id, name)
            }
            None => return,
        }
    };
    let Some(dest) = rfd::FileDialog::new()
        .set_file_name(format!("{name}.gregpack.json"))
        .save_file()
    else {
        return;
    };
    let guard = state.lock().expect("state");
    match guard.pack_service.export_pack(&id) {
        Ok(json) => match std::fs::write(&dest, json) {
            Ok(()) => {
                guard
                    .ui()
                    .set_pack_status(format!("Exported to {}", dest.display()).into());
                AppState::append_log(state, &format!("Exported pack to {}", dest.display()));
            }
            Err(e) => guard
                .ui()
                .set_pack_status(format!("Export failed: {e}").into()),
        },
        Err(e) => guard
            .ui()
            .set_pack_status(format!("Export failed: {e}").into()),
    }
}

/// Imports a shared pack/collection file.
pub fn packs_import(state: &Arc<std::sync::Mutex<AppState>>) {
    let Some(path) = rfd::FileDialog::new()
        .add_filter("Pack files", &["json"])
        .pick_file()
    else {
        return;
    };
    let mut guard = state.lock().expect("state");
    match std::fs::read_to_string(&path) {
        Ok(json) => match guard.pack_service.import_pack(&json) {
            Ok(id) => {
                guard.selected_pack = Some(id);
                save_packs(&guard.pack_service);
                drop(guard);
                packs_refresh(state);
            }
            Err(e) => guard
                .ui()
                .set_pack_status(format!("Import failed: {e}").into()),
        },
        Err(e) => guard
            .ui()
            .set_pack_status(format!("Import failed: {e}").into()),
    }
}

/// Workshop link for pack apply (boxed once, not inline).
type WorkshopLink<'a> = Option<&'a dyn Fn(u64) -> std::result::Result<(), String>>;

/// Applies the selected pack (local enable/disable + Workshop subscribe).
/// The index argument mirrors the UI callback; selection is authoritative.
pub fn packs_apply(state: &Arc<std::sync::Mutex<AppState>>, _index: i32) {
    if guard_game_running(state) {
        return;
    }
    let (pack, game_root, steam) = {
        let guard = state.lock().expect("state");
        let Some(id) = guard.selected_pack.clone() else {
            return;
        };
        let Some(pack) = guard.pack_service.get(&id).cloned() else {
            return;
        };
        (pack, game_root_of(&guard), Arc::clone(&guard.steam))
    };
    let steam_available = matches!(
        steam.status(),
        greg_steam::backend::BackendStatus::Available { .. }
    );
    // Sync Steam calls are quick local IPC; unavailable backends are passed
    // as `None` so workshop entries land in `skipped_no_steam`.
    let subscribe_steam = Arc::clone(&steam);
    let unsubscribe_steam = Arc::clone(&steam);
    let subscribe = move |id: u64| subscribe_steam.subscribe(id).map_err(|e| e.to_string());
    let unsubscribe = move |id: u64| unsubscribe_steam.unsubscribe(id).map_err(|e| e.to_string());
    let (sub, unsub): (WorkshopLink<'_>, WorkshopLink<'_>) = if steam_available {
        (Some(&subscribe), Some(&unsubscribe))
    } else {
        (None, None)
    };
    let report = greg_loader::packs::apply_pack(&pack, &game_root, sub, unsub);
    let summary = match report {
        Ok(report) => {
            for error in &report.errors {
                AppState::append_log(state, &format!("Pack apply: {error}"));
            }
            AppState::append_log(state, &format!("Pack apply: {}", report.summary()));
            report.summary()
        }
        Err(e) => {
            AppState::append_log(state, &format!("Pack apply failed: {e}"));
            format!("Apply failed: {e}")
        }
    };
    let guard = state.lock().expect("state");
    guard.ui().set_pack_status(summary.into());
    drop(guard);
    // Local states may have changed — refresh dependent pages.
    {
        let mut guard = state.lock().expect("state");
        refresh_projects(&mut guard);
        local_refresh(&mut guard);
    }
}

// ---------------------------------------------------------------------------
// Settings.
// ---------------------------------------------------------------------------

/// Loads preferences into the settings page.
pub fn refresh_settings(state: &mut AppState) {
    state.ui().set_se_workspace(
        state
            .prefs
            .get_string(greg_platform::workspace::CUSTOM_WORKSPACE_PATH_KEY, "")
            .into(),
    );
    state
        .ui()
        .set_se_gameroot(state.prefs.get_string("GameRootPath", "").into());
    let code = state.prefs.get_string("AppLanguage", "");
    let codes = ["", "en", "de", "fr", "es", "it", "ja", "pl", "ru", "zh"];
    state
        .ui()
        .set_se_lang(codes.iter().position(|c| *c == code).unwrap_or(1) as i32);
    state
        .ui()
        .set_se_modstore_url(state.prefs.get_string("ModStoreBaseUrl", "").into());
    state.ui().set_se_telemetry_text(
        format!(
            "Telemetry: {} (GREG_TELEMETRY_ENABLED)",
            if greg_core::models::settings::is_telemetry_enabled() {
                "on"
            } else {
                "off"
            }
        )
        .into(),
    );
    push_log(state);
}

fn push_log(state: &mut AppState) {
    let lines = state.log_snapshot();
    state.ui().set_se_log(model_of_strings(&lines));
}

/// Applies the workspace override.
pub fn settings_apply_workspace(state: &Arc<std::sync::Mutex<AppState>>) {
    let mut guard = state.lock().expect("state");
    let value = guard.ui().get_se_workspace().to_string();
    guard.prefs.set_string(
        greg_platform::workspace::CUSTOM_WORKSPACE_PATH_KEY,
        value.trim(),
    );
    let game_root = greg_loader::health::ModDependencyService::new(None).game_root();
    guard.workspace = greg_platform::workspace::Workspace::resolve(
        if value.trim().is_empty() {
            None
        } else {
            Some(value.trim())
        },
        game_root.as_deref(),
    );
    refresh_projects(&mut guard);
}

/// Applies the game-root override.
pub fn settings_apply_gameroot(state: &Arc<std::sync::Mutex<AppState>>) {
    let mut guard = state.lock().expect("state");
    let value = guard.ui().get_se_gameroot().to_string();
    guard.prefs.set_string("GameRootPath", value.trim());
}

/// Applies the Modstore URL override.
pub fn settings_apply_modstore_url(state: &Arc<std::sync::Mutex<AppState>>) {
    let mut guard = state.lock().expect("state");
    let value = guard.ui().get_se_modstore_url().to_string();
    guard.prefs.set_string("ModStoreBaseUrl", value.trim());
}

/// Folder picker into a property.
pub fn pick_folder_into(state: &Arc<std::sync::Mutex<AppState>>, target: &str) {
    let Some(dir) = rfd::FileDialog::new().pick_folder() else {
        return;
    };
    let guard = state.lock().expect("state");
    match target {
        "workspace" => guard
            .ui()
            .set_se_workspace(dir.to_string_lossy().to_string().into()),
        _ => guard
            .ui()
            .set_se_gameroot(dir.to_string_lossy().to_string().into()),
    }
}

/// Runs health checks into the UI.
pub fn settings_run_checks(state: &Arc<std::sync::Mutex<AppState>>) {
    let guard = state.lock().expect("state");
    let root = {
        let raw = guard.ui().get_se_gameroot().to_string();
        if raw.trim().is_empty() {
            None
        } else {
            Some(PathBuf::from(raw))
        }
    };
    let rows: Vec<HealthRow> =
        match greg_loader::health::ModDependencyService::new(root).run_checks() {
            Ok(checks) => checks
                .into_iter()
                .map(|c| HealthRow {
                    label: c.label.into(),
                    detail: c.detail.into(),
                    ok: c.status == greg_loader::content::DependencyStatus::Ok,
                })
                .collect(),
            Err(e) => {
                AppState::append_log(state, &format!("checks failed: {e}"));
                Vec::new()
            }
        };
    guard.ui().set_se_health(model_of_rows(rows));
}

/// Opens the log folder.
pub fn settings_open_logs(_state: &Arc<std::sync::Mutex<AppState>>) {
    let log = greg_platform::filelog::FileLog::shared().clone();
    let dir = log
        .path()
        .parent()
        .map(std::path::Path::to_path_buf)
        .unwrap_or_else(std::env::temp_dir);
    let _ = greg_platform::process::open_folder(&dir);
}

/// Creates a support bundle.
pub fn settings_make_bundle(state: &Arc<std::sync::Mutex<AppState>>) {
    let guard = state.lock().expect("state");
    let log = greg_platform::filelog::FileLog::shared().clone();
    let manifest = serde_json::json!({
        "app": "gregModmanager",
        "version": env!("CARGO_PKG_VERSION"),
        "os": std::env::consts::OS,
    });
    match greg_platform::repro::create_bundle(&log.path(), &log.reports_dir(), &manifest, None) {
        Ok(path) => AppState::append_log(state, &format!("Bundle: {}", path.display())),
        Err(e) => AppState::append_log(state, &format!("Bundle failed: {e}")),
    }
    drop(guard);
    if let Ok(mut guard) = state.lock() {
        push_log(&mut guard);
    }
}

/// Registers the `greg://` protocol.
pub fn settings_register_protocol(state: &Arc<std::sync::Mutex<AppState>>) {
    let message = match std::env::current_exe() {
        Ok(exe) => match greg_platform::protocol::register_protocol(&exe) {
            Ok(()) => "Protocol registered.".to_string(),
            Err(e) => format!("Protocol failed: {e}"),
        },
        Err(e) => format!("No exe path: {e}"),
    };
    AppState::append_log(state, &message);
}

// ---------------------------------------------------------------------------
// Editor assets.
// ---------------------------------------------------------------------------

/// Preview image picker.
pub fn editor_pick_preview(state: &Arc<std::sync::Mutex<AppState>>) {
    let root = {
        let guard = state.lock().expect("state");
        match guard.editor_root.clone() {
            Some(r) => r,
            None => return,
        }
    };
    let Some(path) = rfd::FileDialog::new()
        .add_filter("Images", &["png", "jpg", "jpeg", "gif", "webp"])
        .pick_file()
    else {
        return;
    };
    let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
        return;
    };
    let dest_name = format!("preview.{ext}");
    if std::fs::copy(&path, root.join(&dest_name)).is_ok() {
        let mut guard = state.lock().expect("state");
        guard.editor_meta.preview_image_relative_path = dest_name;
        push_editor(&mut guard);
        run_editor_checks(&mut guard);
    }
}

/// Screenshot adder (max 9).
pub fn editor_add_shot(state: &Arc<std::sync::Mutex<AppState>>) {
    let root = {
        let guard = state.lock().expect("state");
        if guard.editor_meta.additional_previews.len() >= 9 {
            return;
        }
        match guard.editor_root.clone() {
            Some(r) => r,
            None => return,
        }
    };
    let Some(path) = rfd::FileDialog::new()
        .add_filter("Images", &["png", "jpg", "jpeg", "gif", "webp"])
        .pick_file()
    else {
        return;
    };
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return;
    };
    let name = name.to_string();
    let dir = root.join("screenshots");
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    if std::fs::copy(&path, dir.join(&name)).is_ok() {
        let mut guard = state.lock().expect("state");
        guard
            .editor_meta
            .additional_previews
            .push(format!("screenshots/{name}"));
        push_editor(&mut guard);
        run_editor_checks(&mut guard);
    }
}

/// Screenshot remover by index.
pub fn editor_remove_shot(state: &Arc<std::sync::Mutex<AppState>>, index: i32) {
    let mut guard = state.lock().expect("state");
    let idx = index as usize;
    if idx < guard.editor_meta.additional_previews.len() {
        guard.editor_meta.additional_previews.remove(idx);
        push_editor(&mut guard);
        run_editor_checks(&mut guard);
    }
}

/// Workshop dependency adder (validates numeric ids, rejects self/dupes).
pub fn editor_dep_add(state: &Arc<std::sync::Mutex<AppState>>) {
    let mut guard = state.lock().expect("state");
    let raw = guard.ui().get_ed_dep_add_text().to_string();
    let Ok(id) = raw.trim().parse::<u64>() else {
        return;
    };
    if id == 0 || id == guard.editor_meta.published_file_id || guard.editor_dep_ids.contains(&id) {
        return;
    }
    guard.editor_dep_ids.push(id);
    guard.editor_meta.workshop_dependency_ids = guard.editor_dep_ids.clone();
    guard.ui().set_ed_dep_add_text("".into());
    push_editor(&mut guard);
    run_editor_checks(&mut guard);
}

/// Workshop dependency remover by index.
pub fn editor_dep_remove(state: &Arc<std::sync::Mutex<AppState>>, index: i32) {
    let mut guard = state.lock().expect("state");
    let idx = index as usize;
    if idx < guard.editor_dep_ids.len() {
        guard.editor_dep_ids.remove(idx);
        guard.editor_meta.workshop_dependency_ids = guard.editor_dep_ids.clone();
        push_editor(&mut guard);
        run_editor_checks(&mut guard);
    }
}

// ---------------------------------------------------------------------------
// Model helpers.
// ---------------------------------------------------------------------------

fn model_of_strings(items: &[String]) -> ModelRc<SharedString> {
    ModelRc::from(Rc::new(VecModel::from(
        items
            .iter()
            .map(|s| s.clone().into())
            .collect::<Vec<SharedString>>(),
    )))
}

fn model_of_rows<T>(rows: Vec<T>) -> ModelRc<T>
where
    T: Clone + 'static,
{
    ModelRc::from(Rc::new(VecModel::from(rows)))
}

/// Loads a thumbnail (empty image when missing/unreadable — the UI shows
/// the placeholder box). Local files only; remote previews go through the
/// thumbnail cache first.
fn thumb_of(path: Option<&std::path::Path>) -> slint::Image {
    path.and_then(|p| slint::Image::load_from_path(p).ok())
        .unwrap_or_default()
}

/// Thumbnail cache dir for remote previews.
fn thumb_cache_dir() -> PathBuf {
    let dir = std::env::temp_dir().join("gregmodmanager-thumbs");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// Queues missing thumbnails (key -> url), skipping cached/queued ones.
fn queue_thumbs(state: &mut AppState, items: Vec<(String, String)>) {
    for (key, url) in items {
        if url.trim().is_empty()
            || state.thumb_done.contains_key(&key)
            || state.thumb_pending.iter().any(|(k, _)| k == &key)
        {
            continue;
        }
        state.thumb_pending.push((key, url));
    }
    pump_thumb_job(state);
}

/// Starts the download job when idle with pending items.
fn pump_thumb_job(state: &mut AppState) {
    if state.thumb_job.is_some() || state.thumb_pending.is_empty() {
        return;
    }
    let batch = std::mem::take(&mut state.thumb_pending);
    let dir = thumb_cache_dir();
    let job = JobHandle::spawn(move |sink| {
        let mut out = Vec::new();
        for (key, url) in &batch {
            if sink.cancelled().load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }
            match download_thumb(&dir, key, url) {
                Some(path) => out.push(format!("{key}\x1f{}", path.display())),
                None => sink.log(format!("Thumbnail failed: {url}")),
            }
        }
        crate::worker::JobOutcome {
            success: true,
            message: out.join("\x1e"),
        }
    });
    state.thumb_job = Some(job);
}

/// Downloads one preview into the cache (blocking — background only).
fn download_thumb(dir: &std::path::Path, key: &str, url: &str) -> Option<PathBuf> {
    let bytes = reqwest::blocking::get(url)
        .and_then(|r| r.bytes())
        .ok()?
        .to_vec();
    if bytes.is_empty() {
        return None;
    }
    let safe: String = key
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    if safe.is_empty() {
        return None;
    }
    let path = dir.join(format!("{safe}{}", greg_steam::service::image_ext(&bytes)));
    if path.is_file() {
        return Some(path);
    }
    std::fs::write(&path, &bytes).ok()?;
    Some(path)
}

/// Applies finished thumbnail downloads to the cache and repaints rows.
fn drain_thumb(state: &mut AppState) {
    let messages = match state.thumb_job.as_mut() {
        Some(job) => job.drain(),
        None => return,
    };
    let finished = state
        .thumb_job
        .as_ref()
        .is_some_and(|job| job.is_finished());
    let mut done: Option<JobOutcome> = None;
    for msg in messages {
        if let JobMessage::Done(outcome) = msg {
            done = Some(outcome);
        }
    }
    let Some(outcome) = done else {
        if finished {
            state.thumb_job = None;
            pump_thumb_job(state);
        }
        return;
    };
    state.thumb_job = None;
    if outcome.success {
        for record in outcome.message.split('\x1e') {
            let mut parts = record.splitn(2, '\x1f');
            let (Some(key), Some(path)) = (parts.next(), parts.next()) else {
                continue;
            };
            let path = PathBuf::from(path);
            if path.is_file() {
                state.thumb_done.insert(key.to_string(), path);
            }
        }
    }
    repaint_browse_thumbs(state);
    repaint_store_thumbs(state);
    pump_thumb_job(state);
}

/// Repaints browse rows with cached thumbnails (in place, keeps selection).
fn repaint_browse_thumbs(state: &mut AppState) {
    let model = state.ui().get_br_items();
    let Some(view) = model.as_any().downcast_ref::<VecModel<BrowseRow>>() else {
        return;
    };
    for i in 0..view.row_count() {
        let Some(mut row) = view.row_data(i) else {
            continue;
        };
        if let Some(path) = state
            .thumb_done
            .get(row.id.as_str())
            .filter(|p| p.is_file())
            .cloned()
        {
            row.thumb = thumb_of(Some(&path));
            view.set_row_data(i, row);
        }
    }
}

/// Repaints modstore rows with cached thumbnails (in place).
fn repaint_store_thumbs(state: &mut AppState) {
    let model = state.ui().get_st_items();
    let Some(view) = model.as_any().downcast_ref::<VecModel<StoreRow>>() else {
        return;
    };
    for i in 0..view.row_count() {
        let Some(mut row) = view.row_data(i) else {
            continue;
        };
        if let Some(path) = state
            .thumb_done
            .get(row.slug.as_str())
            .filter(|p| p.is_file())
            .cloned()
        {
            row.thumb = thumb_of(Some(&path));
            view.set_row_data(i, row);
        }
    }
}

// AppState helper used by drain_store (avoids borrow issues).
impl AppState {
    fn append_log_ref(state: &mut AppState, line: &str) {
        if let Ok(lines) = state.log_lines.lock().as_deref_mut() {
            lines.push(line.to_string());
            if lines.len() > 500 {
                lines.remove(0);
            }
        }
        state.file_log.info(line);
    }
}

#[cfg(test)]
mod protocol_tests {
    use super::{new_request_id, parse_protocol_url, ProtocolUrl};

    #[test]
    fn request_ids_are_unique_and_server_shaped() {
        let a = new_request_id();
        let b = new_request_id();
        assert_eq!(a.len(), 32, "128-bit hex request id");
        assert_ne!(a, b, "request ids must be unpredictable");
        for id in [&a, &b] {
            assert!(id.len() >= 16 && id.len() <= 128);
            assert!(id.chars().all(|c| c.is_ascii_alphanumeric()));
        }
    }

    #[test]
    fn accepts_both_callback_paths() {
        for url in [
            "greg://auth/callback?requestId=abc&code=CODE&state=s&nonce=n&sig=S",
            "greg://v1/auth/callback?requestId=abc&code=CODE&state=s&nonce=n&sig=S",
        ] {
            match parse_protocol_url(url) {
                Some(ProtocolUrl::AuthCallback {
                    request_id, code, ..
                }) => {
                    assert_eq!(request_id, "abc");
                    assert_eq!(code, "CODE");
                }
                other => panic!("expected callback for {url}, got {}", other.is_some()),
            }
        }
    }

    #[test]
    fn rejects_foreign_urls_and_routes_other() {
        assert!(parse_protocol_url("https://example.com/?code=x").is_none());
        assert!(parse_protocol_url("not a url").is_none());
        match parse_protocol_url("greg://install/intent?modId=5") {
            Some(ProtocolUrl::Other(_)) => {}
            _ => panic!("expected Other"),
        }
    }

    #[test]
    fn decodes_percent_encoding() {
        match parse_protocol_url("greg://auth/callback?code=AB%2FC%20D&requestId=r") {
            Some(ProtocolUrl::AuthCallback { code, .. }) => assert_eq!(code, "AB/C D"),
            _ => panic!("expected callback"),
        }
    }
}
