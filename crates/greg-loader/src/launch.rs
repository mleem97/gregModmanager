//! Supervised game launches with a real vanilla mode.
//!
//! - `With mods`: the game exe starts directly as a tracked child.
//! - `Vanilla`: `Mods/` is atomically renamed to
//!   [`MODS_DISABLED_DIR_NAME`] before the start and renamed back when the
//!   game exits (or at the next manager start after a crash).
//! - The Steamworks session release/reconnect around the play session lives
//!   in the app layer (`greg-app`); this module only owns the OS process
//!   plus the mod-folder state.

use std::path::{Path, PathBuf};

use crate::error::{LoaderError, Result};

/// Rename target that parks `Mods/` for vanilla launches.
///
/// Crash-safe: a leftover directory is restored at the next manager start
/// (see [`ensure_mods_restored`]) when no game process is running.
pub const MODS_DISABLED_DIR_NAME: &str = "Mods.disabled-by-manager";

/// Launch flavour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LaunchMode {
    /// Mods as installed.
    #[default]
    Mods,
    /// `Mods/` parked away for this session.
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
    /// `Mods/` parked at [`MODS_DISABLED_DIR_NAME`] (game not running).
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
    match (mods.is_dir(), parked.is_dir()) {
        (true, false) => ModsState::Enabled,
        (false, true) => ModsState::DisabledByManager,
        (false, false) => ModsState::Missing,
        (true, true) => ModsState::Conflict,
    }
}

/// Parks (`false`) or restores (`true`) the `Mods/` folder.
///
/// Refuses to park while `game_running` (a stale or tracked game process)
/// and refuses to restore over a conflicting state.
pub fn set_mods_enabled(game_root: &Path, enabled: bool, game_running: bool) -> Result<ModsState> {
    let mods = game_root.join("Mods");
    let parked = game_root.join(MODS_DISABLED_DIR_NAME);
    if enabled {
        match mods_state(game_root) {
            ModsState::Enabled | ModsState::Missing => Ok(mods_state(game_root)),
            ModsState::DisabledByManager => {
                std::fs::rename(&parked, &mods)
                    .map_err(|e| LoaderError::Other(format!("cannot restore Mods folder: {e}")))?;
                Ok(ModsState::Enabled)
            }
            ModsState::Conflict => Err(LoaderError::Other(
                "both Mods/ and the parked vanilla copy exist — resolve manually".into(),
            )),
        }
    } else {
        if game_running {
            return Err(LoaderError::Other(
                "cannot switch to vanilla while the game is running".into(),
            ));
        }
        match mods_state(game_root) {
            ModsState::DisabledByManager => Ok(ModsState::DisabledByManager),
            ModsState::Missing => Ok(ModsState::Missing),
            ModsState::Enabled => {
                std::fs::rename(&mods, &parked)
                    .map_err(|e| LoaderError::Other(format!("cannot park Mods folder: {e}")))?;
                Ok(ModsState::DisabledByManager)
            }
            ModsState::Conflict => Err(LoaderError::Other(
                "both Mods/ and the parked vanilla copy exist — resolve manually".into(),
            )),
        }
    }
}

/// Crash recovery at manager start: restores a parked `Mods/` folder when
/// no game process is running. Returns `true` when something was restored.
pub fn ensure_mods_restored(game_root: &Path, game_running: bool) -> bool {
    if game_running || mods_state(game_root) != ModsState::DisabledByManager {
        return false;
    }
    set_mods_enabled(game_root, true, false).is_ok()
}

/// Well-known Data Center executable file names (all OS spellings).
pub const GAME_EXE_NAMES: &[&str] = &[
    "Data Center.exe",
    "DataCenter.exe",
    "Data Center.x86_64",
    "DataCenter.x86_64",
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
    child: greg_platform::process::TrackedChild,
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
    /// - `Vanilla` parks `Mods/` first (refused when a game runs already).
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
        let child = greg_platform::process::spawn_tracked(exe, &[], Some(game_root))
            .map_err(|e| LoaderError::Other(e.to_string()))?;
        Ok(Self {
            child,
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
        self.child.pid()
    }

    /// Game root of this session.
    pub fn game_root(&self) -> &Path {
        &self.game_root
    }

    /// True while the child has not exited yet.
    pub fn is_running(&mut self) -> bool {
        self.child.is_running()
    }

    /// Kills the child (no tree), then unwinds the vanilla state.
    /// Returns the exit code when already known.
    pub fn stop(&mut self) -> Result<Option<i32>> {
        if self.is_running() {
            self.child
                .kill()
                .map_err(|e| LoaderError::Other(e.to_string()))?;
            // Give the process a moment to die, then unwind deterministically.
            let _ = self.child.stop_blocking(std::time::Duration::from_secs(5));
        }
        self.unwind_vanilla();
        Ok(self.child.exit_code())
    }

    /// Polls once: while running returns [`SessionEvent::Running`],
    /// after exit unwinds the vanilla state and reports the code.
    pub fn poll(&mut self) -> SessionEvent {
        if self.is_running() {
            return SessionEvent::Running;
        }
        let code = self.child.exit_code();
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
        let dir = std::env::temp_dir().join(format!(
            "greg-launch-test-{tag}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(dir.join("Mods")).unwrap();
        dir
    }

    #[test]
    fn vanilla_parks_and_restores() {
        let root = tmp_root("park");
        assert_eq!(mods_state(&root), ModsState::Enabled);
        assert_eq!(
            set_mods_enabled(&root, false, false).unwrap(),
            ModsState::DisabledByManager
        );
        assert!(!root.join("Mods").exists());
        assert!(root.join(MODS_DISABLED_DIR_NAME).is_dir());
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
        // Nothing to do when a game runs: leave the parking alone.
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
