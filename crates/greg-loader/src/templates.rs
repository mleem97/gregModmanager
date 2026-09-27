//! New-project scaffolding (`WorkspaceService.CreateTemplateProject` port).
//!
//! Creates `content/`, `metadata.json` data, `README.md`, `CHANGELOG.md` and
//! `.gitignore`. Code-mod sources stay C# (MelonLoader is .NET-only).

use std::path::{Path, PathBuf};

use greg_core::models::WorkshopMetadata;

use crate::content::{AssetModMetadata, WorkshopTemplateKind};
use crate::error::{io_err, Result};

/// Scaffolds a new project folder under `workspace_root`.
/// Returns the folder name and the prepared metadata (caller persists it).
pub fn scaffold(
    workspace_root: &Path,
    folder_name: &str,
    kind: WorkshopTemplateKind,
    title: &str,
) -> Result<(PathBuf, WorkshopMetadata)> {
    let dir_name = greg_core::util::sanitize_folder_name(folder_name);
    if dir_name.is_empty() {
        return Err(crate::error::LoaderError::Game(
            "Project folder name is empty or invalid.".into(),
        ));
    }
    let root = workspace_root.join(&dir_name);
    if root.exists() {
        return Err(crate::error::LoaderError::Game(format!(
            "A project folder named \"{dir_name}\" already exists."
        )));
    }
    let content = root.join("content");
    std::fs::create_dir_all(&content).map_err(io_err)?;

    match kind {
        WorkshopTemplateKind::VanillaObjectDecoration => vanilla_content(&content)?,
        WorkshopTemplateKind::ModdedMelonLoader => {
            modded_mirror_layout(&content)?;
            csharp_sources(&root, &dir_name, false)?;
        }
        WorkshopTemplateKind::ModdedGregCore => {
            modded_mirror_layout(&content)?;
            csharp_sources(&root, &dir_name, true)?;
        }
        WorkshopTemplateKind::UxmlUiOverride => uxml_content(&content, &root, &dir_name)?,
        WorkshopTemplateKind::Standalone3DModel => standalone_asset(&content, "model")?,
        WorkshopTemplateKind::StandaloneTexture => standalone_asset(&content, "texture")?,
        WorkshopTemplateKind::StandaloneAudio => standalone_asset(&content, "audio")?,
    }

    let meta = metadata_for_template(title, kind);
    write_project_docs(&root, title)?;
    write_gitignore(&root)?;
    Ok((root, meta))
}

fn write_file(path: PathBuf, content: &str) -> Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(io_err)?;
    }
    std::fs::write(path, content).map_err(io_err)?;
    Ok(())
}

fn metadata_for_template(title: &str, kind: WorkshopTemplateKind) -> WorkshopMetadata {
    let mut meta = WorkshopMetadata {
        title: title.to_string(),
        ..Default::default()
    };
    match kind {
        WorkshopTemplateKind::VanillaObjectDecoration => {
            meta.description = "[h1]Custom Items for Data Center[/h1]\nAdds custom objects to Data Center using the native mod system.".into();
            meta.tags = vec!["vanilla".into(), "object".into(), "decoration".into()];
            meta.native_config_profile = "decoration".into();
            meta.mod_type = "PlacableObject".into();
        }
        WorkshopTemplateKind::ModdedMelonLoader => {
            meta.description =
                "[h1]MelonLoader Mod[/h1]\nA MelonLoader mod for Data Center.".into();
            meta.tags = vec!["modded".into(), "melonloader".into()];
            meta.native_config_profile = "code".into();
            meta.needs_melon_loader = true;
            meta.mod_type = "DataCenterMod".into();
        }
        WorkshopTemplateKind::ModdedGregCore => {
            meta.description =
                "[h1]gregCoreModFramework Plugin[/h1]\nA gregCoreModFramework plugin for Data Center."
                    .into();
            meta.tags = vec![
                "modded".into(),
                "greg".into(),
                "gregCore-mod-framework".into(),
            ];
            meta.native_config_profile = "code".into();
            meta.needs_melon_loader = true;
            meta.needsgreg = true;
            meta.mod_type = "DataCenterMod".into();
        }
        WorkshopTemplateKind::UxmlUiOverride => {
            meta.description = "[h1]UXML UI Override[/h1]\nReplaces game interfaces with custom UI Toolkit (UXML) designs.".into();
            meta.tags = vec!["modded".into(), "ui".into(), "uxml".into()];
            meta.native_config_profile = "code".into();
            meta.needs_melon_loader = true;
            meta.needsgreg = true;
            meta.mod_type = "DataCenterMod".into();
        }
        WorkshopTemplateKind::Standalone3DModel => {
            meta.description =
                "[h1]Standalone 3D Model[/h1]\nA high-quality 3D asset for use in other mods."
                    .into();
            meta.tags = vec!["asset".into(), "model".into(), "visual".into()];
            meta.native_config_profile = "decoration".into();
            meta.mod_type = "PlacableObject".into();
        }
        WorkshopTemplateKind::StandaloneTexture => {
            meta.description =
                "[h1]Standalone Texture[/h1]\nA high-resolution texture or material pack.".into();
            meta.tags = vec!["asset".into(), "texture".into(), "material".into()];
            meta.native_config_profile = "decoration".into();
            meta.mod_type = "PlacableObject".into();
        }
        WorkshopTemplateKind::StandaloneAudio => {
            meta.description =
                "[h1]Standalone Audio[/h1]\nA music track or sound effect collection.".into();
            meta.tags = vec!["asset".into(), "audio".into(), "radio".into()];
            meta.native_config_profile = "decoration".into();
            meta.mod_type = "PlacableObject".into();
        }
    }
    meta
}

fn write_project_docs(root: &Path, title: &str) -> Result<()> {
    let readme = root.join("README.md");
    if !readme.is_file() {
        write_file(
            readme,
            &format!(
                "# {title}\n\nShort description of your mod.\n\n## Features\n\n- Feature 1\n- Feature 2\n\n## Installation\n\nSubscribe to this Workshop item — the game loads it automatically.\n\n## Requirements\n\n- Data Center\n"
            ),
        )?;
    }
    let changelog = root.join("CHANGELOG.md");
    if !changelog.is_file() {
        let today = crate::date_today();
        write_file(
            changelog,
            &format!(
                "# Changelog\n\nAll notable changes to this project will be documented in this file.\n\nThe format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),\nand this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).\n\n## [Unreleased]\n\n### Added\n\n- Planned changes go here.\n\n## [1.0.0] - {today}\n\n### Added\n\n- Initial release.\n"
            ),
        )?;
    }
    Ok(())
}

fn write_gitignore(root: &Path) -> Result<()> {
    write_file(
        root.join(".gitignore"),
        "bin/\nobj/\n*.user\n.vs/\n.vscode/\n",
    )
}

fn vanilla_content(content: &Path) -> Result<()> {
    write_file(
        content.join("config.json"),
        "{\n  \"modName\": \"My Workshop Mod\",\n  \"shopItems\": [\n    {\n      \"itemName\": \"Custom Server\",\n      \"price\": 500,\n      \"xpToUnlock\": 0,\n      \"sizeInU\": 2,\n      \"mass\": 5.0,\n      \"modelScale\": 1.0,\n      \"colliderSize\": [0.5, 0.5, 0.5],\n      \"colliderCenter\": [0.0, 0.0, 0.0],\n      \"modelFile\": \"server.obj\",\n      \"textureFile\": \"server.png\",\n      \"iconFile\": \"server_icon.png\",\n      \"objectType\": 0\n    }\n  ],\n  \"staticItems\": [],\n  \"dlls\": []\n}\n",
    )?;
    write_file(
        content.join("README.txt"),
        "Native Data Center Mod (Vanilla)\n\nThis folder uses the game's built-in mod system.\n\nStructure:\n  config.json        — mod definition (items, assets, optional DLLs)\n  *.obj              — 3D models (Wavefront OBJ)\n  *.png              — textures and shop icons\n\nPlace your .obj models, .png textures, and icon files in this folder\nalongside config.json.\n\nDo not ship game binaries — only your own assets.\n",
    )
}

/// Workshop layout mirroring `{GameRoot}/Mods`, `Plugins` and greg trees.
fn modded_mirror_layout(content: &Path) -> Result<()> {
    write_file(
        content.join("README_GameMirror.txt"),
        "Workshop content layout (modded)\n\nThis folder is uploaded to Steam as-is. After subscribe, the game places it under:\n  Data Center_Data/StreamingAssets/mods/workshop_<id>/WorkshopUploadContent/\n\nDo not ship game binaries or MelonLoader; only your own assemblies and assets.\n",
    )?;
    for (dir, text) in [
        (
            "Mods",
            "MelonLoader mods ({GameRoot}/Mods)\n\nPlace MelonLoader MelonMod DLLs here.\n",
        ),
        (
            "Plugins",
            "MelonLoader plugins ({GameRoot}/Plugins)\n\nPlace MelonLoader plugin DLLs (MelonPlugin) here.\n",
        ),
        (
            "ModFramework",
            "ModFramework (extra files for greg / tooling)\n\n- greg plugin DLLs -> greg/Plugins/\n",
        ),
        (
            "ModFramework/greg/Plugins",
            "greg plugins ({GameRoot}/greg/Plugins)\n\nPlace greg plugin DLLs (greg.Plugin.*) here.\n",
        ),
    ] {
        write_file(content.join(dir).join("README.txt"), text)?;
    }
    Ok(())
}

/// C# mod sources (game mods stay .NET 6 / MelonLoader).
fn csharp_sources(root: &Path, project_name: &str, is_greg: bool) -> Result<()> {
    let src = root.join("src");
    let (csproj, main_cs) = if is_greg {
        (
            format!(
                "<Project Sdk=\"Microsoft.NET.Sdk\">\n  <PropertyGroup>\n    <TargetFramework>net6.0</TargetFramework>\n    <AssemblyName>{project_name}</AssemblyName>\n    <RootNamespace>{project_name}</RootNamespace>\n    <LangVersion>10.0</LangVersion>\n  </PropertyGroup>\n</Project>\n"
            ),
            format!(
                "using MelonLoader;\nusing UnityEngine;\nusing greg.Core;\n\n[assembly: MelonInfo(typeof({project_name}.Main), \"{project_name}\", \"1.0.0\", \"Author\")]\n[assembly: MelonGame(\"Waseku\", \"Data Center\")]\n\nnamespace {project_name};\n\npublic class Main : greg.Core.Plugins.gregModBase\n{{\n    public override void OnInitializeMod()\n    {{\n        MelonLogger.Msg(\"{project_name} initialized via gregCore!\");\n    }}\n}}\n"
            ),
        )
    } else {
        (
            format!(
                "<Project Sdk=\"Microsoft.NET.Sdk\">\n  <PropertyGroup>\n    <TargetFramework>net6.0</TargetFramework>\n    <AssemblyName>{project_name}</AssemblyName>\n    <RootNamespace>{project_name}</RootNamespace>\n    <LangVersion>10.0</LangVersion>\n  </PropertyGroup>\n</Project>\n"
            ),
            format!(
                "using MelonLoader;\nusing UnityEngine;\n\n[assembly: MelonInfo(typeof({project_name}.Main), \"{project_name}\", \"1.0.0\", \"Author\")]\n[assembly: MelonGame(\"Waseku\", \"Data Center\")]\n\nnamespace {project_name};\n\npublic class Main : MelonMod\n{{\n    public override void OnInitializeMelon()\n    {{\n        MelonLogger.Msg(\"{project_name} (Vanilla) initialized!\");\n    }}\n}}\n"
            ),
        )
    };
    write_file(src.join(format!("{project_name}.csproj")), &csproj)?;
    write_file(src.join("Main.cs"), &main_cs)
}

fn uxml_content(content: &Path, root: &Path, dir_name: &str) -> Result<()> {
    modded_mirror_layout(content)?;
    let src = root.join("src");
    write_file(
        src.join(format!("{dir_name}.csproj")),
        &format!(
            "<Project Sdk=\"Microsoft.NET.Sdk\">\n  <PropertyGroup>\n    <TargetFramework>net6.0</TargetFramework>\n    <AssemblyName>{dir_name}</AssemblyName>\n    <RootNamespace>{dir_name}</RootNamespace>\n    <LangVersion>10.0</LangVersion>\n  </PropertyGroup>\n</Project>\n"
        ),
    )?;
    write_file(
        src.join("Main.cs"),
        &format!(
            "using MelonLoader;\nusing UnityEngine.UIElements;\nusing greg.Core;\n\n[assembly: MelonInfo(typeof({dir_name}.Main), \"{dir_name}\", \"1.0.0\", \"Author\")]\n[assembly: MelonGame(\"Waseku\", \"Data Center\")]\n\nnamespace {dir_name};\n\npublic class Main : greg.Core.Plugins.gregModBase\n{{\n    public override void OnInitializeMod()\n    {{\n        MelonLogger.Msg(\"{dir_name} (UXML UI) initialized!\");\n    }}\n}}\n"
        ),
    )?;
    write_file(
        content.join("Assets").join("UI").join("Sample.uxml"),
        "<ui:UXML xmlns:ui=\"UnityEngine.UIElements\" xmlns:uie=\"UnityEditor.UIElements\" editor-extension-mode=\"False\">\n  <ui:Label text=\"Hello from UXML!\" />\n</ui:UXML>",
    )
}

fn standalone_asset(content: &Path, asset_type: &str) -> Result<()> {
    let dir = content.join("Assets").join(asset_type);
    std::fs::create_dir_all(&dir).map_err(io_err)?;
    let readme = match asset_type {
        "model" => {
            "Place your .obj or .fbx models here. Include .mtl and textures in the same folder."
        }
        "texture" => "Place your .png or .jpg textures here.",
        "audio" => "Place your .wav or .ogg audio files here.",
        _ => "Place your assets here.",
    };
    write_file(dir.join("README.txt"), readme)?;
    let meta = AssetModMetadata {
        asset_type: asset_type.to_string(),
        tags: vec!["asset".to_string(), asset_type.to_string()],
        is_standalone: true,
    };
    let text = serde_json::to_string_pretty(&meta).map_err(crate::error::json_err)?;
    write_file(content.join("modstore.meta.json"), &text)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ws_root(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "greg-tpl-{name}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn scaffolds_vanilla() {
        let ws = ws_root("vanilla");
        let (root, meta) = scaffold(
            &ws,
            "my-mod",
            WorkshopTemplateKind::VanillaObjectDecoration,
            "My Mod",
        )
        .unwrap();
        assert!(root.join("content").join("config.json").is_file());
        assert!(root.join("README.md").is_file());
        assert!(root.join("CHANGELOG.md").is_file());
        assert!(root.join(".gitignore").is_file());
        assert_eq!(meta.mod_type, "PlacableObject");
        std::fs::remove_dir_all(&ws).ok();
    }

    #[test]
    fn scaffolds_melon_with_sources() {
        let ws = ws_root("melon");
        let (root, meta) = scaffold(
            &ws,
            "my-melon",
            WorkshopTemplateKind::ModdedMelonLoader,
            "My Melon",
        )
        .unwrap();
        assert!(root.join("src").join("Main.cs").is_file());
        assert!(!root.join("content").join("Mods").is_file());
        assert!(root.join("content").join("Mods").is_dir());
        assert!(meta.needs_melon_loader);
        std::fs::remove_dir_all(&ws).ok();
    }

    #[test]
    fn rejects_duplicates_and_bad_names() {
        let ws = ws_root("dup");
        scaffold(
            &ws,
            "dup-mod",
            WorkshopTemplateKind::VanillaObjectDecoration,
            "T",
        )
        .unwrap();
        assert!(scaffold(
            &ws,
            "dup-mod",
            WorkshopTemplateKind::VanillaObjectDecoration,
            "T"
        )
        .is_err());
        assert!(scaffold(
            &ws,
            "   ",
            WorkshopTemplateKind::VanillaObjectDecoration,
            "T"
        )
        .is_err());
        std::fs::remove_dir_all(&ws).ok();
    }
}
