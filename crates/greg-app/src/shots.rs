//! Headless screenshot tests (software renderer → PNG).
//!
//! Lets layout be verified without a display: each page renders with sample
//! data into `target/screenshots/`. Run with
//! `cargo test -p greg-app --test shots` — no wait, these are bin unittests:
//! `cargo test -p greg-app shots`.

use std::rc::Rc;

use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType, Rgb565Pixel};
use slint::platform::{Platform, PlatformError, WindowAdapter};
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};

use crate::{
    BrowseRow, CheckRow, HealthRow, NavItem, PackEntryRow, PackRow, ProblemRow, ProjectRow,
    StoreRow, UpdateRow,
};

fn headless_window() -> Rc<MinimalSoftwareWindow> {
    thread_local! {
        static WINDOW: Rc<MinimalSoftwareWindow> = {
            let window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
            struct Holder {
                window: Rc<MinimalSoftwareWindow>,
            }
            impl Platform for Holder {
                fn create_window_adapter(
                    &self,
                ) -> Result<Rc<dyn WindowAdapter>, PlatformError> {
                    Ok(self.window.clone())
                }
            }
            // Only the first test thread wins; the rest reuse the platform.
            let _ = slint::platform::set_platform(Box::new(Holder {
                window: window.clone(),
            }));
            window
        };
    }
    WINDOW.with(|w| w.clone())
}

fn shot(ui: &crate::MainWindow, window: &Rc<MinimalSoftwareWindow>, name: &str) {
    window.request_redraw();
    let out_dir =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/screenshots");
    std::fs::create_dir_all(&out_dir).expect("screenshot dir");
    let drawn = window.draw_if_needed(|renderer| {
        let size = window.size();
        let (w, h) = (size.width, size.height);
        let mut buffer = vec![Rgb565Pixel::default(); (w * h) as usize];
        renderer.render(&mut buffer, w as usize);
        let bytes: Vec<u8> = buffer
            .iter()
            .flat_map(|p| {
                let v = p.0;
                [
                    ((v & 0b1111_1000_0000_0000) >> 8) as u8,
                    ((v & 0b0000_0111_1110_0000) >> 3) as u8,
                    ((v & 0b0000_0000_0001_1111) << 3) as u8,
                ]
            })
            .collect();
        let image = image::RgbImage::from_raw(w, h, bytes).expect("image buffer");
        image
            .save(out_dir.join(format!("{name}.png")))
            .expect("save png");
    });
    assert!(drawn, "nothing redrawn for {name}");
    let _ = ui;
}

fn model<T: Clone + 'static>(items: Vec<T>) -> ModelRc<T> {
    ModelRc::from(Rc::new(VecModel::from(items)))
}

fn strings(items: &[&str]) -> ModelRc<SharedString> {
    model(items.iter().map(|s| SharedString::from(*s)).collect())
}

/// Sample thumbnail: the repo logo (missing file degrades to placeholder).
fn sample_thumb() -> slint::Image {
    let logo = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../G.svg");
    slint::Image::load_from_path(&logo).unwrap_or_default()
}

fn sample_ui() -> crate::MainWindow {
    let _window = headless_window();
    let ui = crate::MainWindow::new().expect("window");
    ui.window().set_size(slint::LogicalSize::new(1400.0, 900.0));
    ui.set_app_version("1.7.0".into());
    ui.set_steam_ok(true);
    ui.set_steam_text("Steam Connected · Tester".into());
    ui.set_gregapi_ok(false);
    ui.set_gregapi_text("gregAPI Offline".into());
    ui.set_modstore_available(false);
    ui.set_login_visible(false);
    ui.set_new_templates(strings(&[
        "Vanilla object (decoration)",
        "MelonLoader mod",
        "gregCore mod",
    ]));
    ui.set_ed_visibilities(strings(&["Public", "FriendsOnly", "Private"]));
    ui.set_ed_profiles(strings(&["decoration", "code"]));
    ui.set_ed_modtypes(strings(&[
        "PlacableObject",
        "MelonloaderPlugin",
        "Userlib",
        "DataCenterMod",
    ]));
    ui.set_br_lists(strings(&["Browse", "Subscribed", "Favorited"]));
    ui.set_br_sorts(strings(&[
        "Updated",
        "Newest",
        "Top rated",
        "Trending",
        "Subscribed",
        "Title A–Z",
    ]));
    ui.set_se_langs(strings(&["System default", "English", "Deutsch"]));
    ui
}

fn nav(ui: &crate::MainWindow, icon: &str, title: &str, items: Vec<(&str, &str)>, page: &str) {
    ui.set_current_icon(icon.into());
    ui.set_current_page(page.into());
    ui.set_nav_visible(true);
    ui.set_nav_title(title.into());
    ui.set_nav_items(model(
        items
            .into_iter()
            .map(|(id, label)| NavItem {
                id: id.into(),
                label: label.into(),
            })
            .collect(),
    ));
}

fn page_mymods(ui: &crate::MainWindow, window: &Rc<MinimalSoftwareWindow>) {
    nav(
        ui,
        "modmanager",
        "MODMANAGER",
        vec![
            ("mymods", "My Mods"),
            ("myplugins", "My Plugins"),
            ("mylibs", "My Libs"),
            ("modpacks", "Modpacks"),
            ("collections", "Collections"),
            ("problems", "Problems"),
            ("report", "Report"),
        ],
        "mymods",
    );
    ui.set_local_title("MY MODS".into());
    let sample: Vec<crate::LocalRow> = (0..6)
        .map(|i| crate::LocalRow {
            name: format!("SomeMod{i}.dll").into(),
            detail: "1.2 MB · Mods".into(),
            enabled: i % 2 == 0,
            index: i,
            thumb: if i == 0 {
                sample_thumb()
            } else {
                slint::Image::default()
            },
        })
        .collect();
    ui.set_local_active_rows(model(
        sample.iter().filter(|r| r.enabled).cloned().collect(),
    ));
    ui.set_local_inactive_rows(model(
        sample.iter().filter(|r| !r.enabled).cloned().collect(),
    ));
    ui.set_local_status("3 active, 3 disabled".into());
    shot(ui, window, "01-mymods");
}

fn page_problems(ui: &crate::MainWindow, window: &Rc<MinimalSoftwareWindow>) {
    nav(
        ui,
        "modmanager",
        "MODMANAGER",
        vec![("problems", "Problems")],
        "problems",
    );
    ui.set_problem_rows(model(vec![
        ProblemRow {
            severity: "error".into(),
            source: "Game".into(),
            message: "Game folder not found — set the game root in Settings.".into(),
            has_action: true,
            action_label: "Open Settings".into(),
            index: 0,
        },
        ProblemRow {
            severity: "warning".into(),
            source: "Projects".into(),
            message: "BetterShop: local changes not published yet.".into(),
            has_action: true,
            action_label: "Open editor".into(),
            index: 1,
        },
        ProblemRow {
            severity: "info".into(),
            source: "Projects".into(),
            message: "Trainer: never published.".into(),
            has_action: false,
            action_label: "".into(),
            index: 2,
        },
    ]));
    ui.set_problem_status("1 errors, 1 warnings".into());
    shot(ui, window, "02-problems");
}

fn page_projects(ui: &crate::MainWindow, window: &Rc<MinimalSoftwareWindow>) {
    nav(
        ui,
        "workshop",
        "WORKSHOP UPLOAD",
        vec![("projects", "Projects"), ("new", "New Project")],
        "projects",
    );
    ui.set_projects(model(
        (0..5)
            .map(|i| ProjectRow {
                name: format!("mod{i}").into(),
                title: format!("Some Mod {i} (mod{i})").into(),
                meta: "v1.0.0 · DataCenterMod · Public".into(),
                state: if i % 2 == 0 { "UPDATED!" } else { "SYNCED" }.into(),
                state_ok: i % 2 != 0,
                root: format!("/ws/mod{i}").into(),
                fileid: format!("{i}").into(),
                thumb: if i == 0 {
                    sample_thumb()
                } else {
                    slint::Image::default()
                },
            })
            .collect(),
    ));
    shot(ui, window, "03-projects");
}

fn page_editor(ui: &crate::MainWindow, window: &Rc<MinimalSoftwareWindow>) {
    nav(ui, "workshop", "WORKSHOP UPLOAD", vec![], "editor");
    ui.set_nav_visible(false);
    ui.set_ed_title("BetterShop".into());
    ui.set_ed_version("1.1.0".into());
    ui.set_ed_desc("Adds better shop items with new prices and unlock levels.".into());
    ui.set_ed_tags("shop, economy".into());
    ui.set_ed_fileid_text("file id: 3807398588".into());
    ui.set_ed_project_name("BetterShop".into());
    ui.set_ed_changelog("Fixed prices.\nAdded unlock levels.".into());
    ui.set_ed_changelog_hint("CHANGELOG.md [1.1.0] → v1.1.0".into());
    ui.set_ed_desc_hint("123 / 8000".into());
    ui.set_ed_preview_text("preview.png".into());
    ui.set_ed_preview_image(sample_thumb());
    ui.set_ed_checks(model(vec![
        CheckRow {
            label: "Title".into(),
            detail: "\"BetterShop\" (10/128)".into(),
            ok: true,
        },
        CheckRow {
            label: "Changelog".into(),
            detail: "42 characters (CHANGELOG.md [1.1.0]).".into(),
            ok: true,
        },
        CheckRow {
            label: "Preview image".into(),
            detail: "File not found: preview.png.".into(),
            ok: false,
        },
    ]));
    ui.set_ed_ready(false);
    ui.set_ed_readiness_text("Issues found — fix errors before uploading".into());
    ui.set_ed_deps(model(vec!["3807398588".to_string().into()]));
    ui.set_ed_status("".into());
    shot(ui, window, "04-editor");
}

fn page_browse(ui: &crate::MainWindow, window: &Rc<MinimalSoftwareWindow>) {
    nav(
        ui,
        "workshop",
        "WORKSHOP UPLOAD",
        vec![("browse", "Browse")],
        "browse",
    );
    ui.set_br_page(1);
    ui.set_br_total(128);
    ui.set_br_items(model(
        (0..8)
            .map(|i| BrowseRow {
                id: format!("{i}").into(),
                title: format!("Workshop Item {i} With A Reasonably Long Title").into(),
                score: "4.5 / 5".into(),
                thumb: if i == 0 {
                    sample_thumb()
                } else {
                    slint::Image::default()
                },
            })
            .collect(),
    ));
    ui.set_br_has_detail(true);
    ui.set_br_d_title("Workshop Item 0 With A Reasonably Long Title".into());
    ui.set_br_d_meta("by SomeAuthor · 4.5 / 5 · 123456 bytes".into());
    ui.set_br_d_tags("economy, shop".into());
    ui.set_br_d_desc(
        "A longer description of the workshop item spanning a couple of lines for layout testing."
            .into(),
    );
    ui.set_br_d_subscribed(false);
    shot(ui, window, "05-browse");
}

fn page_modstore(ui: &crate::MainWindow, window: &Rc<MinimalSoftwareWindow>) {
    ui.set_current_icon("modstore".into());
    ui.set_current_page("modstore".into());
    ui.set_nav_visible(false);
    ui.set_modstore_available(true);
    ui.set_st_items(model(
        (0..5)
            .map(|i| StoreRow {
                title: format!("Store Mod {i}").into(),
                meta: format!("by Author{i} · v1.0.{i} · decoration").into(),
                slug: format!("mod-{i}").into(),
                thumb: if i == 0 {
                    sample_thumb()
                } else {
                    slint::Image::default()
                },
            })
            .collect(),
    ));
    ui.set_st_updates(model(vec![UpdateRow {
        title: "Store Mod 0".into(),
        versions: "1.0.0 → 1.1.0".into(),
    }]));
    ui.set_st_status("Catalog: 5 mods".into());
    shot(ui, window, "06-modstore");
}

fn page_modpacks(ui: &crate::MainWindow, window: &Rc<MinimalSoftwareWindow>) {
    nav(
        ui,
        "modmanager",
        "MODMANAGER",
        vec![("modpacks", "Modpacks")],
        "modpacks",
    );
    ui.set_pack_kind("pack".into());
    ui.set_pack_title("MODPACKS".into());
    ui.set_pack_packs(model(vec![
        PackRow {
            name: "Economy Pack".into(),
            detail: "4 entries".into(),
            enabled: true,
            index: 0,
        },
        PackRow {
            name: "Vanilla Plus".into(),
            detail: "9 entries · off".into(),
            enabled: false,
            index: 1,
        },
    ]));
    ui.set_pack_selected(0);
    ui.set_pack_entries(model(
        (0..5)
            .map(|i| PackEntryRow {
                title: format!("Entry Mod {i}.dll").into(),
                detail: "2.1 MB · Mods".into(),
                enabled: i % 2 == 0,
                index: i,
            })
            .collect(),
    ));
    ui.set_pack_status("Create packs from your installed mods.".into());
    shot(ui, window, "07-modpacks");
}

fn page_settings(ui: &crate::MainWindow, window: &Rc<MinimalSoftwareWindow>) {
    ui.set_current_icon("settings".into());
    ui.set_current_page("settings".into());
    ui.set_nav_visible(false);
    ui.set_se_workspace("/home/test/DataCenterWS".into());
    ui.set_se_gameroot("/home/test/.steam/steamapps/common/Data Center".into());
    ui.set_se_telemetry_text("Telemetry: off (GREG_TELEMETRY_ENABLED)".into());
    ui.set_se_health(model(vec![
        HealthRow {
            label: "Data Center".into(),
            detail: "/home/test/.steam/steamapps/common/Data Center".into(),
            ok: true,
        },
        HealthRow {
            label: "MelonLoader".into(),
            detail: "MelonLoader is not installed.".into(),
            ok: false,
        },
    ]));
    ui.set_se_log(model(vec!["[1] Session started.".into()]));
    shot(ui, window, "08-settings");
}

fn page_bugdialog(ui: &crate::MainWindow, window: &Rc<MinimalSoftwareWindow>) {
    page_mymods(ui, window);
    ui.set_show_bug_dialog(true);
    shot(ui, window, "09-bugdialog");
}

fn page_report(ui: &crate::MainWindow, window: &Rc<MinimalSoftwareWindow>) {
    ui.set_show_bug_dialog(false);
    page_mymods(ui, window);
    ui.set_current_page("report".into());
    ui.set_rp_categories(strings(&["Bug", "Crash", "Other"]));
    shot(ui, window, "11-report");
}

#[test]
fn screenshots() {
    let window = headless_window();
    let ui = sample_ui();
    page_mymods(&ui, &window);
    page_problems(&ui, &window);
    page_projects(&ui, &window);
    page_editor(&ui, &window);
    page_browse(&ui, &window);
    page_modstore(&ui, &window);
    page_modpacks(&ui, &window);
    page_settings(&ui, &window);
    page_bugdialog(&ui, &window);
    page_report(&ui, &window);
    // Profile session menu (logged-in header state).
    ui.set_show_bug_dialog(false);
    page_mymods(&ui, &window);
    ui.set_login_visible(false);
    ui.set_profile_visible(true);
    ui.set_profile_name("Tester".into());
    ui.set_profile_hint("tester@example.com".into());
    ui.set_show_profile_menu(true);
    shot(&ui, &window, "10-profile-menu");
}

/// Regression test for the upload freeze: project sync-state hashing must
/// run in the background job, never synchronously on the UI thread.
/// A published project with a cold cache paints "?" immediately, spawns the
/// hasher, and flips to SYNCED once the job drains.
#[test]
fn project_sync_hashes_off_ui_thread() {
    use greg_core::models::WorkshopMetadata;

    let _window = headless_window();
    let ui = crate::MainWindow::new().expect("window");
    let state = crate::state::AppState::new(ui.as_weak());

    // Temp workspace with one published project (content + recorded hash).
    let tmp = std::env::temp_dir().join(format!("greg-sync-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    let root = tmp.join("proj1");
    std::fs::create_dir_all(root.join("content")).expect("content dir");
    std::fs::write(root.join("content").join("mod.dll"), vec![7u8; 4096]).expect("content");
    let meta = WorkshopMetadata {
        title: "SyncTest".into(),
        published_file_id: 3807398588,
        ..Default::default()
    };
    let desc = state
        .lock()
        .expect("state")
        .steam
        .build_upload_description(&root, &meta);
    let hash = greg_platform::workspace::compute_publish_hash(&root, &meta, &desc);
    assert!(!hash.is_empty(), "hash must cover content");
    let meta = WorkshopMetadata {
        last_published_hash: hash,
        ..meta
    };
    std::fs::write(
        root.join("metadata.json"),
        serde_json::to_string_pretty(&meta).expect("meta json"),
    )
    .expect("metadata");

    {
        let mut guard = state.lock().expect("state");
        guard.workspace =
            greg_platform::workspace::Workspace::resolve(Some(&tmp.to_string_lossy()), None);
        assert_eq!(guard.workspace.root(), tmp.as_path());
        crate::actions::refresh_projects(&mut guard);
        // Cold cache: instant "?" rows + background job running.
        assert!(
            guard.sync_job.is_some(),
            "sync hashing must run in background"
        );
        let states: Vec<String> = ui
            .get_projects()
            .iter()
            .map(|r| r.state.to_string())
            .collect();
        assert!(
            states.iter().any(|s| s == "?"),
            "cold cache paints unknown, got {states:?}"
        );
    }
    // Drain until the job finishes (tick also repaints on completion).
    for _ in 0..200 {
        {
            let mut guard = state.lock().expect("state");
            crate::actions::tick(&mut guard);
            if guard.sync_job.is_none() {
                break;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
    {
        let guard = state.lock().expect("state");
        assert!(guard.sync_job.is_none(), "sync job must finish");
        let states: Vec<String> = ui
            .get_projects()
            .iter()
            .map(|r| r.state.to_string())
            .collect();
        assert!(
            states.iter().any(|s| s == "SYNCED"),
            "hash matches, expected SYNCED, got {states:?}"
        );
    }
    let _ = std::fs::remove_dir_all(&tmp);
}

/// Report Assistant flow: open → fill draft → generate Markdown preview.
/// Guards the phase-2 domain wiring end to end (validation + generation).
#[test]
fn report_flow_generates_markdown_preview() {
    let _window = headless_window();
    let ui = crate::MainWindow::new().expect("window");
    let state = crate::state::AppState::new(ui.as_weak());
    let state = std::sync::Arc::clone(&state);

    crate::actions::report_open(&state);
    {
        let guard = state.lock().expect("state");
        guard.ui().set_rp_title("Crash on launch".into());
        guard.ui().set_rp_category_index(1);
        guard
            .ui()
            .set_rp_description("App crashes when starting the game.".into());
        guard.ui().set_rp_logs("panic: boom".into());
    }
    crate::actions::report_generate(&state);
    {
        let guard = state.lock().expect("state");
        assert_eq!(guard.ui().get_rp_category_index(), 1);
        let preview = guard.ui().get_rp_preview().to_string();
        assert!(preview.contains("## Summary"), "preview: {preview}");
        assert!(preview.contains("panic: boom"), "preview: {preview}");
        assert!(
            preview.contains("## Logs and Diagnostics"),
            "preview: {preview}"
        );
    }
    // Cancel resets the draft.
    crate::actions::report_cancel(&state);
    {
        let guard = state.lock().expect("state");
        assert!(guard.ui().get_rp_preview().to_string().is_empty());
    }
}

/// Report Assistant flow: open → collect buffered logs → generate Markdown
/// preview. Guards the log-collection wiring into the draft end to end.
#[test]
fn report_flow_collects_logs_into_draft() {
    let _window = headless_window();
    let ui = crate::MainWindow::new().expect("window");
    let state = crate::state::AppState::new(ui.as_weak());
    let state = std::sync::Arc::clone(&state);

    crate::actions::report_open(&state);
    {
        let guard = state.lock().expect("state");
        guard.ui().set_rp_title("Crash on launch".into());
        guard.ui().set_rp_category_index(1);
        guard
            .ui()
            .set_rp_description("App crashes when starting the game.".into());
    }
    // Feed buffered log lines, then let the assistant collect them.
    {
        let mut guard = state.lock().expect("state");
        guard.push_log("panic: boom");
        guard.push_log("ModManager: game launch requested");
    }
    crate::actions::report_collect_logs(&state);
    {
        let guard = state.lock().expect("state");
        let logs = guard.ui().get_rp_logs().to_string();
        assert!(logs.contains("panic: boom"), "logs: {logs}");
        assert!(
            logs.contains("ModManager: game launch requested"),
            "logs: {logs}"
        );
        let preview = guard.ui().get_rp_preview().to_string();
        assert!(
            preview.contains("## Logs and Diagnostics"),
            "preview: {preview}"
        );
        assert!(preview.contains("panic: boom"), "preview: {preview}");
    }
}

/// Regression test for My Mods Disable / Enable / Remove actions (including
/// when a search filter is active and when Dolphin / another process has the
/// game directory open).
#[test]
fn local_toggle_and_remove_work_even_with_filter() {
    use greg_core::prefs::Preferences as _;

    let _window = headless_window();
    let ui = crate::MainWindow::new().expect("window");
    let state = crate::state::AppState::new(ui.as_weak());

    let tmp = std::env::temp_dir().join(format!(
        "greg-local-ui-test-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos()
    ));
    let mods_dir = tmp.join("Mods");
    std::fs::create_dir_all(&mods_dir).expect("mods dir");
    std::fs::write(mods_dir.join("Alpha.dll"), b"alpha").expect("alpha");
    std::fs::write(mods_dir.join("Beta.dll"), b"beta").expect("beta");

    let old_root = {
        let mut guard = state.lock().expect("state");
        let prev = guard.prefs.get_string("GameRootPath", "");
        guard
            .prefs
            .set_string("GameRootPath", &tmp.to_string_lossy());
        guard.ui().set_current_page("mymods".into());
        guard.ui().set_local_search("beta".into());
        crate::actions::local_refresh(&mut guard);
        prev
    };

    // Only Beta.dll matches the filter, but its `row.index` is 1 in `local_entries`.
    let beta_idx = {
        let rows = ui.get_local_active_rows();
        assert_eq!(rows.row_count(), 1);
        let row = rows.row_data(0).expect("beta row");
        assert_eq!(row.name.as_str(), "Beta.dll");
        row.index
    };

    // Disable Beta.dll via `local_toggle`.
    crate::actions::local_toggle(&state, beta_idx);
    assert!(
        mods_dir.join("Beta.dll.disabled").is_file(),
        "Beta.dll should be renamed to Beta.dll.disabled"
    );
    assert_eq!(ui.get_local_active_rows().row_count(), 0);
    assert_eq!(ui.get_local_inactive_rows().row_count(), 1);

    // Remove Beta.dll.disabled via `local_remove` + `confirm_yes`.
    let disabled_idx = ui
        .get_local_inactive_rows()
        .row_data(0)
        .expect("disabled beta")
        .index;
    crate::actions::local_remove(&state, disabled_idx);
    assert!(ui.get_show_confirm(), "confirm dialog should open");
    crate::actions::confirm_yes(&state);
    assert!(!mods_dir.join("Beta.dll.disabled").exists());

    // Restore preference and clean up.
    state
        .lock()
        .expect("state")
        .prefs
        .set_string("GameRootPath", &old_root);
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn mylibs_install_modal_populates_and_selects_libraries() {
    use greg_core::prefs::Preferences as _;

    let _window = headless_window();
    let ui = crate::MainWindow::new().expect("window");
    let state = crate::state::AppState::new(ui.as_weak());

    let tmp = std::env::temp_dir().join(format!(
        "greg-mylibs-modal-test-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos()
    ));
    let user_libs = tmp.join("UserLibs");
    std::fs::create_dir_all(&user_libs).expect("UserLibs dir");
    std::fs::write(user_libs.join("Jint.dll"), b"jint").expect("Jint.dll");

    let old_root = {
        let mut guard = state.lock().expect("state");
        let prev = guard.prefs.get_string("GameRootPath", "");
        guard
            .prefs
            .set_string("GameRootPath", &tmp.to_string_lossy());
        guard.ui().set_current_page("mylibs".into());
        crate::actions::local_refresh(&mut guard);
        prev
    };

    // Open the Install Libraries modal.
    crate::actions::lib_open_modal(&state);
    assert!(ui.get_show_lib_modal());
    let catalog = ui.get_lib_catalog();
    assert_eq!(catalog.row_count(), 9);

    // Jint.dll is already in UserLibs -> installed == true, selected == false.
    // Acornima.dll is missing -> installed == false, selected == true.
    let acornima = catalog.row_data(0).expect("acornima");
    assert_eq!(acornima.name.as_str(), "Acornima.dll");
    assert!(!acornima.installed);
    assert!(acornima.selected);

    let jint = catalog.row_data(1).expect("jint");
    assert_eq!(jint.name.as_str(), "Jint.dll");
    assert!(jint.installed);
    assert!(!jint.selected);

    // Select All selects all 9 (including already-installed Jint.dll).
    crate::actions::lib_select_all(&state);
    for i in 0..9 {
        assert!(ui.get_lib_catalog().row_data(i).expect("row").selected);
    }

    // Select All again when all 9 are selected deselects all 9.
    crate::actions::lib_select_all(&state);
    for i in 0..9 {
        assert!(!ui.get_lib_catalog().row_data(i).expect("row").selected);
    }

    // Select Missing restores selection of the 8 missing libraries.
    crate::actions::lib_select_missing(&state);
    assert!(ui.get_lib_catalog().row_data(0).expect("acornima").selected);
    assert!(!ui.get_lib_catalog().row_data(1).expect("jint").selected);

    crate::actions::lib_close_modal(&state);
    assert!(!ui.get_show_lib_modal());

    state
        .lock()
        .expect("state")
        .prefs
        .set_string("GameRootPath", &old_root);
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn myplugins_browse_modal_populates_and_selects_plugins() {
    use greg_core::prefs::Preferences as _;

    let _window = headless_window();
    let ui = crate::MainWindow::new().expect("window");
    let state = crate::state::AppState::new(ui.as_weak());

    let tmp = std::env::temp_dir().join(format!(
        "greg-myplugins-modal-test-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos()
    ));
    let plugins_dir = tmp.join("Plugins");
    std::fs::create_dir_all(&plugins_dir).expect("Plugins dir");
    std::fs::write(plugins_dir.join("gregPlugin.Companion.dll"), b"companion")
        .expect("Companion.dll");

    let old_root = {
        let mut guard = state.lock().expect("state");
        let prev = guard.prefs.get_string("GameRootPath", "");
        guard
            .prefs
            .set_string("GameRootPath", &tmp.to_string_lossy());
        guard.ui().set_current_page("myplugins".into());
        crate::actions::local_refresh(&mut guard);
        prev
    };

    // Open the Browse Plugins modal.
    crate::actions::plugin_open_modal(&state);
    assert!(ui.get_show_plugin_modal());
    let catalog = ui.get_plugin_catalog();
    assert_eq!(
        catalog.row_count(),
        greg_loader::local_content::FRAMEWORK_PLUGINS.len()
    );

    // Il2CppAssemblyFixerPlugin.dll (index 0) is missing -> installed == false, selected == true.
    let fixer = catalog.row_data(0).expect("fixer");
    assert!(fixer
        .detail
        .as_str()
        .contains("Il2CppAssemblyFixerPlugin.dll"));
    assert!(!fixer.installed);
    assert!(fixer.selected);

    // gregPlugin.Companion.dll (index 1) is present -> installed == true, selected == false.
    let companion = catalog.row_data(1).expect("companion");
    assert!(companion
        .detail
        .as_str()
        .contains("gregPlugin.Companion.dll"));
    assert!(companion.installed);
    assert!(!companion.selected);

    // Select All selects all plugins.
    crate::actions::plugin_select_all(&state);
    for i in 0..catalog.row_count() {
        assert!(ui.get_plugin_catalog().row_data(i).expect("row").selected);
    }

    // Select Missing restores missing-only selection.
    crate::actions::plugin_select_missing(&state);
    assert!(ui.get_plugin_catalog().row_data(0).expect("fixer").selected);
    assert!(
        !ui.get_plugin_catalog()
            .row_data(1)
            .expect("companion")
            .selected
    );

    crate::actions::plugin_close_modal(&state);
    assert!(!ui.get_show_plugin_modal());

    state
        .lock()
        .expect("state")
        .prefs
        .set_string("GameRootPath", &old_root);
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn workshop_item_3741628037_metadata_and_preview_binds_to_browse_ui() {
    use greg_core::prefs::Preferences as _;

    let _window = headless_window();
    let ui = crate::MainWindow::new().expect("window");
    let state = crate::state::AppState::new(ui.as_weak());

    // Configure Steam Web API key in Settings UI and apply it.
    let old_key = state
        .lock()
        .expect("state")
        .prefs
        .get_string("SteamWebApiKey", "");
    ui.set_se_steam_web_api_key("TEST_STEAM_WEB_API_KEY_123".into());
    crate::actions::settings_apply_steam_web_api_key(&state);
    assert_eq!(
        state.lock().expect("state").steam.web_api_key().as_deref(),
        Some("TEST_STEAM_WEB_API_KEY_123")
    );

    // Simulate a resolved ItemDetail for item 3741628037 arriving via browse_job + cached preview.
    let tmp = std::env::temp_dir().join(format!(
        "greg-workshop-3741628037-test-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos()
    ));
    std::fs::create_dir_all(&tmp).expect("tmp dir");
    // Minimal 1x1 PNG for preview verification.
    let png_1x1: &[u8] = &[
        0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, b'I', b'H', b'D',
        b'R', 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, b'I', b'D', b'A', b'T', 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, b'I',
        b'E', b'N', b'D', 0xAE, 0x42, 0x60, 0x82,
    ];
    let preview_path = tmp.join("3741628037.png");
    std::fs::write(&preview_path, png_1x1).expect("write png");

    let item_id = greg_steam::models::EXAMPLE_WORKSHOP_ITEM_ID;
    assert_eq!(item_id, 3741628037);
    let detail = greg_steam::models::ItemDetail {
        published_file_id: item_id,
        title: "Data Center Workshop Example 3741628037".into(),
        description: "Full workshop description for item 3741628037".into(),
        owner_steam_id: 76561198000000001,
        owner_name: "teamGreg".into(),
        visibility: "Public".into(),
        status_state: "OK · Public · Updated 1700000000".into(),
        source: "web_api".into(),
        tags: vec!["DataCenterMod".into(), "gregCore".into()],
        preview_image_url: "https://steamuserimages-a.akamaihd.net/ugc/3741628037/preview.png"
            .into(),
        score: 4.9,
        size_bytes: 2048,
        is_subscribed: true,
        dependency_hint_compact: "Requires gregCore".into(),
        ..Default::default()
    };
    let payload = serde_json::to_string(&detail).expect("json");
    {
        let mut guard = state.lock().expect("state");
        guard
            .thumb_done
            .insert(item_id.to_string(), preview_path.clone());
        guard.browse_job = Some(crate::worker::JobHandle::spawn(move |_sink| {
            crate::worker::JobOutcome {
                success: true,
                message: format!("FULL:{payload}"),
            }
        }));
    }
    for _ in 0..20 {
        std::thread::sleep(std::time::Duration::from_millis(10));
        let mut guard = state.lock().expect("state");
        crate::actions::tick(&mut guard);
        if ui.get_br_has_detail() {
            break;
        }
    }
    assert!(ui.get_br_has_detail());
    assert_eq!(
        ui.get_br_d_title().as_str(),
        "Data Center Workshop Example 3741628037"
    );
    assert_eq!(ui.get_br_d_author().as_str(), "teamGreg");
    assert!(ui
        .get_br_d_status_state()
        .as_str()
        .contains("OK · Public · Updated 1700000000"));
    assert_eq!(
        ui.get_br_d_desc().as_str(),
        "Full workshop description for item 3741628037"
    );
    assert_eq!(ui.get_br_d_preview().size().width, 1);

    state
        .lock()
        .expect("state")
        .prefs
        .set_string("SteamWebApiKey", &old_key);
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn i18n_language_switch_updates_tr_global_and_navigation() {
    use greg_core::prefs::Preferences as _;

    let _window = headless_window();
    let ui = crate::MainWindow::new().expect("window");
    let state = crate::state::AppState::new(ui.as_weak());

    let old_lang = state
        .lock()
        .expect("state")
        .prefs
        .get_string("AppLanguage", "");

    // Switch to German (index 2 = "de").
    ui.set_se_lang(2);
    crate::actions::settings_apply_lang(&state);
    assert_eq!(state.lock().expect("state").lang, "de");
    assert_eq!(
        ui.global::<crate::Tr>().get_btn_start_game().as_str(),
        greg_core::l10n::get("de", "Ui_Btn_StartGame")
    );
    assert_eq!(
        ui.global::<crate::Tr>().get_local_install_libs().as_str(),
        greg_core::l10n::get("de", "Ui_Local_InstallLibs")
    );
    assert_eq!(
        ui.global::<crate::Tr>().get_local_browse_plugins().as_str(),
        greg_core::l10n::get("de", "Ui_Local_BrowsePlugins")
    );
    assert_eq!(
        ui.global::<crate::Tr>().get_settings_language().as_str(),
        "Sprache"
    );

    // Switch back to English (index 1 = "en").
    ui.set_se_lang(1);
    crate::actions::settings_apply_lang(&state);
    assert_eq!(state.lock().expect("state").lang, "en");
    assert_eq!(
        ui.global::<crate::Tr>().get_settings_language().as_str(),
        "Language"
    );

    state
        .lock()
        .expect("state")
        .prefs
        .set_string("AppLanguage", &old_lang);
}
