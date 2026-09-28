//! Safe process / URL / folder launching (`SafeProcess` port).

use std::path::Path;

use crate::error::{PlatformError, Result};

/// Opens a URL in the default browser.
pub fn open_url(url: &str) -> Result<()> {
    open::that(url).map_err(|e| PlatformError::Other(format!("cannot open URL: {e}")))
}

/// Opens a folder in the system file manager.
pub fn open_folder(path: &Path) -> Result<()> {
    open::that(path).map_err(|e| PlatformError::Other(format!("cannot open folder: {e}")))
}

/// Reveals a file in the system file manager (falls back to its folder).
pub fn reveal_file(path: &Path) -> Result<()> {
    if open::that(path).is_ok() {
        return Ok(());
    }
    match path.parent() {
        Some(dir) => open_folder(dir),
        None => Err(PlatformError::Other("no parent folder".into())),
    }
}

/// Launches an executable with arguments (detached).
pub fn launch_app(exe: &Path, args: &[String]) -> Result<()> {
    std::process::Command::new(exe)
        .args(args)
        .spawn()
        .map(|_| ())
        .map_err(|e| PlatformError::Other(format!("cannot launch {}: {e}", exe.display())))
}
