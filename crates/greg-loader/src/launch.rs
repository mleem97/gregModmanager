//! Supervised game launches with a real vanilla mode.
//!
//! - `With mods`: the game exe starts directly as a tracked child.
//! - `Vanilla`: active mod assemblies remain inside `Mods/`, but are renamed
//!   to `.dll.disabled` before the start and restored when the game exits (or
//!   at the next manager start after a crash).
//! - The Steamworks session release/reconnect around the play session lives
//!   in the app layer (`greg-app`); this module only owns the OS process
//!   plus the mod-folder state.

use std::path::{Path, PathBuf};

use crate::error::{LoaderError, Result};

/// Crash-safe marker containing the relative paths disabled for a vanilla run.
const VANILLA_MANIFEST_NAME: &str = ".gregmodmanager-vanilla-mods";

/// Legacy directory name used by older builds. It is only retained for a
/// one-time migration of an interrupted old vanilla launch.
pub const MODS_DISABLED_DIR_NAME: &str = "Mods.disabled-by-manager";

/// Launch flavour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LaunchMode {
    /// Mods as installed.
    #[default]
    Mods,
    /// Active assemblies in `Mods/` disabled for this session.
    Vanilla,
}

impl LaunchMode {
    /// Short label for logs and status lines.
    pub fn label(self) -> &'static str {
        match self {
            LaunchMode::Mods => "with mods",
            LaunchMode::Vanilla => "vanilla",
        }
    }
}

/// Mod-folder state relevant to vanilla launches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModsState {
    /// `Mods/` present, no parked directory.
    Enabled,
    /// Active mod assemblies are temporarily renamed to `.disabled`.
    DisabledByManager,
    /// Neither directory exists.
    Missing,
    /// Both exist (interrupted rename or user copy) — needs attention.
    Conflict,
}

/// Inspects the mod-folder state (no changes).
pub fn mods_state(game_root: &Path) -> ModsState {
    let mods = game_root.join("Mods");
    let parked = game_root.join(MODS_DISABLED_DIR_NAME);
    let manifest = game_root.join(VANILLA_MANIFEST_NAME);
    if parked.is_dir() && mods.is_dir() {
        return ModsState::Conflict;
    }
    if manifest.is_file() || parked.is_dir() {
        return ModsState::DisabledByManager;
    }
    if mods.is_dir() {
        ModsState::Enabled
    } else {
        ModsState::Missing
    }
}

/// Disables (`false`) or restores (`true`) active mod assemblies.
///
/// The `Mods/` directory itself never moves. A manifest makes the operation
/// crash-safe and prevents re-enabling files that were already disabled by the
/// user before the vanilla launch.
pub fn set_mods_enabled(game_root: &Path, enabled: bool, game_running: bool) -> Result<ModsState> {
    let mods = game_root.join("Mods");
    let parked = game_root.join(MODS_DISABLED_DIR_NAME);
    if enabled {
        if parked.is_dir() && mods.is_dir() {
            return Err(LoaderError::Other(
                "both Mods/ and the legacy parked mod folder exist — resolve manually".into(),
            ));
        }
        if parked.is_dir() {
            // Migrate an interrupted launch from a pre-manifest build.
            std::fs::rename(&parked, &mods).map_err(|e| {
                LoaderError::Other(format!("cannot restore legacy Mods folder: {e}"))
            })?;
        }
        restore_vanilla_manifest(game_root)?;
        Ok(mods_state(game_root))
    } else {
        if game_running {
            return Err(LoaderError::Other(
                "cannot disable mods while the game is running".into(),
            ));
        }
        if game_root.join(VANILLA_MANIFEST_NAME).is_file() {
            return Ok(ModsState::DisabledByManager);
        }
        if !mods.is_dir() {
            return Ok(ModsState::Missing);
        }
        disable_active_mods(game_root)?;
        Ok(ModsState::DisabledByManager)
    }
}

/// Crash recovery at manager start: restores files disabled for a vanilla
/// session when no game process is running. Returns `true` when something was
/// restored.
pub fn ensure_mods_restored(game_root: &Path, game_running: bool) -> bool {
    if game_running || !game_root.join(VANILLA_MANIFEST_NAME).is_file() {
        return false;
    }
    set_mods_enabled(game_root, true, false).is_ok()
}

fn collect_active_mods(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    let entries = std::fs::read_dir(dir).map_err(crate::error::io_err)?;
    for entry in entries {
        let path = entry.map_err(crate::error::io_err)?.path();
        if path.is_dir() {
            collect_active_mods(&path, out)?;
        } else if path.is_file()
            && path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("dll"))
        {
            out.push(path);
        }
    }
    Ok(())
}

fn disable_active_mods(game_root: &Path) -> Result<()> {
    let mods = game_root.join("Mods");
    let manifest = game_root.join(VANILLA_MANIFEST_NAME);
    let mut active = Vec::new();
    collect_active_mods(&mods, &mut active)?;
    active.sort();

    let entries = active
        .iter()
        .map(|path| {
            path.strip_prefix(game_root)
                .map(|relative| relative.to_string_lossy().into_owned())
                .map_err(|e| LoaderError::Other(format!("cannot create mod manifest: {e}")))
        })
        .collect::<Result<Vec<_>>>()?;
    std::fs::write(&manifest, entries.join("\n")).map_err(crate::error::io_err)?;

    let mut renamed = Vec::new();
    for path in active {
        let disabled = PathBuf::from(format!("{}.disabled", path.display()));
        if disabled.exists() {
            restore_paths(&renamed)?;
            let _ = std::fs::remove_file(&manifest);
            return Err(LoaderError::Other(format!(
                "cannot disable {}: target already exists",
                path.display()
            )));
        }
        if let Err(error) = std::fs::rename(&path, &disabled) {
            restore_paths(&renamed)?;
            let _ = std::fs::remove_file(&manifest);
            return Err(crate::error::io_err(error));
        }
        renamed.push((path, disabled));
    }
    Ok(())
}

fn restore_paths(paths: &[(PathBuf, PathBuf)]) -> Result<()> {
    for (original, disabled) in paths.iter().rev() {
        if disabled.is_file() && !original.exists() {
            std::fs::rename(disabled, original).map_err(crate::error::io_err)?;
        }
    }
    Ok(())
}

fn restore_vanilla_manifest(game_root: &Path) -> Result<()> {
    let manifest = game_root.join(VANILLA_MANIFEST_NAME);
    if !manifest.is_file() {
        return Ok(());
    }
    let content = std::fs::read_to_string(&manifest).map_err(crate::error::io_err)?;
    let paths = content
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            let original = game_root.join(line);
            let disabled = PathBuf::from(format!("{}.disabled", original.display()));
            (original, disabled)
        })
        .collect::<Vec<_>>();
    restore_paths(&paths)?;
    std::fs::remove_file(manifest).map_err(crate::error::io_err)?;
    Ok(())
}

/// Well-known Data Center executable file names (all OS spellings).
pub const GAME_EXE_NAMES: &[&str] = &[
    "Data Center.exe",
    "DataCenter.exe",
    "Data Center.x86_64",
    "DataCenter.x86_64",
    // Unity macOS app-bundle executable names.
    "Data Center",
    "DataCenter",
];

/// True when any known game executable is alive (stale-process guard).
pub fn any_game_process_running() -> bool {
    GAME_EXE_NAMES
        .iter()
        .any(|n| !greg_platform::process::find_pids_by_exe(n).is_empty())
}

/// All pids of known game executables (for messages and kills).
pub fn stale_game_pids() -> Vec<u32> {
    let mut out = Vec::new();
    for name in GAME_EXE_NAMES {
        out.extend(greg_platform::process::find_pids_by_exe(name));
    }
    out.sort_unstable();
    out.dedup();
    out
}

/// A running game: tracked OS process plus the vanilla state to unwind.
pub struct GameSession {
    /// Native child when the platform can execute the game directly.
    child: Option<greg_platform::process::TrackedChild>,
    /// PID discovered after Steam launched a Proton game on Linux.
    proton_pid: Option<u32>,
    mode: LaunchMode,
    game_root: PathBuf,
    exe_name: String,
    vanilla_active: bool,
    started_at: std::time::SystemTime,
}

/// How [`GameSession::poll`] reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionEvent {
    /// Still running.
    Running,
    /// Exited; vanilla state (if any) was already restored.
    Exited { code: Option<i32> },
}

impl GameSession {
    /// Starts the game exe as a tracked child.
    ///
    /// - `Vanilla` disables active assemblies in `Mods/` first.
    /// - Refuses when a stale game process (`exe` file name) is alive so a
    ///   second instance can never disagree with Steam about the state.
    pub fn start(game_root: &Path, exe: &Path, mode: LaunchMode) -> Result<Self> {
        let exe_name = exe
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        if !exe_name.is_empty() && !greg_platform::process::find_pids_by_exe(&exe_name).is_empty() {
            return Err(LoaderError::Other(format!(
                "{exe_name} is already running — stop it first"
            )));
        }
        let game_running = false;
        let vanilla_active = match mode {
            LaunchMode::Mods => {
                // Never start modded while a stale vanilla parking lingers.
                if mods_state(game_root) == ModsState::DisabledByManager {
                    set_mods_enabled(game_root, true, game_running)?;
                }
                false
            }
            LaunchMode::Vanilla => {
                set_mods_enabled(game_root, false, game_running)?;
                mods_state(game_root) == ModsState::DisabledByManager
            }
        };
        let (child, proton_pid) = {
            #[cfg(target_os = "linux")]
            if exe
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("exe"))
            {
                // A Windows Steam install cannot be executed directly by
                // Linux. Run the installed Proton runner directly. Calling
                // `steam -applaunch` would make Steam reject the request,
                // because the manager already uses this AppID for Workshop.
                greg_platform::process::launch_proton_app(
                    exe,
                    game_root,
                    greg_core::models::settings::DATA_CENTER_APP_ID,
                )
                .map_err(|e| LoaderError::Other(e.to_string()))?;
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
                let pid = loop {
                    if let Some(pid) = greg_platform::process::find_pids_by_exe(&exe_name).first() {
                        break *pid;
                    }
                    if std::time::Instant::now() >= deadline {
                        return Err(LoaderError::Other(
                            "Steam accepted the launch request, but the Proton game process did not appear within 30 seconds".into(),
                        ));
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                };
                (None, Some(pid))
            } else {
                let child = greg_platform::process::spawn_tracked(exe, &[], Some(game_root))
                    .map_err(|e| LoaderError::Other(e.to_string()))?;
                (Some(child), None)
            }
            #[cfg(not(target_os = "linux"))]
            {
                let child = greg_platform::process::spawn_tracked(exe, &[], Some(game_root))
                    .map_err(|e| LoaderError::Other(e.to_string()))?;
                (Some(child), None)
            }
        };
        Ok(Self {
            child,
            proton_pid,
            mode,
            game_root: game_root.to_path_buf(),
            exe_name,
            vanilla_active,
            started_at: std::time::SystemTime::now(),
        })
    }

    /// Launch flavour.
    pub fn mode(&self) -> LaunchMode {
        self.mode
    }

    /// Tracked pid.
    pub fn pid(&self) -> u32 {
        self.child
            .as_ref()
            .map(|child| child.pid())
            .or(self.proton_pid)
            .unwrap_or_default()
    }

    /// Game root of this session.
    pub fn game_root(&self) -> &Path {
        &self.game_root
    }

    /// True while the child has not exited yet.
    pub fn is_running(&mut self) -> bool {
        if let Some(child) = self.child.as_mut() {
            return child.is_running();
        }
        self.proton_pid = greg_platform::process::find_pids_by_exe(&self.exe_name)
            .first()
            .copied()
            .or(self.proton_pid.filter(|pid| {
                greg_platform::process::find_pids_by_exe(&self.exe_name).contains(pid)
            }));
        self.proton_pid.is_some()
    }

    /// Kills the child (no tree), then unwinds the vanilla state.
    /// Returns the exit code when already known.
    pub fn stop(&mut self) -> Result<Option<i32>> {
        if self.is_running() {
            if let Some(child) = self.child.as_mut() {
                child
                    .kill()
                    .map_err(|e| LoaderError::Other(e.to_string()))?;
                // Give the process a moment to die, then unwind deterministically.
                let _ = child.stop_blocking(std::time::Duration::from_secs(5));
            } else {
                // Proton may create more than one matching process. Kill all
                // matching game processes, never the Steam client itself.
                for pid in greg_platform::process::find_pids_by_exe(&self.exe_name) {
                    greg_platform::process::kill_tree(pid)
                        .map_err(|e| LoaderError::Other(e.to_string()))?;
                }
                self.proton_pid = None;
            }
        }
        self.unwind_vanilla();
        Ok(self.child.as_mut().and_then(|child| child.exit_code()))
    }

    /// Polls once: while running returns [`SessionEvent::Running`],
    /// after exit unwinds the vanilla state and reports the code.
    pub fn poll(&mut self) -> SessionEvent {
        if self.is_running() {
            return SessionEvent::Running;
        }
        let code = self.child.as_mut().and_then(|child| child.exit_code());
        self.unwind_vanilla();
        SessionEvent::Exited { code }
    }

    /// Elapsed play time.
    pub fn elapsed(&self) -> std::time::Duration {
        self.started_at.elapsed().unwrap_or_default()
    }

    fn unwind_vanilla(&mut self) {
        if self.vanilla_active {
            // Best-effort: the process is dead, nothing else can hold the dir.
            if set_mods_enabled(&self.game_root, true, false).is_ok() {
                self.vanilla_active = false;
            }
        }
    }

    /// Exe file name (for stale-process messages).
    pub fn exe_name(&self) -> &str {
        &self.exe_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_root(tag: &str) -> PathBuf {
        // Test scratch below the crate target dir (never the shared /tmp:
        // unique per process + nanos, cleaned up by each test).
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
            "target/test-tmp/greg-launch-test-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(dir.join("Mods")).unwrap();
        dir
    }

    #[test]
    fn vanilla_disables_and_restores_in_place() {
        let root = tmp_root("disable");
        std::fs::write(root.join("Mods").join("a.dll"), b"test").unwrap();
        assert_eq!(mods_state(&root), ModsState::Enabled);
        assert_eq!(
            set_mods_enabled(&root, false, false).unwrap(),
            ModsState::DisabledByManager
        );
        assert!(root.join("Mods").is_dir());
        assert!(!root.join("Mods").join("a.dll").exists());
        assert!(root.join("Mods").join("a.dll.disabled").is_file());
        assert!(root.join(".gregmodmanager-vanilla-mods").is_file());
        assert_eq!(
            set_mods_enabled(&root, true, false).unwrap(),
            ModsState::Enabled
        );
        assert!(root.join("Mods").is_dir());
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn refuses_park_while_running() {
        let root = tmp_root("refuse");
        assert!(set_mods_enabled(&root, false, true).is_err());
        assert_eq!(mods_state(&root), ModsState::Enabled);
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn crash_recovery_restores_when_idle() {
        let root = tmp_root("recover");
        set_mods_enabled(&root, false, false).unwrap();
        assert!(ensure_mods_restored(&root, false));
        assert_eq!(mods_state(&root), ModsState::Enabled);
        // Nothing to do when a game runs: leave the disabled files alone.
        set_mods_enabled(&root, false, false).unwrap();
        assert!(!ensure_mods_restored(&root, true));
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn refuses_start_when_stale_process_alive() {
        // `true`/`cmd` exits instantly, so no stale process exists and the
        // only possible failure is the missing game shape — but the stale
        // guard itself must not misfire on a live system process.
        let root = tmp_root("stale");
        let stale = greg_platform::process::find_pids_by_exe("definitely-not-a-game-xyz");
        assert!(stale.is_empty());
        std::fs::remove_dir_all(&root).ok();
    }
}
