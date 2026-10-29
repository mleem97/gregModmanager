//! Locally installed content: mods, plugins, libs (`My Mods` pages).
//!
//! Scans the game folders, toggles assemblies via `.disabled` renames and
//! removes files. All paths stay inside the known game folders.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{io_err, LoaderError, Result};

/// Local content kinds for the three pages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocalContentKind {
    /// `{root}/Mods` assemblies.
    Mods,
    /// `{root}/Plugins` + `{root}/greg/Plugins` assemblies.
    Plugins,
    /// `{root}/Plugins/Dependencies` + `{root}/Userlibs` files.
    Libs,
}

impl LocalContentKind {
    /// Stable id used by the UI.
    pub fn id(self) -> &'static str {
        match self {
            Self::Mods => "mymods",
            Self::Plugins => "myplugins",
            Self::Libs => "mylibs",
        }
    }

    /// Page title.
    pub fn title(self) -> &'static str {
        match self {
            Self::Mods => "My Mods",
            Self::Plugins => "My Plugins",
            Self::Libs => "My Libs",
        }
    }
}

/// One installed file entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LocalContentEntry {
    /// File name as shown.
    pub name: String,
    /// Detail line (size + location).
    pub detail: String,
    /// False when renamed to `.disabled`.
    pub enabled: bool,
    /// Absolute path.
    pub path: PathBuf,
}

/// Folders scanned per kind (may not exist).
pub fn folders(game_root: &Path, kind: LocalContentKind) -> Vec<PathBuf> {
    let raw = match kind {
        LocalContentKind::Mods => vec![game_root.join("Mods")],
        LocalContentKind::Plugins => vec![
            game_root.join("Plugins"),
            game_root.join("greg").join("Plugins"),
        ],
        LocalContentKind::Libs => vec![
            game_root.join("UserLibs"),
            game_root.join("Userlibs"),
            game_root.join("Plugins").join("Dependencies"),
        ],
    };
    let mut out: Vec<PathBuf> = Vec::with_capacity(raw.len());
    let mut seen_canonical: Vec<PathBuf> = Vec::with_capacity(raw.len());
    for dir in raw {
        if let Ok(canon) = dir.canonicalize() {
            if seen_canonical.contains(&canon) {
                continue;
            }
            seen_canonical.push(canon);
        }
        out.push(dir);
    }
    out
}

/// Primary directory where `UserLibs` assemblies live (`{game_root}/UserLibs`).
pub fn user_libs_dir(game_root: &Path) -> PathBuf {
    let preferred = game_root.join("UserLibs");
    if preferred.is_dir() {
        return preferred;
    }
    let lower = game_root.join("Userlibs");
    if lower.is_dir() {
        return lower;
    }
    preferred
}

/// Specification for a standard gregFramework runtime library that can be
/// installed into `{game_root}/UserLibs`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameworkLibrarySpec {
    /// Assembly file name inside `UserLibs` (e.g. `"Jint.dll"`).
    pub file_name: &'static str,
    /// Human-readable size and purpose description.
    pub detail: &'static str,
    /// NuGet package ID (lowercase) for v3 flat-container lookup.
    pub nuget_id: &'static str,
    /// NuGet package version matching the gregFramework target.
    pub nuget_version: &'static str,
    /// Entry path inside the `.nupkg` archive.
    pub nupkg_entry: &'static str,
}

/// Official gregFramework runtime libraries available in the "Install Libraries" modal.
pub const FRAMEWORK_LIBRARIES: &[FrameworkLibrarySpec] = &[
    FrameworkLibrarySpec {
        file_name: "Acornima.dll",
        detail: "353 KB · JS parser (Jint dependency)",
        nuget_id: "acornima",
        nuget_version: "1.4.0",
        nupkg_entry: "lib/netstandard2.1/Acornima.dll",
    },
    FrameworkLibrarySpec {
        file_name: "Jint.dll",
        detail: "2.0 MB · JavaScript / TypeScript engine",
        nuget_id: "jint",
        nuget_version: "4.8.0",
        nupkg_entry: "lib/netstandard2.1/Jint.dll",
    },
    FrameworkLibrarySpec {
        file_name: "LiteDB.dll",
        detail: "499 KB · Embedded NoSQL document database",
        nuget_id: "litedb",
        nuget_version: "5.0.21",
        nupkg_entry: "lib/netstandard2.0/LiteDB.dll",
    },
    FrameworkLibrarySpec {
        file_name: "Mono.Cecil.dll",
        detail: "353 KB · .NET IL assembly inspector / weaver",
        nuget_id: "mono.cecil",
        nuget_version: "0.11.6",
        nupkg_entry: "lib/netstandard2.0/Mono.Cecil.dll",
    },
    FrameworkLibrarySpec {
        file_name: "MoonSharp.Interpreter.dll",
        detail: "349 KB · Lua scripting interpreter",
        nuget_id: "moonsharp",
        nuget_version: "2.0.0",
        nupkg_entry: "lib/netstandard1.6/MoonSharp.Interpreter.dll",
    },
    FrameworkLibrarySpec {
        file_name: "NLayer.dll",
        detail: "67 KB · Managed MP3 audio decoder",
        nuget_id: "nlayer",
        nuget_version: "1.11.0",
        nupkg_entry: "lib/net35/NLayer.dll",
    },
    FrameworkLibrarySpec {
        file_name: "Newtonsoft.Json.dll",
        detail: "696 KB · JSON serializer for .NET",
        nuget_id: "newtonsoft.json",
        nuget_version: "13.0.3",
        nupkg_entry: "lib/net6.0/Newtonsoft.Json.dll",
    },
    FrameworkLibrarySpec {
        file_name: "Python.Runtime.dll",
        detail: "440 KB · Python.NET runtime bridge",
        nuget_id: "pythonnet",
        nuget_version: "3.0.5",
        nupkg_entry: "lib/netstandard2.0/Python.Runtime.dll",
    },
    FrameworkLibrarySpec {
        file_name: "dnlib.dll",
        detail: "1.2 MB · .NET module & assembly reader/writer",
        nuget_id: "dnlib",
        nuget_version: "4.5.0",
        nupkg_entry: "lib/net6.0/dnlib.dll",
    },
];

/// Returns true when `file_name` (or `{file_name}.disabled`) is already present
/// in any scanned library folder below `game_root`.
pub fn is_framework_lib_installed(game_root: &Path, file_name: &str) -> bool {
    let disabled = format!("{file_name}.disabled");
    folders(game_root, LocalContentKind::Libs)
        .into_iter()
        .any(|dir| dir.join(file_name).is_file() || dir.join(&disabled).is_file())
}

/// Installs a single framework library into `{game_root}/UserLibs/{file_name}`.
///
/// Prefers a matching DLL in the local NuGet cache (`~/.nuget/packages/...`) when
/// present on the machine, and falls back to downloading the `.nupkg` from the
/// official NuGet v3 flat container and extracting `spec.nupkg_entry`.
pub async fn install_framework_library(
    game_root: &Path,
    spec: &FrameworkLibrarySpec,
    cancel: &std::sync::atomic::AtomicBool,
) -> Result<PathBuf> {
    use std::sync::atomic::Ordering;
    if cancel.load(Ordering::Relaxed) {
        return Err(LoaderError::Cancelled);
    }
    let dest_dir = user_libs_dir(game_root);
    std::fs::create_dir_all(&dest_dir).map_err(io_err)?;
    let dest_file = dest_dir.join(spec.file_name);
    let disabled_sibling = dest_dir.join(format!("{}.disabled", spec.file_name));

    // 1. Fast path: local NuGet package cache (`~/.nuget/packages/{id}/{ver}/{entry}`).
    if let Some(packages_dir) = greg_platform::paths::nuget_packages_dir() {
        let cached = packages_dir
            .join(spec.nuget_id)
            .join(spec.nuget_version)
            .join(spec.nupkg_entry);
        if cached.is_file() {
            std::fs::copy(&cached, &dest_file).map_err(io_err)?;
            if disabled_sibling.is_file() {
                let _ = std::fs::remove_file(&disabled_sibling);
            }
            return Ok(dest_file);
        }
    }

    // 2. Network path: download `.nupkg` from NuGet v3 flatcontainer and extract the DLL.
    let url = format!(
        "https://api.nuget.org/v3-flatcontainer/{id}/{ver}/{id}.{ver}.nupkg",
        id = spec.nuget_id,
        ver = spec.nuget_version,
    );
    let bytes = reqwest::Client::new()
        .get(&url)
        .header("User-Agent", "gregmodmanager")
        .send()
        .await
        .map_err(|e| LoaderError::Network(e.to_string()))?
        .error_for_status()
        .map_err(|e| LoaderError::Network(e.to_string()))?
        .bytes()
        .await
        .map_err(|e| LoaderError::Network(e.to_string()))?;
    if cancel.load(Ordering::Relaxed) {
        return Err(LoaderError::Cancelled);
    }
    extract_single_zip_entry(&bytes, spec.nupkg_entry, &dest_file)?;
    if disabled_sibling.is_file() {
        let _ = std::fs::remove_file(&disabled_sibling);
    }
    Ok(dest_file)
}

/// Extracts `entry_path` from a zip archive into `dest_file`.
pub fn extract_single_zip_entry(bytes: &[u8], entry_path: &str, dest_file: &Path) -> Result<()> {
    use std::io::Cursor;
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|e| LoaderError::InvalidArtefact(format!("unreadable nupkg: {e}")))?;
    let mut found_idx = None;
    for i in 0..zip.len() {
        if let Ok(entry) = zip.by_index(i) {
            if entry.name().eq_ignore_ascii_case(entry_path) {
                found_idx = Some(i);
                break;
            }
        }
    }
    let idx = found_idx.ok_or_else(|| {
        LoaderError::InvalidArtefact(format!("entry '{entry_path}' not found in package"))
    })?;
    let mut entry = zip
        .by_index(idx)
        .map_err(|e| LoaderError::InvalidArtefact(format!("bad zip entry: {e}")))?;
    if let Some(parent) = dest_file.parent() {
        std::fs::create_dir_all(parent).map_err(io_err)?;
    }
    let mut out = std::fs::File::create(dest_file).map_err(io_err)?;
    std::io::copy(&mut entry, &mut out).map_err(io_err)?;
    Ok(())
}

/// Specification for an official MelonLoader plugin in the "My Plugins" catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PluginCatalogSpec {
    /// Target assembly file name inside `{game_root}/Plugins` (e.g. `"Il2CppAssemblyFixerPlugin.dll"`).
    pub file_name: &'static str,
    /// Human-readable plugin title.
    pub display_name: &'static str,
    /// Short description shown in the "Browse Plugins" / "Install Plugins" modal.
    pub detail: &'static str,
    /// Primary release feed API endpoint on Datacentermods Forgejo/Gitea.
    pub datacentermods_api_url: &'static str,
    /// Fallback release feed API endpoint on GitHub.
    pub github_api_url: &'static str,
    /// Candidate asset name patterns in release artefacts.
    pub asset_patterns: &'static [&'static str],
}

/// Official MelonLoader plugins available in the "My Plugins" view ("Browse Plugins" and "Install Plugins").
///
/// Index `0` is the primary **IL2CPP Assembly Fixer** plugin.
pub const FRAMEWORK_PLUGINS: &[PluginCatalogSpec] = &[
    PluginCatalogSpec {
        file_name: "Il2CppAssemblyFixerPlugin.dll",
        display_name: "IL2CPP Assembly Fixer",
        detail: "MelonLoader plugin · Fixes IL2CPP strip/interop assemblies at startup",
        datacentermods_api_url:
            "https://git.datacentermods.com/api/v1/repos/teamGreg/Il2CppAssemblyFixer/releases",
        github_api_url:
            "https://api.github.com/repos/mleem97/Il2CppAssemblyFixer/releases/latest",
        asset_patterns: &[
            "Il2CppAssemblyFixerPlugin",
            "MelonLoader.zip",
            "Il2CppAssemblyFixerPlugin.dll",
        ],
    },
    PluginCatalogSpec {
        file_name: "gregPlugin.Companion.dll",
        display_name: "Modmanager Companion",
        detail: "MelonLoader plugin · In-game companion integration for gregModmanager",
        datacentermods_api_url:
            "https://git.datacentermods.com/api/v1/repos/teamGreg/gregPlugin.Companion/releases",
        github_api_url:
            "https://api.github.com/repos/mleem97/gregPlugin.Companion/releases/latest",
        asset_patterns: &["gregPlugin.Companion.dll", "gregPlugin.Companion"],
    },
    PluginCatalogSpec {
        file_name: "gregPlugin.ExplorerBridge.dll",
        display_name: "Explorer Bridge",
        detail: "MelonLoader plugin · Runtime inspector and UnityExplorer bridge",
        datacentermods_api_url:
            "https://git.datacentermods.com/api/v1/repos/teamGreg/gregPlugin.ExplorerBridge/releases",
        github_api_url:
            "https://api.github.com/repos/mleem97/gregPlugin.ExplorerBridge/releases/latest",
        asset_patterns: &["gregPlugin.ExplorerBridge.dll", "gregPlugin.ExplorerBridge"],
    },
    PluginCatalogSpec {
        file_name: "gregPlugin.SteamModfix.dll",
        display_name: "Steam Modfix",
        detail: "MelonLoader plugin · Steam Workshop compatibility fix",
        datacentermods_api_url:
            "https://git.datacentermods.com/api/v1/repos/teamGreg/gregPlugin.SteamModfix/releases",
        github_api_url:
            "https://api.github.com/repos/mleem97/gregPlugin.SteamModfix/releases/latest",
        asset_patterns: &["gregPlugin.SteamModfix.dll", "gregPlugin.SteamModfix"],
    },
];

/// Returns true when `file_name` (or `{file_name}.disabled`) is present in
/// `{game_root}/Plugins` or `{game_root}/greg/Plugins`.
pub fn is_plugin_installed(game_root: &Path, file_name: &str) -> bool {
    let disabled = format!("{file_name}.disabled");
    folders(game_root, LocalContentKind::Plugins)
        .into_iter()
        .any(|dir| dir.join(file_name).is_file() || dir.join(&disabled).is_file())
}

/// Convenience installer for the primary **IL2CPP Assembly Fixer** plugin (`FRAMEWORK_PLUGINS[0]`).
pub async fn install_latest_il2cpp_fixer(
    game_root: &Path,
    cancel: &std::sync::atomic::AtomicBool,
) -> Result<(String, PathBuf)> {
    install_plugin_from_release(game_root, &FRAMEWORK_PLUGINS[0], cancel).await
}

/// Installs a plugin from its release feeds into `{game_root}/Plugins/{spec.file_name}`.
///
/// Strict source priority:
/// 1. **Datacentermods** Forgejo/Gitea API (`{datacentermods_api_url}/latest`, then `{datacentermods_api_url}?limit=5`)
/// 2. **GitHub** releases API (`{github_api_url}`, then `{github_base}/releases?per_page=5`)
pub async fn install_plugin_from_release(
    game_root: &Path,
    spec: &PluginCatalogSpec,
    cancel: &std::sync::atomic::AtomicBool,
) -> Result<(String, PathBuf)> {
    use std::sync::atomic::Ordering;
    if cancel.load(Ordering::Relaxed) {
        return Err(LoaderError::Cancelled);
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    let dc_base = spec
        .datacentermods_api_url
        .trim_end_matches('/')
        .trim_end_matches("/latest");
    let gh_base = spec
        .github_api_url
        .trim_end_matches('/')
        .trim_end_matches("/latest");

    let candidate_endpoints = [
        (format!("{dc_base}/latest"), false),
        (format!("{dc_base}?limit=5"), false),
        (spec.github_api_url.to_string(), true),
        (format!("{gh_base}?per_page=5"), true),
    ];

    let mut resolved: Option<(String, String, String)> = None;
    let mut last_err: Option<String> = None;

    for (endpoint, is_github) in &candidate_endpoints {
        if cancel.load(Ordering::Relaxed) {
            return Err(LoaderError::Cancelled);
        }
        let mut req = client.get(endpoint).header("User-Agent", "gregmodmanager");
        if *is_github {
            req = req.header("Accept", "application/vnd.github+json");
        } else {
            req = req.header("Accept", "application/json");
        }
        match req.send().await {
            Ok(resp) => match resp.error_for_status() {
                Ok(ok_resp) => match ok_resp.text().await {
                    Ok(body) => {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                            if let Some(found) = match_release_asset(&json, spec) {
                                resolved = Some(found);
                                break;
                            }
                        }
                    }
                    Err(e) => last_err = Some(e.to_string()),
                },
                Err(e) => last_err = Some(e.to_string()),
            },
            Err(e) => last_err = Some(e.to_string()),
        }
    }

    let (version, asset_name, download_url) = resolved.ok_or_else(|| {
        let suffix = last_err.map(|e| format!(" ({e})")).unwrap_or_default();
        LoaderError::InvalidArtefact(format!(
            "no compatible release asset found for {}{suffix}",
            spec.display_name
        ))
    })?;

    if cancel.load(Ordering::Relaxed) {
        return Err(LoaderError::Cancelled);
    }

    let bytes = client
        .get(&download_url)
        .header("User-Agent", "gregmodmanager")
        .send()
        .await
        .map_err(|e| LoaderError::Network(e.to_string()))?
        .error_for_status()
        .map_err(|e| LoaderError::Network(e.to_string()))?
        .bytes()
        .await
        .map_err(|e| LoaderError::Network(e.to_string()))?;

    if cancel.load(Ordering::Relaxed) {
        return Err(LoaderError::Cancelled);
    }

    let dest_path = install_plugin_payload(game_root, spec, &asset_name, &bytes)?;
    Ok((version, dest_path))
}

/// Inspects a single release JSON object or an array of release JSON objects
/// (Forgejo/Gitea or GitHub format) and returns `(version, asset_name, download_url)`
/// for the first release containing a matching asset for `spec`.
pub fn match_release_asset(
    json: &serde_json::Value,
    spec: &PluginCatalogSpec,
) -> Option<(String, String, String)> {
    if let Some(arr) = json.as_array() {
        for item in arr {
            if let Some(found) = match_single_release_object(item, spec) {
                return Some(found);
            }
        }
        None
    } else {
        match_single_release_object(json, spec)
    }
}

fn match_single_release_object(
    release: &serde_json::Value,
    spec: &PluginCatalogSpec,
) -> Option<(String, String, String)> {
    let tag = release.get("tag_name").and_then(|v| v.as_str())?;
    let trimmed = tag.trim_start_matches(['v', 'V']);
    let version = if trimmed.is_empty() {
        tag.to_string()
    } else {
        trimmed.to_string()
    };
    let assets = release.get("assets").and_then(|v| v.as_array())?;
    let stem = spec
        .file_name
        .strip_suffix(".dll")
        .unwrap_or(spec.file_name);
    let stem_lower = stem.to_ascii_lowercase();

    let parsed: Vec<(String, String)> = assets
        .iter()
        .filter_map(|a| {
            let name = a.get("name")?.as_str()?.trim().to_string();
            let url = a.get("browser_download_url")?.as_str()?.trim().to_string();
            if name.is_empty() || url.is_empty() {
                None
            } else {
                Some((name, url))
            }
        })
        .collect();

    // 1. Prefer `{stem}*_MelonLoader.zip` (e.g. `Il2CppAssemblyFixerPlugin_${VERSION}_MelonLoader.zip`).
    if let Some((name, url)) = parsed.iter().find(|(n, _)| {
        let lower = n.to_ascii_lowercase();
        lower.contains(&stem_lower) && lower.ends_with("_melonloader.zip")
    }) {
        return Some((version, name.clone(), url.clone()));
    }

    // 2. Exact DLL match (`spec.file_name`, case-insensitive).
    if let Some((name, url)) = parsed
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(spec.file_name))
    {
        return Some((version, name.clone(), url.clone()));
    }

    // 3. Any `.zip` or `.dll` containing `stem` (excluding BepInEx archives).
    if let Some((name, url)) = parsed.iter().find(|(n, _)| {
        let lower = n.to_ascii_lowercase();
        lower.contains(&stem_lower)
            && !lower.contains("bepinex")
            && (lower.ends_with(".zip") || lower.ends_with(".dll"))
    }) {
        return Some((version, name.clone(), url.clone()));
    }

    None
}

/// Installs a downloaded plugin payload (`.zip` archive or raw `.dll`) into `{game_root}/Plugins`.
///
/// For `.zip` archives (such as `Il2CppAssemblyFixerPlugin_${VERSION}_MelonLoader.zip`):
/// - Extracts `spec.file_name` into `{game_root}/Plugins/{spec.file_name}`
/// - Writes `fixer_config.json` to `{game_root}/Plugins/fixer_config.json` if not already present
/// - Writes helper UserLibs (`dnlib.dll`, `Mono.Cecil.dll`) to `user_libs_dir(game_root)` if not already present
/// - Removes any `{spec.file_name}.disabled` sibling so the newly installed plugin is active
pub fn install_plugin_payload(
    game_root: &Path,
    spec: &PluginCatalogSpec,
    asset_name: &str,
    bytes: &[u8],
) -> Result<PathBuf> {
    let plugins_dir = game_root.join("Plugins");
    std::fs::create_dir_all(&plugins_dir).map_err(io_err)?;
    let dest_file = plugins_dir.join(spec.file_name);

    let is_zip = asset_name.to_ascii_lowercase().ends_with(".zip")
        || (bytes.len() >= 4 && &bytes[0..4] == b"PK\x03\x04");

    if is_zip {
        use std::io::Cursor;
        let mut zip = zip::ZipArchive::new(Cursor::new(bytes))
            .map_err(|e| LoaderError::InvalidArtefact(format!("unreadable plugin zip: {e}")))?;
        let mut extracted_main_dll = false;

        for i in 0..zip.len() {
            let mut entry = zip
                .by_index(i)
                .map_err(|e| LoaderError::InvalidArtefact(format!("bad zip entry: {e}")))?;
            if entry.is_dir() || entry.is_symlink() {
                continue;
            }
            let raw_name = entry.name().to_string();
            if raw_name.contains("..") {
                continue;
            }
            let base_name = raw_name.rsplit(['/', '\\']).next().unwrap_or(&raw_name);

            if base_name.eq_ignore_ascii_case(spec.file_name) {
                let mut out = std::fs::File::create(&dest_file).map_err(io_err)?;
                std::io::copy(&mut entry, &mut out).map_err(io_err)?;
                extracted_main_dll = true;
            } else if base_name.eq_ignore_ascii_case("fixer_config.json") {
                let cfg_dest = plugins_dir.join("fixer_config.json");
                if !cfg_dest.is_file() {
                    let mut out = std::fs::File::create(&cfg_dest).map_err(io_err)?;
                    std::io::copy(&mut entry, &mut out).map_err(io_err)?;
                }
            } else if base_name.eq_ignore_ascii_case("dnlib.dll")
                || base_name.eq_ignore_ascii_case("Mono.Cecil.dll")
            {
                let canonical_lib = if base_name.eq_ignore_ascii_case("dnlib.dll") {
                    "dnlib.dll"
                } else {
                    "Mono.Cecil.dll"
                };
                if !is_framework_lib_installed(game_root, canonical_lib) {
                    let libs_dir = user_libs_dir(game_root);
                    std::fs::create_dir_all(&libs_dir).map_err(io_err)?;
                    let lib_dest = libs_dir.join(canonical_lib);
                    if !lib_dest.is_file() {
                        let mut out = std::fs::File::create(&lib_dest).map_err(io_err)?;
                        std::io::copy(&mut entry, &mut out).map_err(io_err)?;
                    }
                }
            }
        }

        if !extracted_main_dll {
            return Err(LoaderError::InvalidArtefact(format!(
                "'{}' not found in plugin zip '{asset_name}'",
                spec.file_name
            )));
        }
    } else {
        std::fs::write(&dest_file, bytes).map_err(io_err)?;
    }

    // Remove any `.disabled` sibling in known plugin folders so the plugin is active.
    let disabled_name = format!("{}.disabled", spec.file_name);
    for folder in folders(game_root, LocalContentKind::Plugins) {
        let disabled_path = folder.join(&disabled_name);
        if disabled_path.is_file() {
            let _ = std::fs::remove_file(&disabled_path);
        }
    }

    Ok(dest_file)
}

/// Sibling preview image next to an installed file (`Foo.png`/`Foo.jpg`
/// next to `Foo.dll`, stem match, `.disabled` suffix ignored).
pub fn sibling_preview(file: &Path) -> Option<PathBuf> {
    let dir = file.parent()?;
    let name = file.file_name()?.to_str()?;
    let stem = name
        .strip_suffix(".disabled")
        .unwrap_or(name)
        .rsplit_once('.')
        .map(|(s, _)| s)
        .unwrap_or(name);
    if stem.is_empty() {
        return None;
    }
    ["png", "jpg", "jpeg"]
        .into_iter()
        .map(|ext| dir.join(format!("{stem}.{ext}")))
        .find(|p| p.is_file())
}

/// Scans installed entries (`.dll` + `.dll.disabled`; libs: any file).
pub fn scan(game_root: &Path, kind: LocalContentKind) -> Vec<LocalContentEntry> {
    let mut entries = Vec::new();
    for folder in folders(game_root, kind) {
        if !folder.is_dir() {
            continue;
        }
        let Ok(read) = std::fs::read_dir(&folder) else {
            continue;
        };
        let location = folder
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
            .to_string();
        for entry in read.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };
            let (display, enabled) = match name.strip_suffix(".disabled") {
                Some(base) => (base.to_string(), false),
                None => (name.clone(), true),
            };
            let is_dll = display.to_lowercase().ends_with(".dll");
            if kind != LocalContentKind::Libs && !is_dll {
                continue;
            }
            let size = entry
                .metadata()
                .map(|m| greg_core::util::format_bytes(m.len() as i64))
                .unwrap_or_default();
            entries.push(LocalContentEntry {
                name: display,
                detail: format!("{size} · {location}"),
                enabled,
                path,
            });
        }
    }
    entries.sort_by_key(|a| a.name.to_lowercase());
    entries
}

/// Cheap change signature for a kind (`name|size|mtime|enabled` per file).
/// Poll this to rescan only when something actually changed.
pub fn signature(game_root: &Path, kind: LocalContentKind) -> String {
    let mut parts: Vec<String> = Vec::new();
    for folder in folders(game_root, kind) {
        if !folder.is_dir() {
            continue;
        }
        let Ok(read) = std::fs::read_dir(&folder) else {
            continue;
        };
        for entry in read.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?")
                .to_string();
            let (size, mtime) = entry
                .metadata()
                .map(|m| {
                    (
                        m.len(),
                        m.modified()
                            .ok()
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|d| d.as_secs())
                            .unwrap_or(0),
                    )
                })
                .unwrap_or((0, 0));
            parts.push(format!("{name}|{size}|{mtime}"));
        }
    }
    parts.sort();
    parts.join("\n")
}

/// Toggles an entry (`.dll` ↔ `.dll.disabled`). Non-DLL files cannot toggle.
pub fn set_enabled(entry: &LocalContentEntry, enabled: bool) -> Result<PathBuf> {
    if entry.enabled == enabled {
        return Ok(entry.path.clone());
    }
    let target = if enabled {
        entry
            .path
            .to_string_lossy()
            .strip_suffix(".disabled")
            .map(PathBuf::from)
            .ok_or_else(|| LoaderError::Other("not a disabled assembly".into()))?
    } else {
        PathBuf::from(format!("{}.disabled", entry.path.display()))
    };
    if target.is_file() {
        let _ = std::fs::remove_file(&target);
    }
    std::fs::rename(&entry.path, &target).map_err(io_err)?;
    Ok(target)
}

/// Removes an installed file. Callers confirm first (UI dialog).
pub fn remove(entry: &LocalContentEntry) -> Result<()> {
    if !entry.path.is_file() {
        return Err(LoaderError::Other("file is already gone".into()));
    }
    std::fs::remove_file(&entry.path).map_err(io_err)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game_root(name: &str) -> PathBuf {
        let base = std::env::temp_dir().join(format!(
            "greg-local-{name}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(base.join("Mods")).unwrap();
        std::fs::create_dir_all(base.join("Plugins").join("Dependencies")).unwrap();
        std::fs::write(base.join("Mods").join("a.dll"), b"12345").unwrap();
        std::fs::write(base.join("Mods").join("b.dll.disabled"), b"12").unwrap();
        std::fs::write(base.join("Mods").join("notes.txt"), b"hi").unwrap();
        std::fs::write(
            base.join("Plugins").join("Dependencies").join("lib.dll"),
            b"123",
        )
        .unwrap();
        base
    }

    #[test]
    fn scans_and_toggles() {
        let root = game_root("scan");
        let mods = scan(&root, LocalContentKind::Mods);
        assert_eq!(mods.len(), 2);
        assert!(mods.iter().any(|e| e.name == "a.dll" && e.enabled));
        assert!(mods.iter().any(|e| e.name == "b.dll" && !e.enabled));

        let a = mods.iter().find(|e| e.name == "a.dll").unwrap().clone();
        let disabled = set_enabled(&a, false).unwrap();
        assert!(disabled.to_string_lossy().ends_with(".disabled"));
        let back = LocalContentEntry {
            path: disabled,
            ..a
        };
        let re = set_enabled(
            &LocalContentEntry {
                enabled: false,
                ..back
            },
            true,
        )
        .unwrap();
        assert!(re.to_string_lossy().ends_with("a.dll"));

        let libs = scan(&root, LocalContentKind::Libs);
        assert_eq!(libs.len(), 1);
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn signature_tracks_changes() {
        let root = game_root("sig");
        let before = signature(&root, LocalContentKind::Mods);
        std::fs::write(root.join("Mods").join("new.dll"), b"123").unwrap();
        let after = signature(&root, LocalContentKind::Mods);
        assert_ne!(before, after);
        // Toggle renames the file → signature changes too.
        let mods = scan(&root, LocalContentKind::Mods);
        let entry = mods.iter().find(|e| e.name == "new.dll").unwrap();
        set_enabled(entry, false).unwrap();
        assert_ne!(after, signature(&root, LocalContentKind::Mods));
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn removes_files() {
        let root = game_root("remove");
        let mods = scan(&root, LocalContentKind::Mods);
        let a = mods.iter().find(|e| e.name == "a.dll").unwrap();
        remove(a).unwrap();
        assert!(!a.path.exists());
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn finds_sibling_previews() {
        let root = game_root("preview");
        let dir = root.join("Mods");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.dll"), b"12345").unwrap();
        std::fs::write(dir.join("a.png"), b"png").unwrap();
        std::fs::write(dir.join("b.dll.disabled"), b"12").unwrap();
        std::fs::write(dir.join("b.jpg"), b"jpg").unwrap();
        std::fs::write(dir.join("c.dll"), b"1").unwrap();
        assert_eq!(sibling_preview(&dir.join("a.dll")), Some(dir.join("a.png")));
        assert_eq!(
            sibling_preview(&dir.join("b.dll.disabled")),
            Some(dir.join("b.jpg"))
        );
        assert_eq!(sibling_preview(&dir.join("c.dll")), None);
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn scans_userlibs_and_detects_framework_libraries() {
        use std::io::Write as _;
        let root = game_root("userlibs");
        let user_libs = root.join("UserLibs");
        std::fs::create_dir_all(&user_libs).unwrap();
        assert_eq!(FRAMEWORK_LIBRARIES.len(), 9);
        assert!(!is_framework_lib_installed(&root, "Jint.dll"));

        let mut buf = Vec::new();
        {
            let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
            zip.start_file(
                "lib/netstandard2.1/Jint.dll",
                zip::write::SimpleFileOptions::default(),
            )
            .unwrap();
            zip.write_all(b"MZ-fake-jint").unwrap();
            zip.finish().unwrap();
        }
        let target = user_libs.join("Jint.dll");
        extract_single_zip_entry(&buf, "lib/netstandard2.1/Jint.dll", &target).unwrap();
        assert!(is_framework_lib_installed(&root, "Jint.dll"));
        let libs = scan(&root, LocalContentKind::Libs);
        assert!(libs.iter().any(|e| e.name == "Jint.dll" && e.enabled));
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn framework_plugins_catalog_and_detection() {
        let root = game_root("plugins-catalog");
        assert_eq!(FRAMEWORK_PLUGINS.len(), 4);
        assert_eq!(
            FRAMEWORK_PLUGINS[0].file_name,
            "Il2CppAssemblyFixerPlugin.dll"
        );
        assert_eq!(FRAMEWORK_PLUGINS[0].display_name, "IL2CPP Assembly Fixer");
        assert_eq!(
            FRAMEWORK_PLUGINS[0].datacentermods_api_url,
            "https://git.datacentermods.com/api/v1/repos/teamGreg/Il2CppAssemblyFixer/releases"
        );
        assert_eq!(
            FRAMEWORK_PLUGINS[0].github_api_url,
            "https://api.github.com/repos/mleem97/Il2CppAssemblyFixer/releases/latest"
        );
        assert_eq!(FRAMEWORK_PLUGINS[1].file_name, "gregPlugin.Companion.dll");
        assert_eq!(
            FRAMEWORK_PLUGINS[2].file_name,
            "gregPlugin.ExplorerBridge.dll"
        );
        assert_eq!(FRAMEWORK_PLUGINS[3].file_name, "gregPlugin.SteamModfix.dll");

        assert!(!is_plugin_installed(&root, "Il2CppAssemblyFixerPlugin.dll"));

        let greg_plugins = root.join("greg").join("Plugins");
        std::fs::create_dir_all(&greg_plugins).unwrap();
        std::fs::write(
            greg_plugins.join("gregPlugin.Companion.dll.disabled"),
            b"MZ",
        )
        .unwrap();
        assert!(is_plugin_installed(&root, "gregPlugin.Companion.dll"));

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn extracts_il2cpp_fixer_melonloader_zip_and_helper_libs() {
        use std::io::Write as _;
        let root = game_root("il2cpp-fixer-zip");
        let plugins_dir = root.join("Plugins");
        std::fs::create_dir_all(&plugins_dir).unwrap();
        // Pre-seed a `.disabled` sibling to verify it gets cleaned up on install.
        std::fs::write(
            plugins_dir.join("Il2CppAssemblyFixerPlugin.dll.disabled"),
            b"old-disabled",
        )
        .unwrap();

        let release_json = serde_json::json!([
            {
                "tag_name": "v0.1.0",
                "assets": [
                    {
                        "name": "Il2CppAssemblyFixerPlugin_0.1.0_BepInEx.zip",
                        "browser_download_url": "https://git.datacentermods.com/attachments/bepinex.zip"
                    },
                    {
                        "name": "Il2CppAssemblyFixerPlugin_0.1.0_MelonLoader.zip",
                        "browser_download_url": "https://git.datacentermods.com/attachments/melonloader.zip"
                    }
                ]
            }
        ]);

        let (version, asset_name, url) =
            match_release_asset(&release_json, &FRAMEWORK_PLUGINS[0]).unwrap();
        assert_eq!(version, "0.1.0");
        assert_eq!(
            asset_name,
            "Il2CppAssemblyFixerPlugin_0.1.0_MelonLoader.zip"
        );
        assert_eq!(
            url,
            "https://git.datacentermods.com/attachments/melonloader.zip"
        );

        let mut zip_bytes = Vec::new();
        {
            let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut zip_bytes));
            let opts = zip::write::SimpleFileOptions::default();
            zip.start_file("Plugins/Il2CppAssemblyFixerPlugin.dll", opts)
                .unwrap();
            zip.write_all(b"MZ-il2cpp-fixer").unwrap();
            zip.start_file("Plugins/fixer_config.json", opts).unwrap();
            zip.write_all(br#"{"enabled":true}"#).unwrap();
            zip.start_file("UserLibs/dnlib.dll", opts).unwrap();
            zip.write_all(b"MZ-dnlib").unwrap();
            zip.start_file("UserLibs/Mono.Cecil.dll", opts).unwrap();
            zip.write_all(b"MZ-cecil").unwrap();
            zip.finish().unwrap();
        }

        let installed_path =
            install_plugin_payload(&root, &FRAMEWORK_PLUGINS[0], &asset_name, &zip_bytes).unwrap();

        assert_eq!(
            installed_path,
            root.join("Plugins").join("Il2CppAssemblyFixerPlugin.dll")
        );
        assert!(installed_path.is_file());
        assert!(!plugins_dir
            .join("Il2CppAssemblyFixerPlugin.dll.disabled")
            .exists());
        assert!(plugins_dir.join("fixer_config.json").is_file());
        assert!(root.join("UserLibs").join("dnlib.dll").is_file());
        assert!(root.join("UserLibs").join("Mono.Cecil.dll").is_file());
        assert!(is_plugin_installed(&root, "Il2CppAssemblyFixerPlugin.dll"));
        assert!(is_framework_lib_installed(&root, "dnlib.dll"));
        assert!(is_framework_lib_installed(&root, "Mono.Cecil.dll"));

        std::fs::remove_dir_all(&root).ok();
    }
}
