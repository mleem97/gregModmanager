using System.Collections.Generic;
using System.Runtime.InteropServices;
#if WINDOWS
using Microsoft.Win32;
#endif

namespace GregModmanager.Steam;

/// <summary>
/// Loads <c>steam_api64.dll</c> before Facepunch/Steamworks runs so the process uses the same
/// native binary as Data Center: prefer <c>{GameRoot}/Data Center_Data/Plugins/x86_64/</c>, then the
/// copy shipped next to this executable.
/// </summary>
public static class SteamApiNativeLoader
{
	private const string SteamFolderName = "Steam";
	private const string SteamAppsFolderName = "steamapps";
	private const string CommonFolderName = "common";
	private const string GameFolderName = "Data Center";
	private const string PluginsFolderName = "Plugins";
	private const string ArchFolderName = "x86_64";

	private static readonly string DllFileName = RuntimeInformation.IsOSPlatform(OSPlatform.Windows) ? "steam_api64.dll" : "libsteam_api64.so";
	private static readonly string DllFileNameFallback = RuntimeInformation.IsOSPlatform(OSPlatform.Windows) ? "steam_api64.dll" : "libsteam_api.so";
	private const string UnityDataFolderName = "Data Center_Data";
	private static IntPtr _module;
	public static bool IsLoaded => _module != IntPtr.Zero;

	/// <summary>
	/// Idempotent: loads the first existing candidate. Returns true if a module handle was obtained.
	/// </summary>
	private static string? _customGameRoot;

	public static void SetGameRoot(string? gameRoot)
	{
		_customGameRoot = gameRoot;
	}

	private static string? ResolveAutoGameRoot()
	{
		var candidates = new[]
		{
			Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ProgramFilesX86), SteamFolderName, SteamAppsFolderName, CommonFolderName, GameFolderName),
			Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ProgramFiles), SteamFolderName, SteamAppsFolderName, CommonFolderName, GameFolderName),
		};

		return candidates.FirstOrDefault(Directory.Exists);
	}

	public static string? GetGameRoot()
	{
		if (!string.IsNullOrEmpty(_customGameRoot))
		{
			return _customGameRoot;
		}

		var envRoot = Environment.GetEnvironmentVariable("DATA_CENTER_GAME_DIR")?.Trim();
		if (!string.IsNullOrEmpty(envRoot) && Directory.Exists(envRoot))
		{
			return envRoot;
		}

		var autoRoot = ResolveAutoGameRoot();
		if (!string.IsNullOrEmpty(autoRoot))
		{
			return autoRoot;
		}

		return EnumerateHeuristicGameRoots().FirstOrDefault(Directory.Exists);
	}

	public static bool TryPreload()
	{
		if (_module != IntPtr.Zero)
		{
			return true;
		}

		if (string.IsNullOrEmpty(_customGameRoot))
		{
			_customGameRoot = ResolveAutoGameRoot();
		}

		foreach (var path in EnumerateCandidatePaths())
		{
			if (string.IsNullOrEmpty(path) || !File.Exists(path))
			{
				continue;
			}

			try
			{
				if (NativeLibrary.TryLoad(path, out _module))
				{
					return true;
				}
			}
			catch
			{
				// try next
			}
		}

		try
		{
			if (NativeLibrary.TryLoad(DllFileName, out _module))
				return true;
		}
		catch { }

		// Fallback: try alternative library name (e.g. libsteam_api.so if libsteam_api64.so not found)
		if (!string.IsNullOrEmpty(DllFileNameFallback) && DllFileNameFallback != DllFileName)
		{
			try
			{
				if (NativeLibrary.TryLoad(DllFileNameFallback, out _module))
					return true;
			}
			catch { }
		}

		return false;
	}

	private static readonly List<string> _attemptedPaths = new();

	public static IReadOnlyList<string> GetAttemptedPaths() => _attemptedPaths;

	private static IEnumerable<string> EnumerateCandidatePaths()
	{
		_attemptedPaths.Clear();

		if (!string.IsNullOrEmpty(_customGameRoot))
		{
			var path1 = Path.Combine(_customGameRoot, UnityDataFolderName, PluginsFolderName, ArchFolderName, DllFileName);
			var path2 = Path.Combine(_customGameRoot, PluginsFolderName, ArchFolderName, DllFileName);
			_attemptedPaths.Add(path1);
			_attemptedPaths.Add(path2);
			yield return path1;
			yield return path2;
			// Also try fallback
			if (DllFileNameFallback != DllFileName)
			{
				var fb1 = Path.Combine(_customGameRoot, UnityDataFolderName, PluginsFolderName, ArchFolderName, DllFileNameFallback);
				var fb2 = Path.Combine(_customGameRoot, PluginsFolderName, ArchFolderName, DllFileNameFallback);
				_attemptedPaths.Add(fb1);
				_attemptedPaths.Add(fb2);
				yield return fb1;
				yield return fb2;
			}
		}

		var steamCommonPath = Path.Combine(
			Environment.GetFolderPath(Environment.SpecialFolder.ProgramFilesX86),
			SteamFolderName, SteamAppsFolderName, CommonFolderName, GameFolderName);
		if (Directory.Exists(steamCommonPath))
		{
			var path = Path.Combine(steamCommonPath, UnityDataFolderName, PluginsFolderName, ArchFolderName, DllFileName);
			_attemptedPaths.Add(path);
			yield return path;
		}

		var envRoot = Environment.GetEnvironmentVariable("DATA_CENTER_GAME_DIR")?.Trim();
		if (!string.IsNullOrEmpty(envRoot))
		{
			var nativeSubPath = Path.Combine(UnityDataFolderName, PluginsFolderName, ArchFolderName, DllFileName);
			yield return Path.Combine(envRoot, nativeSubPath);
			if (DllFileNameFallback != DllFileName)
			{
				var fbPath = Path.Combine(UnityDataFolderName, PluginsFolderName, ArchFolderName, DllFileNameFallback);
				yield return Path.Combine(envRoot, fbPath);
			}
		}

		foreach (var path in EnumerateWalkingUpFrom(AppContext.BaseDirectory))
		{
			yield return path;
		}

		foreach (var gameRoot in EnumerateHeuristicGameRoots())
		{
			yield return Path.Combine(gameRoot, UnityDataFolderName, PluginsFolderName, ArchFolderName, DllFileName);
			if (DllFileNameFallback != DllFileName)
				yield return Path.Combine(gameRoot, UnityDataFolderName, PluginsFolderName, ArchFolderName, DllFileNameFallback);
		}

		var baseDir = AppContext.BaseDirectory;
		if (!string.IsNullOrEmpty(baseDir))
		{
			yield return Path.Combine(baseDir, DllFileName);
			if (DllFileNameFallback != DllFileName)
				yield return Path.Combine(baseDir, DllFileNameFallback);
		}
	}

	private static IEnumerable<string> EnumerateWalkingUpFrom(string startDir)
	{
		string? dir;
		try
		{
			dir = Path.GetFullPath(startDir.Trim());
		}
		catch
		{
			yield break;
		}

		for (var i = 0; i < 8 && !string.IsNullOrEmpty(dir); i++)
		{
			yield return Path.Combine(dir, UnityDataFolderName, PluginsFolderName, ArchFolderName, DllFileName);
			try
			{
				dir = Path.GetDirectoryName(dir);
			}
			catch
			{
				yield break;
			}
		}
	}

	private static IEnumerable<string> EnumerateHeuristicGameRoots()
	{
		var seen = new HashSet<string>(StringComparer.OrdinalIgnoreCase);

		void Add(string? path)
		{
			if (string.IsNullOrWhiteSpace(path))
			{
				return;
			}

			try
			{
				var full = Path.GetFullPath(path.Trim());
				if (Directory.Exists(full))
				{
					seen.Add(full);
				}
			}
			catch
			{
				// ignored
			}
		}

		if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
		{
#if WINDOWS
			try
			{
				using var key = Registry.LocalMachine.OpenSubKey($@"SOFTWARE\WOW6432Node\Valve\{SteamFolderName}");
				var installPath = key?.GetValue("InstallPath") as string;
				if (!string.IsNullOrEmpty(installPath))
				{
					Add(Path.Combine(installPath, SteamAppsFolderName, CommonFolderName, GameFolderName));
				}
			}
			catch
			{
				// ignored
			}
#endif

			Add(Path.Combine(
				Environment.GetFolderPath(Environment.SpecialFolder.ProgramFilesX86),
				SteamFolderName, SteamAppsFolderName, CommonFolderName, GameFolderName));
		}
		else if (RuntimeInformation.IsOSPlatform(OSPlatform.Linux))
		{
			var home = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);

			// Standard Steam library paths
			var steamRoots = new[]
			{
				Path.Combine(home, ".local", "share", SteamFolderName),
				Path.Combine(home, ".steam", SteamFolderName.ToLowerInvariant()),
				Path.Combine(home, ".steam", "root"),
			};

			foreach (var steamRoot in steamRoots)
			{
				// Direct game install
				Add(Path.Combine(steamRoot, SteamAppsFolderName, CommonFolderName, GameFolderName));

				// Proton prefix (compatdata) — the game runs inside a Wine/Proton prefix
				var compatData = Path.Combine(steamRoot, SteamAppsFolderName, "compatdata", "4170200", "pfx");
				Add(Path.Combine(compatData, "drive_c", "Program Files (x86)", SteamFolderName, SteamAppsFolderName, CommonFolderName, GameFolderName));
				Add(Path.Combine(compatData, "drive_c", "Program Files", SteamFolderName, SteamAppsFolderName, CommonFolderName, GameFolderName));
			}

			// Parse libraryfolders.vdf for additional Steam library paths
			foreach (var libPath in EnumerateSteamLibraryFolders())
			{
				Add(Path.Combine(libPath, SteamAppsFolderName, CommonFolderName, GameFolderName));

				// Proton prefix in additional libraries
				var compatData = Path.Combine(libPath, SteamAppsFolderName, "compatdata", "4170200", "pfx");
				Add(Path.Combine(compatData, "drive_c", "Program Files (x86)", SteamFolderName, SteamAppsFolderName, CommonFolderName, GameFolderName));
				Add(Path.Combine(compatData, "drive_c", "Program Files", SteamFolderName, SteamAppsFolderName, CommonFolderName, GameFolderName));
			}
		}

		foreach (var root in seen)
		{
			yield return root;
		}
	}

	private static IEnumerable<string> EnumerateSteamLibraryFolders()
	{
		if (!RuntimeInformation.IsOSPlatform(OSPlatform.Linux))
		{
			yield break;
		}

		var home = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);
		var vdfPaths = new[]
		{
			Path.Combine(home, ".local", "share", SteamFolderName, SteamAppsFolderName, "libraryfolders.vdf"),
			Path.Combine(home, ".steam", SteamFolderName.ToLowerInvariant(), SteamAppsFolderName, "libraryfolders.vdf"),
			Path.Combine(home, ".steam", "root", SteamAppsFolderName, "libraryfolders.vdf"),
		};

		foreach (var vdfPath in vdfPaths)
		{
			if (!File.Exists(vdfPath))
			{
				continue;
			}

			foreach (var path in ParseLibraryFoldersVdf(vdfPath))
			{
				yield return path;
			}

			yield break;
		}
	}

	/// <summary>
	/// Parses Valve's libraryfolders.vdf to find additional Steam library paths.
	/// Format: "path"		"/mnt/games/SteamLibrary"
	/// </summary>
	private static IEnumerable<string> ParseLibraryFoldersVdf(string vdfPath)
	{
		string[] lines;
		try
		{
			lines = File.ReadAllLines(vdfPath);
		}
		catch
		{
			yield break;
		}

		foreach (var line in lines)
		{
			// Look for lines like: "path"		"/some/path"
			var trimmed = line.Trim();
			if (!trimmed.StartsWith("\"path\"", StringComparison.OrdinalIgnoreCase))
			{
				continue;
			}

			var parts = trimmed.Split('\t', StringSplitOptions.RemoveEmptyEntries);
			if (parts.Length < 2)
			{
				continue;
			}

			var value = parts[^1].Trim().Trim('"');
			if (!string.IsNullOrEmpty(value) && Directory.Exists(value))
			{
				yield return value;
			}
		}
	}
}

