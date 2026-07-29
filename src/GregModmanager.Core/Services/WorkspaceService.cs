using System.Text.Json;
using Steamworks;
using GregModmanager.Localization;
using GregModmanager.Models;
using GregModmanager.Steam;

namespace GregModmanager.Services;

public sealed class WorkspaceService
{
	public const string CustomWorkspacePathKey = "CustomWorkspacePath";

	private static readonly JsonSerializerOptions JsonOptions = AppJsonContext.SharedOptions;

	private static readonly string LegacyFallbackPath =
		Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), "DataCenterWS");

	private readonly SteamWorkshopService? _steam;
	private string? _cachedWorkspaceRoot;

	/// <summary>GUI: resolves workspace via Custom > Game > Fallback. Headless: use parameterless ctor.</summary>
	public WorkspaceService(SteamWorkshopService steam)
	{
		_steam = steam;
	}

	/// <summary>Headless CLI — no Steam injection.</summary>
	public WorkspaceService()
	{
	}

	/// <summary>
	/// Resolution order: 1) User-configured custom path (Settings), 2) <c>&lt;GameRoot&gt;/workshop</c> via Steam, 3) <c>%USERPROFILE%/DataCenterWS</c>.
	/// </summary>
	public string WorkspaceRoot
	{
		get
		{
			if (_cachedWorkspaceRoot != null)
			{
				return _cachedWorkspaceRoot;
			}

			var custom = S.Preferences.GetString(CustomWorkspacePathKey, "");
			if (!string.IsNullOrWhiteSpace(custom) && Directory.Exists(custom))
			{
				_cachedWorkspaceRoot = custom;
				return _cachedWorkspaceRoot;
			}

			_steam?.EnsureInitialized(null);
			var fromGame = TryGetGameWorkshopDirectory();
			_cachedWorkspaceRoot = !string.IsNullOrEmpty(fromGame)
				? fromGame!
				: LegacyFallbackPath;
			return _cachedWorkspaceRoot;
		}
	}

	/// <summary>Clears cached root so the next access re-resolves (call after changing the custom path preference).</summary>
	public void InvalidateCache() => _cachedWorkspaceRoot = null;

	/// <summary>Moves all project folders from the legacy <c>DataCenterWS</c> location into the current <see cref="WorkspaceRoot"/>. Returns number of folders moved.</summary>
	public int MigrateLegacyProjects()
	{
		var target = WorkspaceRoot;
		if (string.Equals(Path.GetFullPath(LegacyFallbackPath), Path.GetFullPath(target), StringComparison.OrdinalIgnoreCase))
			return 0;

		if (!Directory.Exists(LegacyFallbackPath))
			return 0;

		var moved = 0;
		foreach (var dir in Directory.EnumerateDirectories(LegacyFallbackPath))
		{
			var name = Path.GetFileName(dir);
			if (name.StartsWith(".", StringComparison.Ordinal)) continue;

			var metaFile = Path.Combine(dir, "metadata.json");
			var contentDir = Path.Combine(dir, "content");
			if (!File.Exists(metaFile) && !Directory.Exists(contentDir)) continue;

			var dest = Path.Combine(target, name);
			if (Directory.Exists(dest)) continue;

			try
			{
				Directory.CreateDirectory(target);
				Directory.Move(dir, dest);
				moved++;
			}
			catch
			{
				// skip folders that can't be moved (locked, permissions, etc.)
			}
		}

		if (moved > 0)
		{
			try
			{
				var remaining = Directory.EnumerateFileSystemEntries(LegacyFallbackPath).Any();
				if (!remaining)
					Directory.Delete(LegacyFallbackPath, false);
			}
			catch
			{
				// legacy folder still in use or not empty — leave it
			}
		}

		return moved;
	}

	private static string? TryGetGameWorkshopDirectory()
	{
		if (!SteamClient.IsValid)
		{
			return null;
		}

		try
		{
			var install = SteamApps.AppInstallDir(SteamConstants.DataCenterAppId);
			if (string.IsNullOrWhiteSpace(install))
			{
				return null;
			}

			return Path.Combine(install.TrimEnd(Path.DirectorySeparatorChar, Path.AltDirectorySeparatorChar), "workshop");
		}
		catch
		{
			return null;
		}
	}

	public string TemplatesDirectory => Path.Combine(WorkspaceRoot, ".templates");

	public void EnsureWorkspaceStructure()
	{
		Directory.CreateDirectory(WorkspaceRoot);
		Directory.CreateDirectory(TemplatesDirectory);

		var sample = Path.Combine(TemplatesDirectory, "metadata.sample.json");
		if (!File.Exists(sample))
		{
			var meta = new WorkshopMetadata
			{
				Title = "My Mod",
				Description = "Description",
				Visibility = "Public",
				PreviewImageRelativePath = "preview.png",
			};
			File.WriteAllText(sample, JsonSerializer.Serialize(meta, AppJsonContext.Default.WorkshopMetadata));
		}
	}

	public IReadOnlyList<WorkshopProject> ScanProjects()
	{
		EnsureWorkspaceStructure();
		if (!Directory.Exists(WorkspaceRoot))
		{
			return Array.Empty<WorkshopProject>();
		}

		var list = new List<WorkshopProject>();
		foreach (var dir in Directory.EnumerateDirectories(WorkspaceRoot))
		{
			var name = Path.GetFileName(dir);
			if (name.StartsWith(".", StringComparison.Ordinal))
			{
				continue;
			}

			var content = Path.Combine(dir, "content");
			var metadataPath = Path.Combine(dir, "metadata.json");
			var hasContent = Directory.Exists(content);
			list.Add(new WorkshopProject
			{
				Name = name,
				RootPath = dir,
				ContentPath = content,
				MetadataPath = metadataPath,
				IsValidLayout = hasContent,
			});
		}

		return list.OrderBy(p => p.Name, StringComparer.OrdinalIgnoreCase).ToList();
	}

	/// <summary>Finds the local project that owns an existing Steam Workshop item.</summary>
	public WorkshopProject? FindProjectByPublishedFileId(ulong publishedFileId)
	{
		if (publishedFileId == 0) return null;

		return ScanProjects().FirstOrDefault(project =>
		{
			var metadata = LoadMetadata(project.RootPath);
			return metadata.PublishedFileId == publishedFileId;
		});
	}

	private static readonly string[] PreviewExtensions = [".png", ".jpg", ".jpeg", ".gif", ".webp"];

	public WorkshopMetadata LoadMetadata(string projectRoot)
	{
		projectRoot = Path.GetFullPath(projectRoot);
		var path = Path.Combine(projectRoot, "metadata.json");
		if (!File.Exists(path))
		{
			return new WorkshopMetadata();
		}

		WorkshopMetadata meta;
		try
		{
			var json = File.ReadAllText(path);
			meta = JsonSerializer.Deserialize(json, AppJsonContext.Default.WorkshopMetadata) ?? new WorkshopMetadata();
		}
		catch
		{
			meta = new WorkshopMetadata();
		}
		meta.Title ??= string.Empty;
		meta.Version ??= "1.0.0";
		meta.Description ??= string.Empty;
		meta.Visibility ??= "Public";
		meta.PreviewImageRelativePath ??= "preview.png";
		meta.Tags ??= new List<string>();
		meta.NativeConfigProfile ??= "decoration";
		meta.AdditionalPreviews ??= new List<string>();
		meta.WorkshopDependencyIds ??= new List<ulong>();

		AutoDetectPreviewImage(projectRoot, meta);

		return meta;
	}

	/// <summary>
	/// If the configured preview image doesn't exist, scan for common formats.
	/// </summary>
	private static void AutoDetectPreviewImage(string projectRoot, WorkshopMetadata meta)
	{
		var configured = Path.Combine(projectRoot, meta.PreviewImageRelativePath);
		if (File.Exists(configured)) return;

		foreach (var ext in PreviewExtensions)
		{
			var candidate = Path.Combine(projectRoot, $"preview{ext}");
			if (File.Exists(candidate))
			{
				meta.PreviewImageRelativePath = $"preview{ext}";
				return;
			}
		}
	}

	/// <summary>Creates <c>content/</c>, <c>metadata.json</c>, and template readmes. <paramref name="folderName"/> is sanitized for a single folder segment.</summary>
	public string CreateTemplateProject(string folderName, WorkshopTemplateKind kind)
	{
		EnsureWorkspaceStructure();
		var dirName = SanitizeFolderName(folderName);
		if (string.IsNullOrEmpty(dirName))
		{
			throw new ArgumentException("Project folder name is empty or invalid.");
		}

		var root = Path.Combine(WorkspaceRoot, dirName);
		if (Directory.Exists(root))
		{
			throw new InvalidOperationException($"A project folder named \"{dirName}\" already exists.");
		}

		var content = Path.Combine(root, "content");
		Directory.CreateDirectory(content);

		switch (kind)
		{
			case WorkshopTemplateKind.VanillaObjectDecoration:
				CreateVanillaObjectDecorationTemplate(content);
				break;
			case WorkshopTemplateKind.ModdedMelonLoader:
				CreateMelonLoaderTemplate(content);
				CreateSourceTemplates(root, dirName, isGreg: false);
				break;
			case WorkshopTemplateKind.ModdedgregCoreModFramework:
				CreategregCoreModFrameworkTemplate(content);
				CreateSourceTemplates(root, dirName, isGreg: true);
				break;
			case WorkshopTemplateKind.UxmlUiOverride:
				CreateUxmlTemplate(content, root, dirName);
				break;
			case WorkshopTemplateKind.Standalone3DModel:
				CreateStandaloneAssetTemplate(content, "model");
				break;
			case WorkshopTemplateKind.StandaloneTexture:
				CreateStandaloneAssetTemplate(content, "texture");
				break;
			case WorkshopTemplateKind.StandaloneAudio:
				CreateStandaloneAssetTemplate(content, "audio");
				break;
			default:
				throw new ArgumentOutOfRangeException(nameof(kind), kind, null);
		}

		var meta = BuildMetadataForTemplate(dirName, kind);
		SaveMetadata(root, meta);
		CreateProjectGitignore(root);
		return root;
	}

	private static void CreateProjectGitignore(string root)
	{
		const string gitignore = """
			bin/
			obj/
			*.user
			.vs/
			.vscode/
			""";
		File.WriteAllText(Path.Combine(root, ".gitignore"), gitignore.Trim());
	}

	private static void CreateSourceTemplates(string root, string projectName, bool isGreg)
	{
		var srcDir = Path.Combine(root, "src");
		Directory.CreateDirectory(srcDir);

		string csproj;
		string mainCs;

		if (isGreg)
		{
			csproj = $$"""
				<Project Sdk="Microsoft.NET.Sdk">
				  <PropertyGroup>
				    <TargetFramework>net6.0</TargetFramework>
				    <AssemblyName>{{projectName}}</AssemblyName>
				    <RootNamespace>{{projectName}}</RootNamespace>
				    <LangVersion>10.0</LangVersion>
				  </PropertyGroup>

				  <ItemGroup>
				    <!-- Reference gregCore.dll from game Mods folder or local path -->
				    <Reference Include="gregCore">
				      <HintPath>C:\Program Files (x86)\Steam\steamapps\common\Data Center\Mods\gregCore.dll</HintPath>
				      <Private>false</Private>
				    </Reference>
				    <Reference Include="MelonLoader">
				      <HintPath>C:\Program Files (x86)\Steam\steamapps\common\Data Center\MelonLoader\MelonLoader.dll</HintPath>
				      <Private>false</Private>
				    </Reference>
				    <Reference Include="UnityEngine.CoreModule">
				      <HintPath>C:\Program Files (x86)\Steam\steamapps\common\Data Center\Data Center_Data\Managed\UnityEngine.CoreModule.dll</HintPath>
				      <Private>false</Private>
				    </Reference>
				  </ItemGroup>
				</Project>
				""";

			mainCs = $$"""
				using MelonLoader;
				using UnityEngine;
				using greg.Core;
				using greg.Sdk.Services;

				[assembly: MelonInfo(typeof({{projectName}}.Main), "{{projectName}}", "1.0.0", "Author")]
				[assembly: MelonGame("Waseku", "Data Center")]

				namespace {{projectName}};

				public class Main : greg.Core.Plugins.gregModBase
				{
				    public override void OnInitializeMod()
				    {
				        MelonLogger.Msg("{{projectName}} initialized via gregCore!");
				        GregModRegistry.Register("{{projectName}}", "1.0.0");
				    }

				    public override void OnUpdateMod()
				    {
				        if (Input.GetKeyDown(KeyCode.F5))
				        {
				            MelonLogger.Msg("F5 pressed in {{projectName}}");
				        }
				    }
				}
				""";
		}
		else
		{
			csproj = $$"""
				<Project Sdk="Microsoft.NET.Sdk">
				  <PropertyGroup>
				    <TargetFramework>net6.0</TargetFramework>
				    <AssemblyName>{{projectName}}</AssemblyName>
				    <RootNamespace>{{projectName}}</RootNamespace>
				    <LangVersion>10.0</LangVersion>
				  </PropertyGroup>

				  <ItemGroup>
				    <Reference Include="MelonLoader">
				      <HintPath>C:\Program Files (x86)\Steam\steamapps\common\Data Center\MelonLoader\MelonLoader.dll</HintPath>
				      <Private>false</Private>
				    </Reference>
				    <Reference Include="UnityEngine.CoreModule">
				      <HintPath>C:\Program Files (x86)\Steam\steamapps\common\Data Center\Data Center_Data\Managed\UnityEngine.CoreModule.dll</HintPath>
				      <Private>false</Private>
				    </Reference>
				  </ItemGroup>
				</Project>
				""";

			mainCs = $$"""
				using MelonLoader;
				using UnityEngine;

				[assembly: MelonInfo(typeof({{projectName}}.Main), "{{projectName}}", "1.0.0", "Author")]
				[assembly: MelonGame("Waseku", "Data Center")]

				namespace {{projectName}};

				public class Main : MelonMod
				{
				    public override void OnInitializeMelon()
				    {
				        MelonLogger.Msg("{{projectName}} (Vanilla) initialized!");
				    }

				    public override void OnUpdate()
				    {
				        if (Input.GetKeyDown(KeyCode.F5))
				        {
				            MelonLogger.Msg("F5 pressed in {{projectName}}");
				        }
				    }
				}
				""";
		}

		File.WriteAllText(Path.Combine(srcDir, $"{projectName}.csproj"), csproj.Trim());
		File.WriteAllText(Path.Combine(srcDir, "Main.cs"), mainCs.Trim());
	}

	private static WorkshopMetadata BuildMetadataForTemplate(string title, WorkshopTemplateKind kind)
	{
		var meta = new WorkshopMetadata
		{
			Title = title,
			PreviewImageRelativePath = "preview.png",
		};

		switch (kind)
		{
			case WorkshopTemplateKind.VanillaObjectDecoration:
				meta.Description =
					"[h1]Custom Items for Data Center[/h1]\n" +
					"Adds custom objects to Data Center using the native mod system.\n\n" +
					"[h2]Contents[/h2]\n" +
					"[list]\n[*] Custom shop items (purchasable in-game)\n[/list]\n\n" +
					"[h2]Installation[/h2]\n" +
					"Subscribe to this Workshop item — the game loads it automatically. No additional tools required.\n\n" +
					"[h2]Mod Format[/h2]\n" +
					"This mod uses the native Data Center mod system ([b]config.json[/b] + OBJ/PNG assets).\n\n" +
					"[hr][/hr]\n" +
					"[i]Replace this placeholder description with your own.[/i]";
				meta.Tags.AddRange(new[] { "vanilla", "object", "decoration" });
				meta.NativeConfigProfile = "decoration";
				meta.NeedsMelonLoader = false;
				meta.Needsgreg = false;
				meta.ModType = "PlacableObject";
				break;
			case WorkshopTemplateKind.ModdedMelonLoader:
				meta.Description =
					"[h1]MelonLoader Mod[/h1]\n" +
					"A MelonLoader mod for Data Center.\n\n" +
					"[h2]Features[/h2]\n" +
					"[list]\n[*] Feature 1\n[*] Feature 2\n[/list]\n\n" +
					"[h2]Requirements[/h2]\n" +
					"[list]\n[*] [url=https://melonwiki.xyz]MelonLoader[/url]\n[/list]\n\n" +
					"[h2]Installation[/h2]\n" +
					"[list]\n[*] Install MelonLoader for Data Center\n[*] Subscribe to this Workshop item\n[*] Restart the game\n[/list]\n\n" +
					"[hr][/hr]\n" +
					"[i]Replace this placeholder description with your own.[/i]";
				meta.Tags.AddRange(new[] { "modded", "melonloader" });
				meta.NativeConfigProfile = "code";
				meta.NeedsMelonLoader = true;
				meta.Needsgreg = false;
				meta.ModType = "DataCenterMod";
				break;
			case WorkshopTemplateKind.ModdedgregCoreModFramework:
				meta.Description =
					"[h1]gregCoreModFramework Plugin[/h1]\n" +
					"A gregCoreModFramework plugin for Data Center.\n\n" +
					"[h2]Features[/h2]\n" +
					"[list]\n[*] Feature 1\n[*] Feature 2\n[/list]\n\n" +
					"[h2]Requirements[/h2]\n" +
					"[list]\n[*] [url=https://melonwiki.xyz]MelonLoader[/url]\n[*] [url=https://gregframework.eu]gregCoreModFramework[/url]\n[/list]\n\n" +
					"[h2]Installation[/h2]\n" +
					"[list]\n[*] Install MelonLoader for Data Center\n[*] Install gregCoreModFramework\n[*] Subscribe to this Workshop item\n[*] Restart the game\n[/list]\n\n" +
					"[hr][/hr]\n" +
					"[i]Replace this placeholder description with your own.[/i]";
				meta.Tags.AddRange(new[] { "modded", "greg", "gregCore-mod-framework" });
				meta.NativeConfigProfile = "code";
				meta.NeedsMelonLoader = true;
				meta.Needsgreg = true;
				meta.ModType = "DataCenterMod";
				break;
			case WorkshopTemplateKind.UxmlUiOverride:
				meta.Description = "[h1]UXML UI Override[/h1]\nReplaces game interfaces with custom UI Toolkit (UXML) designs.";
				meta.Tags.AddRange(new[] { "modded", "ui", "uxml" });
				meta.NativeConfigProfile = "code";
				meta.NeedsMelonLoader = true;
				meta.Needsgreg = true;
				meta.ModType = "DataCenterMod";
				break;
			case WorkshopTemplateKind.Standalone3DModel:
				meta.Description = "[h1]Standalone 3D Model[/h1]\nA high-quality 3D asset for use in other mods or game scenes.";
				meta.Tags.AddRange(new[] { "asset", "model", "visual" });
				meta.NativeConfigProfile = "decoration";
				meta.ModType = "PlacableObject";
				break;
			case WorkshopTemplateKind.StandaloneTexture:
				meta.Description = "[h1]Standalone Texture[/h1]\nA high-resolution texture or material pack.";
				meta.Tags.AddRange(new[] { "asset", "texture", "material" });
				meta.NativeConfigProfile = "decoration";
				meta.ModType = "PlacableObject";
				break;
			case WorkshopTemplateKind.StandaloneAudio:
				meta.Description = "[h1]Standalone Audio[/h1]\nA music track or sound effect collection.";
				meta.Tags.AddRange(new[] { "asset", "audio", "radio" });
				meta.NativeConfigProfile = "decoration";
				meta.ModType = "PlacableObject";
				break;
			default:
				throw new ArgumentOutOfRangeException(nameof(kind), kind, "Unknown workshop template kind.");
		}

		return meta;
	}

	private static void CreateVanillaObjectDecorationTemplate(string contentRoot)
	{
		Directory.CreateDirectory(contentRoot);

		const string sampleConfig = """
			{
			  "modName": "My Workshop Mod",
			  "shopItems": [
			    {
			      "itemName": "Custom Server",
			      "price": 500,
			      "xpToUnlock": 0,
			      "sizeInU": 2,
			      "mass": 5.0,
			      "modelScale": 1.0,
			      "colliderSize": [0.5, 0.5, 0.5],
			      "colliderCenter": [0.0, 0.0, 0.0],
			      "modelFile": "server.obj",
			      "textureFile": "server.png",
			      "iconFile": "server_icon.png",
			      "objectType": 0
			    }
			  ],
			  "staticItems": [],
			  "dlls": []
			}
			""";

		File.WriteAllText(Path.Combine(contentRoot, "config.json"), sampleConfig.Trim());

		File.WriteAllText(
			Path.Combine(contentRoot, "README.txt"),
			"""
			Native Data Center Mod (Vanilla)

			This folder uses the game's built-in mod system.

			Structure:
			  config.json        — mod definition (items, assets, optional DLLs)
			  *.obj              — 3D models (Wavefront OBJ)
			  *.png              — textures and shop icons

			config.json fields:
			  modName            — display name of your mod
			  shopItems[]        — purchasable objects for the in-game shop
			  staticItems[]      — statically placed decorations
			  dlls[]             — optional plugin DLLs (untested/experimental)

			shopItems fields:
			  itemName, price, xpToUnlock, sizeInU, mass, modelScale,
			  colliderSize [x,y,z], colliderCenter [x,y,z],
			  modelFile (.obj), textureFile (.png), iconFile (.png),
			  objectType (0 = passive object)

			staticItems fields:
			  itemName, modelFile, textureFile, modelScale,
			  colliderSize [x,y,z], colliderCenter [x,y,z],
			  position [x,y,z], rotation [x,y,z]

			Place your .obj models, .png textures, and icon files in this folder
			alongside config.json.

			Do not ship game binaries — only your own assets.
			""".Trim());
	}

	/// <summary>
	/// Standard layout under <c>content/</c> for modded Workshop items: mirrors <c>{GameRoot}/Mods</c>, <c>{GameRoot}/Plugins</c>,
	/// and <c>{GameRoot}/greg/...</c> under a single <c>ModFramework/</c> umbrella. Steam uploads this tree as workshop content;
	/// the game delivers it under <c>.../StreamingAssets/mods/workshop_&lt;id&gt;/WorkshopUploadContent/</c>. Players typically use
	/// directory junctions from the game root to those subfolders (see README_GameMirror.txt).
	/// </summary>
	private static void CreateModdedGameMirrorLayout(string contentRoot)
	{
		Directory.CreateDirectory(contentRoot);

		File.WriteAllText(
			Path.Combine(contentRoot, "README_GameMirror.txt"),
			"""
			Workshop content layout (modded)

			This folder is uploaded to Steam as-is. After subscribe, the game places it under:
			  Data Center_Data/StreamingAssets/mods/workshop_<id>/WorkshopUploadContent/

			Recommended mapping next to the game executable ({GameRoot}):
			  Mods/           -> MelonLoader MelonMods  (same relative path under WorkshopUploadContent)
			  Plugins/        -> MelonLoader Plugins
			  ModFramework/     -> umbrella for greg and other framework files; use ModFramework/greg/Plugins for greg.Plugin DLLs

			MelonLoader does not read arbitrary paths from Loader.cfg — link {GameRoot}/Mods (or a subfolder) to the
			workshop folder if you want Workshop DLLs loaded like local mods.

			Do not ship game binaries or MelonLoader; only your own assemblies and assets.
			""".Trim());

		var mods = Path.Combine(contentRoot, "Mods");
		Directory.CreateDirectory(mods);
		File.WriteAllText(
			Path.Combine(mods, "README.txt"),
			"""
			MelonLoader mods ({GameRoot}/Mods)

			Place MelonLoader MelonMod DLLs here. Relative path matches the game Mods folder when you use a junction
			from this folder to {GameRoot}/Mods (or a subfolder under Mods).

			Do not redistribute the game or MelonLoader; only your mod assemblies and your own assets.
			""".Trim());

		var plugins = Path.Combine(contentRoot, "Plugins");
		Directory.CreateDirectory(plugins);
		File.WriteAllText(
			Path.Combine(plugins, "README.txt"),
			"""
			MelonLoader plugins ({GameRoot}/Plugins)

			Place MelonLoader plugin DLLs (MelonPlugin) here. Same relative path as next to the game executable.

			Do not redistribute the game or MelonLoader; only your plugin assemblies and your own assets.
			""".Trim());

		var modFw = Path.Combine(contentRoot, "ModFramework");
		Directory.CreateDirectory(modFw);
		File.WriteAllText(
			Path.Combine(modFw, "README.txt"),
			"""
			ModFramework (extra files for greg / tooling)

			Use this tree for everything that belongs to the mod framework but is not a MelonMod/MelonPlugin:
			  - greg plugin DLLs -> greg/Plugins/
			  - optional configs, catalogs, or assets your greg plugins expect

			At the game root, greg expects greg/Plugins next to the executable. From Workshop delivery, create a junction:
			  {GameRoot}/greg  ->  .../WorkshopUploadContent/ModFramework/greg
			so that greg/Plugins resolves to ModFramework/greg/Plugins in the workshop folder.

			Do not ship game binaries; only your own files.
			""".Trim());

		var gregPlugins = Path.Combine(modFw, "greg", "Plugins");
		Directory.CreateDirectory(gregPlugins);
		File.WriteAllText(
			Path.Combine(gregPlugins, "README.txt"),
			"""
			greg plugins ({GameRoot}/greg/Plugins)

			Place greg plugin DLLs (greg.Plugin.*) here. With a junction from {GameRoot}/greg to ModFramework/greg in the
			workshop content folder, this path matches the live game layout.

			Do not ship game binaries; only your plugin DLLs and allowed content.
			""".Trim());
	}

	private static void CreateMelonLoaderTemplate(string contentRoot) => CreateModdedGameMirrorLayout(contentRoot);

	private static void CreategregCoreModFrameworkTemplate(string contentRoot) => CreateModdedGameMirrorLayout(contentRoot);

	/// <summary>Copies all files from <paramref name="sourceDir"/> into <paramref name="destDir"/> (recursive).</summary>
	public static void CopyDirectoryRecursive(string sourceDir, string destDir)
	{
		if (!Directory.Exists(sourceDir))
		{
			throw new DirectoryNotFoundException(sourceDir);
		}

		Directory.CreateDirectory(destDir);
foreach (var file in Directory.EnumerateFiles(sourceDir, "*", new EnumerationOptions { RecurseSubdirectories = true, IgnoreInaccessible = true }))
		{
			var rel = Path.GetRelativePath(sourceDir, file);
			var target = Path.Combine(destDir, rel);
			var targetDir = Path.GetDirectoryName(target);
			if (!string.IsNullOrEmpty(targetDir))
			{
				Directory.CreateDirectory(targetDir);
			}

			File.Copy(file, target, overwrite: true);
		}
	}

	/// <summary>Total size of <c>content/</c> and largest direct children.</summary>
	public ContentStats GetContentStats(string projectRoot)
	{
		var content = Path.Combine(projectRoot, "content");
		if (!Directory.Exists(content))
		{
			return new ContentStats { Exists = false };
		}

		var total = GetDirectorySizeRecursive(content);
		var entries = new List<ContentFolderSize>();

		foreach (var file in Directory.EnumerateFiles(content))
		{
			try
			{
				var len = new FileInfo(file).Length;
				entries.Add(new ContentFolderSize(Path.GetFileName(file), len));
			}
			catch
			{
				// ignored
			}
		}

		foreach (var dir in Directory.EnumerateDirectories(content))
		{
			var name = Path.GetFileName(dir) + "/";
			entries.Add(new ContentFolderSize(name, GetDirectorySizeRecursive(dir)));
		}

		var top = entries.OrderByDescending(e => e.Bytes).Take(16).ToList();
		return new ContentStats
		{
			Exists = true,
			TotalBytes = total,
			TopEntries = top,
		};
	}

	private static long GetDirectorySizeRecursive(string dir)
	{
		long n = 0;
		try
		{
			foreach (var f in Directory.EnumerateFiles(dir, "*", SearchOption.AllDirectories))
			{
				try
				{
					n += new FileInfo(f).Length;
				}
				catch
				{
					// ignored
				}
			}
		}
		catch
		{
			// ignored
		}

		return n;
	}

	public static string FormatBytes(long bytes)
	{
		if (bytes < 0)
		{
			bytes = 0;
		}

		string[] suf = { "B", "KB", "MB", "GB", "TB" };
		double v = bytes;
		var i = 0;
		while (v >= 1024 && i < suf.Length - 1)
		{
			v /= 1024;
			i++;
		}

		return $"{v:0.##} {suf[i]}";
	}

	/// <summary>Single path segment under <see cref="WorkspaceRoot"/>; strips invalid characters.</summary>
	public static string SanitizeFolderName(string raw)
	{
		if (string.IsNullOrWhiteSpace(raw))
		{
			return string.Empty;
		}

		var trimmed = raw.Trim();
		var invalid = Path.GetInvalidFileNameChars();
		var chars = trimmed.Select(c => invalid.Contains(c) ? '_' : c).ToArray();
		var s = new string(chars);
		while (s.Contains("..", StringComparison.Ordinal))
		{
			s = s.Replace("..", "_", StringComparison.Ordinal);
		}

		s = s.Trim('.', ' ');
		if (s.Length > 80)
		{
			s = s[..80].TrimEnd();
		}

		return s;
	}

	public static void SaveMetadata(string projectRoot, WorkshopMetadata metadata)
	{
		projectRoot = Path.GetFullPath(projectRoot);
		var title = metadata.Title ?? string.Empty;
		var description = metadata.Description ?? string.Empty;
		if (title.Length > SteamConstants.MaxTitleLength)
		{
			throw new InvalidOperationException($"Title exceeds {SteamConstants.MaxTitleLength} characters.");
		}

		if (description.Length > SteamConstants.MaxDescriptionLength)
		{
			throw new InvalidOperationException($"Description exceeds {SteamConstants.MaxDescriptionLength} characters.");
		}

		var path = Path.Combine(projectRoot, "metadata.json");
		File.WriteAllText(path, JsonSerializer.Serialize(metadata, AppJsonContext.Default.WorkshopMetadata));
	}

	private static void CreateUxmlTemplate(string contentRoot, string root, string dirName)
	{
		CreateModdedGameMirrorLayout(contentRoot);

		// Source template for UXML registration
		var srcDir = Path.Combine(root, "src");
		Directory.CreateDirectory(srcDir);

		string csproj = $$"""
			<Project Sdk="Microsoft.NET.Sdk">
			  <PropertyGroup>
			    <TargetFramework>net6.0</TargetFramework>
			    <AssemblyName>{{dirName}}</AssemblyName>
			    <RootNamespace>{{dirName}}</RootNamespace>
			    <LangVersion>10.0</LangVersion>
			  </PropertyGroup>

			  <ItemGroup>
			    <Reference Include="gregCore">
			      <HintPath>C:\Program Files (x86)\Steam\steamapps\common\Data Center\Mods\gregCore.dll</HintPath>
			      <Private>false</Private>
			    </Reference>
			    <Reference Include="MelonLoader">
			      <HintPath>C:\Program Files (x86)\Steam\steamapps\common\Data Center\MelonLoader\MelonLoader.dll</HintPath>
			      <Private>false</Private>
			    </Reference>
			    <Reference Include="UnityEngine.CoreModule">
			      <HintPath>C:\Program Files (x86)\Steam\steamapps\common\Data Center\Data Center_Data\Managed\UnityEngine.CoreModule.dll</HintPath>
			      <Private>false</Private>
			    </Reference>
			    <Reference Include="UnityEngine.UIElementsModule">
			      <HintPath>C:\Program Files (x86)\Steam\steamapps\common\Data Center\Data Center_Data\Managed\UnityEngine.UIElementsModule.dll</HintPath>
			      <Private>false</Private>
			    </Reference>
			  </ItemGroup>
			</Project>
			""";

		string mainCs = $$"""
			using MelonLoader;
			using UnityEngine;
			using UnityEngine.UIElements;
			using greg.Core;
			using greg.Sdk.Services;

			[assembly: MelonInfo(typeof({{dirName}}.Main), "{{dirName}}", "1.0.0", "Author")]
			[assembly: MelonGame("Waseku", "Data Center")]

			namespace {{dirName}};

			public class Main : greg.Core.Plugins.gregModBase
			{
			    public override void OnInitializeMod()
			    {
			        // Load your AssetBundle containing UXML/USS
			        // var bundle = AssetBundle.LoadFromFile(Path.Combine(AppContext.BaseDirectory, "Mods", "{{dirName}}.bundle"));
			        // var uxml = bundle.LoadAsset<VisualTreeAsset>("MyMenu.uxml");
			        
			        // Register override for MainMenu
			        // GregUxmlService.RegisterOverride("MainMenu", () => {
			        //    GregUxmlService.CreateInterface("CustomMenu", uxml);
			        // });

			        MelonLogger.Msg("{{dirName}} (UXML UI) initialized!");
			    }
			}
			""";

		File.WriteAllText(Path.Combine(srcDir, $"{dirName}.csproj"), csproj.Trim());
		File.WriteAllText(Path.Combine(srcDir, "Main.cs"), mainCs.Trim());

		// UXML/USS Assets folder in content
		var assets = Path.Combine(contentRoot, "Assets", "UI");
		Directory.CreateDirectory(assets);
		File.WriteAllText(Path.Combine(assets, "Sample.uxml"), "<ui:UXML xmlns:ui=\"UnityEngine.UIElements\" xmlns:uie=\"UnityEditor.UIElements\" editor-extension-mode=\"False\">\n  <ui:Label text=\"Hello from UXML!\" />\n</ui:UXML>");
	}

	private static void CreateStandaloneAssetTemplate(string contentRoot, string type)
	{
		Directory.CreateDirectory(contentRoot);
		var assetDir = Path.Combine(contentRoot, "Assets", type);
		Directory.CreateDirectory(assetDir);

		string readme = type switch
		{
			"model" => "Place your .obj or .fbx models here. Include .mtl and textures in the same folder.",
			"texture" => "Place your .png or .jpg textures here.",
			"audio" => "Place your .wav or .ogg audio files here.",
			_ => "Place your assets here."
		};

		File.WriteAllText(Path.Combine(assetDir, "README.txt"), readme);

		// ModStore metadata for Asset Store indexing
		var modstoreMeta = new AssetModMetadata
		{
			AssetType = type,
			Tags = new[] { "asset", type },
			IsStandalone = true
		};
		File.WriteAllText(Path.Combine(contentRoot, "modstore.meta.json"), JsonSerializer.Serialize(modstoreMeta, AppJsonContext.Default.AssetModMetadata));
	}
}


