//! Headless CLI (`HeadlessRunner` port): `publish`, `status`, `list`.
//!
//! Legacy `--mode publish --path …` invocations are translated to the
//! subcommand form, so existing scripts keep working.

use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use clap::{Parser, Subcommand};
use greg_core::docs::project_docs;
use greg_core::models::{ProjectSyncState, RalphTaskStatus};
use greg_core::prefs::Preferences;

const APP_ID: u32 = greg_core::models::settings::DATA_CENTER_APP_ID;

/// gregModmanager headless uploader.
#[derive(Debug, Parser)]
#[command(name = "gregcli", version, about = "Headless Steam Workshop publish")]
struct Cli {
    /// Mirror the file log to stdout.
    #[arg(long, global = true)]
    verbose: bool,

    /// Same as --verbose.
    #[arg(long = "console-log", global = true)]
    console_log: bool,

    /// Write the log file here instead of the default.
    #[arg(long = "log-file", global = true)]
    log_file: Option<PathBuf>,

    /// Legacy mode selector (`publish`, `status`, `list`).
    #[arg(long, global = true)]
    mode: Option<String>,

    /// Legacy alias for `--mode publish`.
    #[arg(long, global = true)]
    upload: bool,

    /// Project root (must contain `content/` and `metadata.json`).
    #[arg(long, global = true)]
    path: Option<PathBuf>,

    /// Change note attached to the Steam update.
    #[arg(long, global = true)]
    changelog: Option<String>,

    /// Alias for --changelog.
    #[arg(long = "change-note", global = true)]
    change_note: Option<String>,

    /// Write `.ralph/tasks/status.json` on completion.
    #[arg(long, global = true)]
    autocommit: bool,

    /// Publish target: `steam` (default) or `modstore`.
    #[arg(long, global = true)]
    to: Option<String>,

    /// Modstore API token (or `GREG_MODSTORE_TOKEN`).
    #[arg(long, global = true)]
    token: Option<String>,

    /// Modstore mod id for updates (or stored from the last publish).
    #[arg(long = "mod-id", global = true)]
    mod_id: Option<String>,

    /// Modstore category (defaults to the project profile).
    #[arg(long, global = true)]
    category: Option<String>,

    /// Modstore license (server default when omitted).
    #[arg(long, global = true)]
    license: Option<String>,

    /// Modstore API base URL (or `GREG_MODSTORE_API_URL`).
    #[arg(long = "api-url", global = true)]
    api_url: Option<String>,

    /// Upload one raw file instead of the zipped `content/` folder.
    #[arg(long, global = true)]
    file: Option<PathBuf>,

    /// Action to run.
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Clone, Subcommand)]
enum Command {
    /// Publish a local workshop project.
    Publish,
    /// Show sync state of one project (exit 0 = synced).
    Status,
    /// List all workspace projects with sync state.
    List,
}

fn main() {
    let code = run(std::env::args().collect());
    std::process::exit(code);
}

fn run(raw_args: Vec<String>) -> i32 {
    let args = translate_legacy(raw_args);
    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(e) => {
            // Clap handles --help/--version with exit code 0 via its own error.
            let _ = e.print();
            return if e.kind() == clap::error::ErrorKind::DisplayHelp
                || e.kind() == clap::error::ErrorKind::DisplayVersion
            {
                0
            } else {
                2
            };
        }
    };

    let log = greg_platform::filelog::FileLog::open(cli.log_file.clone());
    if cli.verbose || cli.console_log {
        log.set_mirror_to_console(true);
    }
    log.start_session();

    let command = cli.command.clone().or_else(|| {
        cli.mode
            .as_deref()
            .and_then(|m| match m.to_lowercase().as_str() {
                "publish" => Some(Command::Publish),
                "status" => Some(Command::Status),
                "list" => Some(Command::List),
                _ => None,
            })
    });
    let command = match command {
        Some(c) => c,
        None if cli.upload => Command::Publish,
        None => {
            eprintln!("No command given. Use publish, status or list (--help).");
            return 2;
        }
    };

    let manual = cli
        .changelog
        .clone()
        .or(cli.change_note.clone())
        .filter(|s| !s.trim().is_empty());
    let code = match command {
        Command::Publish => match cli.path.clone() {
            Some(path) => {
                let target = cli.to.as_deref().unwrap_or("steam");
                if target.eq_ignore_ascii_case("modstore") {
                    run_publish_modstore(&log, &cli, &path, manual.as_deref(), cli.autocommit)
                } else {
                    run_publish(&log, &path, manual.as_deref(), cli.autocommit)
                }
            }
            None => {
                eprintln!("Missing --path <dir>.");
                2
            }
        },
        Command::Status => match cli.path.clone() {
            Some(path) => run_status(&path),
            None => {
                eprintln!("Missing --path <dir>.");
                2
            }
        },
        Command::List => run_list(),
    };
    log.end_session();
    code
}

/// Translates legacy `--mode X` / `--upload` invocations to subcommands.
fn translate_legacy(raw: Vec<String>) -> Vec<String> {
    let has_subcommand = raw
        .iter()
        .any(|a| matches!(a.as_str(), "publish" | "status" | "list"));
    if has_subcommand {
        return raw;
    }
    let mode = raw
        .iter()
        .position(|a| a == "--mode")
        .and_then(|i| raw.get(i + 1))
        .map(|s| s.to_lowercase());
    let upload = raw.iter().any(|a| a == "--upload");
    let subcommand = match mode.as_deref() {
        Some("publish") => Some("publish"),
        Some("status") => Some("status"),
        Some("list") => Some("list"),
        _ if upload => Some("publish"),
        _ => None,
    };
    match subcommand {
        Some(sub) => {
            let mut out = vec![raw[0].clone(), sub.to_string()];
            let mut skip_next = false;
            for arg in raw.iter().skip(1) {
                if skip_next {
                    skip_next = false;
                    continue;
                }
                if arg == "--mode" {
                    skip_next = true;
                    continue;
                }
                if arg == "--upload" {
                    continue;
                }
                out.push(arg.clone());
            }
            out
        }
        None => raw,
    }
}

fn workspace_for_listing() -> greg_platform::workspace::Workspace {
    let prefs = greg_platform::prefs::JsonFilePreferences::open();
    let custom = prefs.get_string(greg_platform::workspace::CUSTOM_WORKSPACE_PATH_KEY, "");
    let custom = if custom.trim().is_empty() {
        std::env::var("GREG_WORKSPACE").ok()
    } else {
        Some(custom)
    };
    greg_platform::workspace::Workspace::resolve(custom.as_deref(), None)
}

fn effective_description(project_root: &Path, lang: &str) -> String {
    let meta = greg_platform::workspace::load_metadata(project_root).unwrap_or_default();
    project_docs::build_effective_steam_description(lang, project_root, &meta)
}

fn run_status(project_root: &Path) -> i32 {
    let root = project_root.to_path_buf();
    if !root.is_dir() {
        eprintln!("Project path not found: {}", root.display());
        return 2;
    }
    let meta = greg_platform::workspace::load_metadata(&root).unwrap_or_default();
    let desc = effective_description(&root, "en");
    let ws = workspace_for_listing();
    let state = ws
        .sync_state(&root, &desc)
        .unwrap_or(ProjectSyncState::Unknown);
    let name = root.file_name().and_then(|n| n.to_str()).unwrap_or("?");
    println!("project: {name}");
    println!("path: {}", root.display());
    println!(
        "workshop-id: {}",
        if meta.published_file_id == 0 {
            "none".to_string()
        } else {
            meta.published_file_id.to_string()
        }
    );
    println!("sync-state: {}", format!("{state:?}").to_uppercase());
    println!(
        "last-published-hash: {}",
        if meta.last_published_hash.is_empty() {
            "none".to_string()
        } else {
            format!(
                "{}…",
                &meta.last_published_hash[..meta.last_published_hash.len().min(16)]
            )
        }
    );
    if state == ProjectSyncState::Synced {
        0
    } else {
        1
    }
}

fn run_list() -> i32 {
    let ws = workspace_for_listing();
    let projects = match ws.scan_projects() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Workspace unreadable ({}): {e}", ws.root().display());
            return 2;
        }
    };
    println!("workspace: {}", ws.root().display());
    println!("projects: {}", projects.len());
    for project in projects {
        let meta = greg_platform::workspace::load_metadata(&project.root_path).unwrap_or_default();
        let desc = effective_description(&project.root_path, "en");
        let state = ws
            .sync_state(&project.root_path, &desc)
            .unwrap_or(ProjectSyncState::Unknown);
        let id = if meta.published_file_id == 0 {
            "none".to_string()
        } else {
            meta.published_file_id.to_string()
        };
        println!("{} | id={id} | {:?}", project.name, state);
    }
    0
}

fn run_publish(
    log: &greg_platform::filelog::FileLog,
    project_root: &Path,
    manual_changelog: Option<&str>,
    autocommit: bool,
) -> i32 {
    let root = project_root.to_path_buf();
    if !root.is_dir() {
        eprintln!("Project path not found: {}", root.display());
        if autocommit {
            write_ralph_status(&root, "publish", false, "Project path not found.");
        }
        return 1;
    }
    if !root.join("content").is_dir() {
        eprintln!("Missing content folder: {}", root.join("content").display());
        if autocommit {
            write_ralph_status(&root, "publish", false, "Missing content folder.");
        }
        return 1;
    }
    let mut meta = greg_platform::workspace::load_metadata(&root).unwrap_or_default();

    // Readiness gate (same checks as the UI).
    let checks = greg_core::upload::check("en", &root, &meta, manual_changelog);
    if !greg_core::models::is_ready_to_upload(&checks) {
        for result in checks
            .iter()
            .filter(|r| r.severity == greg_core::models::UploadCheckSeverity::Error)
        {
            eprintln!("CHECK {}: {}", result.label, result.detail);
        }
        if autocommit {
            write_ralph_status(&root, "publish", false, "Upload checks failed.");
        }
        return 1;
    }

    let backend = greg_steam::backend::connect(APP_ID);
    let service = greg_steam::service::WorkshopService::new(Arc::from(backend), "en");
    let cancel = AtomicBool::new(false);
    let file_log = log.clone();
    let outcome = service.publish(
        &root,
        &mut meta,
        manual_changelog,
        &|p| println!("Upload {:.0}%", p * 100.0),
        &|s| {
            println!("{s}");
            file_log.info(s);
        },
        &cancel,
    );
    if !outcome.success {
        eprintln!("{}", outcome.message);
        log.error(&format!("Headless publish failed: {}", outcome.message));
        if autocommit {
            write_ralph_status(&root, "publish", false, &outcome.message);
        }
        return 1;
    }
    println!("Published. Workshop file id: {}", outcome.published_file_id);
    if autocommit {
        write_ralph_status(
            &root,
            "publish",
            true,
            &format!("Published file id {}", outcome.published_file_id),
        );
    }
    0
}

/// Publishes a project to the Modstore (presigned upload + submit).
fn run_publish_modstore(
    log: &greg_platform::filelog::FileLog,
    cli: &Cli,
    project_root: &Path,
    manual_changelog: Option<&str>,
    autocommit: bool,
) -> i32 {
    use greg_modstore::publish::{PublishMetadata, PublishPayload, PublishPlan};
    use greg_modstore::upload_client::ModStoreUploadClient;

    let root = project_root.to_path_buf();
    if !root.is_dir() {
        eprintln!("Project path not found: {}", root.display());
        return 2;
    }
    let token = cli
        .token
        .clone()
        .or_else(|| std::env::var("GREG_MODSTORE_TOKEN").ok())
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty());
    let Some(token) = token else {
        eprintln!("Missing Modstore API token (--token or GREG_MODSTORE_TOKEN).");
        eprintln!("Create one in the Modstore: Profile -> API Keys.");
        return 2;
    };
    let base_url = cli
        .api_url
        .clone()
        .or_else(|| std::env::var("GREG_MODSTORE_API_URL").ok())
        .unwrap_or_else(greg_core::models::settings::modstore_api_url);

    let mut meta = greg_platform::workspace::load_metadata(&root).unwrap_or_default();
    meta.normalize();

    // Readiness gate (same content checks as the UI/Steam flow).
    let checks = greg_core::upload::check("en", &root, &meta, manual_changelog);
    if !greg_core::models::is_ready_to_upload(&checks) {
        for result in checks
            .iter()
            .filter(|r| r.severity == greg_core::models::UploadCheckSeverity::Error)
        {
            eprintln!("CHECK {}: {}", result.label, result.detail);
        }
        if autocommit {
            write_ralph_status(&root, "modstore-publish", false, "Upload checks failed.");
        }
        return 1;
    }

    // Payload: one raw file or the zipped content folder.
    let payload = match cli.file.clone() {
        Some(file) if file.is_file() => PublishPayload::File { path: file },
        Some(file) => {
            eprintln!("File not found: {}", file.display());
            return 2;
        }
        None => {
            let content = root.join("content");
            if !content.is_dir() {
                eprintln!("Missing content folder: {}", content.display());
                return 2;
            }
            let folder = root.file_name().and_then(|n| n.to_str()).unwrap_or("mod");
            PublishPayload::Directory {
                dir: content,
                file_name: format!("{folder}.zip"),
            }
        }
    };

    let description = project_docs::resolve_modstore_description(&root, &meta.description).text;
    let changelog =
        project_docs::resolve_modstore_changelog(&root, &meta.version, manual_changelog).text;
    let mod_id = cli
        .mod_id
        .clone()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| (!meta.modstore_id.trim().is_empty()).then(|| meta.modstore_id.clone()));
    let category = cli.category.clone().or_else(|| {
        let profile = meta.native_config_profile.trim();
        (!profile.is_empty()).then(|| profile.to_string())
    });
    let plan = match PublishPlan::from_project(
        &payload,
        &PublishMetadata {
            mod_id,
            title: meta.title.clone(),
            version: meta.version.clone(),
            description,
            changelog,
            category,
            license: cli.license.clone(),
            tags: meta.tags.clone(),
            installation: None,
            support_url: None,
            min_game_version: None,
            recommended_game_version: None,
        },
    ) {
        Ok(plan) => plan,
        Err(e) => {
            eprintln!("Packaging failed: {e}");
            return 1;
        }
    };

    let client = match ModStoreUploadClient::new(vec![base_url]) {
        Ok(client) => client.with_token(token),
        Err(e) => {
            eprintln!("Modstore client: {e}");
            return 2;
        }
    };
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("Runtime: {e}");
            return 2;
        }
    };
    println!(
        "Uploading {} ({} bytes) as {}...",
        plan.file_name,
        plan.bytes.len(),
        plan.method
    );
    let outcome = runtime.block_on(plan.execute(&client, &|p| {
        if p % 25 == 0 {
            println!("Upload {p}%");
        }
    }));
    match outcome {
        Ok(response) => {
            println!("Submitted. Mod id: {}", response.mod_id);
            println!("Release id: {}", response.release_id);
            println!("Scan status: {}", response.scan_status);
            if response.manual_review_required {
                println!("Manual review required — watch the moderation status.");
            }
            if !response.mod_id.trim().is_empty() {
                meta.modstore_id = response.mod_id.clone();
                let _ = greg_platform::workspace::save_metadata(&root, &meta);
            }
            log.info(&format!(
                "Headless modstore publish: {} -> mod {}",
                root.display(),
                response.mod_id
            ));
            if autocommit {
                write_ralph_status(
                    &root,
                    "modstore-publish",
                    true,
                    &format!("Submitted mod {}", response.mod_id),
                );
            }
            0
        }
        Err(e) => {
            eprintln!("Modstore publish failed: {e}");
            log.error(&format!("Headless modstore publish failed: {e}"));
            if autocommit {
                write_ralph_status(&root, "modstore-publish", false, &e.to_string());
            }
            1
        }
    }
}

/// Writes `.ralph/tasks/status.json` beside the project on `--autocommit`.
fn write_ralph_status(project_root: &Path, command: &str, ok: bool, message: &str) {
    let status = RalphTaskStatus {
        last_command: command.to_string(),
        ok,
        message: message.to_string(),
        timestamp: greg_core::util::today_ymd(),
    };
    if let Ok(text) = serde_json::to_string_pretty(&status) {
        let dir = project_root.join(".ralph").join("tasks");
        if std::fs::create_dir_all(&dir).is_ok() {
            let _ = std::fs::write(dir.join("status.json"), text);
        }
    }
}
