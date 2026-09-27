//! Build orchestration (`cargo xtask <cmd>`).
//!
//! - `check`: fmt + clippy + tests.
//! - `setup`: copy the Steam native lib next to dev binaries.
//! - `dist`: release build + staged portable package incl. native libs.
//!
//! Native Steam libraries per OS (from the `steamworks-sys` build output):
//! Windows `steam_api64.dll`, Linux `libsteam_api.so`, macOS
//! `libsteam_api.dylib`. The staged package always carries the matching
//! file next to the binaries (rpath/`$ORIGIN` handles Linux/macOS,
//! DLL-search handles Windows).

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};

/// Steam AppID for Data Center (also written to `steam_appid.txt`).
const STEAM_APP_ID: &str = "4170200";
/// Crate version doubles as the product version (workspace package version).
const BIN_APP: &str = "gregmodmanager";
const BIN_CLI: &str = "gregcli";

/// Target OS family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Os {
    Windows,
    Linux,
    Macos,
    Other,
}

/// Target CPU (for package tags only).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Arch {
    X64,
    Arm64,
    Other,
}

/// Native Steam library file name per OS.
fn steam_lib_name(os: Os) -> &'static str {
    match os {
        Os::Windows => "steam_api64.dll",
        Os::Linux => "libsteam_api.so",
        Os::Macos => "libsteam_api.dylib",
        Os::Other => "libsteam_api.so",
    }
}

/// Archive extension per OS (`.zip` on Windows, `.tar.gz` elsewhere).
fn archive_extension(os: Os) -> &'static str {
    match os {
        Os::Windows => "zip",
        _ => "tar.gz",
    }
}

/// Binary suffix per OS.
fn exe_suffix(os: Os) -> &'static str {
    match os {
        Os::Windows => ".exe",
        _ => "",
    }
}

/// Splits a target triple (or host default) into `(Os, Arch, tag)`.
fn parse_target(triple: Option<&str>) -> (Os, Arch, String) {
    let triple = triple
        .map(str::to_string)
        .unwrap_or_else(|| format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS));
    let lower = triple.to_lowercase();
    let os = if lower.contains("windows") {
        Os::Windows
    } else if lower.contains("apple") || lower.contains("darwin") || lower.contains("macos") {
        Os::Macos
    } else if lower.contains("linux") {
        Os::Linux
    } else {
        Os::Other
    };
    let arch = if lower.contains("aarch64") || lower.contains("arm64") {
        Arch::Arm64
    } else if lower.contains("x86_64") || lower.contains("x64") || lower.contains("amd64") {
        Arch::X64
    } else {
        Arch::Other
    };
    let tag = match (os, arch) {
        (Os::Windows, Arch::Arm64) => "win-arm64".to_string(),
        (Os::Windows, _) => "win-x64".to_string(),
        (Os::Linux, Arch::Arm64) => "linux-arm64".to_string(),
        (Os::Linux, _) => "linux-x64".to_string(),
        (Os::Macos, Arch::Arm64) => "macos-arm64".to_string(),
        (Os::Macos, _) => "macos-x64".to_string(),
        (Os::Other, _) => triple.replace(['/', '\\', ' '], "_"),
    };
    (os, arch, tag)
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

/// Cargo artifact dir for a profile/target triple.
fn artifact_dir(root: &Path, triple: Option<&str>, profile: &str) -> PathBuf {
    match triple {
        Some(t) => root.join("target").join(t).join(profile),
        None => root.join("target").join(profile),
    }
}

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("check") => check(),
        Some("setup") => setup_steam_lib(),
        Some("dist") => {
            let mut target: Option<String> = None;
            let mut out: Option<PathBuf> = None;
            let rest: Vec<String> = args.collect();
            let mut i = 0;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--target" => {
                        i += 1;
                        target = rest.get(i).cloned();
                    }
                    "--out" => {
                        i += 1;
                        if let Some(dir) = rest.get(i) {
                            out = Some(PathBuf::from(dir));
                        }
                    }
                    other => bail!("unknown dist flag: {other}"),
                }
                i += 1;
            }
            dist(target.as_deref(), out)
        }
        _ => {
            println!("Usage: cargo xtask <check|setup|dist [--target TRIPLE] [--out DIR]>");
            println!("  check  cargo fmt --check, clippy, test");
            println!("  setup  copy libsteam_api next to dev binaries");
            println!("  dist   release build + portable package incl. native Steam libs");
            Ok(())
        }
    }
}

fn cargo(args: &[&str]) -> Result<()> {
    let status = Command::new("cargo").args(args).status()?;
    if status.success() {
        Ok(())
    } else {
        bail!("cargo {} failed", args.join(" "))
    }
}

fn check() -> Result<()> {
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
fn find_steam_lib(search_root: &Path, file_name: &str) -> Option<PathBuf> {
    let build = search_root.join("build");
    let Ok(entries) = std::fs::read_dir(&build) else {
        return None;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with("steamworks-sys-") {
            let candidate = entry.path().join("out").join(file_name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

/// Copies the Steam native lib next to dev binaries (with rpath it resolves).
fn setup_steam_lib() -> Result<()> {
    let root = workspace_root();
    for profile in ["debug", "release"] {
        let dir = artifact_dir(&root, None, profile);
        for lib in ["libsteam_api.so", "steam_api64.dll", "libsteam_api.dylib"] {
            if let Some(src) = find_steam_lib(&dir, lib) {
                for dest_dir in [&dir, &dir.join("deps")] {
                    if dest_dir.is_dir() {
                        let dest = dest_dir.join(lib);
                        if !dest.is_file() {
                            std::fs::copy(&src, &dest)?;
                            println!("xtask setup: {} -> {}", src.display(), dest.display());
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

/// Release build + staged portable package incl. native Steam libraries.
fn dist(target: Option<&str>, out: Option<PathBuf>) -> Result<()> {
    let root = workspace_root();
    let (os, _arch, tag) = parse_target(target);
    let version = workspace_version(&root)?;
    println!("xtask dist: version={version} tag={tag}");

    // 1. Release build.
    let mut build_args = vec!["build", "--release", "-p", "greg-app", "-p", "greg-cli"];
    let target_owned;
    if let Some(t) = target {
        target_owned = t.to_string();
        build_args.push("--target");
        build_args.push(&target_owned);
    }
    cargo(&build_args)?;

    // 2. Locate binaries + native lib.
    let artifacts = artifact_dir(&root, target, "release");
    let suffix = exe_suffix(os);
    let app_bin = artifacts.join(format!("{BIN_APP}{suffix}"));
    let cli_bin = artifacts.join(format!("{BIN_CLI}{suffix}"));
    for bin in [&app_bin, &cli_bin] {
        if !bin.is_file() {
            bail!("binary missing after build: {}", bin.display());
        }
    }
    let lib_name = steam_lib_name(os);
    let steam_lib = find_steam_lib(&artifacts, lib_name).with_context(|| {
        format!(
            "native Steam lib {lib_name} not found under {}",
            artifacts.display()
        )
    })?;
    println!("xtask dist: native lib {}", steam_lib.display());

    // 3. Stage the portable tree.
    let out_dir = out.unwrap_or_else(|| root.join("dist"));
    let stage = out_dir.join(format!("gregmodmanager-{version}-{tag}"));
    if stage.exists() {
        std::fs::remove_dir_all(&stage)?;
    }
    std::fs::create_dir_all(&stage)?;
    std::fs::copy(&app_bin, stage.join(format!("{BIN_APP}{suffix}")))?;
    std::fs::copy(&cli_bin, stage.join(format!("{BIN_CLI}{suffix}")))?;
    std::fs::copy(&steam_lib, stage.join(lib_name))?;
    std::fs::write(stage.join("steam_appid.txt"), format!("{STEAM_APP_ID}\n"))?;
    for doc in ["README.md", "LICENSE", "CHANGELOG.md"] {
        let src = root.join(doc);
        if src.is_file() {
            std::fs::copy(&src, stage.join(doc))?;
        }
    }
    // 4. Archive + checksum.
    let archive_name = format!("gregmodmanager-{version}-{tag}.{}", archive_extension(os));
    let archive = out_dir.join(&archive_name);
    if archive.exists() {
        std::fs::remove_file(&archive)?;
    }
    if os == Os::Windows {
        zip_dir(&stage, &archive)?;
    } else {
        tar_gz_dir(&stage, &archive)?;
    }
    let digest = sha256_file(&archive)?;
    std::fs::write(
        out_dir.join(format!("{archive_name}.sha256")),
        format!("{digest}  {archive_name}\n"),
    )?;
    println!("xtask dist: {}", archive.display());
    println!("xtask dist: sha256 {digest}");
    // 5. Manifest listing.
    for entry in walk_files(&stage)? {
        println!("  staged: {}", entry.display());
    }
    Ok(())
}

/// Workspace version from the root `Cargo.toml`.
fn workspace_version(root: &Path) -> Result<String> {
    let text = std::fs::read_to_string(root.join("Cargo.toml"))?;
    for line in text.lines() {
        let line = line.trim();
        if let Some(version) = line.strip_prefix("version") {
            let version = version
                .trim()
                .trim_start_matches('=')
                .trim()
                .trim_matches('"');
            if !version.is_empty() {
                // First `version =` under [workspace.package]; naive but stable here.
                return Ok(version.to_string());
            }
        }
    }
    bail!("workspace version not found")
}

/// All files below a dir (relative display paths).
fn walk_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        for entry in std::fs::read_dir(&current)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                out.push(path.strip_prefix(dir).unwrap_or(&path).to_path_buf());
            }
        }
    }
    out.sort();
    Ok(out)
}

/// SHA-256 hex of a file.
fn sha256_file(path: &Path) -> Result<String> {
    use sha2::Digest as _;
    let bytes = std::fs::read(path)?;
    Ok(hex::encode(sha2::Sha256::digest(&bytes)))
}

/// Zips a directory (portable packages, Windows default).
fn zip_dir(src_dir: &Path, archive: &Path) -> Result<()> {
    let file = std::fs::File::create(archive)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    let top = src_dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    for rel in walk_files(src_dir)? {
        let name = format!("{top}/{}", rel.to_string_lossy().replace('\\', "/"));
        zip.start_file(name, options)?;
        let bytes = std::fs::read(src_dir.join(&rel))?;
        use std::io::Write as _;
        zip.write_all(&bytes)?;
    }
    zip.finish()?;
    Ok(())
}

/// Creates a `.tar.gz` of a directory (Unix default).
fn tar_gz_dir(src_dir: &Path, archive: &Path) -> Result<()> {
    let file = std::fs::File::create(archive)?;
    let encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
    let mut tar = tar::Builder::new(encoder);
    let top = src_dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    for rel in walk_files(src_dir)? {
        tar.append_path_with_name(src_dir.join(&rel), format!("{top}/{}", rel.display()))?;
    }
    let encoder = tar.into_inner()?;
    encoder.finish()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_platforms() {
        assert_eq!(steam_lib_name(Os::Windows), "steam_api64.dll");
        assert_eq!(steam_lib_name(Os::Linux), "libsteam_api.so");
        assert_eq!(steam_lib_name(Os::Macos), "libsteam_api.dylib");
        assert_eq!(archive_extension(Os::Windows), "zip");
        assert_eq!(archive_extension(Os::Linux), "tar.gz");
        assert_eq!(exe_suffix(Os::Macos), "");
    }

    #[test]
    fn parses_triples() {
        let (os, _, tag) = parse_target(Some("x86_64-pc-windows-gnu"));
        assert_eq!(os, Os::Windows);
        assert_eq!(tag, "win-x64");
        let (os, _, tag) = parse_target(Some("aarch64-apple-darwin"));
        assert_eq!(os, Os::Macos);
        assert_eq!(tag, "macos-arm64");
        let (os, _, tag) = parse_target(Some("x86_64-unknown-linux-gnu"));
        assert_eq!(os, Os::Linux);
        assert_eq!(tag, "linux-x64");
    }

    #[test]
    fn roundtrips_archives() {
        let base = std::env::temp_dir().join(format!(
            "greg-xtask-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let src = base.join("stage");
        std::fs::create_dir_all(src.join("sub")).unwrap();
        std::fs::write(src.join("a.txt"), b"hello").unwrap();
        std::fs::write(src.join("sub").join("b.bin"), [0u8, 1, 2, 3]).unwrap();
        let zip_path = base.join("out.zip");
        zip_dir(&src, &zip_path).unwrap();
        assert!(zip_path.is_file());
        let tar_path = base.join("out.tar.gz");
        tar_gz_dir(&src, &tar_path).unwrap();
        assert!(tar_path.is_file());
        let digest = sha256_file(&tar_path).unwrap();
        assert_eq!(digest.len(), 64);
        std::fs::remove_dir_all(&base).ok();
    }
}
