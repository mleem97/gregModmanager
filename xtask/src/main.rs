//! Build orchestration (`cargo xtask <cmd>`).

use std::path::PathBuf;
use std::process::Command;

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("check") => check(),
        Some("setup") => setup_steam_lib(),
        Some("dist") => {
            setup_steam_lib()?;
            dist()
        }
        _ => {
            println!("Usage: cargo xtask <check|setup|dist>");
            println!("  check  cargo fmt --check, clippy, test");
            println!("  setup  copy libsteam_api next to dev binaries");
            println!("  dist   setup + release build info");
            Ok(())
        }
    }
}

fn cargo(args: &[&str]) -> anyhow::Result<()> {
    let status = Command::new("cargo").args(args).status()?;
    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("cargo {} failed", args.join(" "))
    }
}

fn check() -> anyhow::Result<()> {
    cargo(&["fmt", "--all", "--", "--check"])?;
    cargo(&[
        "clippy",
        "--workspace",
        "--all-targets",
        "--",
        "-D",
        "warnings",
    ])?;
    cargo(&["test", "--workspace"])?;
    Ok(())
}

/// Finds the Steam native lib in a `steamworks-sys` build output dir.
fn find_steam_lib() -> Option<PathBuf> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest.parent()?.to_path_buf();
    for profile in ["debug", "release"] {
        let build = root.join("target").join(profile).join("build");
        let Ok(entries) = std::fs::read_dir(&build) else {
            continue;
        };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("steamworks-sys-") {
                for lib in ["libsteam_api.so", "steam_api64.dll", "libsteam_api.dylib"] {
                    let candidate = entry.path().join("out").join(lib);
                    if candidate.is_file() {
                        return Some(candidate);
                    }
                }
            }
        }
    }
    None
}

/// Copies the Steam native lib next to dev binaries (with rpath it resolves).
fn setup_steam_lib() -> anyhow::Result<()> {
    let Some(lib) = find_steam_lib() else {
        println!(
            "xtask setup: libsteam_api not found (build greg-steam with the `steam` feature first)"
        );
        return Ok(());
    };
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest.parent().expect("workspace root");
    let name = lib.file_name().expect("file name");
    for profile in ["debug", "release"] {
        for dir in [
            root.join("target").join(profile),
            root.join("target").join(profile).join("deps"),
        ] {
            if dir.is_dir() {
                let dest = dir.join(name);
                if !dest.is_file() {
                    std::fs::copy(&lib, &dest)?;
                    println!("xtask setup: {} -> {}", lib.display(), dest.display());
                }
            }
        }
    }
    Ok(())
}

fn dist() -> anyhow::Result<()> {
    println!("dist: release binaries are in target/release/ (gregmodmanager, gregcli)");
    println!("dist: ship libsteam_api.so / steam_api64.dll beside them (see xtask setup)");
    Ok(())
}
