//! Background build verification for local authoring projects.
//!
//! Executes only explicit SDK tools, never a shell command from metadata. A
//! build runs project code and is therefore an explicit user action, not an
//! automatic side effect of viewing/downloading a Workshop item.

use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use greg_core::models::WorkshopMetadata;
use serde::{Deserialize, Serialize};

use crate::error::{io_err, LoaderError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildPlan {
    pub tool: String,
    pub project: PathBuf,
    pub arguments: Vec<String>,
    pub dependency_dirs: Vec<PathBuf>,
    pub log_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildResult {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub cancelled: bool,
    pub timed_out: bool,
    pub elapsed_ms: u128,
    pub log_path: PathBuf,
    pub output_tail: String,
}

/// Resolve one .NET project/solution or Cargo manifest inside the upload project.
pub fn plan(
    project_root: &Path,
    meta: &WorkshopMetadata,
    manager_base: &Path,
    game_root: &Path,
) -> Result<BuildPlan> {
    let root = project_root.canonicalize().map_err(io_err)?;
    let mut candidates = Vec::new();
    // `build.project` is optional; old metadata remains compatible.
    if let Some(build) = &meta.build {
        let relative = Path::new(&build.project);
        if relative.is_absolute()
            || relative
                .components()
                .any(|c| !matches!(c, std::path::Component::Normal(_)))
        {
            return Err(LoaderError::InvalidArtefact(
                "build.project must be a relative project path".into(),
            ));
        }
        candidates.push(root.join(relative));
    } else {
        for entry in walkdir::WalkDir::new(&root)
            .max_depth(5)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| {
                !matches!(
                    e.file_name().to_str(),
                    Some("bin" | "obj" | "target" | ".git")
                )
            })
        {
            let entry = entry.map_err(|e| LoaderError::Other(e.to_string()))?;
            if entry.file_type().is_file() && supported(entry.path()) {
                candidates.push(entry.path().to_path_buf());
            }
        }
        candidates.sort();
        // A single solution is the deliberate aggregate when present.
        let solutions: Vec<_> = candidates
            .iter()
            .filter(|p| p.extension().is_some_and(|e| e == "sln" || e == "slnx"))
            .cloned()
            .collect();
        if solutions.len() == 1 {
            candidates = solutions;
        }
    }
    if candidates.len() != 1 {
        return Err(LoaderError::InvalidArtefact("Choose exactly one build project in metadata.json: build.project (.csproj, .sln, .slnx or Cargo.toml). Native assets without source have no build test.".into()));
    }
    let project = candidates.pop().unwrap().canonicalize().map_err(io_err)?;
    if !project.starts_with(&root) || !supported(&project) || !project.is_file() {
        return Err(LoaderError::InvalidArtefact(
            "build project must remain inside the upload project".into(),
        ));
    }
    let configuration = meta
        .build
        .as_ref()
        .map(|b| b.configuration.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("Release");
    if !configuration
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(LoaderError::InvalidArtefact(
            "invalid build configuration".into(),
        ));
    }
    let mut dependency_dirs = Vec::new();
    for dependency_root in [
        manager_base.join("GameModding/UserLibs"),
        manager_base.join("GameModding/Melons"),
        game_root.join("MelonLoader"),
        game_root.join("Data Center_Data/Managed"),
    ] {
        if !dependency_root.is_dir() {
            continue;
        }
        for entry in walkdir::WalkDir::new(&dependency_root)
            .follow_links(false)
            .max_depth(5)
        {
            let entry = entry.map_err(|e| LoaderError::Other(e.to_string()))?;
            if entry.file_type().is_file()
                && entry
                    .path()
                    .extension()
                    .is_some_and(|e| e.eq_ignore_ascii_case("dll"))
            {
                if let Some(parent) = entry.path().parent() {
                    dependency_dirs.push(parent.to_path_buf());
                }
            }
        }
    }
    dependency_dirs.sort();
    dependency_dirs.dedup();
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let output = manager_base.join("Cache/Builds").join(stamp.to_string());
    let log_path = manager_base.join("Logs").join(format!("build-{stamp}.log"));
    let (tool, arguments) = if project.file_name().is_some_and(|n| n == "Cargo.toml") {
        (
            "cargo",
            vec![
                "build".into(),
                "--manifest-path".into(),
                project.to_string_lossy().into_owned(),
                "--release".into(),
                "--target-dir".into(),
                output.to_string_lossy().into_owned(),
            ],
        )
    } else {
        let reference_path = dependency_dirs
            .iter()
            .map(|p| msbuild_escape(&p.to_string_lossy()))
            .collect::<Vec<_>>()
            .join(";");
        (
            "dotnet",
            vec![
                "build".into(),
                project.to_string_lossy().into_owned(),
                "--configuration".into(),
                configuration.into(),
                "--nologo".into(),
                "--disable-build-servers".into(),
                "--artifacts-path".into(),
                output.to_string_lossy().into_owned(),
                format!("-p:AssemblySearchPaths=\"{{CandidateAssemblyFiles}};{{HintPathFromItem}};{{TargetFrameworkDirectory}};{{RawFileName}};{reference_path}\""),
                format!(
                    "-p:GameDir={}",
                    msbuild_escape(&game_root.to_string_lossy())
                ),
                format!(
                    "-p:UserLibsDir={}",
                    msbuild_escape(&manager_base.join("GameModding/UserLibs").to_string_lossy())
                ),
            ],
        )
    };
    Ok(BuildPlan {
        tool: tool.into(),
        project,
        arguments,
        dependency_dirs,
        log_path,
    })
}

fn supported(path: &Path) -> bool {
    path.file_name().is_some_and(|p| p == "Cargo.toml")
        || path
            .extension()
            .is_some_and(|p| p == "csproj" || p == "sln" || p == "slnx")
}

fn msbuild_escape(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '%' | ';' | ',' | '$' | '@' | '\'' | '(' | ')' | '"' | '*' | '?' => {
                escaped.push_str(&format!("%{:02X}", ch as u32))
            }
            _ => escaped.push(ch),
        }
    }
    escaped
}

/// Run without blocking the UI; output goes directly to a log rather than an
/// unbounded pipe buffer. Requires .NET SDK 8+ or a stable Rust SDK.
pub fn run(plan: &BuildPlan, cancel: &AtomicBool, timeout: Duration) -> Result<BuildResult> {
    if !matches!(plan.tool.as_str(), "dotnet" | "cargo") {
        return Err(LoaderError::InvalidArtefact(
            "unsupported build tool".into(),
        ));
    }
    if let Some(parent) = plan.log_path.parent() {
        std::fs::create_dir_all(parent).map_err(io_err)?;
    }
    let log = std::fs::File::create(&plan.log_path).map_err(io_err)?;
    let mut command = Command::new(&plan.tool);
    command
        .args(&plan.arguments)
        .current_dir(plan.project.parent().unwrap())
        .env("DOTNET_CLI_TELEMETRY_OPTOUT", "1")
        .env("DOTNET_NOLOGO", "1")
        .env(
            "DOTNET_CLI_HOME",
            plan.log_path
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("Cache/Dotnet"),
        )
        .stdin(Stdio::null())
        .stdout(Stdio::from(log.try_clone().map_err(io_err)?))
        .stderr(Stdio::from(log));
    greg_platform::process::isolate_command(&mut command);
    let started = Instant::now();
    let mut child = command
        .spawn()
        .map_err(|e| LoaderError::Other(format!("cannot start {}: {e}", plan.tool)))?;
    let (status, cancelled, timed_out) = loop {
        if let Some(status) = child.try_wait().map_err(io_err)? {
            break (Some(status), false, false);
        }
        let cancelled = cancel.load(Ordering::Relaxed);
        let timed_out = started.elapsed() >= timeout;
        if cancelled || timed_out {
            let _ = greg_platform::process::stop_isolated_command(child.id());
            let _ = child.kill();
            let _ = child.wait();
            break (None, cancelled, timed_out);
        }
        std::thread::sleep(Duration::from_millis(100));
    };
    let mut file = std::fs::File::open(&plan.log_path).map_err(io_err)?;
    let length = file.metadata().map_err(io_err)?.len();
    file.seek(SeekFrom::Start(length.saturating_sub(256 * 1024)))
        .map_err(io_err)?;
    let mut bytes = Vec::new();
    file.take(256 * 1024)
        .read_to_end(&mut bytes)
        .map_err(io_err)?;
    Ok(BuildResult {
        success: status.as_ref().is_some_and(|s| s.success()),
        exit_code: status.and_then(|s| s.code()),
        cancelled,
        timed_out,
        elapsed_ms: started.elapsed().as_millis(),
        log_path: plan.log_path.clone(),
        output_tail: String::from_utf8_lossy(&bytes).into_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "greg-build-plan-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(dir.join("project/content")).unwrap();
        std::fs::create_dir_all(dir.join("GameModding/UserLibs/42")).unwrap();
        std::fs::write(
            dir.join("GameModding/UserLibs/42/Dependency.dll"),
            "fixture",
        )
        .unwrap();
        std::fs::write(dir.join("project/content/Test.csproj"), "<Project/>").unwrap();
        dir
    }
    #[test]
    fn common_project_uses_managed_dependencies_and_no_shell() {
        let dir = fixture();
        let plan = plan(
            &dir.join("project"),
            &WorkshopMetadata::default(),
            &dir,
            &dir.join("game"),
        )
        .unwrap();
        assert_eq!(plan.tool, "dotnet");
        assert!(plan.arguments.contains(&"--disable-build-servers".into()));
        assert!(plan
            .dependency_dirs
            .contains(&dir.join("GameModding/UserLibs/42")));
        assert!(plan
            .arguments
            .iter()
            .any(|s| s.starts_with("-p:AssemblySearchPaths=")));
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn ambiguous_projects_require_explicit_shared_configuration() {
        let dir = fixture();
        std::fs::write(dir.join("project/Other.csproj"), "<Project/>").unwrap();
        assert!(plan(
            &dir.join("project"),
            &WorkshopMetadata::default(),
            &dir,
            &dir.join("game")
        )
        .is_err());
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn explicit_build_cannot_escape_upload_project() {
        let dir = fixture();
        let meta = WorkshopMetadata {
            build: Some(greg_core::models::ProjectBuildConfig {
                project: "../Outside.csproj".into(),
                configuration: "Release".into(),
            }),
            ..Default::default()
        };
        assert!(plan(&dir.join("project"), &meta, &dir, &dir.join("game")).is_err());
        std::fs::remove_dir_all(dir).unwrap();
    }

    /// Opt-in integration check exercises actual SDK dependency resolution.
    #[test]
    #[ignore = "requires installed .NET SDK 8+; run explicitly for build-runner changes"]
    fn dotnet_build_reports_missing_and_resolved_dependency() {
        let dir = fixture();
        let project = dir.join("project/content/Test.csproj");
        std::fs::write(&project, r#"<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><TargetFramework>net8.0</TargetFramework></PropertyGroup><ItemGroup><Reference Include="FixtureDependency" /></ItemGroup></Project>"#).unwrap();
        std::fs::write(
            dir.join("project/content/Consumer.cs"),
            "public class Consumer { public int Value => FixtureDependency.Api.Value; }",
        )
        .unwrap();
        let cancel = AtomicBool::new(false);
        let missing = plan(
            &dir.join("project"),
            &WorkshopMetadata::default(),
            &dir,
            &dir.join("game"),
        )
        .unwrap();
        let result = run(&missing, &cancel, Duration::from_secs(90)).unwrap();
        assert!(
            !result.success,
            "missing dependency must fail: {}",
            result.output_tail
        );
        assert!(
            result.output_tail.contains("FixtureDependency"),
            "{}",
            result.output_tail
        );
        let dep = dir.join("dependency");
        std::fs::create_dir(&dep).unwrap();
        std::fs::write(dep.join("FixtureDependency.csproj"), r#"<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><TargetFramework>net8.0</TargetFramework></PropertyGroup></Project>"#).unwrap();
        std::fs::write(dep.join("Api.cs"), "namespace FixtureDependency { public static class Api { public static int Value => 42; } }").unwrap();
        let mut dep_plan =
            plan(&dep, &WorkshopMetadata::default(), &dir, &dir.join("game")).unwrap();
        dep_plan.arguments.extend([
            "--output".into(),
            dir.join("GameModding/UserLibs/42")
                .to_string_lossy()
                .into_owned(),
        ]);
        let built = run(&dep_plan, &cancel, Duration::from_secs(90)).unwrap();
        assert!(built.success, "{}", built.output_tail);
        let resolved = plan(
            &dir.join("project"),
            &WorkshopMetadata::default(),
            &dir,
            &dir.join("game"),
        )
        .unwrap();
        let result = run(&resolved, &cancel, Duration::from_secs(90)).unwrap();
        assert!(result.success, "{}", result.output_tail);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
