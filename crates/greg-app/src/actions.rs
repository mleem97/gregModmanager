//! Slint callback implementations (one section per shell area).

use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use greg_core::l10n;
use greg_core::models::ProjectSyncState;
use greg_core::prefs::Preferences;
use slint::{Model, ModelRc, SharedString, VecModel};

use crate::state::{AppState, UiEvent, ICON_MODSTORE, ICON_PROJECTS, ICON_SETTINGS, ICON_WORKSHOP};
use crate::worker::{JobHandle, JobMessage, JobOutcome};
use crate::{BrowseRow, CheckRow, HealthRow, ProjectRow, StoreRow, UpdateRow};

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
        }
    }
    // Steam re-check roughly every 60 s (400 × 150 ms).
    state.tick_count += 1;
    if state.tick_count.is_multiple_of(400) {
        state.refresh_steam_status();
    }
    drain_publish(state);
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
        ICON_PROJECTS => AppState::apply_icon_state(&guard.ui(), ICON_PROJECTS, "projects"),
        ICON_WORKSHOP => {
            AppState::apply_icon_state(&guard.ui(), ICON_WORKSHOP, "browse");
            guard.ui().set_br_list(0);
        }
        ICON_MODSTORE => AppState::apply_icon_state(&guard.ui(), ICON_MODSTORE, "modstore"),
        ICON_SETTINGS => {
            AppState::apply_icon_state(&guard.ui(), ICON_SETTINGS, "settings");
            refresh_settings(&mut guard);
        }
        _ => {}
    }
}

/// Handles navigation clicks (page ids).
pub fn on_nav(state: &Arc<std::sync::Mutex<AppState>>, id: &str) {
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
        "uploads" => {
            guard.ui().set_br_list(3);
            guard.ui().set_current_page("browse".into());
        }
        _ => {}
    }
}

/// Launches the game: `steam`/`mods` via Steam URL, `vanilla` via exe.
pub fn start_game(state: &Arc<std::sync::Mutex<AppState>>, mode: &str) {
    // Gathers the plan first so no lock is held during launches/logs.
    let vanilla_plan = if mode == "vanilla" {
        let registry = greg_loader::game::GameAdapterRegistry::with_defaults();
        let game_root = std::env::var("GREG_GAME_ROOT").ok().map(PathBuf::from);
        Some(match registry.detect(game_root.as_deref()) {
            Some((adapter, installation)) => adapter.plan_launch(&installation.root_path, vec![]),
            None => greg_loader::game::GameOperationPlan {
                adapter_id: "datacenter".into(),
                operation: "launch".into(),
                game_root: PathBuf::new(),
                executable: None,
            },
        })
    } else {
        None
    };
    match vanilla_plan {
        Some(plan) => match plan.executable {
            Some(exe) => {
                if let Err(e) = greg_platform::process::launch_app(&exe, &[]) {
                    AppState::append_log(state, &format!("Launch failed: {e}"));
                }
            }
            None => AppState::append_log(state, "Launch: game executable not found."),
        },
        None => {
            if let Err(e) = greg_platform::process::open_url("steam://rungameid/4170200") {
                AppState::append_log(state, &format!("Steam launch failed: {e}"));
            }
        }
    }
}

/// Opens the login URL in a browser (session completes via `greg://` later).
pub fn login(state: &Arc<std::sync::Mutex<AppState>>) {
    let request_id = format!(
        "{:x}{:x}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0),
        std::process::id()
    );
    let formats = login_url_formats();
    let redirect = percent_encode(greg_modstore::auth_client::AUTH_CALLBACK_REDIRECT_URI);
    let candidate = formats.first().map(|f| {
        f.replacen("{0}", &redirect, 1)
            .replacen("{1}", &percent_encode(&request_id), 1)
    });
    match candidate {
        Some(url) => {
            AppState::append_log(state, &format!("Opening login: {url}"));
            let _ = greg_platform::process::open_url(&url);
        }
        None => AppState::append_log(state, "Login: no auth endpoint configured."),
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

/// Profile click (logout arrives with the session flow).
pub fn profile_clicked(state: &Arc<std::sync::Mutex<AppState>>) {
    AppState::append_log(state, "Profile: session management follows with OAuth.");
}

// ---------------------------------------------------------------------------
// Projects.
// ---------------------------------------------------------------------------

/// Refreshes list rows (metadata + sync states).
pub fn refresh_projects(state: &mut AppState) {
    let projects = state.workspace.scan_projects().unwrap_or_default();
    let mut rows: Vec<ProjectRow> = Vec::new();
    let mut cache: Vec<(PathBuf, u64)> = Vec::new();
    for project in &projects {
        let meta = greg_platform::workspace::load_metadata(&project.root_path).unwrap_or_default();
        let desc = state
            .steam
            .build_upload_description(&project.root_path, &meta);
        let sync = state
            .workspace
            .sync_state(&project.root_path, &desc)
            .unwrap_or(ProjectSyncState::Unknown);
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
    AppState::apply_icon_state(&guard.ui(), ICON_PROJECTS, "editor");
}

/// Pushes editor state into properties.
fn push_editor(state: &mut AppState) {
    let ui = &state.ui();
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
    let Some(root) = guard.editor_root.clone() else {
        return;
    };
    if guard.publish_job.is_some() {
        return;
    }
    if !guard.ui().get_ed_ready() {
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
        if outcome.success {
            let _ = greg_platform::workspace::save_metadata(&root, &meta);
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
                    3 => steam.my_published(page as u32, &cancel),
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
                let rows: Vec<BrowseRow> = items
                    .into_iter()
                    .map(|i| BrowseRow {
                        id: i.published_file_id.to_string().into(),
                        title: i.title.into(),
                        score: format!("★ {:.1}", i.score).into(),
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
    let mut guard = state.lock().expect("state");
    if guard.store_job.is_some() || !guard.ui().get_modstore_available() {
        return;
    }
    let urls = store_base_urls(&guard);
    let rt = guard.runtime.handle().clone();
    guard.ui().set_br_busy(false);
    guard.ui().set_st_busy(true);
    guard.ui().set_st_status("Loading catalog…".into());
    let job = JobHandle::spawn(move |sink| {
        let outcome = match greg_modstore::client::ModStoreClient::new(urls) {
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
        let outcome = match greg_modstore::client::ModStoreClient::new(urls) {
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
            let rows: Vec<StoreRow> = items
                .into_iter()
                .map(|i| StoreRow {
                    title: i.title.into(),
                    meta: format!("by {} · v{} · {}", i.author, i.release.version, i.category)
                        .into(),
                    slug: i.slug.into(),
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
