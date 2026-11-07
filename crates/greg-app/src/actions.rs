//! Slint callback implementations (one section per shell area).

use std::path::{Path, PathBuf};
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
    BrowseRow, CheckRow, DiagnosticRow, HealthRow, LibInstallRow, LocalRow, PackEntryRow, PackRow,
    ProblemRow, ProjectRow, StoreRow, UpdateRow,
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
                let prev_key = state.steam.web_api_key();
                state.backend = Arc::clone(&backend);
                let steam = Arc::new(greg_steam::service::WorkshopService::new(
                    backend,
                    state.lang.clone(),
                ));
                steam.set_web_api_key(prev_key);
                state.steam = steam;
                state.refresh_steam_status();
                refresh_projects(state);
            }
            UiEvent::Probe { gregapi_ok } => {
                let ui = state.ui();
                ui.set_gregapi_known(true);
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
                state.push_log("Login failed.");
                let ui = state.ui();
                ui.set_ed_status(format!("Login failed: {message}").into());
            }
            UiEvent::LocalScanDone {
                generation,
                root,
                kind,
                signature,
                entries,
            } => {
                finish_local_scan(state, generation, root, kind, signature, entries);
            }
            UiEvent::ReportSubmissionDone {
                success,
                message,
                issue_url,
            } => {
                let ui = state.ui();
                if success {
                    ui.set_rp_submission("submitted".into());
                    ui.set_rp_status(if let Some(url) = &issue_url {
                        format!("Report submitted: {url}").into()
                    } else {
                        format!("Report submitted. {message}").into()
                    });
                } else {
                    state.push_log("Report submission failed.");
                    let ui = state.ui();
                    ui.set_rp_submission("failed".into());
                    ui.set_rp_status(format!("Report submission failed: {message}").into());
                }
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
    drain_build(state);
    drain_publish(state);
    drain_sync(state);
    drain_thumb(state);
    drain_browse(state);
    drain_store(state);
    drain_lib_install(state);
    drain_plugin_install(state);
    // Language switch watch (also wired via se-apply-lang callback).
    let lang_index = state.ui().get_se_lang();
    if state.settings_lang_index != lang_index {
        state.settings_lang_index = lang_index;
        settings_apply_lang_index(state, lang_index);
    }
}

/// Immediately applies the selected language from the Settings ComboBox.
pub fn settings_apply_lang(state: &Arc<std::sync::Mutex<AppState>>) {
    let mut guard = state.lock().expect("state");
    let lang_index = guard.ui().get_se_lang();
    guard.settings_lang_index = lang_index;
    settings_apply_lang_index(&mut guard, lang_index);
}

/// Applies a settings language index (shared by watch and callback).
fn settings_apply_lang_index(state: &mut AppState, index: i32) {
    let codes = ["", "en", "de", "fr", "es", "it", "ja", "pl", "ru", "zh"];
    let code = codes.get(index.max(0) as usize).copied().unwrap_or("en");
    state.lang = l10n::resolve_lang_code(code).to_string();
    state.prefs.set_string("AppLanguage", code);
    let prev_key = state.steam.web_api_key();
    let backend = Arc::clone(&state.backend);
    let steam = Arc::new(greg_steam::service::WorkshopService::new(
        backend,
        state.lang.clone(),
    ));
    steam.set_web_api_key(prev_key);
    state.steam = steam;
    apply_ui_translations(state);
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

/// Handles navigation clicks (page ids + the report entry).
pub fn on_nav(state: &Arc<std::sync::Mutex<AppState>>, id: &str) {
    if id == "report" {
        // Universal Report Assistant page (central entry; category →
        // description → preview → submit).
        report_open(state);
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
/// Modes: `mods` (ensures files disabled by an interrupted vanilla session are
/// restored first), or `vanilla` (the default Start Game action, disables
/// active assemblies in `Mods/` for this session and restores them on exit).
/// The Steamworks session is
/// released before the start — otherwise Steam keeps seeing the game as
/// running through the manager and the next launch fails until
/// everything is killed (Windows zombie case).
pub fn start_game(state: &Arc<std::sync::Mutex<AppState>>, mode: &str) {
    poll_game_state(&mut state.lock().expect("state"));
    let launch_mode = if mode == "mods" {
        greg_loader::launch::LaunchMode::Mods
    } else {
        // Start Game intentionally means Without Mods. Modded launching is
        // an explicit choice through the separate With Mods button.
        greg_loader::launch::LaunchMode::Vanilla
    };
    // Refuse while our own session runs.
    // Bind first: a temporary guard in the `if let` scrutinee would live
    // through the whole body and deadlock on the re-lock below.
    let tracked_pid = tracked_game_pid(&mut state.lock().expect("state"));
    if tracked_pid.is_some() {
        AppState::append_log(state, "Game launch skipped: game process already running.");
        return;
    }
    // Refuse while a stale game process lives (would disagree with Steam).
    let stale = greg_loader::launch::stale_game_pids();
    if !stale.is_empty() {
        AppState::append_log(state, "Game launch skipped: stale game process detected.");
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
                (
                    installation
                        .root_path
                        .canonicalize()
                        .unwrap_or(installation.root_path),
                    plan.executable,
                )
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
    if launch_mode == greg_loader::launch::LaunchMode::Mods {
        let melon_loader = greg_loader::installers::detect_melon_loader(&game_root);
        if !melon_loader.installed {
            let mut guard = state.lock().expect("state");
            let message = l10n::get(&guard.lang, "Game_MelonLoaderMissing");
            guard.push_log(&message);
            guard.ui().set_game_text(message.into());
            return;
        }
    }
    let base = state.lock().expect("state").workspace.root().to_path_buf();
    if launch_mode == greg_loader::launch::LaunchMode::Mods {
        if let Err(_error) =
            greg_loader::managed_runtime::validate_managed_runtime(&game_root, &base)
        {
            let guard = state.lock().expect("state");
            let message = l10n::get(&guard.lang, "Workspace_LoaderUnavailable");
            guard.log_line("Game launch failed.");
            guard.ui().set_game_text(message.into());
            return;
        }
    }
    // Free the Steam session first so the play session owns the app state.
    release_steam_for_game(&mut state.lock().expect("state"));
    let result = if launch_mode == greg_loader::launch::LaunchMode::Mods {
        greg_loader::launch::GameSession::start_managed(&game_root, &exe, &base)
    } else {
        greg_loader::launch::GameSession::start(&game_root, &exe, launch_mode)
    };
    match result {
        Ok(session) => {
            let pid = session.pid();
            let mut guard = state.lock().expect("state");
            AppState::append_log(state, "Game process started.");
            guard.game_session = Some(session);
            set_game_ui(&mut guard, true, Some(pid));
        }
        Err(_e) => {
            AppState::append_log(state, "Game launch failed.");
            let mut guard = state.lock().expect("state");
            reconnect_steam(&mut guard);
        }
    }
}

/// Performs the dependency check once during manager startup.
///
/// Vanilla launches remain available without MelonLoader, but the result is
/// logged immediately and the With Mods action performs the same check again
/// immediately before launch so an installation changed while the manager was
/// open cannot be missed.
pub fn check_melon_loader_on_start(state: &Arc<std::sync::Mutex<AppState>>) {
    let mut guard = state.lock().expect("state");
    let root = game_root_of(&guard);
    if !root.is_dir() {
        guard.ui().set_melonloader_known(false);
        guard.ui().set_melonloader_ok(false);
        guard
            .ui()
            .set_melonloader_text(l10n::get(&guard.lang, "MelonLoader_StatusUnknown").into());
        guard.push_log("Startup check: game folder not found; MelonLoader check deferred.");
        return;
    }
    let install = greg_loader::installers::detect_melon_loader(&root);
    guard.ui().set_melonloader_known(true);
    if install.installed {
        guard.ui().set_melonloader_ok(true);
        guard
            .ui()
            .set_melonloader_text(l10n::get(&guard.lang, "MelonLoader_StatusInstalled").into());
        guard.push_log("Startup check: MelonLoader is installed.");
    } else {
        guard.ui().set_melonloader_ok(false);
        guard
            .ui()
            .set_melonloader_text(l10n::get(&guard.lang, "MelonLoader_StatusMissing").into());
        let message = l10n::get(&guard.lang, "Game_MelonLoaderMissing");
        guard.push_log("Startup check: MelonLoader is not installed.");
        guard.ui().set_game_text(message.into());
    }
}

/// Stops the tracked game session, or kills stale game processes when no
/// session exists (the Windows zombie escape hatch). Restores a parked
/// `Mods/` folder and reconnects Steamworks afterwards.
pub fn stop_game(state: &Arc<std::sync::Mutex<AppState>>) {
    if stop_tracked_session(state) {
        return;
    }
    stop_stale_processes(state);
}

/// Stops the tracked session. Returns `true` when one was stopped.
fn stop_tracked_session(state: &Arc<std::sync::Mutex<AppState>>) -> bool {
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
                    Err(_) => {
                        AppState::append_log(state, "Game stop failed.");
                        false
                    }
                },
                None => false,
            }
        }
    };
    if !stopped {
        return false;
    }
    let mut guard = state.lock().expect("state");
    guard.game_session = None;
    reconnect_steam(&mut guard);
    set_game_ui(&mut guard, false, None);
    local_refresh(&mut guard);
    true
}

/// Kills stale game processes by exe name (no live session exists).
fn stop_stale_processes(state: &Arc<std::sync::Mutex<AppState>>) {
    let stale = greg_loader::launch::stale_game_pids();
    if stale.is_empty() {
        AppState::append_log(state, "Stop: no game process running.");
        return;
    }
    for pid in &stale {
        match greg_platform::process::kill_tree(*pid) {
            Ok(()) => AppState::append_log(state, "Stopped stale game process."),
            Err(_) => AppState::append_log(state, "Could not stop stale game process."),
        }
    }
    // A stale kill may have left vanilla-disabled files: restore when idle.
    let mut guard = state.lock().expect("state");
    recover_parked_mods(&mut guard);
    reconnect_steam(&mut guard);
    set_game_ui(&mut guard, false, None);
    local_refresh(&mut guard);
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
        greg_loader::launch::SessionEvent::ManagedFailed { reason } => {
            let _detail = state
                .game_session
                .as_ref()
                .and_then(|s| s.managed_error())
                .unwrap_or(reason)
                .to_string();
            let still_running = state
                .game_session
                .as_mut()
                .is_some_and(|session| session.is_running());
            if still_running {
                let pid = state.game_session.as_ref().map(|s| s.pid());
                set_game_ui(state, true, pid);
            } else {
                state.log_line("Managed game launch stopped.");
                state.game_session = None;
                reconnect_steam(state);
                set_game_ui(state, false, None);
            }
            state
                .ui()
                .set_game_text(l10n::get(&state.lang, "Workspace_LoaderUnavailable").into());
        }
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
            state.game_session = None;
            reconnect_steam(state);
            set_game_ui(state, false, None);
            let _ = code;
            state.push_log("Game process exited.");
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
        let msg = l10n::get(&guard.lang, "Game_BlockedWhileRunning");
        guard.ui().set_local_status(msg.clone().into());
        AppState::append_log(state, &msg);
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

/// Restores files left disabled by a killed vanilla session.
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
        state.push_log("Recovered mods disabled by an unclean vanilla session.");
        local_refresh(state);
    }
}

/// Releases the Steamworks session so Steam stops seeing the game as
/// running through the manager. In-flight jobs are cancelled first; the
/// native shutdown lands when their handles drop.
fn release_steam_for_game(state: &mut AppState) {
    // A worker may hold its own Arc<WorkshopService>, and therefore its own
    // steamworks::Client. Cancelling the worker is not enough: the JobHandle
    // must be dropped so its thread is joined and the last Client can run
    // SteamAPI_Shutdown before Steam receives the game launch request.
    for job in [
        state.store_job.take(),
        state.browse_job.take(),
        state.publish_job.take(),
    ]
    .into_iter()
    .flatten()
    {
        job.cancel();
        drop(job);
    }

    // Replace both service references only after background workers have
    // stopped. This makes the ownership transition deterministic instead of
    // relying on a later UI tick to release the Steam API session.
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
            AppState::append_log(state, "Opening Steam login flow.");
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
    Other,
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
    let _ = url;
    Some(ProtocolUrl::Other)
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
        ProtocolUrl::Other => {
            AppState::append_log(state, "Ignored unsupported protocol URL.");
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
    state.push_log("Login completed; account identity omitted from diagnostics.");
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
        AppState::append_log(state, "Protocol handoff received.");
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
        "PlacableObject" => 0,
        _ => -1,
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
        0 => "PlacableObject".into(),
        _ => meta.mod_type.clone(),
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
    let (root, meta) = {
        let mut guard = state.lock().expect("state");
        pull_editor(&mut guard);
        (guard.editor_root.clone(), guard.editor_meta.clone())
    };
    // Release the lock before append_log (it locks state internally) to avoid
    // a self-deadlock on the UI thread.
    if let Some(root) = root {
        match greg_platform::workspace::save_metadata(&root, &meta) {
            Ok(()) => AppState::append_log(state, "Project saved."),
            Err(_) => AppState::append_log(state, "Project save failed."),
        }
    }
    // NOTE: run_editor_checks() and refresh_projects() are intentionally NOT
    // called here anymore. They run on the UI thread and can block for large
    // mods (hashing every content byte, converting every README to BBCode).
    // The editor tick re-runs checks reactively on field changes; after a
    // publish, drain_publish() refreshes projects asynchronously.
}

/// Show the resolved SDK build and dependency paths before executing project code.
pub fn editor_review_build(state: &Arc<std::sync::Mutex<AppState>>) {
    editor_save(state);
    let mut guard = state.lock().expect("state");
    if guard.build_job.is_some() {
        guard.ui().set_show_build_modal(true);
        return;
    }
    let Some(root) = guard.editor_root.clone() else {
        return;
    };
    let plan = greg_loader::build_test::plan(
        &root,
        &guard.editor_meta,
        guard.workspace.root(),
        &game_root_of(&guard),
    );
    guard.ui().set_build_output("".into());
    match plan {
        Ok(plan) => {
            let text = format!(
                "{} {}\n\n{}",
                plan.tool,
                plan.project.display(),
                plan.dependency_dirs
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect::<Vec<_>>()
                    .join("\n")
            );
            guard.ui().set_build_summary(text.into());
            guard.build_plan = Some(plan);
            guard.ui().set_build_can_run(true);
        }
        Err(error) => {
            guard.ui().set_build_summary(error.to_string().into());
            guard.build_plan = None;
            guard.ui().set_build_can_run(false);
        }
    }
    guard.ui().set_show_build_modal(true);
}

pub fn editor_run_build(state: &Arc<std::sync::Mutex<AppState>>) {
    let mut guard = state.lock().expect("state");
    if guard.build_job.is_some() {
        return;
    }
    let Some(plan) = guard.build_plan.clone() else {
        return;
    };
    guard.ui().set_build_busy(true);
    guard.ui().set_build_can_run(false);
    guard
        .ui()
        .set_build_output(l10n::get(&guard.lang, "Working").into());
    guard.build_job = Some(JobHandle::spawn(
        move |sink| match greg_loader::build_test::run(
            &plan,
            sink.cancelled(),
            std::time::Duration::from_secs(300),
        ) {
            Ok(result) => JobOutcome {
                success: result.success,
                message: serde_json::to_string(&result).unwrap_or_default(),
            },
            Err(error) => JobOutcome {
                success: false,
                message: error.to_string(),
            },
        },
    ));
}

pub fn editor_cancel_build(state: &Arc<std::sync::Mutex<AppState>>) {
    if let Some(job) = state.lock().expect("state").build_job.as_ref() {
        job.cancel();
    }
}

fn drain_build(state: &mut AppState) {
    let Some(job) = state.build_job.as_mut() else {
        return;
    };
    let done = job.drain().into_iter().find_map(|message| match message {
        JobMessage::Done(outcome) => Some(outcome),
        _ => None,
    });
    let Some(done) = done else {
        return;
    };
    state.build_job = None;
    state.ui().set_build_busy(false);
    state.ui().set_build_can_run(state.build_plan.is_some());
    let output = match serde_json::from_str::<greg_loader::build_test::BuildResult>(&done.message) {
        Ok(result) => format!(
            "{} · {} ms · {}\n{}",
            l10n::get(
                &state.lang,
                if result.success {
                    "Build_Passed"
                } else {
                    "Build_Failed"
                }
            ),
            result.elapsed_ms,
            result.log_path.display(),
            result.output_tail
        ),
        Err(_) => done.message,
    };
    state.ui().set_build_output(output.into());
}

/// Starts the background publish.
pub fn editor_publish(state: &Arc<std::sync::Mutex<AppState>>) {
    // Save first (separate lock scope).
    editor_save(state);
    // Extract everything we need while holding the lock, then release it
    // before calling append_log (which locks state internally) to avoid a
    // self-deadlock on the UI thread — that froze the app on upload.
    let (root, steam, manual, mut meta) = {
        let guard = state.lock().expect("state");
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
        let manual = if guard.editor_changelog.trim().is_empty() {
            None
        } else {
            Some(guard.editor_changelog.clone())
        };
        let steam = Arc::clone(&guard.steam);
        let meta = guard.editor_meta.clone();
        (root, steam, manual, meta)
    };
    // Safe: no state lock held here — append_log locks state itself.
    AppState::append_log(state, "Upload started.");
    let mut guard = state.lock().expect("state");
    guard.ui().set_ed_publishing(true);
    guard.ui().set_ed_progress(0.0);
    guard.ui().set_ed_status("Uploading…".into());
    let job = JobHandle::spawn(move |sink| {
        let outcome = steam.publish(
            &root,
            &mut meta,
            manual.as_deref(),
            &|p| sink.progress(p),
            &|s| {
                sink.status(s.to_string());
                sink.log("Upload progress updated.".to_string());
            },
            sink.cancelled(),
        );
        // File-log the lifecycle (the UI log alone is lost on a hang/kill).
        if outcome.success {
            let _ = greg_platform::workspace::save_metadata(&root, &meta);
            greg_platform::filelog::FileLog::shared().info("Upload finished.");
        } else {
            greg_platform::filelog::FileLog::shared().info("Upload failed.");
        }
        crate::worker::JobOutcome {
            success: outcome.success,
            message: if outcome.success {
                "Upload finished.".to_string()
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

/// Starts a Modstore publish of the open project (presigned upload +
/// submit, shared flow with the CLI). Reuses the publish job slot,
/// progress bar and log; the outcome names mod, release and scan status.
pub fn editor_publish_modstore(state: &Arc<std::sync::Mutex<AppState>>) {
    use greg_modstore::publish::{PublishMetadata, PublishPayload, PublishPlan};
    use greg_modstore::upload_client::ModStoreUploadClient;

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
    let Some(token) = guard
        .session
        .as_ref()
        .map(|s| s.access_token.clone())
        .filter(|t| !t.trim().is_empty())
    else {
        ui.set_ed_status("Login required — connect the Modstore first.".into());
        return;
    };
    let meta = guard.editor_meta.clone();
    let manual = guard.editor_changelog.clone();
    let urls = store_base_urls(&guard);
    let rt = guard.runtime.handle().clone();
    guard.ui().set_ed_publishing(true);
    guard.ui().set_ed_progress(0.0);
    guard
        .ui()
        .set_ed_status("Packaging for the Modstore…".into());
    AppState::append_log(state, "Modstore upload started.");
    let job = JobHandle::spawn(move |sink| {
        let content = root.join("content");
        let folder = root.file_name().and_then(|n| n.to_str()).unwrap_or("mod");
        let payload = PublishPayload::Directory {
            dir: content,
            file_name: format!("{folder}.zip"),
        };
        let description =
            greg_core::docs::project_docs::resolve_modstore_description(&root, &meta.description)
                .text;
        let changelog =
            greg_core::docs::project_docs::resolve_modstore_changelog(&root, &meta.version, {
                let manual = manual.trim();
                (!manual.is_empty()).then_some(manual)
            })
            .text;
        let plan = match PublishPlan::from_project(
            &payload,
            &PublishMetadata::from_workshop(&meta, description, changelog),
        ) {
            Ok(plan) => plan,
            Err(e) => {
                return crate::worker::JobOutcome {
                    success: false,
                    message: format!("Packaging failed: {e}"),
                }
            }
        };
        let client = match ModStoreUploadClient::new(urls).map(|c| c.with_token(token)) {
            Ok(client) => client,
            Err(e) => {
                return crate::worker::JobOutcome {
                    success: false,
                    message: e.to_string(),
                }
            }
        };
        let progress_sink = sink.clone();
        let outcome = rt.block_on(plan.execute(&client, &|p| {
            progress_sink.progress(f32::from(p) / 100.0);
            progress_sink.status(format!("Modstore upload {p}%"));
        }));
        match outcome {
            Ok(response) => {
                if !response.mod_id.trim().is_empty() {
                    let mut saved = meta.clone();
                    saved.modstore_id = response.mod_id.clone();
                    let _ = greg_platform::workspace::save_metadata(&root, &saved);
                }
                greg_platform::filelog::FileLog::shared().info("Modstore upload finished.");
                crate::worker::JobOutcome {
                    success: true,
                    message: format!(
                        "Modstore: mod {} · release {} · scan {}",
                        response.mod_id, response.release_id, response.scan_status
                    ),
                }
            }
            Err(e) => {
                greg_platform::filelog::FileLog::shared().info("Modstore upload failed.");
                crate::worker::JobOutcome {
                    success: false,
                    message: format!("Modstore upload failed: {e}"),
                }
            }
        }
    });
    guard.publish_job = Some(job);
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
        // NOTE: run_editor_checks() is intentionally NOT called here. It runs
        // synchronously on the UI thread (hashes every content byte, converts
        // every README to BBCode) and blocks the UI for large mods. Checks are
        // re-run reactively via editor_changed() on the next UI tick anyway.
        refresh_projects(state);
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
    let root = guard.workspace.workshop_dir();
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
    let target_id = detail_id.or_else(|| {
        search
            .trim()
            .parse::<u64>()
            .ok()
            .filter(|&id| greg_core::limits::is_usable_workshop_id(id))
    });
    let mut page = guard.ui().get_br_page();
    if page < 1 {
        page = 1;
        guard.ui().set_br_page(1);
    }
    guard.ui().set_br_busy(true);
    let loading = l10n::get(&guard.lang, "Loading");
    guard.ui().set_br_status(loading.into());
    let job = JobHandle::spawn(move |sink| {
        let cancel = sink.cancelled().clone();
        let outcome = if let Some(id) = target_id {
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
            guard.log_line("Workshop subscription changed.");
            drop(guard);
            browse_go(state, Some(id));
        }
        Err(_) => guard.log_line("Workshop subscription change failed."),
    }
}

/// Review a managed install independently of the authoring-project import.
pub fn browse_review_install(state: &Arc<std::sync::Mutex<AppState>>) {
    let guard = state.lock().expect("state");
    if guard.browse_job.is_some() || guard.browse_detail_id.is_none() {
        return;
    }
    guard.ui().set_managed_install_target(
        guard
            .workspace
            .game_modding_dir()
            .to_string_lossy()
            .into_owned()
            .into(),
    );
    guard.ui().set_show_managed_install_modal(true);
}

/// Download to Steam's cache and atomically import into the manager base.
pub fn browse_install(state: &Arc<std::sync::Mutex<AppState>>) {
    if guard_game_running(state) {
        return;
    }
    let mut guard = state.lock().expect("state");
    if guard.browse_job.is_some() {
        return;
    }
    let Some(id) = guard.browse_detail_id else {
        return;
    };
    let backend = Arc::clone(&guard.backend);
    let base = guard.workspace.root().to_path_buf();
    guard.ui().set_show_managed_install_modal(false);
    guard.ui().set_br_busy(true);
    guard
        .ui()
        .set_br_status(l10n::get(&guard.lang, "Working").into());
    guard.browse_job = Some(JobHandle::spawn(move |sink| {
        let cancel = sink.cancelled().clone();
        let source = match backend.download(id, &|p| sink.progress(p), &cancel) {
            Ok(source) => source,
            Err(error) => {
                return JobOutcome {
                    success: false,
                    message: error.to_string(),
                }
            }
        };
        if cancel.load(std::sync::atomic::Ordering::Relaxed) {
            return JobOutcome {
                success: false,
                message: "Cancelled".into(),
            };
        }
        let outcome = greg_loader::sync::sync_managed_item(id, &source, &base);
        match outcome.path.filter(|_| outcome.success) {
            Some(path) => JobOutcome {
                success: true,
                message: format!("MANAGED:{}", path.display()),
            },
            None => JobOutcome {
                success: false,
                message: outcome
                    .error
                    .unwrap_or_else(|| "Managed import failed".into()),
            },
        }
    }));
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
    if let Some(path) = outcome.message.strip_prefix("MANAGED:") {
        state
            .ui()
            .set_br_status(l10n::format(&state.lang, "Workspace_InstallDone", &[&path]).into());
        local_refresh(state);
        return;
    }
    if let Some(payload) = outcome.message.strip_prefix("FULL:") {
        match serde_json::from_str::<greg_steam::models::ItemDetail>(payload) {
            Ok(detail) => {
                let id_str = detail.published_file_id.to_string();
                state.browse_detail_id = Some(detail.published_file_id);
                state.ui().set_br_has_detail(true);
                state.ui().set_br_d_title(detail.title.clone().into());
                let author = if detail.owner_name.trim().is_empty() {
                    if detail.owner_steam_id != 0 {
                        detail.owner_steam_id.to_string()
                    } else {
                        "—".to_string()
                    }
                } else {
                    detail.owner_name.clone()
                };
                state.ui().set_br_d_author(author.clone().into());
                let status_state = if detail.status_state.trim().is_empty() {
                    detail.visibility.clone()
                } else {
                    format!("{} · {}", detail.status_state, detail.source)
                };
                state.ui().set_br_d_status_state(status_state.into());
                state.ui().set_br_d_meta(
                    format!(
                        "ID {} · by {} · {:.1} / 5 · {}",
                        detail.published_file_id,
                        author,
                        detail.score,
                        greg_core::util::format_bytes(detail.size_bytes as i64)
                    )
                    .into(),
                );
                state.ui().set_br_d_tags(detail.tags.join(", ").into());
                state.ui().set_br_d_desc(detail.description.clone().into());
                state
                    .ui()
                    .set_br_d_deps(detail.dependency_hint_compact.clone().into());
                state.ui().set_br_d_subscribed(detail.is_subscribed);
                let cached_preview = state
                    .thumb_done
                    .get(&format!("workshop:{id_str}"))
                    .filter(|p| p.is_file())
                    .cloned();
                state
                    .ui()
                    .set_br_d_preview(thumb_of(cached_preview.as_deref()));
                if !detail.preview_image_url.trim().is_empty() {
                    queue_thumbs(
                        state,
                        vec![(
                            format!("workshop:{id_str}"),
                            detail.preview_image_url.clone(),
                        )],
                    );
                }
                if !state
                    .browse_items
                    .iter()
                    .any(|(existing, _)| *existing == detail.published_file_id)
                {
                    state
                        .browse_items
                        .push((detail.published_file_id, detail.title.clone()));
                    let row = BrowseRow {
                        id: id_str.into(),
                        title: detail.title.into(),
                        score: format!("{:.1} / 5", detail.score).into(),
                        thumb: thumb_of(cached_preview.as_deref()),
                    };
                    state
                        .ui()
                        .set_br_items(ModelRc::from(Rc::new(VecModel::from(vec![row]))));
                    state.ui().set_br_total(1);
                }
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
                        .map(|i| {
                            (
                                format!("workshop:{}", i.published_file_id),
                                i.preview_image_url.clone(),
                            )
                        })
                        .collect(),
                );
                let rows: Vec<BrowseRow> = items
                    .into_iter()
                    .map(|i| BrowseRow {
                        id: i.published_file_id.to_string().into(),
                        title: i.title.into(),
                        score: format!("{:.1} / 5", i.score).into(),
                        thumb: thumb_of(
                            state
                                .thumb_done
                                .get(&format!("workshop:{}", i.published_file_id))
                                .map(PathBuf::as_path),
                        ),
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
    let game_root = guard.workspace.game_modding_dir();
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
    let game_root = guard.workspace.game_modding_dir();
    let target = if is_update {
        guard
            .store_updates
            .get(index as usize)
            .map(|u| StoreTarget::Update(u.clone()))
    } else {
        guard
            .ui()
            .get_st_items()
            .row_data(index as usize)
            .and_then(|row| {
                guard
                    .store_items
                    .iter()
                    .find(|item| item.slug == row.slug.as_str())
                    .cloned()
            })
            .map(StoreTarget::Item)
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
                    .map(|i| {
                        (
                            format!("store:{}", i.slug),
                            i.image_url.clone().unwrap_or_default(),
                        )
                    })
                    .collect(),
            );
            store_filter(state);
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

// Slint Image is deliberately not Send. Keep presentation values on the UI
// thread, while AppState and its domain/worker data remain Send behind Mutex.
#[derive(Default)]
struct LocalRows {
    root: PathBuf,
    kind: Option<greg_loader::local_content::LocalContentKind>,
    rows: Vec<LocalRow>,
}

thread_local! {
    static LOCAL_ROWS: std::cell::RefCell<LocalRows> = std::cell::RefCell::new(LocalRows::default());
}

fn local_kind_of(state: &AppState) -> greg_loader::local_content::LocalContentKind {
    use greg_loader::local_content::LocalContentKind as K;
    match state.ui().get_current_page().as_str() {
        "myplugins" => K::Plugins,
        "mylibs" => K::Libs,
        _ => K::Mods,
    }
}

/// Requests a background scan; cached rows remain available immediately.
/// Repeated requests coalesce into one follow-up scan rather than spawning
/// unbounded filesystem workers while a user switches pages.
pub fn local_refresh(state: &mut AppState) {
    request_local_scan(state, true);
}

fn request_local_scan(state: &mut AppState, force: bool) {
    let kind = local_kind_of(state);
    let root = state.workspace.game_modding_dir();
    let title_key = match kind {
        greg_loader::local_content::LocalContentKind::Mods => "Ui_Nav_MyMods",
        greg_loader::local_content::LocalContentKind::Plugins => "Ui_Nav_MyPlugins",
        greg_loader::local_content::LocalContentKind::Libs => "Ui_Nav_MyLibs",
    };
    state
        .ui()
        .set_local_title(l10n::get(&state.lang, title_key).to_uppercase().into());
    let same_cache = state.local_cache_kind == Some(kind) && state.local_cache_root == root;
    if !same_cache {
        state.local_entries.clear();
        LOCAL_ROWS.with(|cache| *cache.borrow_mut() = LocalRows::default());
        state.local_sig.clear();
        state.local_cache_root = root.clone();
        state.local_cache_kind = Some(kind);
        local_filter_changed(state);
    }
    if state.local_scan_running {
        if force {
            state.local_scan_generation = state.local_scan_generation.wrapping_add(1);
            state.local_scan_pending = true;
        }
        return;
    }
    state.local_scan_generation = state.local_scan_generation.wrapping_add(1);
    let generation = state.local_scan_generation;
    let previous_signature = state.local_sig.clone();
    let tx = state.events_tx.clone();
    state.local_scan_running = true;
    state.runtime.spawn_blocking(move || {
        let signature = greg_loader::local_content::signature_managed(&root, kind);
        let entries = if force || !same_cache || signature != previous_signature {
            Some(greg_loader::local_content::scan_managed(&root, kind))
        } else {
            None
        };
        let _ = tx.send(UiEvent::LocalScanDone {
            generation,
            root,
            kind,
            signature,
            entries,
        });
    });
}

fn finish_local_scan(
    state: &mut AppState,
    generation: u64,
    root: PathBuf,
    kind: greg_loader::local_content::LocalContentKind,
    signature: String,
    entries: Option<Vec<greg_loader::local_content::LocalContentEntry>>,
) {
    state.local_scan_running = false;
    if generation == state.local_scan_generation
        && root == state.workspace.game_modding_dir()
        && kind == local_kind_of(state)
    {
        state.local_sig = signature;
        if let Some(entries) = entries {
            let rows = entries
                .iter()
                .enumerate()
                .map(|(i, entry)| LocalRow {
                    name: entry.name.clone().into(),
                    detail: entry.detail.clone().into(),
                    enabled: entry.enabled,
                    index: i as i32,
                    thumb: thumb_of(
                        greg_loader::local_content::sibling_preview(&entry.path).as_deref(),
                    ),
                })
                .collect();
            LOCAL_ROWS.with(|cache| {
                *cache.borrow_mut() = LocalRows {
                    root: root.clone(),
                    kind: Some(kind),
                    rows,
                }
            });
            state.local_entries = entries;
            state.local_cache_root = root;
            state.local_cache_kind = Some(kind);
            local_filter_changed(state);
        }
    }
    if std::mem::take(&mut state.local_scan_pending) {
        request_local_scan(state, true);
    }
}

/// Repaints a search query from cached rows only: no directory scans, metadata
/// lookups or thumbnail reads occur on a keystroke. Stable row indices continue
/// to address the full local entry cache for toggle/remove actions.
pub fn local_filter_changed(state: &mut AppState) {
    let query = state.ui().get_local_search().to_string().to_lowercase();
    let mut active = Vec::new();
    let mut inactive = Vec::new();
    LOCAL_ROWS.with(|cache| {
        let cache = cache.borrow();
        if cache.root != state.local_cache_root || cache.kind != state.local_cache_kind {
            return;
        }
        for row in &cache.rows {
            if !query.is_empty() && !row.name.to_lowercase().contains(&query) {
                continue;
            }
            if row.enabled {
                active.push(row.clone());
            } else {
                inactive.push(row.clone());
            }
        }
    });
    let (n_active, n_inactive) = (active.len(), inactive.len());
    state.ui().set_local_active_rows(model_of_rows(active));
    state.ui().set_local_inactive_rows(model_of_rows(inactive));
    let root = state.workspace.game_modding_dir();
    let summary = l10n::format(
        &state.lang,
        "Ui_Local_StatusSummary",
        &[&n_active, &n_inactive, &root.display()],
    );
    state.ui().set_local_status(summary.into());
}

/// True on My Mods / My Plugins / My Libs.
fn is_local_page(state: &AppState) -> bool {
    matches!(
        state.ui().get_current_page().as_str(),
        "mymods" | "myplugins" | "mylibs"
    )
}

/// Periodic metadata checks run on the worker too, avoiding disk stalls while
/// the UI mutex is held. Existing scans coalesce automatic checks.
fn auto_refresh_local(state: &mut AppState) {
    if is_local_page(state) && !state.local_scan_running {
        request_local_scan(state, false);
    }
}

/// Toggles an entry by `local_entries` index (`LocalRow.index`).
pub fn local_toggle(state: &Arc<std::sync::Mutex<AppState>>, index: i32) {
    if guard_game_running(state) {
        return;
    }
    let entry = {
        let guard = state.lock().expect("state");
        guard.local_entries.get(index as usize).cloned()
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
        Err(e) => {
            let msg = format!("Toggle failed: {e}");
            AppState::append_log(state, &msg);
            state
                .lock()
                .expect("state")
                .ui()
                .set_local_status(msg.into());
        }
    }
}

/// Opens the confirm dialog for a removal.
pub fn local_remove(state: &Arc<std::sync::Mutex<AppState>>, index: i32) {
    if guard_game_running(state) {
        return;
    }
    let mut guard = state.lock().expect("state");
    let entry = guard.local_entries.get(index as usize).cloned();
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
    let dir = match local_kind_of(&guard) {
        greg_loader::local_content::LocalContentKind::Mods => guard.workspace.mods_dir(),
        greg_loader::local_content::LocalContentKind::Plugins => guard.workspace.melons_dir(),
        greg_loader::local_content::LocalContentKind::Libs => guard.workspace.userlibs_dir(),
    };
    let _ = greg_platform::process::open_folder(&dir);
}

/// Rebuilds the `lib-catalog` model for the "Install Libraries" modal.
pub fn lib_refresh_catalog(state: &mut AppState) {
    use greg_loader::local_content::{is_framework_lib_installed, FRAMEWORK_LIBRARIES};
    let root = state.workspace.game_modding_dir();
    let has_root = root.is_dir();
    if state.lib_selected.len() != FRAMEWORK_LIBRARIES.len() {
        state.lib_selected = vec![false; FRAMEWORK_LIBRARIES.len()];
    }
    let mut rows: Vec<LibInstallRow> = Vec::with_capacity(FRAMEWORK_LIBRARIES.len());
    let mut installed_count = 0usize;
    let mut selected_count = 0usize;
    for (i, spec) in FRAMEWORK_LIBRARIES.iter().enumerate() {
        let installed = has_root && is_framework_lib_installed(&root, spec.file_name);
        if installed {
            installed_count += 1;
        }
        let selected = state.lib_selected.get(i).copied().unwrap_or(false);
        if selected {
            selected_count += 1;
        }
        rows.push(LibInstallRow {
            name: spec.file_name.into(),
            detail: format!("v{} · {}", spec.nuget_version, spec.detail).into(),
            installed,
            selected,
            index: i as i32,
        });
    }
    state.ui().set_lib_catalog(model_of_rows(rows));
    if !state.ui().get_lib_busy() {
        let total = FRAMEWORK_LIBRARIES.len();
        let status = if !has_root {
            "Managed content folder is unavailable — check the workspace in Settings.".to_string()
        } else {
            format!(
                "{selected_count} selected · {installed_count}/{total} installed in {}",
                greg_loader::local_content::user_libs_dir(&root).display()
            )
        };
        state.ui().set_lib_status(status.into());
    }
}

/// Opens the "Install Libraries" modal and pre-selects missing libraries.
pub fn lib_open_modal(state: &Arc<std::sync::Mutex<AppState>>) {
    use greg_loader::local_content::{is_framework_lib_installed, FRAMEWORK_LIBRARIES};
    let mut guard = state.lock().expect("state");
    let root = guard.workspace.game_modding_dir();
    let has_root = root.is_dir();
    guard.lib_selected = FRAMEWORK_LIBRARIES
        .iter()
        .map(|spec| !has_root || !is_framework_lib_installed(&root, spec.file_name))
        .collect();
    lib_refresh_catalog(&mut guard);
    guard.ui().set_show_lib_modal(true);
}

/// Toggles the selection state of a framework library row in the modal.
pub fn lib_toggle_select(state: &Arc<std::sync::Mutex<AppState>>, index: i32) {
    let mut guard = state.lock().expect("state");
    if let Some(slot) = guard.lib_selected.get_mut(index as usize) {
        *slot = !*slot;
    }
    lib_refresh_catalog(&mut guard);
}

/// Selects all currently missing framework libraries (deselects already-installed ones).
pub fn lib_select_missing(state: &Arc<std::sync::Mutex<AppState>>) {
    use greg_loader::local_content::{is_framework_lib_installed, FRAMEWORK_LIBRARIES};
    let mut guard = state.lock().expect("state");
    let root = guard.workspace.game_modding_dir();
    let has_root = root.is_dir();
    guard.lib_selected = FRAMEWORK_LIBRARIES
        .iter()
        .map(|spec| !has_root || !is_framework_lib_installed(&root, spec.file_name))
        .collect();
    lib_refresh_catalog(&mut guard);
}

/// Selects all framework libraries (or deselects all if all are already selected).
pub fn lib_select_all(state: &Arc<std::sync::Mutex<AppState>>) {
    let mut guard = state.lock().expect("state");
    let all_selected = guard.lib_selected.iter().all(|&b| b);
    guard.lib_selected.fill(!all_selected);
    lib_refresh_catalog(&mut guard);
}

/// Closes the "Install Libraries" modal when not busy.
pub fn lib_close_modal(state: &Arc<std::sync::Mutex<AppState>>) {
    let guard = state.lock().expect("state");
    if !guard.ui().get_lib_busy() {
        guard.ui().set_show_lib_modal(false);
    }
}

/// Starts a background job to install all selected framework libraries into `UserLibs`.
pub fn lib_install_selected(state: &Arc<std::sync::Mutex<AppState>>) {
    use greg_loader::local_content::{
        install_framework_library, FrameworkLibrarySpec, FRAMEWORK_LIBRARIES,
    };
    if guard_game_running(state) {
        let guard = state.lock().expect("state");
        let msg = guard.ui().get_local_status();
        guard.ui().set_lib_status(msg);
        return;
    }
    let mut guard = state.lock().expect("state");
    if guard.lib_job.is_some() {
        return;
    }
    let root = guard.workspace.game_modding_dir();
    if !root.is_dir() {
        guard.ui().set_lib_status(
            "Managed content folder is unavailable — check the workspace in Settings.".into(),
        );
        return;
    }
    let chosen: Vec<FrameworkLibrarySpec> = FRAMEWORK_LIBRARIES
        .iter()
        .enumerate()
        .filter(|(i, _)| guard.lib_selected.get(*i).copied().unwrap_or(false))
        .map(|(_, spec)| *spec)
        .collect();
    if chosen.is_empty() {
        guard
            .ui()
            .set_lib_status("Select at least one library to install.".into());
        return;
    }
    let handle = guard.runtime.handle().clone();
    guard.ui().set_lib_busy(true);
    guard.ui().set_lib_status(
        format!(
            "Installing {} {}…",
            chosen.len(),
            if chosen.len() == 1 {
                "library"
            } else {
                "libraries"
            }
        )
        .into(),
    );
    guard.lib_job = Some(JobHandle::spawn(move |sink| {
        let total = chosen.len();
        let mut installed = 0usize;
        for (idx, spec) in chosen.iter().enumerate() {
            sink.status(format!(
                "Installing {} ({}/{})…",
                spec.file_name,
                idx + 1,
                total
            ));
            let cancel = sink.cancelled().clone();
            match handle.block_on(install_framework_library(&root, spec, &cancel)) {
                Ok(path) => {
                    installed += 1;
                    sink.log(format!(
                        "Installed {} -> {}",
                        spec.file_name,
                        path.display()
                    ));
                }
                Err(e) => {
                    return JobOutcome {
                        success: false,
                        message: format!("Failed to install {}: {e}", spec.file_name),
                    };
                }
            }
        }
        JobOutcome {
            success: true,
            message: format!(
                "Installed {installed} {} into UserLibs.",
                if installed == 1 {
                    "library"
                } else {
                    "libraries"
                }
            ),
        }
    }));
}

fn drain_lib_install(state: &mut AppState) {
    let mut done: Option<JobOutcome> = None;
    let mut finished = false;
    let mut status_lines: Vec<String> = Vec::new();
    let messages = match state.lib_job.as_mut() {
        Some(job) => {
            let messages = job.drain();
            finished = job.is_finished();
            messages
        }
        None => Vec::new(),
    };
    for msg in messages {
        match msg {
            JobMessage::Progress(_) => {}
            JobMessage::Status(s) => status_lines.push(s),
            JobMessage::Log(line) => AppState::append_log_ref(state, &line),
            JobMessage::Done(outcome) => done = Some(outcome),
        }
    }
    if let Some(last) = status_lines.last() {
        state.ui().set_lib_status(last.clone().into());
    }
    let Some(outcome) = done else {
        if finished {
            state.lib_job = None;
            state.ui().set_lib_busy(false);
        }
        return;
    };
    state.lib_job = None;
    state.ui().set_lib_busy(false);
    local_refresh(state);
    if outcome.success {
        state.lib_selected.fill(false);
        lib_refresh_catalog(state);
        state.ui().set_lib_status(outcome.message.clone().into());
        state.ui().set_local_status(outcome.message.into());
        state.ui().set_show_lib_modal(false);
    } else {
        lib_refresh_catalog(state);
        state.ui().set_lib_status(outcome.message.into());
    }
}

/// Rebuilds the `plugin-catalog` model for the "Browse Plugins" modal.
pub fn plugin_refresh_catalog(state: &mut AppState) {
    use greg_loader::local_content::{is_managed_plugin_installed, FRAMEWORK_PLUGINS};
    let root = state.workspace.game_modding_dir();
    let has_root = root.is_dir();
    if state.plugin_selected.len() != FRAMEWORK_PLUGINS.len() {
        state.plugin_selected = vec![false; FRAMEWORK_PLUGINS.len()];
    }
    let mut rows: Vec<LibInstallRow> = Vec::with_capacity(FRAMEWORK_PLUGINS.len());
    let mut installed_count = 0usize;
    let mut selected_count = 0usize;
    for (i, spec) in FRAMEWORK_PLUGINS.iter().enumerate() {
        let installed = has_root && is_managed_plugin_installed(&root, spec.file_name);
        if installed {
            installed_count += 1;
        }
        let selected = state.plugin_selected.get(i).copied().unwrap_or(false);
        if selected {
            selected_count += 1;
        }
        rows.push(LibInstallRow {
            name: spec.display_name.into(),
            detail: format!("{} · {}", spec.file_name, spec.detail).into(),
            installed,
            selected,
            index: i as i32,
        });
    }
    state.ui().set_plugin_catalog(model_of_rows(rows));
    if !state.ui().get_plugin_busy() {
        let total = FRAMEWORK_PLUGINS.len();
        let status = if !has_root {
            l10n::get(&state.lang, "Ui_Local_GameNotFound")
        } else {
            format!(
                "{selected_count} selected · {installed_count}/{total} installed in {}",
                root.join("Melons").display()
            )
        };
        state.ui().set_plugin_status(status.into());
    }
}

/// Opens the "Browse Plugins" modal and pre-selects missing plugins.
pub fn plugin_open_modal(state: &Arc<std::sync::Mutex<AppState>>) {
    use greg_loader::local_content::{is_managed_plugin_installed, FRAMEWORK_PLUGINS};
    let mut guard = state.lock().expect("state");
    let root = guard.workspace.game_modding_dir();
    let has_root = root.is_dir();
    guard.plugin_selected = FRAMEWORK_PLUGINS
        .iter()
        .map(|spec| !has_root || !is_managed_plugin_installed(&root, spec.file_name))
        .collect();
    plugin_refresh_catalog(&mut guard);
    guard.ui().set_show_plugin_modal(true);
}

/// Toggles the selection state of a plugin row in the "Browse Plugins" modal.
pub fn plugin_toggle_select(state: &Arc<std::sync::Mutex<AppState>>, index: i32) {
    let mut guard = state.lock().expect("state");
    if let Some(slot) = guard.plugin_selected.get_mut(index as usize) {
        *slot = !*slot;
    }
    plugin_refresh_catalog(&mut guard);
}

/// Selects all currently missing plugins (deselects already-installed ones).
pub fn plugin_select_missing(state: &Arc<std::sync::Mutex<AppState>>) {
    use greg_loader::local_content::{is_managed_plugin_installed, FRAMEWORK_PLUGINS};
    let mut guard = state.lock().expect("state");
    let root = guard.workspace.game_modding_dir();
    let has_root = root.is_dir();
    guard.plugin_selected = FRAMEWORK_PLUGINS
        .iter()
        .map(|spec| !has_root || !is_managed_plugin_installed(&root, spec.file_name))
        .collect();
    plugin_refresh_catalog(&mut guard);
}

/// Selects all plugins in the catalog (or deselects all if all are already selected).
pub fn plugin_select_all(state: &Arc<std::sync::Mutex<AppState>>) {
    let mut guard = state.lock().expect("state");
    let all_selected = guard.plugin_selected.iter().all(|&b| b);
    guard.plugin_selected.fill(!all_selected);
    plugin_refresh_catalog(&mut guard);
}

/// Closes the "Browse Plugins" modal when not busy.
pub fn plugin_close_modal(state: &Arc<std::sync::Mutex<AppState>>) {
    let guard = state.lock().expect("state");
    if !guard.ui().get_plugin_busy() {
        guard.ui().set_show_plugin_modal(false);
    }
}

/// Starts a background job to install all selected plugins into `Plugins/`.
pub fn plugin_install_selected(state: &Arc<std::sync::Mutex<AppState>>) {
    use greg_loader::local_content::{
        install_managed_plugin_from_release, PluginCatalogSpec, FRAMEWORK_PLUGINS,
    };
    if guard_game_running(state) {
        let guard = state.lock().expect("state");
        let msg = guard.ui().get_local_status();
        guard.ui().set_plugin_status(msg);
        return;
    }
    let mut guard = state.lock().expect("state");
    if guard.plugin_job.is_some() {
        return;
    }
    let root = guard.workspace.game_modding_dir();
    if !root.is_dir() {
        let msg = l10n::get(&guard.lang, "Ui_Local_GameNotFound");
        guard.ui().set_plugin_status(msg.into());
        return;
    }
    let chosen: Vec<PluginCatalogSpec> = FRAMEWORK_PLUGINS
        .iter()
        .enumerate()
        .filter(|(i, _)| guard.plugin_selected.get(*i).copied().unwrap_or(false))
        .map(|(_, spec)| *spec)
        .collect();
    if chosen.is_empty() {
        guard
            .ui()
            .set_plugin_status("Select at least one plugin to install.".into());
        return;
    }
    let handle = guard.runtime.handle().clone();
    guard.ui().set_plugin_busy(true);
    let installing = l10n::get(&guard.lang, "Ui_Modal_Installing");
    guard.ui().set_plugin_status(installing.into());
    guard.plugin_job = Some(JobHandle::spawn(move |sink| {
        let total = chosen.len();
        let mut installed = 0usize;
        for (idx, spec) in chosen.iter().enumerate() {
            sink.status(format!(
                "Installing {} ({}/{})…",
                spec.display_name,
                idx + 1,
                total
            ));
            let cancel = sink.cancelled().clone();
            match handle.block_on(install_managed_plugin_from_release(&root, spec, &cancel)) {
                Ok((tag, path)) => {
                    installed += 1;
                    sink.log(format!(
                        "Installed {} ({tag}) -> {}",
                        spec.file_name,
                        path.display()
                    ));
                }
                Err(e) => {
                    return JobOutcome {
                        success: false,
                        message: format!("Failed to install {}: {e}", spec.display_name),
                    };
                }
            }
        }
        JobOutcome {
            success: true,
            message: format!(
                "Installed {installed} {} into GameModding/Melons.",
                if installed == 1 { "plugin" } else { "plugins" }
            ),
        }
    }));
}

/// "Install Plugins" button on the My Plugins view: automatically downloads and installs
/// the latest version of the IL2CPP Assembly Fixer directly from Datacentermods (preferred)
/// with automatic fallback to GitHub releases.
pub fn plugin_install_default(state: &Arc<std::sync::Mutex<AppState>>) {
    use greg_loader::local_content::install_managed_latest_il2cpp_fixer;
    if guard_game_running(state) {
        return;
    }
    let mut guard = state.lock().expect("state");
    if guard.plugin_job.is_some() {
        return;
    }
    let root = guard.workspace.game_modding_dir();
    if !root.is_dir() {
        let msg = l10n::get(&guard.lang, "Ui_Local_GameNotFound");
        guard.ui().set_local_status(msg.clone().into());
        guard.ui().set_plugin_status(msg.into());
        return;
    }
    let handle = guard.runtime.handle().clone();
    guard.ui().set_plugin_busy(true);
    let status_msg =
        "Downloading latest IL2CPP Assembly Fixer (Datacentermods / GitHub)…".to_string();
    guard.ui().set_local_status(status_msg.clone().into());
    guard.ui().set_plugin_status(status_msg.into());
    guard.plugin_job = Some(JobHandle::spawn(move |sink| {
        sink.status(
            "Downloading latest IL2CPP Assembly Fixer (Datacentermods / GitHub)…".to_string(),
        );
        let cancel = sink.cancelled().clone();
        match handle.block_on(install_managed_latest_il2cpp_fixer(&root, &cancel)) {
            Ok((tag, path)) => {
                sink.log(format!(
                    "Installed IL2CPP Assembly Fixer ({tag}) -> {}",
                    path.display()
                ));
                JobOutcome {
                    success: true,
                    message: format!(
                        "Installed IL2CPP Assembly Fixer ({tag}) into {}.",
                        path.display()
                    ),
                }
            }
            Err(e) => JobOutcome {
                success: false,
                message: format!("Failed to install IL2CPP Assembly Fixer: {e}"),
            },
        }
    }));
}

fn drain_plugin_install(state: &mut AppState) {
    let mut done: Option<JobOutcome> = None;
    let mut finished = false;
    let mut status_lines: Vec<String> = Vec::new();
    let messages = match state.plugin_job.as_mut() {
        Some(job) => {
            let messages = job.drain();
            finished = job.is_finished();
            messages
        }
        None => Vec::new(),
    };
    for msg in messages {
        match msg {
            JobMessage::Progress(_) => {}
            JobMessage::Status(s) => status_lines.push(s),
            JobMessage::Log(line) => AppState::append_log_ref(state, &line),
            JobMessage::Done(outcome) => done = Some(outcome),
        }
    }
    if let Some(last) = status_lines.last() {
        state.ui().set_plugin_status(last.clone().into());
    }
    let Some(outcome) = done else {
        if finished {
            state.plugin_job = None;
            state.ui().set_plugin_busy(false);
        }
        return;
    };
    state.plugin_job = None;
    state.ui().set_plugin_busy(false);
    local_refresh(state);
    if outcome.success {
        state.plugin_selected.fill(false);
        plugin_refresh_catalog(state);
        state.ui().set_plugin_status(outcome.message.clone().into());
        state.ui().set_local_status(outcome.message.into());
        state.ui().set_show_plugin_modal(false);
    } else {
        plugin_refresh_catalog(state);
        state.ui().set_plugin_status(outcome.message.clone().into());
        state.ui().set_local_status(outcome.message.into());
    }
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
            Ok(()) => AppState::append_log(state, "Local content removed."),
            Err(_) => AppState::append_log(state, "Could not remove local content."),
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

// ---------------------------------------------------------------------------
// Universal Report Assistant (central "Report" page).
// ---------------------------------------------------------------------------

/// Opens the Report Assistant page and resets the draft.
pub fn report_open(state: &Arc<std::sync::Mutex<AppState>>) {
    let guard = state.lock().expect("state");
    guard.ui().set_current_page("report".into());
    guard.ui().set_rp_title(String::new().into());
    guard.ui().set_rp_category_index(0);
    guard.ui().set_rp_description(String::new().into());
    guard.ui().set_rp_steps(String::new().into());
    guard.ui().set_rp_expected(String::new().into());
    guard.ui().set_rp_actual(String::new().into());
    guard.ui().set_rp_logs(String::new().into());
    guard.ui().set_rp_environment(String::new().into());
    guard.ui().set_rp_preview(String::new().into());
    guard.ui().set_rp_status(String::new().into());
    guard.ui().set_rp_submission(String::new().into());
    guard.ui().set_rp_attachments(model_of_strings(&[]));
    guard.ui().set_rp_submit_confirm(false);
    guard.ui().set_rp_submit_mode(String::new().into());
    let labels = [
        "Bug",
        "Crash",
        "Feature",
        "Improvement",
        "Performance",
        "Compatibility",
        "UI/UX",
        "Documentation",
        "Installation",
        "Configuration",
        "Security",
        "Other",
    ];
    guard.ui().set_rp_categories(model_of_strings(
        &labels.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
    ));
}

/// Builds the assistant draft from the current UI state and validates it.
///
/// The draft is stamped with the shared report schema version and source
/// product; the transmission mode is resolved by the caller (operator service
/// when a Modstore session is active, local export otherwise).
fn report_draft(
    ui: &crate::MainWindow,
) -> (
    greg_core::report::ReportData,
    Vec<greg_core::report::ReportValidationIssue>,
) {
    let category = greg_core::report::ReportCategory::all()
        .get(ui.get_rp_category_index().max(0) as usize)
        .map(|c| c.as_str().to_string())
        .unwrap_or_else(|| "other".to_string());
    let mut draft = greg_core::report::ReportData::new(
        greg_core::report::SOURCE_MODMANAGER,
        greg_core::report::MODE_LOCAL_EXPORT,
    );
    draft.title = ui.get_rp_title().to_string();
    draft.category = category;
    draft.description = ui.get_rp_description().to_string();
    draft.steps_to_reproduce = ui.get_rp_steps().to_string();
    draft.expected_behavior = ui.get_rp_expected().to_string();
    draft.actual_behavior = ui.get_rp_actual().to_string();
    draft.logs = ui.get_rp_logs().to_string();
    {
        let model = ui.get_rp_attachments();
        let Some(view) = model.as_any().downcast_ref::<VecModel<SharedString>>() else {
            return (draft, Vec::new());
        };
        for i in 0..view.row_count() {
            if let Some(s) = view.row_data(i) {
                draft.attachments.push(s.to_string());
            }
        }
    }
    let mut environment = ui.get_rp_environment().to_string();
    if environment.trim().is_empty() {
        // Auto-collect version + OS context instead of guessing.
        environment = greg_core::report::ReportEnvironment {
            os: os_display_name(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            game: String::new(),
            loader: String::new(),
        }
        .describe();
    }
    draft.environment = environment;
    let issues = greg_core::report::validate(&draft);
    (draft, issues)
}

/// Human-readable OS name for the auto-collected report environment.
fn os_display_name() -> String {
    let raw = std::env::consts::OS;
    let mut chars = raw.chars();
    let first = chars
        .next()
        .map(|c| c.to_ascii_uppercase())
        .unwrap_or_default();
    let rest: String = chars.collect();
    format!("{first}{rest}")
}

/// Collects buffered application log lines into the report draft.
///
/// Snapshots the bounded UI log buffer, sanitizes the joined lines
/// (redaction + size cap) and fills the report logs field; the Markdown
/// preview is refreshed afterwards so the collected evidence is visible
/// immediately. No log content is ever persisted or transmitted here.
pub fn report_collect_logs(state: &Arc<std::sync::Mutex<AppState>>) {
    let (logs, collected) = {
        let guard = state.lock().expect("state");
        let lines = guard.log_snapshot();
        let logs = lines.join("\n");
        (greg_core::report::sanitize(&logs), lines.len())
    };
    {
        let guard = state.lock().expect("state");
        guard.ui().set_rp_logs(logs.clone().into());
        if collected == 0 {
            guard.ui().set_rp_status(
                "No log entries were collected yet. Add or attach log output manually.".into(),
            );
        }
    }
    report_generate(state);
}

/// Pulls the assistant draft from the UI, validates it and fills the
/// Markdown preview. The generated body is sanitized (redaction + size cap)
/// before it reaches the preview.
pub fn report_generate(state: &Arc<std::sync::Mutex<AppState>>) {
    let guard = state.lock().expect("state");
    let ui = &guard.ui();
    let (mut draft, issues) = report_draft(ui);
    draft.set_transmission_mode(if guard.session.is_some() {
        greg_core::report::ReportTransmissionMode::OperatorService
    } else {
        greg_core::report::ReportTransmissionMode::LocalExport
    });
    if issues.iter().any(|i| i.is_error()) {
        let summary = issues
            .iter()
            .map(|i| format!("{}: {}", i.field, i.message))
            .collect::<Vec<_>>()
            .join(" · ");
        ui.set_rp_status(format!("Validation: {summary}").into());
        return;
    }
    ui.set_rp_preview(greg_core::report::generate_markdown(&draft).into());
    ui.set_rp_status(String::new().into());
}

/// Discards the assistant draft and resets the page.
pub fn report_cancel(state: &Arc<std::sync::Mutex<AppState>>) {
    report_open(state);
}

/// Exports the generated Markdown report body to a user-chosen local file.
///
/// Local export is always available (offline / no backend channel); the written
/// content is sanitized before it leaves the process.
pub fn report_export(state: &Arc<std::sync::Mutex<AppState>>) {
    let (draft, issues) = {
        let guard = state.lock().expect("state");
        let ui = &guard.ui();
        let (draft, issues) = report_draft(ui);
        (draft, issues)
    };
    if issues.iter().any(|i| i.is_error()) {
        let summary = issues
            .iter()
            .map(|i| format!("{}: {}", i.field, i.message))
            .collect::<Vec<_>>()
            .join(" · ");
        let guard = state.lock().expect("state");
        let ui = &guard.ui();
        ui.set_rp_status(format!("Validation: {summary}").into());
        return;
    }
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let Some(path) = rfd::FileDialog::new()
        .set_file_name(format!("gregmodmanager-report-{stamp}.md"))
        .save_file()
    else {
        return;
    };
    if let Err(e) = greg_core::report::export_markdown(&draft, &path) {
        let guard = state.lock().expect("state");
        let ui = &guard.ui();
        ui.set_rp_status(format!("Export failed: {e}").into());
        return;
    }
    let guard = state.lock().expect("state");
    let ui = &guard.ui();
    ui.set_rp_status(format!("Report exported to {}", path.display()).into());
    guard.log_line("Report exported.");
}

/// Opens the submission confirmation step for the current draft.
pub fn report_submit(state: &Arc<std::sync::Mutex<AppState>>) {
    let guard = state.lock().expect("state");
    let ui = &guard.ui();
    ui.set_rp_submit_confirm(true);
    let mode = if guard.session.is_some() {
        "operator service"
    } else {
        "local export"
    };
    ui.set_rp_submit_mode(mode.into());
    ui.set_rp_status("Review the generated report before submitting.".into());
}

/// Closes the submission confirmation without submitting.
pub fn report_confirm_no(state: &Arc<std::sync::Mutex<AppState>>) {
    let guard = state.lock().expect("state");
    guard.ui().set_rp_submit_confirm(false);
}

/// GitHub issue submission configuration.
///
/// Sourced exclusively from environment variables: `GREG_GITHUB_TOKEN`,
/// `GREG_GITHUB_ISSUE_TARGET_REPO` and `GREG_GITHUB_API_URL`.
///
/// Returns `None` when the credentials are not configured — the local export
/// delivery then remains the supported fallback.
fn github_submit_config() -> Option<(String, String, String)> {
    let token = std::env::var("GREG_GITHUB_TOKEN").ok()?.trim().to_string();
    let repo = std::env::var("GREG_GITHUB_ISSUE_TARGET_REPO")
        .ok()?
        .trim()
        .to_string();
    if token.is_empty() || repo.is_empty() {
        return None;
    }
    // Guard against malformed / path-traversing repository identifiers.
    let parts: Vec<&str> = repo.split('/').collect();
    if parts.len() != 2
        || parts
            .iter()
            .any(|p| p.trim().is_empty() || *p == ".." || *p == ".")
    {
        return None;
    }
    let api = std::env::var("GREG_GITHUB_API_URL")
        .ok()
        .map(|v| v.trim().to_string())
        .unwrap_or_else(|| "https://api.github.com".to_string());
    Some((token, repo, api))
}

/// Creates a GitHub issue from the sanitized report markdown.
///
/// Uses the canonical GitHub Issues REST endpoint. The body is already
/// sanitized (redaction + size cap) via `generate_markdown`.
async fn github_create_issue(
    token: String,
    repo: String,
    api_base: String,
    _draft: greg_core::report::ReportData,
    markdown: String,
) -> (bool, String, Option<String>) {
    let url = format!("{api_base}/repos/{repo}/issues");
    let client = reqwest::Client::new();
    // User-authored titles may contain account names, library titles or IDs.
    // Keep them local and use a fixed generic title for remote issue creation.
    let body = serde_json::json!({ "title": "gregModmanager report", "body": markdown });
    match client
        .post(&url)
        .bearer_auth(&token)
        .json(&body)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            let created: serde_json::Value = resp.json().await.unwrap_or(serde_json::json!({}));
            let issue_url = created
                .get("html_url")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            (true, "Issue created.".to_string(), issue_url)
        }
        Ok(resp) => (
            false,
            format!("GitHub API error: status {}", resp.status()),
            None,
        ),
        Err(_e) => (false, "GitHub API unreachable.".to_string(), None),
    }
}

/// Confirms the submission and executes the delivery for the current draft.
///
/// Delivery modes implemented today:
/// - `local_export`: the user picks a save location; the sanitized Markdown
///   issue body is written locally (local fallback / offline support).
/// - `user_direct` / `operator_service`: the corresponding backend channel is
///   not wired in this build yet — the user gets an explanatory status and can
///   still export the report locally. No unverified success is ever claimed.
///
/// When GitHub submission credentials are configured via the environment
/// (`GREG_GITHUB_TOKEN` + `GREG_GITHUB_ISSUE_TARGET_REPO`), the report is
/// delivered as a GitHub issue through the GitHub Issues API instead.
pub fn report_confirm_yes(state: &Arc<std::sync::Mutex<AppState>>) {
    let (draft, issues) = {
        let guard = state.lock().expect("state");
        let ui = &guard.ui();
        let (draft, issues) = report_draft(ui);
        (draft, issues)
    };
    {
        let guard = state.lock().expect("state");
        let ui = &guard.ui();
        if issues.iter().any(|i| i.is_error()) {
            let summary = issues
                .iter()
                .map(|i| format!("{}: {}", i.field, i.message))
                .collect::<Vec<_>>()
                .join(" · ");
            ui.set_rp_status(format!("Validation: {summary}").into());
            ui.set_rp_submit_confirm(false);
            return;
        }
        ui.set_rp_submission("in_progress".into());
        ui.set_rp_status("Submitting…".into());
    }
    // GitHub delivery (env-gated): user_direct-style issue creation via the
    // GitHub Issues API using an environment-provided credential.
    if let Some((token, repo, api_base)) = github_submit_config() {
        let markdown = greg_core::report::generate_markdown(&draft);
        let (tx, rt) = {
            let guard = state.lock().expect("state");
            (guard.events_tx.clone(), guard.runtime.handle().clone())
        };
        rt.spawn(async move {
            let (success, message, issue_url) =
                github_create_issue(token, repo, api_base, draft, markdown).await;
            let _ = tx.send(UiEvent::ReportSubmissionDone {
                success,
                message,
                issue_url,
            });
        });
        return;
    }
    let mode = draft.effective_transmission_mode();
    if mode != greg_core::report::ReportTransmissionMode::LocalExport {
        // User-direct / operator-service channels are not wired in this build
        // yet — never pretend a delivery succeeded; keep the draft local.
        let mode_label = mode.as_str().to_string();
        let guard = state.lock().expect("state");
        let ui = &guard.ui();
        ui.set_rp_submit_confirm(false);
        ui.set_rp_status(
            format!(
                "Submission via {mode_label} is not available in this build yet. \
             The report stays local — use local export to save it."
            )
            .into(),
        );
        ui.set_rp_submission("draft".into());
        return;
    }
    // Local fallback delivery: user-chosen save location.
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let Some(path) = rfd::FileDialog::new()
        .set_file_name(format!("gregmodmanager-report-{stamp}.md"))
        .save_file()
    else {
        let guard = state.lock().expect("state");
        let ui = &guard.ui();
        ui.set_rp_submit_confirm(false);
        ui.set_rp_submission("cancelled".into());
        ui.set_rp_status("Submission cancelled.".into());
        return;
    };
    if let Err(e) = greg_core::report::export_markdown(&draft, &path) {
        let guard = state.lock().expect("state");
        let ui = &guard.ui();
        ui.set_rp_submit_confirm(false);
        ui.set_rp_submission("failed".into());
        ui.set_rp_status(format!("Local export failed: {e}").into());
        return;
    }
    let guard = state.lock().expect("state");
    let ui = &guard.ui();
    ui.set_rp_submit_confirm(false);
    ui.set_rp_submission("submitted".into());
    ui.set_rp_status(format!("Report saved locally: {}", path.display()).into());
    guard.log_line("Report exported.");
}

/// Adds a user-selected attachment file name to the draft.
pub fn report_attach(state: &Arc<std::sync::Mutex<AppState>>) {
    let Some(path) = rfd::FileDialog::new()
        .add_filter(
            "Report attachments",
            &[
                "log", "txt", "md", "json", "cfg", "ini", "png", "jpg", "jpeg", "gif", "pdf",
            ],
        )
        .pick_file()
    else {
        return;
    };
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("attachment")
        .to_string();
    let guard = state.lock().expect("state");
    let mut names: Vec<String> = Vec::new();
    {
        let model = guard.ui().get_rp_attachments();
        let Some(view) = model.as_any().downcast_ref::<VecModel<SharedString>>() else {
            return;
        };
        for i in 0..view.row_count() {
            if let Some(s) = view.row_data(i) {
                names.push(s.to_string());
            }
        }
    }
    if names.iter().any(|n| n == &name) {
        return;
    }
    names.push(name);
    guard.ui().set_rp_attachments(model_of_strings(&names));
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
    if guard.game_session.is_none() {
        let stale = greg_loader::launch::stale_game_pids();
        if let Some(pid) = stale.first() {
            problems.push(Problem {
                severity: "warning",
                source: "Game".into(),
                message: l10n::format(&guard.lang, "Game_StaleRunning", &[pid]),
                action: Some(("stop-game".into(), String::new(), "Stop game".into())),
            });
        }
    }
    // Mods disabled by an unclean vanilla session.
    if game_root.is_dir()
        && greg_loader::launch::mods_state(&game_root)
            == greg_loader::launch::ModsState::DisabledByManager
        && !greg_loader::launch::any_game_process_running()
    {
        problems.push(Problem {
            severity: "warning",
            source: "Game".into(),
            message: "Mods are disabled for vanilla mode — restore them to play with mods.".into(),
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

/// Catalog belongs to the configured manager base.
pub fn packs_path(workspace: &greg_platform::workspace::Workspace) -> PathBuf {
    workspace.configs_dir().join("collections.json")
}

/// Import the former catalog without removing its source.
pub fn load_packs(
    workspace: &greg_platform::workspace::Workspace,
) -> greg_core::models::ModCollectionService {
    let path = packs_path(workspace);
    if !path.exists() {
        if let Some(legacy) =
            greg_platform::paths::app_data_dir().map(|p| p.join("collections.json"))
        {
            if legacy.is_file() {
                let _ = std::fs::copy(legacy, &path);
            }
        }
    }
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|text| greg_core::models::ModCollectionService::load_json(&text).ok())
        .unwrap_or_default()
}

/// Persists the catalog alongside the settings database.
pub fn save_packs(
    workspace: &greg_platform::workspace::Workspace,
    svc: &greg_core::models::ModCollectionService,
) {
    if let Ok(json) = svc.to_json() {
        let _ = std::fs::write(packs_path(workspace), json);
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
    save_packs(&guard.workspace, &guard.pack_service);
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
        save_packs(&guard.workspace, &guard.pack_service);
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
        save_packs(&guard.workspace, &guard.pack_service);
    }
    drop(guard);
    packs_refresh(state);
}

/// Toggles one entry of the selected pack.
pub fn packs_toggle_entry(state: &Arc<std::sync::Mutex<AppState>>, index: i32) {
    let mut guard = state.lock().expect("state");
    if let Some(id) = guard.selected_pack.clone() {
        guard.pack_service.toggle_entry(&id, index as usize);
        save_packs(&guard.workspace, &guard.pack_service);
        refresh_pack_entries(&mut guard);
    }
}

/// Removes one entry of the selected pack.
pub fn packs_remove_entry(state: &Arc<std::sync::Mutex<AppState>>, index: i32) {
    let mut guard = state.lock().expect("state");
    if let Some(id) = guard.selected_pack.clone() {
        guard.pack_service.remove_entry_at(&id, index as usize);
        save_packs(&guard.workspace, &guard.pack_service);
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
    save_packs(&guard.workspace, &guard.pack_service);
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
                guard.log_line("Pack exported.");
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
                save_packs(&guard.workspace, &guard.pack_service);
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
        (
            pack,
            guard.workspace.game_modding_dir(),
            Arc::clone(&guard.steam),
        )
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
            for _error in &report.errors {
                AppState::append_log(state, "Pack apply failed.");
            }
            AppState::append_log(state, "Pack apply completed.");
            report.summary()
        }
        Err(e) => {
            AppState::append_log(state, "Pack apply failed.");
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
    state
        .ui()
        .set_se_workspace(state.workspace.root().to_string_lossy().into_owned().into());
    state
        .ui()
        .set_se_gameroot(state.prefs.get_string("GameRootPath", "").into());
    state.ui().set_st_kinds(model_of_strings(&[
        l10n::get(&state.lang, "StoreKind_All"),
        l10n::get(&state.lang, "StoreKind_Mods"),
        l10n::get(&state.lang, "StoreKind_Libraries"),
        l10n::get(&state.lang, "StoreKind_Plugins"),
    ]));
    let code = state.prefs.get_string("AppLanguage", "");
    let codes = ["", "en", "de", "fr", "es", "it", "ja", "pl", "ru", "zh"];
    let lang_idx = codes.iter().position(|c| *c == code).unwrap_or(1) as i32;
    state.settings_lang_index = lang_idx;
    state.ui().set_se_lang(lang_idx);
    state
        .ui()
        .set_se_modstore_url(state.prefs.get_string("ModStoreBaseUrl", "").into());
    state
        .ui()
        .set_se_steam_web_api_key(state.prefs.get_string("SteamWebApiKey", "").into());
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
    let labels = [
        "AppVersion",
        "Platform",
        "Workspace",
        "GameRoot",
        "Steam",
        "SteamAccount",
        "SteamApiKey",
        "Discord",
        "RichPresence",
        "GregApi",
        "MelonLoader",
    ];
    let app_version = env!("CARGO_PKG_VERSION");
    let (steam_connected, steam_account) = match state.backend.status() {
        greg_steam::backend::BackendStatus::Available { user_name } => (true, user_name),
        greg_steam::backend::BackendStatus::Unavailable { .. } => (false, String::new()),
    };
    let gregapi = if state.ui().get_gregapi_known() {
        if state.ui().get_gregapi_ok() {
            "Online"
        } else {
            "Offline"
        }
    } else {
        "Unknown"
    };
    let melonloader = if state.ui().get_melonloader_known() {
        if state.ui().get_melonloader_ok() {
            "Installed"
        } else {
            "Missing"
        }
    } else {
        "Unknown"
    };
    let values = [
        app_version.to_string(),
        format!("{} / {}", std::env::consts::OS, std::env::consts::ARCH),
        l10n::get(&state.lang, "Ui_Settings_Meta_Configured"),
        if state.prefs.get_string("GameRootPath", "").trim().is_empty() {
            l10n::get(&state.lang, "Ui_Settings_Meta_NotConfigured")
        } else {
            l10n::get(&state.lang, "Ui_Settings_Meta_Configured")
        },
        if steam_connected {
            l10n::get(&state.lang, "Ui_Settings_Meta_ConnectedHidden")
        } else {
            l10n::get(&state.lang, "Ui_Settings_Meta_Disconnected")
        },
        if steam_connected && !steam_account.trim().is_empty() {
            steam_account
        } else {
            l10n::get(&state.lang, "Ui_Settings_Meta_Unavailable")
        },
        if state.prefs.get_string("SteamWebApiKey", "").is_empty() {
            l10n::get(&state.lang, "Ui_Settings_Meta_NotConfigured")
        } else {
            l10n::get(&state.lang, "Ui_Settings_Meta_Configured")
        },
        l10n::get(&state.lang, "Ui_Settings_Meta_NotConfigured"),
        l10n::get(&state.lang, "Ui_Settings_Meta_Unavailable"),
        l10n::get(&state.lang, &format!("Ui_Settings_Meta_{gregapi}")),
        l10n::get(&state.lang, &format!("Ui_Settings_Meta_{melonloader}")),
    ];
    state.ui().set_se_diagnostics(model_of_rows(
        labels
            .iter()
            .zip(values.iter())
            .map(|(key, value)| DiagnosticRow {
                label: l10n::get(&state.lang, &format!("Ui_Settings_Meta_{key}")).into(),
                value: value.as_str().into(),
            })
            .collect::<Vec<_>>(),
    ));
    push_log(state);
}

fn push_log(state: &mut AppState) {
    let lines = state.log_snapshot();
    state.ui().set_se_log(model_of_strings(&lines));
}

/// Preview a base-directory change before touching any persistent settings.
pub fn settings_apply_workspace(state: &Arc<std::sync::Mutex<AppState>>) {
    let guard = state.lock().expect("state");
    let value = guard.ui().get_se_workspace().to_string();
    let workspace = greg_platform::workspace::Workspace::resolve(
        if value.trim().is_empty() {
            None
        } else {
            Some(value.trim())
        },
        None,
    );
    guard
        .ui()
        .set_workspace_target(workspace.root().to_string_lossy().into_owned().into());
    guard.ui().set_workspace_error("".into());
    guard.ui().set_show_workspace_modal(true);
}

/// Commit the reviewed base; existing content stays in the former base.
pub fn settings_confirm_workspace(state: &Arc<std::sync::Mutex<AppState>>) {
    if guard_game_running(state) {
        return;
    }
    let mut guard = state.lock().expect("state");
    if guard.publish_job.is_some()
        || guard.browse_job.is_some()
        || guard.store_job.is_some()
        || guard.lib_job.is_some()
        || guard.plugin_job.is_some()
        || guard.sync_job.is_some()
        || guard.build_job.is_some()
    {
        guard
            .ui()
            .set_workspace_error(l10n::get(&guard.lang, "Workspace_Busy").into());
        return;
    }
    let target = PathBuf::from(guard.ui().get_workspace_target().as_str());
    if !target.is_absolute() {
        guard
            .ui()
            .set_workspace_error(l10n::get(&guard.lang, "Workspace_AbsolutePath").into());
        return;
    }
    if let Ok(value) = std::env::var("GREG_WORKSPACE") {
        if !value.trim().is_empty() && Path::new(value.trim()) != target {
            guard
                .ui()
                .set_workspace_error(l10n::get(&guard.lang, "Workspace_EnvOverride").into());
            return;
        }
    }
    let workspace = greg_platform::workspace::Workspace::resolve(target.to_str(), None);
    let outcome = workspace
        .ensure_structure()
        .and_then(|()| guard.prefs.relocate(&target));
    if let Err(error) = outcome {
        guard.ui().set_workspace_error(error.to_string().into());
        return;
    }
    guard.prefs.set_string(
        greg_platform::workspace::CUSTOM_WORKSPACE_PATH_KEY,
        &target.to_string_lossy(),
    );
    if let Some(error) = guard.prefs.take_error() {
        guard.ui().set_workspace_error(error.into());
        return;
    }
    if guard.file_log.relocate(&target).is_err() {
        guard.log_line("Log directory relocation failed.");
    }
    greg_platform::filelog::register_sensitive_path(&target);
    guard.file_log.scrub_history(&target.join("Logs"));
    guard.workspace = workspace;
    guard.editor_root = None;
    guard.project_sync.clear();
    guard.local_sig.clear();
    guard.pack_service = load_packs(&guard.workspace);
    guard.ui().set_show_workspace_modal(false);
    refresh_projects(&mut guard);
    refresh_settings(&mut guard);
    local_refresh(&mut guard);
}

/// Applies the game-root override.
pub fn settings_apply_gameroot(state: &Arc<std::sync::Mutex<AppState>>) {
    {
        let mut guard = state.lock().expect("state");
        let value = guard.ui().get_se_gameroot().to_string();
        guard.prefs.set_string("GameRootPath", value.trim());
        if !value.trim().is_empty() {
            greg_platform::filelog::register_sensitive_path(Path::new(value.trim()));
            let logs_dir = guard.workspace.logs_dir();
            guard.file_log.scrub_history(&logs_dir);
        }
    }
    check_melon_loader_on_start(state);
}

/// Applies the Modstore URL override.
pub fn settings_apply_modstore_url(state: &Arc<std::sync::Mutex<AppState>>) {
    let mut guard = state.lock().expect("state");
    let value = guard.ui().get_se_modstore_url().to_string();
    guard.prefs.set_string("ModStoreBaseUrl", value.trim());
}

/// Applies the optional Steam Web API Key (Tier-2 fallback when Steam is offline).
pub fn settings_apply_steam_web_api_key(state: &Arc<std::sync::Mutex<AppState>>) {
    let mut guard = state.lock().expect("state");
    let value = guard.ui().get_se_steam_web_api_key().to_string();
    let trimmed = value.trim().to_string();
    guard.prefs.set_string("SteamWebApiKey", &trimmed);
    guard.steam.set_web_api_key(if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    });
    guard.log_line("Updated Steam Web API Key preference.");
    push_log(&mut guard);
}

/// Updates all localized UI strings (`Tr` global singleton, dropdown models,
/// navigation items, and active page titles/status summaries) for `state.lang`.
pub fn apply_ui_translations(state: &mut AppState) {
    let lang = state.lang.clone();
    let t = |key: &str| -> SharedString { l10n::get(&lang, key).into() };
    let ui = state.ui();
    {
        let tr = ui.global::<crate::Tr>();
        // Shell & Navigation
        tr.set_tip_modmanager(t("Ui_Tip_Modmanager"));
        tr.set_tip_workshop(t("Ui_Tip_WorkshopUpload"));
        tr.set_tip_modstore(t("Ui_Tip_Modstore"));
        tr.set_tip_settings(t("Ui_Tip_Settings"));
        tr.set_tip_report_bug(t("Ui_Tip_ReportBug"));
        tr.set_tip_login(t("Ui_Tip_Login"));
        tr.set_tip_logout(t("Ui_Tip_Logout"));
        tr.set_btn_start_game(t("Ui_Btn_StartGame"));
        tr.set_tip_start_game_vanilla(t("Ui_Tip_StartGameVanilla"));
        tr.set_btn_start_game_mods(t("Ui_Btn_StartGameMods"));
        tr.set_tip_start_game_mods(t("Ui_Tip_StartGameMods"));
        tr.set_btn_stop_game(t("Ui_Btn_StopGame"));
        tr.set_tip_stop_game(t("Ui_Tip_StopGame"));
        tr.set_btn_login(t("Ui_Btn_Login"));
        tr.set_btn_logout(t("Ui_Btn_Logout"));
        tr.set_btn_my_mods(t("Ui_Btn_MyMods"));
        tr.set_btn_upload(t("Ui_Btn_Upload"));
        tr.set_btn_settings(t("Ui_Btn_Settings"));
        tr.set_nav_modmanager(t("Ui_Nav_Modmanager"));
        tr.set_nav_workshop_upload(t("Ui_Nav_WorkshopUpload"));
        tr.set_nav_my_mods(t("Ui_Nav_MyMods"));
        tr.set_nav_my_plugins(t("Ui_Nav_MyPlugins"));
        tr.set_nav_my_libs(t("Ui_Nav_MyLibs"));
        tr.set_nav_modpacks(t("Ui_Nav_Modpacks"));
        tr.set_nav_collections(t("Ui_Nav_Collections"));
        tr.set_nav_problems(t("Ui_Nav_Problems"));
        tr.set_nav_report(t("Ui_Nav_Report"));
        tr.set_nav_projects(t("Ui_Nav_Projects"));
        tr.set_nav_new_project(t("Ui_Nav_NewProject"));
        tr.set_nav_browse(t("Ui_Nav_Browse"));
        tr.set_nav_subscribed(t("Ui_Nav_Subscribed"));
        tr.set_nav_favorited(t("Ui_Nav_Favorited"));
        tr.set_status_steam_connected(t("Ui_Status_SteamConnected"));
        tr.set_status_steam_disconnected(t("Ui_Status_SteamDisconnected"));
        tr.set_status_steam_connecting(t("Ui_Status_SteamConnecting"));
        tr.set_status_gregapi_connected(t("Ui_Status_GregApiConnected"));
        tr.set_status_gregapi_disconnected(t("Ui_Status_GregApiDisconnected"));
        tr.set_status_gregapi_probing(t("Ui_Status_GregApiProbing"));
        tr.set_status_melonloader_checking(t("Ui_Status_MelonLoaderChecking"));

        // Modals
        tr.set_new_modal_title(t("Ui_NewModal_Title"));
        tr.set_new_modal_folder_name(t("Ui_NewModal_FolderName"));
        tr.set_new_modal_folder_placeholder(t("Ui_NewModal_FolderPlaceholder"));
        tr.set_new_modal_template(t("Ui_NewModal_Template"));
        tr.set_new_modal_create_open(t("Ui_NewModal_CreateAndOpen"));
        tr.set_confirm_delete(t("Ui_Confirm_Delete"));
        tr.set_confirm_cancel(t("Ui_Confirm_Cancel"));
        tr.set_bug_dialog_title(t("Ui_BugDialog_Title"));
        tr.set_bug_dialog_subtitle(t("Ui_BugDialog_Subtitle"));
        tr.set_bug_dialog_this_app(t("Ui_BugDialog_ThisApp"));
        tr.set_bug_dialog_other_mods(t("Ui_BugDialog_OtherMods"));
        tr.set_lib_modal_title(t("Ui_LibModal_Title"));
        tr.set_lib_modal_subtitle(t("Ui_LibModal_Subtitle"));
        tr.set_plugin_modal_title(t("Ui_PluginModal_Title"));
        tr.set_plugin_modal_subtitle(t("Ui_PluginModal_Subtitle"));
        tr.set_modal_select_missing(t("Ui_Modal_SelectMissing"));
        tr.set_modal_select_all(t("Ui_Modal_SelectAll"));
        tr.set_modal_installed(t("Ui_Modal_Installed"));
        tr.set_modal_missing(t("Ui_Modal_Missing"));
        tr.set_modal_cancel(t("Ui_Modal_Cancel"));
        tr.set_modal_installing(t("Ui_Modal_Installing"));
        tr.set_modal_install_selected(t("Ui_Modal_InstallSelected"));

        // Local Content Page
        tr.set_local_filter_placeholder(t("Ui_Local_FilterPlaceholder"));
        tr.set_local_install_libs(t("Ui_Local_InstallLibs"));
        tr.set_local_browse_plugins(t("Ui_Local_BrowsePlugins"));
        tr.set_local_install_plugins(t("Ui_Local_InstallPlugins"));
        tr.set_local_rescan(t("Ui_Local_Rescan"));
        tr.set_local_open_folder(t("Ui_Local_OpenFolder"));
        tr.set_local_active_section(t("Ui_Local_ActiveSection"));
        tr.set_local_disabled_section(t("Ui_Local_DisabledSection"));
        tr.set_local_no_active(t("Ui_Local_NoActive"));
        tr.set_local_no_disabled(t("Ui_Local_NoDisabled"));
        tr.set_local_disable(t("Ui_Local_Disable"));
        tr.set_local_enable(t("Ui_Local_Enable"));
        tr.set_local_remove(t("Ui_Local_Remove"));

        // Packs & Collections Page
        tr.set_packs_new_name_placeholder(t("Ui_Packs_NewNamePlaceholder"));
        tr.set_packs_new(t("Ui_Packs_New"));
        tr.set_packs_import(t("Ui_Packs_Import"));
        tr.set_packs_on(t("Ui_Packs_On"));
        tr.set_packs_off(t("Ui_Packs_Off"));
        tr.set_packs_apply(t("Ui_Packs_Apply"));
        tr.set_packs_add_local_file(t("Ui_Packs_AddLocalFile"));
        tr.set_packs_export(t("Ui_Packs_Export"));
        tr.set_packs_delete_pack(t("Ui_Packs_DeletePack"));
        tr.set_packs_remove(t("Ui_Packs_Remove"));

        // Problems Page
        tr.set_problems_title(t("Ui_Problems_Title"));
        tr.set_problems_recheck(t("Ui_Problems_Recheck"));

        // Projects Page
        tr.set_projects_title(t("Ui_Projects_Title"));
        tr.set_projects_new_project(t("Ui_Projects_NewProject"));
        tr.set_projects_search_placeholder(t("Ui_Projects_SearchPlaceholder"));
        tr.set_projects_refresh(t("Ui_Projects_Refresh"));
        tr.set_projects_open_folder(t("Ui_Projects_OpenFolder"));
        tr.set_projects_steam(t("Ui_Projects_Steam"));
        tr.set_projects_edit(t("Ui_Projects_Edit"));

        // Editor Page
        tr.set_editor_section_details(t("Ui_Editor_SectionDetails"));
        tr.set_editor_section_deps(t("Ui_Editor_SectionDeps"));
        tr.set_editor_section_assets(t("Ui_Editor_SectionAssets"));
        tr.set_editor_section_changelog(t("Ui_Editor_SectionChangelog"));
        tr.set_editor_section_checks(t("Ui_Editor_SectionChecks"));
        tr.set_editor_title(t("Ui_Editor_Title"));
        tr.set_editor_version(t("Ui_Editor_Version"));
        tr.set_editor_view_on_steam(t("Ui_Editor_ViewOnSteam"));
        tr.set_editor_description(t("Ui_Editor_Description"));
        tr.set_editor_tags(t("Ui_Editor_Tags"));
        tr.set_editor_visibility(t("Ui_Editor_Visibility"));
        tr.set_editor_profile(t("Ui_Editor_Profile"));
        tr.set_editor_modtype(t("Ui_Editor_ModType"));
        tr.set_editor_needs_greg(t("Ui_Editor_NeedsGreg"));
        tr.set_editor_needs_ml(t("Ui_Editor_NeedsMl"));
        tr.set_editor_remove(t("Ui_Editor_Remove"));
        tr.set_editor_workshop_id_placeholder(t("Ui_Editor_WorkshopIdPlaceholder"));
        tr.set_editor_add_by_id(t("Ui_Editor_AddById"));
        tr.set_editor_preview(t("Ui_Editor_Preview"));
        tr.set_editor_pick_preview(t("Ui_Editor_PickPreview"));
        tr.set_editor_screenshots(t("Ui_Editor_Screenshots"));
        tr.set_editor_add_screenshot(t("Ui_Editor_AddScreenshot"));
        tr.set_editor_save(t("Ui_Editor_Save"));
        tr.set_editor_save_and_upload(t("Ui_Editor_SaveAndUpload"));
        tr.set_editor_modstore_upload(t("Ui_Editor_ModstoreUpload"));
        tr.set_editor_cancel(t("Ui_Editor_Cancel"));

        // Browse Workshop Page
        tr.set_browse_title(t("Ui_Browse_Title"));
        tr.set_browse_search_placeholder(t("Ui_Browse_SearchPlaceholder"));
        tr.set_browse_tag(t("Ui_Browse_Tag"));
        tr.set_browse_tag_placeholder(t("Ui_Browse_TagPlaceholder"));
        tr.set_browse_search(t("Ui_Browse_Search"));
        tr.set_browse_lookup_id_placeholder(t("Ui_Browse_LookupIdPlaceholder"));
        tr.set_browse_lookup_id_btn(t("Ui_Browse_LookupIdBtn"));
        tr.set_browse_cancel(t("Ui_Browse_Cancel"));
        tr.set_browse_select_hint(t("Ui_Browse_SelectHint"));
        tr.set_browse_author_label(t("Ui_Browse_Author"));
        tr.set_browse_status_label(t("Ui_Browse_Status"));
        tr.set_browse_subscribe(t("Ui_Browse_Subscribe"));
        tr.set_browse_unsubscribe(t("Ui_Browse_Unsubscribe"));
        tr.set_browse_import(t("Ui_Browse_Import"));
        tr.set_browse_steam_page(t("Ui_Browse_SteamPage"));
        tr.set_browse_prev(t("Ui_Browse_Prev"));
        tr.set_browse_next(t("Ui_Browse_Next"));
        tr.set_browse_page_word(t("Ui_Browse_PageWord"));
        tr.set_browse_total_word(t("Ui_Browse_TotalWord"));

        // Modstore Page
        tr.set_store_title(t("Ui_Store_Title"));
        tr.set_store_refresh(t("Ui_Store_Refresh"));
        tr.set_store_updates(t("Ui_Store_Updates"));
        tr.set_store_search_placeholder(t("Ui_Store_SearchPlaceholder"));
        tr.set_store_cancel(t("Ui_Store_Cancel"));
        tr.set_store_catalog_tab(t("Ui_Store_CatalogTab"));
        tr.set_store_install(t("Ui_Store_Install"));
        tr.set_store_update(t("Ui_Store_Update"));

        // Settings Page
        tr.set_settings_section_general(t("Ui_Settings_SectionGeneral"));
        tr.set_settings_section_diagnostics(t("Ui_Settings_SectionDiagnostics"));
        tr.set_settings_app_details(t("Ui_Settings_AppDetails"));
        tr.set_settings_diagnostics_privacy(t("Ui_Settings_DiagnosticsPrivacy"));
        tr.set_settings_section_support(t("Ui_Settings_SectionSupport"));
        tr.set_settings_language(t("Ui_Settings_Language"));
        tr.set_managed_install_title(t("Workspace_InstallTitle"));
        tr.set_managed_install_description(t("Workspace_InstallDescription"));
        tr.set_managed_install_button(t("Workspace_Install"));
        tr.set_build_test_title(t("Build_Title"));
        tr.set_build_test_description(t("Build_Description"));
        tr.set_build_test_run(t("Build_Run"));
        tr.set_workspace_title(t("Workspace_Title"));
        tr.set_workspace_description(t("Workspace_Description"));
        tr.set_settings_workspace(t("Ui_Settings_Workspace"));
        tr.set_settings_gameroot(t("Ui_Settings_GameRoot"));
        tr.set_settings_modstore_api(t("Ui_Settings_ModstoreApi"));
        tr.set_settings_steam_web_api_key(t("Ui_Settings_SteamWebApiKey"));
        tr.set_settings_steam_web_api_key_placeholder(t("Ui_Settings_SteamWebApiKeyPlaceholder"));
        tr.set_settings_browse(t("Ui_Settings_Browse"));
        tr.set_settings_apply(t("Ui_Settings_Apply"));
        tr.set_settings_check_updates(t("Ui_Settings_CheckUpdates"));
        tr.set_settings_run_checks(t("Ui_Settings_RunChecks"));
        tr.set_settings_open_log_folder(t("Ui_Settings_OpenLogFolder"));
        tr.set_settings_support_bundle(t("Ui_Settings_SupportBundle"));
        tr.set_settings_register_protocol(t("Ui_Settings_RegisterProtocol"));

        // Report Assistant Page
        tr.set_report_title(t("Ui_Report_Title"));
        tr.set_report_subtitle(t("Ui_Report_Subtitle"));
        tr.set_report_category(t("Ui_Report_Category"));
        tr.set_report_report_title(t("Ui_Report_ReportTitle"));
        tr.set_report_description(t("Ui_Report_Description"));
        tr.set_report_steps(t("Ui_Report_Steps"));
        tr.set_report_expected(t("Ui_Report_Expected"));
        tr.set_report_actual(t("Ui_Report_Actual"));
        tr.set_report_logs_crash(t("Ui_Report_LogsCrash"));
        tr.set_report_measurements(t("Ui_Report_Measurements"));
        tr.set_report_environment(t("Ui_Report_Environment"));
        tr.set_report_attach_file(t("Ui_Report_AttachFile"));
        tr.set_report_preview_section(t("Ui_Report_PreviewSection"));
        tr.set_report_collect_logs(t("Ui_Report_CollectLogs"));
        tr.set_report_generate_preview(t("Ui_Report_GeneratePreview"));
        tr.set_report_export(t("Ui_Report_Export"));
        tr.set_report_submit(t("Ui_Report_Submit"));
        tr.set_report_cancel(t("Ui_Report_Cancel"));
        tr.set_report_confirm_title(t("Ui_Report_ConfirmTitle"));
    }

    // Localized ComboBox & filter models (preserving active indices).
    let prev_new_tpl = ui.get_new_template();
    ui.set_new_templates(model_of_strings(&[
        l10n::get(&lang, "Ui_Template_VanillaObject"),
        l10n::get(&lang, "Ui_Template_MelonMod"),
        l10n::get(&lang, "Ui_Template_GregCoreMod"),
        l10n::get(&lang, "Ui_Template_UxmlOverride"),
        l10n::get(&lang, "Ui_Template_Standalone3d"),
        l10n::get(&lang, "Ui_Template_StandaloneTexture"),
        l10n::get(&lang, "Ui_Template_StandaloneAudio"),
    ]));
    ui.set_new_template(prev_new_tpl);

    let prev_vis = ui.get_ed_visibility();
    ui.set_ed_visibilities(model_of_strings(&[
        l10n::get(&lang, "Ui_Vis_Public"),
        l10n::get(&lang, "Ui_Vis_FriendsOnly"),
        l10n::get(&lang, "Ui_Vis_Private"),
    ]));
    ui.set_ed_visibility(prev_vis);

    let prev_prof = ui.get_ed_profile();
    ui.set_ed_profiles(model_of_strings(&[
        l10n::get(&lang, "Ui_Profile_Decoration"),
        l10n::get(&lang, "Ui_Profile_Code"),
    ]));
    ui.set_ed_profile(prev_prof);

    let prev_modtype = ui.get_ed_modtype();
    ui.set_ed_modtypes(model_of_strings(&[
        l10n::get(&lang, "Ui_ModType_PlacableObject"),
        l10n::get(&lang, "Ui_ModType_MelonPlugin"),
        l10n::get(&lang, "Ui_ModType_Userlib"),
        l10n::get(&lang, "Ui_ModType_DataCenterMod"),
    ]));
    ui.set_ed_modtype(prev_modtype);

    let prev_br_list = ui.get_br_list();
    ui.set_br_lists(model_of_strings(&[
        l10n::get(&lang, "Ui_Browse_ListBrowse"),
        l10n::get(&lang, "Ui_Browse_ListSubscribed"),
        l10n::get(&lang, "Ui_Browse_ListFavorited"),
    ]));
    ui.set_br_list(prev_br_list);

    let prev_br_sort = ui.get_br_sort();
    ui.set_br_sorts(model_of_strings(&[
        l10n::get(&lang, "Ui_Browse_SortUpdated"),
        l10n::get(&lang, "Ui_Browse_SortNewest"),
        l10n::get(&lang, "Ui_Browse_SortTopRated"),
        l10n::get(&lang, "Ui_Browse_SortTrending"),
        l10n::get(&lang, "Ui_Browse_SortSubscribed"),
        l10n::get(&lang, "Ui_Browse_SortTitleAz"),
    ]));
    ui.set_br_sort(prev_br_sort);

    let prev_proj_filter = ui.get_project_filter_index();
    ui.set_project_filters(model_of_strings(&[
        l10n::get(&lang, "Ui_Projects_FilterAll"),
        l10n::get(&lang, "Ui_Projects_FilterUnreleased"),
        l10n::get(&lang, "Ui_Projects_FilterPublished"),
        l10n::get(&lang, "Ui_Projects_FilterUpdated"),
    ]));
    ui.set_project_filter_index(prev_proj_filter);

    let prev_se_lang = ui.get_se_lang();
    ui.set_se_langs(model_of_strings(&[
        l10n::get(&lang, "Ui_Lang_SystemDefault"),
        "English".to_string(),
        "Deutsch".to_string(),
        "Français".to_string(),
        "Español".to_string(),
        "Italiano".to_string(),
        "日本語".to_string(),
        "Polski".to_string(),
        "Русский".to_string(),
        "中文".to_string(),
    ]));
    ui.set_se_lang(prev_se_lang);

    // Refresh current navigation sidebar & status labels.
    let current_icon = ui.get_current_icon().to_string();
    let current_page = ui.get_current_page().to_string();
    AppState::apply_icon_state(&ui, &current_icon, &current_page);
    state.refresh_steam_status();
    if !ui.get_game_running() {
        ui.set_game_text(l10n::get(&lang, "Game_StatusStopped").into());
    }
    if is_local_page(state) {
        local_refresh(state);
    }
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
                guard.log_line(&format!("checks failed: {e}"));
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
        Ok(_) => guard.log_line("Support bundle created."),
        Err(_) => guard.log_line("Support bundle creation failed."),
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
    crate::image_cache::thumbnail(path)
}

/// Thumbnail cache dir for remote previews.
fn thumb_cache_dir(state: &AppState) -> PathBuf {
    state.workspace.root().join("Cache").join("Thumbnails")
}

const MAX_THUMBNAILS: usize = 128;
const THUMB_BATCH_SIZE: usize = 16;

/// Queues bounded missing thumbnails; URL changes invalidate reused item keys.
fn queue_thumbs(state: &mut AppState, items: Vec<(String, String)>) {
    for (key, url) in items {
        if url.trim().is_empty() {
            continue;
        }
        let same_url = state.thumb_urls.get(&key) == Some(&url);
        let fresh = same_url
            && state.thumb_done.get(&key).is_some_and(|path| {
                std::fs::metadata(path)
                    .ok()
                    .and_then(|metadata| metadata.modified().ok())
                    .and_then(|time| time.elapsed().ok())
                    .is_some_and(|age| age < std::time::Duration::from_secs(24 * 60 * 60))
            });
        if fresh
            || (same_url
                && (state.thumb_pending.iter().any(|(queued, _)| queued == &key)
                    || (state.thumb_job.is_some() && !state.thumb_done.contains_key(&key))))
        {
            continue;
        }
        if !same_url {
            state.thumb_done.remove(&key);
            state.thumb_pending.retain(|(queued, _)| queued != &key);
        }
        if state.thumb_pending.len() >= MAX_THUMBNAILS {
            break;
        }
        state.thumb_urls.insert(key.clone(), url.clone());
        state.thumb_pending.push((key, url));
    }
    pump_thumb_job(state);
}

/// Starts at most one bounded batch, leaving the remainder queued.
fn pump_thumb_job(state: &mut AppState) {
    if state.thumb_job.is_some() || state.thumb_pending.is_empty() {
        return;
    }
    let batch: Vec<_> = state
        .thumb_pending
        .drain(..state.thumb_pending.len().min(THUMB_BATCH_SIZE))
        .collect();
    let dir = thumb_cache_dir(state);
    let job = JobHandle::spawn(move |sink| {
        let mut out: Vec<(String, String, PathBuf)> = Vec::new();
        let runtime = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(runtime) => runtime,
            Err(_) => {
                return JobOutcome {
                    success: false,
                    message: String::new(),
                }
            }
        };
        let client = match reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(20))
            .build()
        {
            Ok(client) => client,
            Err(_) => {
                return JobOutcome {
                    success: false,
                    message: String::new(),
                }
            }
        };
        for (key, url) in &batch {
            if sink.cancelled().load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }
            match download_thumb(&runtime, &client, &dir, url, sink.cancelled()) {
                Some(path) => out.push((key.clone(), url.clone(), path)),
                None => sink.log("Thumbnail unavailable; showing placeholder."),
            }
        }
        trim_thumbnail_disk_cache(&dir);
        JobOutcome {
            success: true,
            message: serde_json::to_string(&out).unwrap_or_default(),
        }
    });
    state.thumb_job = Some(job);
}

/// Bounded HTTP and genuine image validation happen on the background worker.
fn download_thumb(
    runtime: &tokio::runtime::Runtime,
    client: &reqwest::Client,
    dir: &std::path::Path,
    url: &str,
    cancel: &std::sync::atomic::AtomicBool,
) -> Option<PathBuf> {
    runtime
        .block_on(
            greg_steam::web_api::download_and_cache_preview_image_with_client(
                url, 0, dir, cancel, client,
            ),
        )
        .ok()?
        .cached_path
}

/// Keep the persistent preview cache bounded too. Only files belonging to our
/// URL-keyed preview cache are candidates; unrelated temporary files stay put.
fn trim_thumbnail_disk_cache(dir: &std::path::Path) {
    const MAX_DISK_BYTES: u64 = 64 * 1024 * 1024;
    let Ok(read) = std::fs::read_dir(dir) else {
        return;
    };
    let mut files: Vec<_> = read
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.starts_with("preview_") && !name.starts_with("workshop_") {
                return None;
            }
            let metadata = entry.metadata().ok()?;
            if !metadata.is_file() {
                return None;
            }
            Some((metadata.modified().ok(), metadata.len(), entry.path()))
        })
        .collect();
    files.sort_by_key(|(modified, _, _)| *modified);
    let mut size = files
        .iter()
        .fold(0u64, |sum, (_, len, _)| sum.saturating_add(*len));
    let mut count = files.len();
    for (_, len, path) in files {
        if size <= MAX_DISK_BYTES && count <= MAX_THUMBNAILS {
            break;
        }
        if std::fs::remove_file(path).is_ok() {
            size = size.saturating_sub(len);
            count -= 1;
        }
    }
}

/// Applies only current-URL results; stale in-flight downloads cannot replace
/// a newer item preview. Cache bookkeeping and decoded pixels are both bounded.
fn drain_thumb(state: &mut AppState) {
    let messages = match state.thumb_job.as_mut() {
        Some(job) => job.drain(),
        None => return,
    };
    let finished = state
        .thumb_job
        .as_ref()
        .is_some_and(|job| job.is_finished());
    let mut done = None;
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
        let records: Vec<(String, String, PathBuf)> =
            serde_json::from_str(&outcome.message).unwrap_or_default();
        for (key, url, path) in records {
            if state.thumb_urls.get(&key) == Some(&url) && path.is_file() {
                state.thumb_done.insert(key, path);
            }
        }
    }
    while state.thumb_done.len() > MAX_THUMBNAILS {
        let Some(key) = state.thumb_done.keys().next().cloned() else {
            break;
        };
        state.thumb_done.remove(&key);
    }
    state.thumb_urls.retain(|key, _| {
        state.thumb_done.contains_key(key)
            || state.thumb_pending.iter().any(|(queued, _)| queued == key)
    });
    repaint_browse_thumbs(state);
    repaint_store_thumbs(state);
    pump_thumb_job(state);
}

/// Repaints browse rows and the active detail preview with cached thumbnails.
fn repaint_browse_thumbs(state: &mut AppState) {
    if let Some(detail_id) = state.browse_detail_id {
        let key = format!("workshop:{detail_id}");
        if let Some(path) = state.thumb_done.get(&key).filter(|p| p.is_file()).cloned() {
            state.ui().set_br_d_preview(thumb_of(Some(&path)));
        }
    }
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
            .get(&format!("workshop:{}", row.id))
            .filter(|p| p.is_file())
            .cloned()
        {
            row.thumb = thumb_of(Some(&path));
            view.set_row_data(i, row);
        }
    }
}

/// Filters the cached catalog by shared content type and search text.
/// Row slugs remain provider identities regardless of the filtered row index.
pub fn store_filter(state: &mut AppState) {
    use greg_core::models::ProjectContentKind;
    let ui = state.ui();
    let query = ui.get_st_search().to_string().trim().to_lowercase();
    let selected_kind = ui.get_st_kind();
    let rows: Vec<StoreRow> = state
        .store_items
        .iter()
        .filter_map(|item| {
            let kind = item.content_kind().ok();
            let matches_kind = match selected_kind {
                1 => matches!(
                    kind,
                    Some(ProjectContentKind::PlacableObject | ProjectContentKind::DataCenterMod)
                ),
                2 => kind == Some(ProjectContentKind::Userlib),
                3 => kind == Some(ProjectContentKind::MelonloaderPlugin),
                _ => true,
            };
            if !matches_kind
                || (!query.is_empty()
                    && ![
                        item.title.as_str(),
                        item.slug.as_str(),
                        item.author.as_str(),
                        item.category.as_str(),
                    ]
                    .iter()
                    .any(|text| text.to_lowercase().contains(&query)))
            {
                return None;
            }
            Some(StoreRow {
                title: item.title.clone().into(),
                meta: format!(
                    "by {} · v{} · {} · {}",
                    item.author,
                    item.release.version,
                    item.category,
                    kind.map(ProjectContentKind::name).unwrap_or("?")
                )
                .into(),
                slug: item.slug.clone().into(),
                thumb: thumb_of(
                    state
                        .thumb_done
                        .get(&format!("store:{}", item.slug))
                        .map(PathBuf::as_path),
                ),
            })
        })
        .collect();
    ui.set_st_items(model_of_rows(rows));
}

/// New image results use the same catalog filter, preserving current selection.
fn repaint_store_thumbs(state: &mut AppState) {
    store_filter(state);
}

// AppState helper used by drain_store (avoids borrow issues).
impl AppState {
    fn append_log_ref(state: &mut AppState, line: &str) {
        let _ = line;
        let line = "Background operation status updated.";
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
            Some(ProtocolUrl::Other) => {}
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

#[cfg(test)]
mod thumbnail_cache_tests {
    #[test]
    fn disk_cache_trimming_bounds_owned_files_and_preserves_other_files() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("greg-thumb-disk-{}-{nonce}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("unrelated.txt"), b"keep").unwrap();
        let huge = std::fs::File::create(dir.join("preview_big.png")).unwrap();
        huge.set_len(65 * 1024 * 1024).unwrap();
        drop(huge);
        for index in 0..140 {
            std::fs::write(dir.join(format!("preview_{index}.png")), b"x").unwrap();
        }
        super::trim_thumbnail_disk_cache(&dir);
        let owned: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .filter(|entry| entry.file_name().to_string_lossy().starts_with("preview_"))
            .collect();
        assert!(owned.len() <= super::MAX_THUMBNAILS);
        assert!(
            owned
                .iter()
                .map(|entry| entry.metadata().unwrap().len())
                .sum::<u64>()
                <= 64 * 1024 * 1024
        );
        assert_eq!(std::fs::read(dir.join("unrelated.txt")).unwrap(), b"keep");
        std::fs::remove_dir_all(dir).unwrap();
    }
}
