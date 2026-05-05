using System.Collections.Generic;
using System.Runtime.InteropServices;
using Microsoft.Win32;

namespace GregModmanager.Steam;

/// <summary>
/// Loads <c>steam_api64.dll</c> before Facepunch/Steamworks runs so the process uses the same
/// native binary as Data Center: prefer <c>{GameRoot}/Data Center_Data/Plugins/x86_64/</c>, then the
/// copy shipped next to this executable.
/// </summary>
public static class SteamApiNativeLoader
{
	private static readonly string DllFileName = RuntimeInformation.IsOSPlatform(OSPlatform.Windows) ? "steam_api64.dll" : "libsteam_api.so";
	private const string UnityDataFolderName = "Data Center_Data";
	private static IntPtr _module;

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
			Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ProgramFilesX86), "Steam", "steamapps", "common", "Data Center"),
			Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ProgramFiles), "Steam", "steamapps", "common", "Data Center"),
		};

		foreach (var candidate in candidates)
		{
			if (Directory.Exists(candidate))
			{
				return candidate;
			}
		}

		return null;
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

		foreach (var gameRoot in EnumerateHeuristicGameRoots())
		{
			if (Directory.Exists(gameRoot))
			{
				return gameRoot;
			}
		}

		return null;
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
			return NativeLibrary.TryLoad(DllFileName, out _module);
		}
		catch
		{
			return false;
		}
	}

	private static readonly List<string> _attemptedPaths = new();

	public static IReadOnlyList<string> GetAttemptedPaths() => _attemptedPaths;

	private static IEnumerable<string> EnumerateCandidatePaths()
	{
		_attemptedPaths.Clear();

		if (!string.IsNullOrEmpty(_customGameRoot))
		{
			var path1 = Path.Combine(_customGameRoot, UnityDataFolderName, "Plugins", "x86_64", DllFileName);
			var path2 = Path.Combine(_customGameRoot, "Plugins", "x86_64", DllFileName);
			_attemptedPaths.Add(path1);
			_attemptedPaths.Add(path2);
			yield return path1;
			yield return path2;
		}

		var steamCommonPath = Path.Combine(
			Environment.GetFolderPath(Environment.SpecialFolder.ProgramFilesX86),
			"Steam", "steamapps", "common", "Data Center");
		if (Directory.Exists(steamCommonPath))
		{
			var path = Path.Combine(steamCommonPath, UnityDataFolderName, "Plugins", "x86_64", DllFileName);
			_attemptedPaths.Add(path);
			yield return path;
		}

		var envRoot = Environment.GetEnvironmentVariable("DATA_CENTER_GAME_DIR")?.Trim();
		if (!string.IsNullOrEmpty(envRoot))
		{
			var nativeSubPath = RuntimeInformation.IsOSPlatform(OSPlatform.Windows) 
				? Path.Combine(UnityDataFolderName, "Plugins", "x86_64", DllFileName)
				: Path.Combine(UnityDataFolderName, "Plugins", "x86_64", DllFileName); // Unity Linux paths usually match
			
			yield return Path.Combine(envRoot, nativeSubPath);
		}

		foreach (var path in EnumerateWalkingUpFrom(AppContext.BaseDirectory))
		{
			yield return path;
		}

		foreach (var gameRoot in EnumerateHeuristicGameRoots())
		{
			yield return Path.Combine(gameRoot, UnityDataFolderName, "Plugins", "x86_64", DllFileName);
		}

		var baseDir = AppContext.BaseDirectory;
		if (!string.IsNullOrEmpty(baseDir))
		{
			yield return Path.Combine(baseDir, DllFileName);
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
			yield return Path.Combine(dir, UnityDataFolderName, "Plugins", "x86_64", DllFileName);
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
			try
			{
				using var key = Registry.LocalMachine.OpenSubKey(@"SOFTWARE\WOW6432Node\Valve\Steam");
				var installPath = key?.GetValue("InstallPath") as string;
				if (!string.IsNullOrEmpty(installPath))
				{
					Add(Path.Combine(installPath, "steamapps", "common", "Data Center"));
				}
			}
			catch
			{
				// ignored
			}

			Add(Path.Combine(
				Environment.GetFolderPath(Environment.SpecialFolder.ProgramFilesX86),
				"Steam", "steamapps", "common", "Data Center"));
		}
		else if (RuntimeInformation.IsOSPlatform(OSPlatform.Linux))
		{
			var home = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);
			Add(Path.Combine(home, ".local/share/Steam/steamapps/common/Data Center"));
			Add(Path.Combine(home, ".steam/steam/steamapps/common/Data Center"));
		}

		foreach (var root in seen)
		{
			yield return root;
		}
	}
}

