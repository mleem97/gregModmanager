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

/// Launches a Windows Steam game through the installed Proton runner on Linux.
///
/// Calling `steam -applaunch` is not suitable here: this process itself uses
/// the game's AppID for Workshop access, so Steam sees the manager as the
/// already-running game. Starting Proton directly avoids that duplicate app
/// launch while the injected IDs still let the game initialize Steamworks.
#[cfg(target_os = "linux")]
pub fn launch_proton_app(exe: &Path, game_root: &Path, app_id: u32) -> Result<()> {
    let common_dir = game_root
        .parent()
        .ok_or_else(|| PlatformError::Other("game path has no steamapps/common parent".into()))?;
    let library_root = common_dir
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| PlatformError::Other("cannot determine Steam library root".into()))?;
    let proton = find_proton_runner(common_dir)?;
    let compat_data = library_root
        .join("steamapps/compatdata")
        .join(app_id.to_string());
    std::fs::create_dir_all(&compat_data)
        .map_err(|e| PlatformError::Other(format!("cannot create Proton prefix: {e}")))?;

    // Steamworks uses these variables to recognize the game when it was not
    // started by Steam's `-applaunch` command. They are intentionally set on
    // the Proton child only; they must not change the manager's own process.
    let mut command = std::process::Command::new(&proton);
    command
        .arg("run")
        .arg(exe)
        .current_dir(game_root)
        .env("SteamAppId", app_id.to_string())
        .env("SteamAppID", app_id.to_string())
        .env("SteamGameId", app_id.to_string())
        // Steam's Proton launch wrapper sets this for MelonLoader's native
        // version.dll proxy. Without it the game starts, but MelonLoader is
        // silently skipped even though the installation is present.
        .env("WINEDLLOVERRIDES", "version=n,b")
        .env("STEAM_COMPAT_DATA_PATH", &compat_data)
        .env("STEAM_COMPAT_INSTALL_PATH", game_root)
        .env("STEAM_COMPAT_CLIENT_INSTALL_PATH", library_root);
    command.spawn().map(|_| ()).map_err(|e| {
        PlatformError::Other(format!(
            "cannot launch {} through Proton ({}): {e}",
            exe.display(),
            proton.display()
        ))
    })
}

#[cfg(target_os = "linux")]
fn find_proton_runner(common_dir: &Path) -> Result<std::path::PathBuf> {
    let preferred = [
        "Proton - Experimental",
        "Proton - Hotfix",
        "Proton 9.0",
        "Proton 8.0",
        "Proton 7.0",
    ];
    for name in preferred {
        let runner = common_dir.join(name).join("proton");
        if runner.is_file() {
            return Ok(runner);
        }
    }
    let mut candidates = std::fs::read_dir(common_dir)
        .map_err(|e| PlatformError::Other(format!("cannot scan Steam runtimes: {e}")))?
        .flatten()
        .map(|entry| entry.path().join("proton"))
        .filter(|runner| runner.is_file())
        .collect::<Vec<_>>();
    candidates.sort();
    candidates.into_iter().next().ok_or_else(|| {
        PlatformError::Other("no Proton runner found in Steam/steamapps/common".into())
    })
}

/// A supervised child process (game launch tracking).
///
/// The handle owns the `Child`: dropping it without `stop` leaves the
/// process running (detached), so callers must poll or stop explicitly.
pub struct TrackedChild {
    child: std::process::Child,
    pid: u32,
    started_at: std::time::SystemTime,
}

/// Spawns `exe` with `args` in `workdir` (or its parent dir) and tracks it.
pub fn spawn_tracked(exe: &Path, args: &[String], workdir: Option<&Path>) -> Result<TrackedChild> {
    let dir: &Path = match workdir {
        Some(d) => d,
        None => exe.parent().unwrap_or_else(|| Path::new(".")),
    };
    let mut child = std::process::Command::new(exe)
        .args(args)
        .current_dir(dir)
        .spawn()
        .map_err(|e| PlatformError::Other(format!("cannot launch {}: {e}", exe.display())))?;
    // Reap the pid eagerly: `id()` is only valid before any wait.
    let pid = child.id();
    // Touch the handle so a double-spawn bug surfaces early in logs/tests.
    let _ = child.try_wait();
    Ok(TrackedChild {
        child,
        pid,
        started_at: std::time::SystemTime::now(),
    })
}

impl TrackedChild {
    /// OS process id.
    pub fn pid(&self) -> u32 {
        self.pid
    }

    /// Spawn timestamp.
    pub fn started_at(&self) -> std::time::SystemTime {
        self.started_at
    }

    /// True while the process has not exited yet.
    pub fn is_running(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    /// Exit code when the process already finished.
    pub fn exit_code(&mut self) -> Option<i32> {
        match self.child.try_wait() {
            Ok(Some(status)) => status.code(),
            _ => None,
        }
    }

    /// Kills the process (SIGKILL / TerminateProcess, no tree).
    pub fn kill(&mut self) -> Result<()> {
        self.child
            .kill()
            .map_err(|e| PlatformError::Other(format!("cannot kill pid {}: {e}", self.pid)))
    }

    /// Waits for exit (up to `timeout`), killing the process when it
    /// outlives the wait. Returns the exit code when known.
    pub fn stop_blocking(&mut self, timeout: std::time::Duration) -> Option<i32> {
        let deadline = std::time::Instant::now() + timeout;
        loop {
            match self.child.try_wait() {
                Ok(Some(status)) => return status.code(),
                Ok(None) => {}
                Err(_) => return None,
            }
            if std::time::Instant::now() >= deadline {
                let _ = self.child.kill();
                std::thread::sleep(std::time::Duration::from_millis(250));
                return self.exit_code();
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}

/// Kills `pid` including its child tree.
///
/// Windows uses `taskkill /T /F` (kills zombies the game left behind);
/// Unix sends SIGKILL to the single pid (games here do not daemonize).
pub fn kill_tree(pid: u32) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        let status = std::process::Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .status()
            .map_err(|e| PlatformError::Other(format!("taskkill failed: {e}")))?;
        if status.success() {
            Ok(())
        } else {
            Err(PlatformError::Other(format!(
                "taskkill exited with {status} for pid {pid}"
            )))
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let status = std::process::Command::new("kill")
            .args(["-KILL", &pid.to_string()])
            .status()
            .map_err(|e| PlatformError::Other(format!("kill failed: {e}")))?;
        if status.success() {
            Ok(())
        } else {
            Err(PlatformError::Other(format!(
                "kill exited with {status} for pid {pid}"
            )))
        }
    }
}

/// Finds running pids whose executable file name matches (case-insensitive).
///
/// Used to detect stale game processes the manager did not start itself
/// (Windows zombie case): launch is refused until they are gone so Steam
/// and the game do not disagree about the running state.
pub fn find_pids_by_exe(file_name: &str) -> Vec<u32> {
    let want = file_name.to_lowercase();
    #[cfg(target_os = "windows")]
    {
        parse_tasklist_csv(&tasklist_csv(), &want)
    }
    #[cfg(target_os = "linux")]
    {
        scan_proc_exe(&want)
    }
    #[cfg(target_os = "macos")]
    {
        // Keep the original spelling: macOS process names are case-sensitive
        // for `pgrep -x`, while Linux matching below normalizes case.
        scan_pgrep(file_name)
    }
}

#[cfg(target_os = "windows")]
fn tasklist_csv() -> String {
    std::process::Command::new("tasklist")
        .args(["/FO", "CSV", "/NH"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default()
}

/// Parses `tasklist /FO CSV /NH` output (`"name","pid",...` per line).
#[cfg(target_os = "windows")]
fn parse_tasklist_csv(csv: &str, want_lower: &str) -> Vec<u32> {
    let own = std::process::id();
    let mut out = Vec::new();
    for line in csv.lines() {
        let mut cells = line.split("\",\"");
        let name = cells.next().unwrap_or("").trim_matches('"').to_lowercase();
        let pid: Option<u32> = cells.next().unwrap_or("").trim_matches('"').parse().ok();
        if let Some(pid) = pid {
            if pid != own && name == *want_lower {
                out.push(pid);
            }
        }
    }
    out.sort_unstable();
    out.dedup();
    out
}

/// Scans `/proc/<pid>/exe` symlinks on Linux.
#[cfg(target_os = "linux")]
fn scan_proc_exe(want_lower: &str) -> Vec<u32> {
    let own = std::process::id();
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return out;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let Ok(pid): std::result::Result<u32, _> = name.parse() else {
            continue;
        };
        if pid == own {
            continue;
        }
        let exe = std::fs::read_link(format!("/proc/{pid}/exe"));
        let file = exe
            .ok()
            .and_then(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map(str::to_lowercase)
            })
            .unwrap_or_default();
        // Native Linux processes expose the executable through `/proc`. Wine
        // / Proton processes often expose the Wine loader instead, so inspect
        // the command line as a fallback and match the Windows game name.
        let command_line_matches = std::fs::read(format!("/proc/{pid}/cmdline"))
            .ok()
            .map(|bytes| {
                bytes.split(|byte| *byte == 0).any(|arg| {
                    std::path::Path::new(std::str::from_utf8(arg).unwrap_or_default())
                        .file_name()
                        .and_then(|name| name.to_str())
                        .map(|name| name.to_lowercase() == *want_lower)
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false);
        if file == *want_lower || command_line_matches {
            out.push(pid);
        }
    }
    out.sort_unstable();
    out.dedup();
    out
}

/// Finds exact executable-name matches on macOS.
///
/// macOS has no `/proc` filesystem by default. `pgrep -x` is provided by the
/// operating system and avoids parsing localized process-list output.
#[cfg(target_os = "macos")]
fn scan_pgrep(file_name: &str) -> Vec<u32> {
    let own = std::process::id();
    let output = std::process::Command::new("pgrep")
        .args(["-x", file_name])
        .output();
    let Ok(output) = output else {
        return Vec::new();
    };

    let mut out: Vec<u32> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.trim().parse().ok())
        .filter(|pid| *pid != own)
        .collect();
    out.sort_unstable();
    out.dedup();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracked_child_lifecycle() {
        #[cfg(target_os = "windows")]
        let prog = ("cmd", vec!["/C".to_string(), "exit 0".to_string()]);
        #[cfg(not(target_os = "windows"))]
        let prog = ("/bin/true", vec![]);
        let mut tracked = spawn_tracked(Path::new(prog.0), &prog.1, None).expect("spawn helper");
        assert!(tracked.pid() != std::process::id());
        // `true` exits immediately; poll until reaped (max ~2 s).
        let mut alive = true;
        for _ in 0..20 {
            if !tracked.is_running() {
                alive = false;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        assert!(!alive, "helper process should have exited");
    }

    #[test]
    fn finds_spawned_process_by_exe_name() {
        // Spawn a sleeper, prove the scan sees its pid, then clean up.
        // No `current_exe`: the scan must work for foreign processes.
        #[cfg(target_os = "windows")]
        let (prog, name, args) = (
            Path::new("powershell.exe"),
            "powershell.exe",
            vec![
                "-NoProfile".to_string(),
                "-Command".to_string(),
                "Start-Sleep -Seconds 30".to_string(),
            ],
        );
        #[cfg(not(target_os = "windows"))]
        let (prog, name, args) = (Path::new("/bin/sleep"), "sleep", vec!["30".to_string()]);
        let mut tracked = spawn_tracked(prog, &args, None).expect("spawn sleeper");
        let pid = tracked.pid();
        assert!(find_pids_by_exe(name).contains(&pid));
        assert!(find_pids_by_exe("definitely-not-a-real-process-xyz123").is_empty());
        let _ = tracked.stop_blocking(std::time::Duration::from_secs(5));
        assert!(!tracked.is_running());
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn parses_tasklist() {
        let csv = "\"game.exe\",\"1234\",\"Console\",\"1\",\"10.000 K\"\r\n\"other.exe\",\"99\",\"Console\",\"1\",\"1 K\"\r\n";
        assert_eq!(parse_tasklist_csv(csv, "game.exe"), vec![1234]);
        assert!(parse_tasklist_csv(csv, "missing.exe").is_empty());
    }
}
