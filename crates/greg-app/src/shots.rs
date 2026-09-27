//! Headless screenshot tests (software renderer → PNG).
//!
//! Lets layout be verified without a display: each page renders with sample
//! data into `target/screenshots/`. Run with
//! `cargo test -p greg-app --test shots` — no wait, these are bin unittests:
//! `cargo test -p greg-app shots`.

use std::rc::Rc;

use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType, Rgb565Pixel};
use slint::platform::{Platform, PlatformError, WindowAdapter};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

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

fn sample_ui() -> crate::MainWindow {
    let _window = headless_window();
    let ui = crate::MainWindow::new().expect("window");
    ui.window().set_size(slint::LogicalSize::new(1400.0, 900.0));
    ui.set_app_version("1.6.1".into());
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
            ("report-bug", "Report a Bug"),
        ],
        "mymods",
    );
    ui.set_local_title("MY MODS".into());
    ui.set_local_rows(model(
        (0..6)
            .map(|i| crate::LocalRow {
                name: format!("SomeMod{i}.dll").into(),
                detail: "1.2 MB · Mods".into(),
                enabled: i % 2 == 0,
                index: i,
            })
            .collect(),
    ));
    ui.set_local_status("6 entries".into());
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
                score: "★ 4.5".into(),
            })
            .collect(),
    ));
    ui.set_br_has_detail(true);
    ui.set_br_d_title("Workshop Item 0 With A Reasonably Long Title".into());
    ui.set_br_d_meta("by SomeAuthor · ★ 4.5 · 123456 bytes".into());
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
}
