//! `greg://` URL protocol registration (`ProtocolRegistryService` port).

use std::path::Path;

use crate::error::{PlatformError, Result};

/// Registered scheme.
pub const PROTOCOL_SCHEME: &str = "greg";
/// Human-readable description.
pub const PROTOCOL_DESCRIPTION: &str = "gregModmanager Protocol";

/// Registers `greg://` URLs to open with `app_exe`.
/// Best-effort per platform; macOS needs a bundled plist (see docs).
pub fn register_protocol(app_exe: &Path) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        register_windows(app_exe)
    }
    #[cfg(target_os = "linux")]
    {
        register_linux(app_exe)
    }
    #[cfg(target_os = "macos")]
    {
        let _ = app_exe;
        Err(PlatformError::Other(
            "protocol registration on macOS requires a bundled Info.plist; see docs".into(),
        ))
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        let _ = app_exe;
        Err(PlatformError::Other("unsupported OS".into()))
    }
}

#[cfg(target_os = "windows")]
fn register_windows(app_exe: &Path) -> Result<()> {
    // Uses reg.exe so no extra crates are needed.
    let exe = app_exe.to_string_lossy().to_string();
    let key = format!(r"HKCU\Software\Classes\{PROTOCOL_SCHEME}");
    let runs: &[&[&str]] = &[
        &["add", &key, "/ve", "/d", PROTOCOL_DESCRIPTION, "/f"],
        &["add", &key, "/v", "URL Protocol", "/d", "", "/f"],
        &[
            "add",
            &format!(r"{key}\shell\open\command"),
            "/ve",
            "/d",
            &format!("\"{exe}\" \"%1\""),
            "/f",
        ],
    ];
    for args in runs {
        let status = std::process::Command::new("reg")
            .args(*args)
            .status()
            .map_err(|e| PlatformError::Other(format!("reg.exe failed: {e}")))?;
        if !status.success() {
            return Err(PlatformError::Other("reg.exe reported failure".into()));
        }
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn register_linux(app_exe: &Path) -> Result<()> {
    let Some(data_home) = dirs::data_dir() else {
        return Err(PlatformError::Other("no data dir".into()));
    };
    let apps_dir = data_home.join("applications");
    std::fs::create_dir_all(&apps_dir)
        .map_err(|e| PlatformError::Other(format!("cannot create {apps_dir:?}: {e}")))?;
    let desktop = format!(
        "[Desktop Entry]\nType=Application\nName=gregModmanager\nComment={PROTOCOL_DESCRIPTION}\nExec=\"{exe}\" %u\nMimeType=x-scheme-handler/{PROTOCOL_SCHEME};\nNoDisplay=true\n",
        exe = app_exe.display()
    );
    std::fs::write(apps_dir.join("gregmodmanager-protocol.desktop"), desktop)
        .map_err(|e| PlatformError::Other(format!("cannot write desktop entry: {e}")))?;
    Ok(())
}
