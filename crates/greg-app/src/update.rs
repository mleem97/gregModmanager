//! Interactive application updates; all network and disk work stays off the UI thread.
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

use greg_core::l10n;
use greg_platform::update;

use crate::state::AppState;

static CHECKING: AtomicBool = AtomicBool::new(false);

pub fn check(state: &Arc<Mutex<AppState>>, manual: bool) {
    if CHECKING.swap(true, Ordering::AcqRel) {
        return;
    }
    let lang = state.lock().expect("state").lang.clone();
    std::thread::spawn(move || {
        let result = update::latest().and_then(|release| {
            if !update::newer(&release.tag_name, env!("CARGO_PKG_VERSION"))? {
                return Ok(None);
            }
            let kind = update::install_kind();
            let asset = update::select_asset(
                &release,
                &kind,
                std::env::consts::OS,
                std::env::consts::ARCH,
            )
            .cloned();
            Ok(Some((release.tag_name, kind, asset)))
        });
        let _ = slint::invoke_from_event_loop(move || {
            CHECKING.store(false, Ordering::Release);
            match result {
                Ok(None) if manual => info(&lang, &l10n::get(&lang, "Ui_Update_Current")),
                Ok(None) => {}
                Err(error) if manual => error_dialog(&lang, &error.to_string()),
                Err(_) => {}
                Ok(Some((version, kind, asset))) => {
                    let Some(asset) = asset else {
                        info(&lang, &l10n::get(&lang, "Ui_Update_Missing"));
                        return;
                    };
                    let prompt = l10n::format(&lang, "Ui_Update_Available", &[&version]);
                    if rfd::MessageDialog::new()
                        .set_title(l10n::get(&lang, "Ui_Update_Title"))
                        .set_description(prompt)
                        .set_buttons(rfd::MessageButtons::YesNo)
                        .show()
                        != rfd::MessageDialogResult::Yes
                    {
                        return;
                    }
                    std::thread::spawn(move || {
                        let result = update::download(&asset);
                        let _ = slint::invoke_from_event_loop(move || match result {
                            Ok(path) if kind == "portable" => {
                                let path = path.display().to_string();
                                info(&lang, &l10n::format(&lang, "Ui_Update_Portable", &[&path]));
                            }
                            Ok(path) => {
                                if let Err(error) = greg_platform::process::open_folder(&path) {
                                    error_dialog(&lang, &error.to_string());
                                }
                            }
                            Err(error) => error_dialog(&lang, &error.to_string()),
                        });
                    });
                }
            }
        });
    });
}

fn info(lang: &str, message: &str) {
    rfd::MessageDialog::new()
        .set_title(l10n::get(lang, "Ui_Update_Title"))
        .set_description(message)
        .show();
}

fn error_dialog(lang: &str, error: &str) {
    info(lang, &l10n::format(lang, "Ui_Update_Error", &[&error]));
}
