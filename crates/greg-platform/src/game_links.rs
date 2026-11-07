//! Reviewed links from an installation-scoped manager tree into a game install.
//!
//! Only explicitly mapped game-relative directories are eligible. The link
//! itself lives below the selected manager base; its target remains the
//! corresponding directory in the selected game installation.

use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::{io_err, PlatformError, Result};

/// Directories that can be managed independently for a game installation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ManagedDirectory {
    Mods,
    MelonPlugins,
    UserLibs,
    UserData,
    CustomObjects,
}

impl ManagedDirectory {
    /// Manager-side relative path. The existing `Melons` name is retained for
    /// compatibility with the managed plugin storage already used by the app.
    pub fn manager_relative(self) -> &'static str {
        match self {
            Self::Mods => "Mods",
            Self::MelonPlugins => "Melons",
            Self::UserLibs => "UserLibs",
            Self::UserData => "UserData",
            Self::CustomObjects => "Custom/Objects",
        }
    }
}

/// A game-adapter-provided mapping. `None` means that the game has no
/// verified legacy destination for this category; no guessed link is made.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedLinkSpec {
    pub directory: ManagedDirectory,
    pub game_relative: Option<PathBuf>,
}

impl ManagedLinkSpec {
    pub fn mapped(directory: ManagedDirectory, game_relative: impl Into<PathBuf>) -> Self {
        Self {
            directory,
            game_relative: Some(game_relative.into()),
        }
    }

    pub fn unavailable(directory: ManagedDirectory) -> Self {
        Self {
            directory,
            game_relative: None,
        }
    }
}

/// Per-installation folder below `GameModding`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameModdingSlot {
    pub game_id: String,
    pub installation_id: String,
    pub path: PathBuf,
}

/// Reviewable state for a planned symlink.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LinkPlanStatus {
    Create,
    ReplaceEmptyPlaceholder,
    AlreadyCorrect,
    Unavailable,
}

/// One manager-to-game link to show in the review modal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannedGameLink {
    pub directory: ManagedDirectory,
    pub manager_path: PathBuf,
    pub game_path: Option<PathBuf>,
    pub status: LinkPlanStatus,
}

/// All validated links for one game installation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameLinkPlan {
    pub game_id: String,
    pub installation_id: String,
    pub manager_base: PathBuf,
    pub game_root: PathBuf,
    pub links: Vec<PlannedGameLink>,
}

/// Applied result, suitable for a post-apply refresh or modal summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppliedGameLink {
    pub directory: ManagedDirectory,
    pub manager_path: PathBuf,
    pub game_path: Option<PathBuf>,
    pub status: LinkPlanStatus,
}

/// Resolves an opaque stable installation slot without storing the user's
/// absolute game path in the manager's directory names.
pub fn game_modding_slot(
    manager_base: &Path,
    game_id: &str,
    game_root: &Path,
) -> Result<GameModdingSlot> {
    validate_game_id(game_id)?;
    let canonical_manager = manager_base.canonicalize().map_err(io_err)?;
    let canonical_game = game_root.canonicalize().map_err(io_err)?;
    if !canonical_game.is_dir() {
        return Err(PlatformError::Project(
            "game root is not a directory".into(),
        ));
    }
    if canonical_manager.starts_with(&canonical_game)
        || canonical_game.starts_with(&canonical_manager)
    {
        return Err(PlatformError::Project(
            "manager base and game installation must not overlap".into(),
        ));
    }
    let installation_id = installation_id(&canonical_game);
    Ok(GameModdingSlot {
        game_id: game_id.to_string(),
        installation_id: installation_id.clone(),
        path: canonical_manager
            .join("GameModding")
            .join(game_id)
            .join(&installation_id),
    })
}

/// Returns a stable, non-reversible directory key for a canonical game root.
pub fn installation_id(canonical_game_root: &Path) -> String {
    let digest = Sha256::digest(canonical_game_root.as_os_str().as_encoded_bytes());
    hex::encode(&digest[..12])
}

/// Builds a side-effect-free plan. This can be presented to the user before
/// [`apply_game_links`] changes any filesystem entry.
pub fn plan_game_links(
    manager_base: &Path,
    game_id: &str,
    game_root: &Path,
    specs: &[ManagedLinkSpec],
) -> Result<GameLinkPlan> {
    let manager_root = manager_base.canonicalize().map_err(io_err)?;
    let slot = game_modding_slot(&manager_root, game_id, game_root)?;
    let game_root = game_root.canonicalize().map_err(io_err)?;
    let slot_relative = slot
        .path
        .strip_prefix(&manager_root)
        .map_err(|_| PlatformError::Project("managed game slot is outside manager base".into()))?;
    validate_relative(slot_relative)?;
    validate_manager_components(&manager_root, slot_relative, false)?;

    let mut seen = std::collections::HashSet::new();
    let mut links = Vec::with_capacity(specs.len());
    for spec in specs {
        let relative = PathBuf::from(spec.directory.manager_relative());
        if !seen.insert(relative.clone()) {
            return Err(PlatformError::Project(format!(
                "duplicate managed directory mapping: {}",
                relative.display()
            )));
        }
        validate_relative(&relative)?;
        let manager_path = slot.path.join(&relative);
        let manager_relative = slot_relative.join(&relative);
        validate_manager_components(&manager_root, &manager_relative, true)?;

        let Some(game_relative) = &spec.game_relative else {
            links.push(PlannedGameLink {
                directory: spec.directory,
                manager_path,
                game_path: None,
                status: LinkPlanStatus::Unavailable,
            });
            continue;
        };
        validate_relative(game_relative)?;
        let game_path = game_root.join(game_relative);
        validate_game_components(&game_root, game_relative, false)?;
        let status = inspect_link(&manager_path, &game_path)?;
        links.push(PlannedGameLink {
            directory: spec.directory,
            manager_path,
            game_path: Some(game_path),
            status,
        });
    }
    Ok(GameLinkPlan {
        game_id: slot.game_id,
        installation_id: slot.installation_id,
        manager_base: manager_root,
        game_root,
        links,
    })
}

/// Applies a previously reviewed plan. Only empty manager-side placeholder
/// directories can be removed. Existing game content is never moved/deleted.
pub fn apply_game_links(plan: &GameLinkPlan) -> Result<Vec<AppliedGameLink>> {
    let manager_root = plan.manager_base.canonicalize().map_err(io_err)?;
    let game_root = plan.game_root.canonicalize().map_err(io_err)?;
    if manager_root != plan.manager_base || game_root != plan.game_root {
        return Err(PlatformError::Project(
            "manager or game root changed after link review".into(),
        ));
    }
    if manager_root.starts_with(&game_root) || game_root.starts_with(&manager_root) {
        return Err(PlatformError::Project(
            "manager base and game installation must not overlap".into(),
        ));
    }
    let reviewed_slot = game_modding_slot(&manager_root, &plan.game_id, &game_root)?;
    if reviewed_slot.installation_id != plan.installation_id {
        return Err(PlatformError::Project(
            "selected game installation changed after link review".into(),
        ));
    }
    // Rebuild the entire read-only plan before changing the first link. This
    // catches edited plans and filesystem changes since the review modal.
    let mut current_specs = Vec::with_capacity(plan.links.len());
    for link in &plan.links {
        let expected_manager_path = reviewed_slot.path.join(link.directory.manager_relative());
        if link.manager_path != expected_manager_path {
            return Err(PlatformError::Project(format!(
                "manager link path changed after review: {}",
                link.manager_path.display()
            )));
        }
        match (&link.status, &link.game_path) {
            (LinkPlanStatus::Unavailable, None) => {
                current_specs.push(ManagedLinkSpec::unavailable(link.directory));
            }
            (LinkPlanStatus::Unavailable, Some(_)) => {
                return Err(PlatformError::Project(
                    "unavailable mapping has a game target".into(),
                ));
            }
            (_, Some(target)) => {
                let relative = target.strip_prefix(&game_root).map_err(|_| {
                    PlatformError::Project(
                        "game link target is outside the reviewed installation".into(),
                    )
                })?;
                current_specs.push(ManagedLinkSpec::mapped(link.directory, relative));
            }
            (_, None) => {
                return Err(PlatformError::Project(
                    "reviewed mapped link has no game target".into(),
                ));
            }
        }
    }
    let fresh_plan = plan_game_links(&manager_root, &plan.game_id, &game_root, &current_specs)?;
    if fresh_plan != *plan {
        return Err(PlatformError::Project(
            "game-link review is stale; rebuild and review the plan again".into(),
        ));
    }
    let mut results = Vec::with_capacity(plan.links.len());
    for link in &plan.links {
        let expected_manager_path = reviewed_slot.path.join(link.directory.manager_relative());
        if link.manager_path != expected_manager_path {
            return Err(PlatformError::Project(format!(
                "manager link path changed after review: {}",
                link.manager_path.display()
            )));
        }
        validate_manager_path_in_root(&manager_root, &expected_manager_path, true)?;
        if link.status == LinkPlanStatus::Unavailable
            || link.status == LinkPlanStatus::AlreadyCorrect
        {
            if link.status == LinkPlanStatus::Unavailable {
                if link.game_path.is_some() {
                    return Err(PlatformError::Project(
                        "unavailable mapping has a game target".into(),
                    ));
                }
            } else {
                let target = link.game_path.as_ref().ok_or_else(|| {
                    PlatformError::Project("reviewed link has no game target".into())
                })?;
                let relative = target.strip_prefix(&game_root).map_err(|_| {
                    PlatformError::Project(
                        "game link target is outside the reviewed installation".into(),
                    )
                })?;
                validate_relative(relative)?;
                validate_game_components(&game_root, relative, false)?;
                if inspect_link(&expected_manager_path, target)? != LinkPlanStatus::AlreadyCorrect {
                    return Err(PlatformError::Project(format!(
                        "managed link changed after review: {}",
                        expected_manager_path.display()
                    )));
                }
            }
            results.push(AppliedGameLink {
                directory: link.directory,
                manager_path: link.manager_path.clone(),
                game_path: link.game_path.clone(),
                status: link.status.clone(),
            });
            continue;
        }
        let game_path = link
            .game_path
            .as_ref()
            .ok_or_else(|| PlatformError::Project("mapped link has no game target".into()))?;
        let game_relative = game_path.strip_prefix(&game_root).map_err(|_| {
            PlatformError::Project("game link target is outside the reviewed installation".into())
        })?;
        validate_relative(game_relative)?;
        validate_game_components(&game_root, game_relative, false)?;
        let current_status = inspect_link(&link.manager_path, game_path)?;
        if current_status == LinkPlanStatus::AlreadyCorrect {
            results.push(AppliedGameLink {
                directory: link.directory,
                manager_path: link.manager_path.clone(),
                game_path: Some(game_path.clone()),
                status: LinkPlanStatus::AlreadyCorrect,
            });
            continue;
        }
        if current_status != link.status {
            return Err(PlatformError::Project(format!(
                "managed link changed after review: {}",
                link.manager_path.display()
            )));
        }
        let manager_parent = link
            .manager_path
            .parent()
            .ok_or_else(|| PlatformError::Project("managed link has no parent directory".into()))?;
        create_plain_directories(manager_parent)?;
        create_game_target(&game_root, game_relative)?;
        if link.status == LinkPlanStatus::ReplaceEmptyPlaceholder {
            std::fs::remove_dir(&link.manager_path).map_err(io_err)?;
        }
        create_directory_symlink(game_path, &link.manager_path)?;
        results.push(AppliedGameLink {
            directory: link.directory,
            manager_path: link.manager_path.clone(),
            game_path: Some(game_path.clone()),
            status: link.status.clone(),
        });
    }
    Ok(results)
}

/// Resolve an active managed directory only when its symlink still matches the
/// currently selected game installation and adapter mapping exactly.
pub fn resolve_active_game_link(
    manager_base: &Path,
    game_id: &str,
    game_root: &Path,
    spec: &ManagedLinkSpec,
) -> Result<PathBuf> {
    let game_relative = spec.game_relative.as_ref().ok_or_else(|| {
        PlatformError::Project("managed directory has no verified game target".into())
    })?;
    validate_relative(game_relative)?;
    let slot = game_modding_slot(manager_base, game_id, game_root)?;
    let manager_root = manager_base.canonicalize().map_err(io_err)?;
    let canonical_game = game_root.canonicalize().map_err(io_err)?;
    let manager_relative = slot
        .path
        .strip_prefix(&manager_root)
        .map_err(|_| PlatformError::Project("managed slot escaped manager base".into()))?
        .join(spec.directory.manager_relative());
    validate_relative(&manager_relative)?;
    validate_manager_components(&manager_root, &manager_relative, true)?;
    let manager_link = manager_root.join(&manager_relative);
    let target = canonical_game.join(game_relative);
    validate_game_components(&canonical_game, game_relative, false)?;
    if !target.is_dir() {
        return Err(PlatformError::Project(
            "managed game target is missing".into(),
        ));
    }
    match inspect_link(&manager_link, &target)? {
        LinkPlanStatus::AlreadyCorrect => Ok(target),
        _ => Err(PlatformError::Project(format!(
            "managed directory is not linked to the active game installation: {}",
            manager_link.display()
        ))),
    }
}

fn validate_game_id(game_id: &str) -> Result<()> {
    if game_id.is_empty()
        || !game_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(PlatformError::Project(
            "game id must contain only ASCII letters, digits, '_' or '-'".into(),
        ));
    }
    Ok(())
}

fn validate_relative(path: &Path) -> Result<()> {
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(PlatformError::Project(format!(
            "managed link path must be relative and contain no traversal: {}",
            path.display()
        )));
    }
    Ok(())
}

fn validate_manager_components(root: &Path, relative: &Path, allow_missing: bool) -> Result<()> {
    let mut current = root.to_path_buf();
    let count = relative.components().count();
    for (index, component) in relative.components().enumerate() {
        let Component::Normal(name) = component else {
            return Err(PlatformError::Project(
                "invalid manager-relative path".into(),
            ));
        };
        current.push(name);
        match std::fs::symlink_metadata(&current) {
            Ok(meta) if meta.file_type().is_symlink() => {
                // The final path may be the reviewed game link on a re-plan.
                if allow_missing && index + 1 == count {
                    continue;
                }
                return Err(PlatformError::Project(format!(
                    "manager path contains an unexpected symlink: {}",
                    current.display()
                )));
            }
            Ok(meta) if index + 1 < count && !meta.is_dir() => {
                return Err(PlatformError::Project(format!(
                    "manager path component is not a directory: {}",
                    current.display()
                )));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(io_err(error)),
        }
    }
    Ok(())
}

fn validate_manager_path_in_root(root: &Path, path: &Path, allow_final_link: bool) -> Result<()> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| PlatformError::Project("manager link path escaped selected base".into()))?;
    validate_relative(relative)?;
    validate_manager_components(root, relative, allow_final_link)
}

fn validate_game_components(root: &Path, relative: &Path, create: bool) -> Result<()> {
    let mut current = root.to_path_buf();
    for (index, component) in relative.components().enumerate() {
        let Component::Normal(name) = component else {
            return Err(PlatformError::Project("invalid game-relative path".into()));
        };
        current.push(name);
        match std::fs::symlink_metadata(&current) {
            Ok(meta) if meta.file_type().is_symlink() => {
                return Err(PlatformError::Project(format!(
                    "game target uses an unexpected symlink component: {}",
                    current.display()
                )));
            }
            Ok(meta) if index + 1 < relative.components().count() && !meta.is_dir() => {
                return Err(PlatformError::Project(format!(
                    "game target parent is not a directory: {}",
                    current.display()
                )));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound && create => {
                std::fs::create_dir(&current).map_err(io_err)?;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(io_err(error)),
        }
    }
    Ok(())
}

fn inspect_link(manager_path: &Path, game_path: &Path) -> Result<LinkPlanStatus> {
    match std::fs::symlink_metadata(manager_path) {
        Ok(meta) if meta.file_type().is_symlink() => {
            let raw = std::fs::read_link(manager_path).map_err(io_err)?;
            let resolved = if raw.is_absolute() {
                raw
            } else {
                manager_path
                    .parent()
                    .ok_or_else(|| PlatformError::Project("link has no parent".into()))?
                    .join(raw)
            };
            if resolved == game_path {
                Ok(LinkPlanStatus::AlreadyCorrect)
            } else {
                Err(PlatformError::Project(format!(
                    "manager path is already a symlink to a different target: {}",
                    manager_path.display()
                )))
            }
        }
        Ok(meta) if meta.is_dir() => {
            let mut entries = std::fs::read_dir(manager_path).map_err(io_err)?;
            if entries.next().is_none() {
                Ok(LinkPlanStatus::ReplaceEmptyPlaceholder)
            } else {
                Err(PlatformError::Project(format!(
                    "refusing to replace non-empty manager directory: {}",
                    manager_path.display()
                )))
            }
        }
        Ok(_) => Err(PlatformError::Project(format!(
            "refusing to replace a non-directory manager path: {}",
            manager_path.display()
        ))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(LinkPlanStatus::Create),
        Err(error) => Err(io_err(error)),
    }
}

fn create_plain_directories(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        create_plain_directories(parent)?;
    }
    match std::fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_symlink() => Err(PlatformError::Project(format!(
            "refusing symlinked manager parent: {}",
            path.display()
        ))),
        Ok(meta) if meta.is_dir() => Ok(()),
        Ok(_) => Err(PlatformError::Project(format!(
            "manager parent is not a directory: {}",
            path.display()
        ))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            std::fs::create_dir(path).map_err(io_err)
        }
        Err(error) => Err(io_err(error)),
    }
}

fn create_game_target(root: &Path, relative: &Path) -> Result<()> {
    let mut current = root.to_path_buf();
    for component in relative.components() {
        let Component::Normal(name) = component else {
            return Err(PlatformError::Project("invalid game target path".into()));
        };
        current.push(name);
        match std::fs::symlink_metadata(&current) {
            Ok(meta) if meta.file_type().is_symlink() => {
                return Err(PlatformError::Project(format!(
                    "refusing symlinked game target: {}",
                    current.display()
                )));
            }
            Ok(meta) if meta.is_dir() => {}
            Ok(_) => {
                return Err(PlatformError::Project(format!(
                    "game target is not a directory: {}",
                    current.display()
                )));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                std::fs::create_dir(&current).map_err(io_err)?;
            }
            Err(error) => return Err(io_err(error)),
        }
    }
    Ok(())
}

#[cfg(unix)]
fn create_directory_symlink(target: &Path, link: &Path) -> Result<()> {
    std::os::unix::fs::symlink(target, link).map_err(io_err)
}

#[cfg(windows)]
fn create_directory_symlink(target: &Path, link: &Path) -> Result<()> {
    std::os::windows::fs::symlink_dir(target, link).map_err(|error| {
        PlatformError::Other(format!(
            "could not create directory symlink; enable Windows Developer Mode or grant the current user symbolic-link privilege: {error}"
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(label: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "greg-game-links-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    fn specs() -> Vec<ManagedLinkSpec> {
        vec![
            ManagedLinkSpec::mapped(ManagedDirectory::Mods, "Mods"),
            ManagedLinkSpec::mapped(ManagedDirectory::MelonPlugins, "Plugins"),
            ManagedLinkSpec::mapped(ManagedDirectory::UserLibs, "UserLibs"),
            ManagedLinkSpec::mapped(ManagedDirectory::UserData, "UserData"),
            ManagedLinkSpec::unavailable(ManagedDirectory::CustomObjects),
        ]
    }

    #[test]
    fn slots_are_installation_scoped_and_do_not_expose_game_paths() {
        let root = temp_root("slots");
        let base = root.join("manager");
        let game = root.join("secret-user-name/Game");
        std::fs::create_dir_all(&base).unwrap();
        std::fs::create_dir_all(&game).unwrap();
        let a = game_modding_slot(&base, "datacenter", &game).unwrap();
        let other = root.join("secret-user-name/OtherGame");
        std::fs::create_dir_all(&other).unwrap();
        let b = game_modding_slot(&base, "datacenter", &other).unwrap();
        assert_ne!(a.installation_id, b.installation_id);
        assert!(!a.path.to_string_lossy().contains("secret-user-name"));
        assert!(a
            .path
            .ends_with(format!("datacenter/{}", a.installation_id)));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn reviewed_links_replace_only_empty_placeholders_and_are_idempotent() {
        let root = temp_root("empty");
        let base = root.join("manager");
        let game = root.join("game");
        std::fs::create_dir_all(base.join("GameModding/datacenter/old/Mods")).unwrap();
        std::fs::create_dir_all(&game).unwrap();
        // Keep only the matching installation slot by deriving it after the game exists.
        let slot = game_modding_slot(&base, "datacenter", &game).unwrap();
        std::fs::create_dir_all(slot.path.join("Mods")).unwrap();
        let plan = plan_game_links(&base, "datacenter", &game, &specs()).unwrap();
        assert_eq!(
            plan.links[0].status,
            LinkPlanStatus::ReplaceEmptyPlaceholder
        );
        assert_eq!(plan.links[4].status, LinkPlanStatus::Unavailable);
        let applied = apply_game_links(&plan).unwrap();
        assert_eq!(applied.len(), 5);
        assert!(std::fs::symlink_metadata(slot.path.join("Mods"))
            .unwrap()
            .file_type()
            .is_symlink());
        let resolved = resolve_active_game_link(&base, "datacenter", &game, &specs()[0]).unwrap();
        assert_eq!(resolved, game.canonicalize().unwrap().join("Mods"));
        assert!(resolve_active_game_link(&base, "another-game", &game, &specs()[0]).is_err());
        let again = plan_game_links(&base, "datacenter", &game, &specs()).unwrap();
        assert_eq!(again.links[0].status, LinkPlanStatus::AlreadyCorrect);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn existing_content_and_symlink_chains_are_rejected_without_changes() {
        use std::os::unix::fs::symlink;
        let root = temp_root("reject");
        let base = root.join("manager");
        let game = root.join("game");
        std::fs::create_dir_all(&base).unwrap();
        std::fs::create_dir_all(&game).unwrap();
        let slot = game_modding_slot(&base, "datacenter", &game).unwrap();
        std::fs::create_dir_all(slot.path.join("Mods")).unwrap();
        std::fs::write(slot.path.join("Mods/personal.dll"), b"keep").unwrap();
        let error = plan_game_links(&base, "datacenter", &game, &specs()).unwrap_err();
        assert!(error.to_string().contains("non-empty manager directory"));
        assert_eq!(
            std::fs::read(slot.path.join("Mods/personal.dll")).unwrap(),
            b"keep"
        );

        let outside = root.join("outside");
        std::fs::create_dir_all(&outside).unwrap();
        symlink(&outside, game.join("Plugins")).unwrap();
        let plugin_only = [ManagedLinkSpec::mapped(
            ManagedDirectory::MelonPlugins,
            "Plugins",
        )];
        assert!(plan_game_links(&base, "datacenter", &game, &plugin_only).is_err());
        assert!(outside.is_dir());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn existing_game_content_is_kept_when_the_manager_link_is_created() {
        let root = temp_root("target-content");
        let base = root.join("manager");
        let game = root.join("game");
        std::fs::create_dir_all(&base).unwrap();
        std::fs::create_dir_all(game.join("Mods")).unwrap();
        std::fs::write(game.join("Mods/installed-by-game.dll"), b"foreign").unwrap();

        let plan = plan_game_links(
            &base,
            "datacenter",
            &game,
            &[ManagedLinkSpec::mapped(ManagedDirectory::Mods, "Mods")],
        )
        .unwrap();
        assert_eq!(plan.links[0].status, LinkPlanStatus::Create);
        apply_game_links(&plan).unwrap();
        assert_eq!(
            std::fs::read(game.join("Mods/installed-by-game.dll")).unwrap(),
            b"foreign"
        );
        let slot = game_modding_slot(&base, "datacenter", &game).unwrap();
        assert!(std::fs::symlink_metadata(slot.path.join("Mods"))
            .unwrap()
            .file_type()
            .is_symlink());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_invalid_game_ids_and_traversal() {
        let root = temp_root("invalid");
        let base = root.join("manager");
        let game = root.join("game");
        std::fs::create_dir_all(&base).unwrap();
        std::fs::create_dir_all(&game).unwrap();
        assert!(game_modding_slot(&base, "../escape", &game).is_err());
        let invalid = vec![ManagedLinkSpec::mapped(
            ManagedDirectory::Mods,
            "../outside",
        )];
        assert!(plan_game_links(&base, "datacenter", &game, &invalid).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn apply_rejects_a_link_path_changed_after_review() {
        let root = temp_root("tamper");
        let base = root.join("manager");
        let game = root.join("game");
        std::fs::create_dir_all(&base).unwrap();
        std::fs::create_dir_all(&game).unwrap();
        let mut plan = plan_game_links(&base, "datacenter", &game, &specs()).unwrap();
        plan.links[0].manager_path = root.join("outside");
        assert!(apply_game_links(&plan).is_err());
        assert!(!root.join("outside").exists());
        std::fs::remove_dir_all(root).unwrap();
    }
}
