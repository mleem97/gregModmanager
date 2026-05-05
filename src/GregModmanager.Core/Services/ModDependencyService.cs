using Steamworks;
using GregModmanager.Models;
using GregModmanager.Steam;

namespace GregModmanager.Services;

/// <summary>
/// Probes the local Data Center installation for MelonLoader, gregCoreModFramework,
/// Il2Cpp interop assemblies, and mod configuration directories.
/// All checks are read-only unless explicitly stated.
/// </summary>
public sealed class ModDependencyService
{
	private readonly SteamWorkshopService _steam;
	private string? _cachedGameRoot;

	public ModDependencyService(SteamWorkshopService steam)
	{
		_steam = steam;
	}

	/// <summary>Resolved game root via <see cref="SteamApps.AppInstallDir"/> or common default.</summary>
	public string? GameRoot
	{
		get
		{
			if (_cachedGameRoot != null)
			{
				return _cachedGameRoot;
			}

			_steam.EnsureInitialized(null);
			_cachedGameRoot = ResolveGameRoot();
			return _cachedGameRoot;
		}
	}

	public void InvalidateCache()
	{
		_cachedGameRoot = null;
	}

	public string MelonLoaderDir => Path.Combine(GameRoot ?? string.Empty, "MelonLoader");
	public string MelonLoaderNet6Dir => Path.Combine(MelonLoaderDir, "net6");
	public string Il2CppAssembliesDir => Path.Combine(MelonLoaderDir, "Il2CppAssemblies");
	public string ModsDir => Path.Combine(GameRoot ?? string.Empty, "Mods");
	public string gregPluginsDir => Path.Combine(GameRoot ?? string.Empty, "greg", "Plugins");
	public string ModCfgDir => Path.Combine(GameRoot ?? string.Empty, "UserData", "ModCfg");

	/// <summary>Run all dependency checks and return an ordered list of results.</summary>
	public IReadOnlyList<DependencyCheckResult> RunChecks()
	{
		var results = new List<DependencyCheckResult>();

		var root = GameRoot;
		if (string.IsNullOrEmpty(root) || !Directory.Exists(root))
		{
			results.Add(new DependencyCheckResult
			{
				Label = "Data Center",
				Status = DependencyStatus.Missing,
				Detail = "Spielordner nicht gefunden. Steam starten und Data Center installieren.",
				Path = root,
			});
			return results;
		}

		results.Add(new DependencyCheckResult
		{
			Label = "Data Center",
			Status = DependencyStatus.Ok,
			Detail = root,
			Path = root,
		});

		CheckMelonLoader(results);
		CheckIl2Cpp(results);
		CheckgregCoreModFramework(results);
		CheckgregPluginsDir(results);
		CheckModCfg(results);

		return results;
	}

	private void CheckMelonLoader(List<DependencyCheckResult> results)
	{
		var dll = Path.Combine(MelonLoaderNet6Dir, "MelonLoader.dll");
		if (File.Exists(dll))
		{
			results.Add(new DependencyCheckResult
			{
				Label = "MelonLoader",
				Status = DependencyStatus.Ok,
				Detail = MelonLoaderNet6Dir,
				Path = MelonLoaderNet6Dir,
			});
		}
		else
		{
			results.Add(new DependencyCheckResult
			{
				Label = "MelonLoader",
				Status = DependencyStatus.Missing,
				Detail = "MelonLoader nicht installiert. Bitte MelonLoader herunterladen und installieren.",
				Path = MelonLoaderDir,
			});
		}
	}

	private void CheckIl2Cpp(List<DependencyCheckResult> results)
	{
		var asmCs = Path.Combine(Il2CppAssembliesDir, "Assembly-CSharp.dll");
		if (File.Exists(asmCs))
		{
			results.Add(new DependencyCheckResult
			{
				Label = "Il2Cpp Interop",
				Status = DependencyStatus.Ok,
				Detail = Il2CppAssembliesDir,
				Path = Il2CppAssembliesDir,
			});
		}
		else if (Directory.Exists(MelonLoaderNet6Dir))
		{
			results.Add(new DependencyCheckResult
			{
				Label = "Il2Cpp Interop",
				Status = DependencyStatus.Warning,
				Detail = "Assembly-CSharp.dll fehlt. Spiel einmal mit MelonLoader starten, damit Il2Cpp-Assemblies generiert werden.",
				Path = Il2CppAssembliesDir,
			});
		}
		else
		{
			results.Add(new DependencyCheckResult
			{
				Label = "Il2Cpp Interop",
				Status = DependencyStatus.Missing,
				Detail = "Erst MelonLoader installieren, dann Spiel einmal starten.",
				Path = Il2CppAssembliesDir,
			});
		}
	}

	private void CheckgregCoreModFramework(List<DependencyCheckResult> results)
	{
		var gregDll = Path.Combine(ModsDir, "gregCoreModdingFramework.dll");
		if (File.Exists(gregDll))
		{
			results.Add(new DependencyCheckResult
			{
				Label = "gregCoreModFramework",
				Status = DependencyStatus.Ok,
				Detail = gregDll,
				Path = ModsDir,
			});
		}
		else
		{
			results.Add(new DependencyCheckResult
			{
				Label = "gregCoreModFramework",
				Status = DependencyStatus.Missing,
				Detail = "gregCoreModdingFramework.dll fehlt unter Mods/. Aus GitHub Release herunterladen oder selbst bauen.",
				Path = ModsDir,
			});
		}
	}

	private void CheckgregPluginsDir(List<DependencyCheckResult> results)
	{
		if (Directory.Exists(gregPluginsDir))
		{
			var count = Directory.GetFiles(gregPluginsDir, "*.dll").Length;
			results.Add(new DependencyCheckResult
			{
				Label = "greg Plugins",
				Status = count > 0 ? DependencyStatus.Ok : DependencyStatus.Warning,
				Detail = count > 0 ? $"{count} Plugin-DLL(s) unter greg/Plugins/" : "greg/Plugins/ existiert, aber enthält keine DLLs.",
				Path = gregPluginsDir,
			});
		}
		else
		{
			results.Add(new DependencyCheckResult
			{
				Label = "greg Plugins",
				Status = DependencyStatus.Warning,
				Detail = "greg/Plugins/ existiert noch nicht (wird beim ersten Plugin-Install angelegt).",
				Path = Path.Combine(GameRoot ?? string.Empty, "greg"),
			});
		}
	}

	private void CheckModCfg(List<DependencyCheckResult> results)
	{
		if (Directory.Exists(ModCfgDir))
		{
			results.Add(new DependencyCheckResult
			{
				Label = "Mod-Konfiguration",
				Status = DependencyStatus.Ok,
				Detail = ModCfgDir,
				Path = ModCfgDir,
			});
		}
		else
		{
			results.Add(new DependencyCheckResult
			{
				Label = "Mod-Konfiguration",
				Status = DependencyStatus.Warning,
				Detail = "UserData/ModCfg/ fehlt (wird beim ersten Mod-Start angelegt).",
				Path = Path.Combine(GameRoot ?? string.Empty, "UserData"),
			});
		}
	}

	/// <summary>Create the ModCfg directory if it doesn't exist.</summary>
	public void EnsureModCfgDirectory()
	{
		if (!string.IsNullOrEmpty(GameRoot))
		{
			Directory.CreateDirectory(ModCfgDir);
		}
	}

	/// <summary>Create the greg/Plugins directory if it doesn't exist.</summary>
	public void EnsuregregPluginsDirectory()
	{
		if (!string.IsNullOrEmpty(GameRoot))
		{
			Directory.CreateDirectory(gregPluginsDir);
		}
	}

	private static string? ResolveGameRoot()
	{
		if (SteamClient.IsValid)
		{
			try
			{
				var install = SteamApps.AppInstallDir(SteamConstants.DataCenterAppId);
				if (!string.IsNullOrWhiteSpace(install))
				{
					var trimmed = install.TrimEnd(Path.DirectorySeparatorChar, Path.AltDirectorySeparatorChar);
					if (Directory.Exists(trimmed))
					{
						return trimmed;
					}
				}
			}
			catch
			{
				// fall through
			}
		}

		var defaultPath = Path.Combine(
			Environment.GetFolderPath(Environment.SpecialFolder.ProgramFilesX86),
			"Steam", "steamapps", "common", "Data Center");
		return Directory.Exists(defaultPath) ? defaultPath : null;
	}
}

