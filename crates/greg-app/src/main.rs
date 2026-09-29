//! Desktop UI entry point (Slint main window).

mod actions;
mod state;
mod worker;

#[cfg(test)]
mod shots;

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
    // Single instance: a second launch forwards `greg://` URLs to the
    // primary through the handoff file and exits (OAuth browser roundtrip).
    // Stale locks (killed predecessor) are taken over by the guard itself.
    let instance = match greg_platform::instance::InstanceGuard::acquire("gregmodmanager") {
        Ok(guard) => Some(guard),
        Err(_) => {
            for arg in std::env::args().skip(1) {
                if arg.starts_with("greg://") {
                    actions::forward_to_primary(&arg);
                }
            }
            std::process::exit(0);
        }
    };
    let state = AppState::new(ui.as_weak());
    state.lock().expect("state").instance_guard = instance;
    wire(&ui, &state);
    AppState::append_log(&state, "gregModmanager started.");
    // Session restore + protocol URLs from our own launch arguments.
    actions::restore_session(&state);
    for arg in std::env::args().skip(1) {
        if arg.starts_with("greg://") {
            actions::handle_protocol_url(&state, &arg);
        }
    }
    // Repeating UI tick (kept alive until the event loop exits).
    let weak_state = std::sync::Arc::downgrade(&state);
    let _timer = slint::Timer::default();
    let mut tick_count: u64 = 0;
    _timer.start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_millis(150),
        move || {
            if let Some(state) = weak_state.upgrade() {
                if let Ok(mut guard) = state.lock() {
                    actions::tick(&mut guard);
                }
                // Second-instance `greg://` handoffs (~2 s cadence).
                tick_count += 1;
                if tick_count.is_multiple_of(13) {
                    actions::drain_handoff(&state);
                }
                // Single-instance heartbeat (~4.5 s cadence).
                if tick_count.is_multiple_of(30) {
                    if let Ok(guard) = state.lock() {
                        if let Some(instance) = guard.instance_guard.as_ref() {
                            instance.heartbeat();
                        }
                    }
                }
            }
        },
    );
    ui.run()?;
    actions::finalize_game_state(&state);
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
        ui.on_stop_game(move || actions::stop_game(&state));
    }
    {
        let state = Arc::clone(state);
        ui.on_login_clicked(move || actions::login(&state));
    }
    {
        let state = Arc::clone(state);
        ui.on_profile_clicked(move || actions::profile_clicked(&state));
    }
    {
        let state = Arc::clone(state);
        ui.on_profile_menu_clicked(move |id| actions::profile_menu(&state, &id));
    }
    // Custom window chrome.
    {
        let state = Arc::clone(state);
        ui.on_window_minimize(move || actions::window_minimize(&state));
    }
    {
        let state = Arc::clone(state);
        ui.on_window_maximize(move || actions::window_maximize(&state));
    }
    {
        let state = Arc::clone(state);
        ui.on_window_close(move || actions::window_close(&state));
    }

    // Local content.
    {
        let state = Arc::clone(state);
        ui.on_local_refresh(move || {
            if let Ok(mut guard) = state.lock() {
                actions::local_refresh(&mut guard);
            }
        });
    }
    {
        let state = Arc::clone(state);
        ui.on_local_toggle(move |idx| actions::local_toggle(&state, idx));
    }
    {
        let state = Arc::clone(state);
        ui.on_local_remove(move |idx| actions::local_remove(&state, idx));
    }
    {
        let state = Arc::clone(state);
        ui.on_local_open_folder(move || actions::local_open_folder(&state));
    }
    {
        let state = Arc::clone(state);
        ui.on_confirm_yes(move || actions::confirm_yes(&state));
    }
    {
        let state = Arc::clone(state);
        ui.on_confirm_no(move || actions::confirm_no(&state));
    }
    // Problems.
    {
        let state = Arc::clone(state);
        ui.on_problems_refresh(move || actions::refresh_problems(&state));
    }
    {
        let state = Arc::clone(state);
        ui.on_problems_act(move |idx| actions::problems_act(&state, idx));
    }
    // Packs (Modpacks + Collections share one component).
    {
        let state = Arc::clone(state);
        ui.on_packs_create(move || actions::packs_create(&state));
    }
    {
        let state = Arc::clone(state);
        ui.on_packs_select(move |idx| actions::packs_select(&state, idx));
    }
    {
        let state = Arc::clone(state);
        ui.on_packs_toggle(move |idx| actions::packs_toggle(&state, idx));
    }
    {
        let state = Arc::clone(state);
        ui.on_packs_delete(move |idx| actions::packs_delete(&state, idx));
    }
    {
        let state = Arc::clone(state);
        ui.on_packs_toggle_entry(move |idx| actions::packs_toggle_entry(&state, idx));
    }
    {
        let state = Arc::clone(state);
        ui.on_packs_remove_entry(move |idx| actions::packs_remove_entry(&state, idx));
    }
    {
        let state = Arc::clone(state);
        ui.on_packs_add_file(move || actions::packs_add_file(&state));
    }
    {
        let state = Arc::clone(state);
        ui.on_packs_export(move |_idx| actions::packs_export(&state));
    }
    {
        let state = Arc::clone(state);
        ui.on_packs_import(move || actions::packs_import(&state));
    }
    {
        let state = Arc::clone(state);
        ui.on_packs_apply(move |idx| actions::packs_apply(&state, idx));
    }
    // Bug report dialog.
    {
        let state = Arc::clone(state);
        ui.on_bug_open_manager(move || actions::bug_open_manager(&state));
    }
    {
        let state = Arc::clone(state);
        ui.on_bug_open_org(move || actions::bug_open_org(&state));
    }
    {
        let state = Arc::clone(state);
        ui.on_bug_close(move || actions::bug_close(&state));
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
        ui.on_br_filter_changed(move || actions::browse_filter_changed(&state));
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
