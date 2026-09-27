//! Desktop UI entry point (Slint main window).

mod actions;
mod state;
mod worker;

slint::include_modules!();

use std::sync::{Arc, Mutex};

use greg_platform::filelog::FileLog;
use state::AppState;

fn main() -> anyhow::Result<()> {
    let log = FileLog::shared().clone();
    log.start_session();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let ui = MainWindow::new()?;
    let state = AppState::new(ui.as_weak());
    wire(&ui, &state);
    AppState::append_log(&state, "gregModmanager started.");
    // Repeating UI tick (kept alive until the event loop exits).
    let weak_state = std::sync::Arc::downgrade(&state);
    let _timer = slint::Timer::default();
    _timer.start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_millis(150),
        move || {
            if let Some(state) = weak_state.upgrade() {
                if let Ok(mut guard) = state.lock() {
                    actions::tick(&mut guard);
                }
            }
        },
    );
    ui.run()?;
    log.end_session();
    Ok(())
}

/// Connects every Slint callback to its action.
fn wire(ui: &MainWindow, state: &Arc<Mutex<AppState>>) {
    // Shell.
    {
        let state = Arc::clone(state);
        ui.on_icon_clicked(move |id| actions::on_icon(&state, &id));
    }
    {
        let state = Arc::clone(state);
        ui.on_nav_clicked(move |id| actions::on_nav(&state, &id));
    }
    {
        let state = Arc::clone(state);
        ui.on_start_game(move |mode| actions::start_game(&state, &mode));
    }
    {
        let state = Arc::clone(state);
        ui.on_login_clicked(move || actions::login(&state));
    }
    {
        let state = Arc::clone(state);
        ui.on_profile_clicked(move || actions::profile_clicked(&state));
    }
    // Projects.
    {
        let state = Arc::clone(state);
        ui.on_project_refresh(move || {
            if let Ok(mut guard) = state.lock() {
                actions::refresh_projects(&mut guard);
            }
        });
    }
    {
        let state = Arc::clone(state);
        ui.on_project_open(move |root| actions::open_project(&state, &root));
    }
    {
        let state = Arc::clone(state);
        ui.on_project_open_folder(move || {
            let root = {
                let guard = state.lock().expect("state");
                guard.workspace.root().to_path_buf()
            };
            let _ = greg_platform::process::open_folder(&root);
        });
    }
    {
        let state = Arc::clone(state);
        ui.on_project_create(move || actions::create_project(&state));
    }
    {
        let state = Arc::clone(state);
        ui.on_project_view_steam(move |id| actions::view_on_steam(&state, &id));
    }
    // Editor.
    {
        let state = Arc::clone(state);
        ui.on_ed_changed(move || actions::editor_changed(&state));
    }
    {
        let state = Arc::clone(state);
        ui.on_ed_save(move || actions::editor_save(&state));
    }
    {
        let state = Arc::clone(state);
        ui.on_ed_publish(move || actions::editor_publish(&state));
    }
    {
        let state = Arc::clone(state);
        ui.on_ed_cancel_publish(move || actions::editor_cancel_publish(&state));
    }
    {
        let state = Arc::clone(state);
        ui.on_ed_pick_preview(move || actions::editor_pick_preview(&state));
    }
    {
        let state = Arc::clone(state);
        ui.on_ed_add_shot(move || actions::editor_add_shot(&state));
    }
    {
        let state = Arc::clone(state);
        ui.on_ed_remove_shot(move |idx| actions::editor_remove_shot(&state, idx));
    }
    {
        let state = Arc::clone(state);
        ui.on_ed_dep_add(move || actions::editor_dep_add(&state));
    }
    {
        let state = Arc::clone(state);
        ui.on_ed_dep_remove(move |idx| actions::editor_dep_remove(&state, idx));
    }
    {
        let state = Arc::clone(state);
        ui.on_ed_view_steam(move || {
            let file_id = {
                let guard = state.lock().expect("state");
                guard.editor_meta.published_file_id.to_string()
            };
            actions::view_on_steam(&state, &file_id);
        });
    }
    // Browse.
    {
        let state = Arc::clone(state);
        ui.on_br_go(move || actions::browse_go(&state, None));
    }
    {
        let state = Arc::clone(state);
        ui.on_br_cancel(move || actions::browse_cancel(&state));
    }
    {
        let state = Arc::clone(state);
        ui.on_br_select(move |id| actions::browse_select(&state, &id));
    }
    {
        let state = Arc::clone(state);
        ui.on_br_prev(move || actions::browse_page(&state, -1));
    }
    {
        let state = Arc::clone(state);
        ui.on_br_next(move || actions::browse_page(&state, 1));
    }
    {
        let state = Arc::clone(state);
        ui.on_br_subscribe_toggle(move || actions::browse_subscribe_toggle(&state));
    }
    {
        let state = Arc::clone(state);
        ui.on_br_import(move || actions::browse_import(&state));
    }
    {
        let state = Arc::clone(state);
        ui.on_br_view_steam(move || {
            let id = {
                let guard = state.lock().expect("state");
                guard.browse_detail_id.unwrap_or(0).to_string()
            };
            actions::view_on_steam(&state, &id);
        });
    }
    // Modstore.
    {
        let state = Arc::clone(state);
        ui.on_st_refresh(move || actions::store_refresh(&state));
    }
    {
        let state = Arc::clone(state);
        ui.on_st_check_updates(move || actions::store_check_updates(&state));
    }
    {
        let state = Arc::clone(state);
        ui.on_st_cancel(move || actions::store_cancel(&state));
    }
    {
        let state = Arc::clone(state);
        ui.on_st_install(move |idx| actions::store_install(&state, idx, false));
    }
    {
        let state = Arc::clone(state);
        ui.on_st_install_update(move |idx| actions::store_install(&state, idx, true));
    }
    // Settings.
    {
        let state = Arc::clone(state);
        ui.on_se_apply_workspace(move || actions::settings_apply_workspace(&state));
    }
    {
        let state = Arc::clone(state);
        ui.on_se_browse_workspace(move || actions::pick_folder_into(&state, "workspace"));
    }
    {
        let state = Arc::clone(state);
        ui.on_se_apply_gameroot(move || actions::settings_apply_gameroot(&state));
    }
    {
        let state = Arc::clone(state);
        ui.on_se_browse_gameroot(move || actions::pick_folder_into(&state, "gameroot"));
    }
    {
        let state = Arc::clone(state);
        ui.on_se_apply_modstore_url(move || actions::settings_apply_modstore_url(&state));
    }
    {
        let state = Arc::clone(state);
        ui.on_se_run_checks(move || actions::settings_run_checks(&state));
    }
    {
        let state = Arc::clone(state);
        ui.on_se_open_logs(move || actions::settings_open_logs(&state));
    }
    {
        let state = Arc::clone(state);
        ui.on_se_make_bundle(move || actions::settings_make_bundle(&state));
    }
    {
        let state = Arc::clone(state);
        ui.on_se_register_protocol(move || actions::settings_register_protocol(&state));
    }
}
