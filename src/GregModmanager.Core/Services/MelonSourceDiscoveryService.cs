using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using System.Text.RegularExpressions;
using GregModmanager.Steam;

namespace GregModmanager.Services;

public enum MelonSourceKind { GameRoot, GregModmanager, StreamingAssets, SteamWorkshop }
public enum MelonContentKind { Mods, Plugins, UserLibs }

public sealed record MelonSourceLocation(
	string Path,
	MelonSourceKind Source,
	MelonContentKind Kind,
	int Priority,
	ulong? PublishedFileId = null,
	string? DisplayName = null);

/// <summary>Discovers the same external MelonLoader source locations used by SteamModfix.</summary>
public sealed class MelonSourceDiscoveryService
{
	private static readonly Regex SteamLibraryRegex = new("\\\"path\\\"\\s*\\\"([^\\\"]+)\\\"", RegexOptions.Compiled);

	public IReadOnlyList<MelonSourceLocation> Discover(string gameRoot, uint appId = 0)
	{
		if (string.IsNullOrWhiteSpace(gameRoot) || !Directory.Exists(gameRoot)) return Array.Empty<MelonSourceLocation>();
		if (appId == 0) appId = SteamConstants.DataCenterAppId;
		var results = new List<MelonSourceLocation>();
		AddStandard(results, gameRoot, MelonSourceKind.GameRoot, 0, null);
		AddGregSources(results);
		AddStreamingAssets(results, gameRoot);
		AddWorkshopSources(results, appId, gameRoot);

		return results
			.Where(x => Directory.Exists(x.Path))
			.GroupBy(x => $"{Canonical(x.Path)}|{x.Kind}", StringComparer.OrdinalIgnoreCase)
			.Select(g => g.OrderBy(x => x.Priority).First())
			.OrderBy(x => x.Priority)
			.ThenBy(x => x.Path, StringComparer.OrdinalIgnoreCase)
			.ToArray();
	}

	private static void AddStandard(List<MelonSourceLocation> results, string root, MelonSourceKind source, int priority, ulong? id)
	{
		Add(results, root, "Mods", source, MelonContentKind.Mods, priority, id);
		Add(results, root, "Plugins", source, MelonContentKind.Plugins, priority, id);
		Add(results, root, "UserLibs", source, MelonContentKind.UserLibs, priority, id);
	}

	private static void AddStreamingAssets(List<MelonSourceLocation> results, string gameRoot)
	{
		foreach (var dataDir in SafeDirectories(gameRoot, "*_Data"))
		{
			var streaming = Path.Combine(dataDir, "StreamingAssets");
			AddStandard(results, streaming, MelonSourceKind.StreamingAssets, 20, null);
			AddStandard(results, Path.Combine(streaming, "MelonLoader"), MelonSourceKind.StreamingAssets, 21, null);
		}
	}

	private static void AddGregSources(List<MelonSourceLocation> results)
	{
		var raw = Environment.GetEnvironmentVariable("GREGMODMANAGER_SOURCES");
		if (string.IsNullOrWhiteSpace(raw)) return;
		foreach (var source in raw.Split(Path.PathSeparator, StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries))
			AddStandard(results, source, MelonSourceKind.GregModmanager, 10, null);
	}

	private static void AddWorkshopSources(List<MelonSourceLocation> results, uint appId, string gameRoot)
	{
		foreach (var library in FindSteamLibraries(gameRoot))
		{
			var root = Path.Combine(library, "steamapps", "workshop", "content", appId.ToString());
			foreach (var item in SafeDirectories(root, "*"))
			{
				if (!ulong.TryParse(Path.GetFileName(item), out var id)) continue;
				AddStandard(results, item, MelonSourceKind.SteamWorkshop, 30, id);
				AddStandard(results, Path.Combine(item, "MelonLoader"), MelonSourceKind.SteamWorkshop, 31, id);
				ClassifyRootDlls(results, item, id);
			}
		}
	}

	private static void ClassifyRootDlls(List<MelonSourceLocation> results, string item, ulong id)
	{
		foreach (var file in SafeFiles(item, "*.dll"))
		{
			var kind = Inspect(file);
			Add(results, item, MelonSourceKind.SteamWorkshop, 32, id, kind, Path.GetFileName(file));
		}
	}

	private static MelonContentKind Inspect(string path)
	{
		try
		{
			using var stream = File.OpenRead(path);
			using var pe = new PEReader(stream);
			if (!pe.HasMetadata) return MelonContentKind.UserLibs;
			var reader = pe.GetMetadataReader();
			foreach (var handle in reader.CustomAttributes)
			{
				var attr = reader.GetCustomAttribute(handle);
				var name = GetAttributeName(reader, attr.Constructor);
				if (name.Contains("MelonInfoAttribute", StringComparison.OrdinalIgnoreCase))
					return MelonContentKind.Mods;
			}
		}
		catch { }
		return MelonContentKind.UserLibs;
	}

	private static string GetAttributeName(MetadataReader reader, EntityHandle constructor)
	{
		return constructor.Kind switch
		{
			HandleKind.MemberReference => reader.GetString(reader.GetMemberReference((MemberReferenceHandle)constructor).Name),
			HandleKind.MethodDefinition => reader.GetString(reader.GetMethodDefinition((MethodDefinitionHandle)constructor).Name),
			_ => string.Empty,
		};
	}

	private static void Add(List<MelonSourceLocation> results, string root, string child, MelonSourceKind source, MelonContentKind kind, int priority, ulong? id)
		=> Add(results, Path.Combine(root, child), source, priority, id, kind, null);

	private static void Add(List<MelonSourceLocation> results, string path, MelonSourceKind source, int priority, ulong? id, MelonContentKind kind, string? displayName)
	{
		if (Directory.Exists(path)) results.Add(new(path, source, kind, priority, id, displayName));
	}

	private static IEnumerable<string> FindSteamLibraries(string gameRoot)
	{
		var candidates = new HashSet<string>(StringComparer.OrdinalIgnoreCase);
		// A normal Steam game path is <library>/steamapps/common/<game>.
		// The library root is therefore three levels above the game directory.
		var gameLibrary = Path.GetFullPath(Path.Combine(gameRoot, "..", "..", ".."));
		if (Directory.Exists(Path.Combine(gameLibrary, "steamapps"))) candidates.Add(gameLibrary);
		var env = Environment.GetEnvironmentVariable("STEAM_LIBRARY_PATHS");
		if (!string.IsNullOrWhiteSpace(env)) foreach (var path in env.Split(Path.PathSeparator, StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries)) candidates.Add(path);
		foreach (var file in new[]
		{
			Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".local/share/Steam/steamapps/libraryfolders.vdf"),
			Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".steam/steam/steamapps/libraryfolders.vdf"),
			Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".steam/root/steamapps/libraryfolders.vdf")
		})
		{
			if (!File.Exists(file)) continue;
			foreach (Match match in SteamLibraryRegex.Matches(File.ReadAllText(file))) candidates.Add(match.Groups[1].Value.Replace("\\\\", "/"));
		}
		return candidates.Where(Directory.Exists);
	}

	private static IEnumerable<string> SafeDirectories(string root, string pattern)
	{
		try { return Directory.Exists(root) ? Directory.EnumerateDirectories(root, pattern, SearchOption.TopDirectoryOnly).ToArray() : Array.Empty<string>(); }
		catch { return Array.Empty<string>(); }
	}

	private static IEnumerable<string> SafeFiles(string root, string pattern)
	{
		try { return Directory.Exists(root) ? Directory.EnumerateFiles(root, pattern, SearchOption.TopDirectoryOnly).ToArray() : Array.Empty<string>(); }
		catch { return Array.Empty<string>(); }
	}

	private static string Canonical(string path)
	{
		try { return Path.GetFullPath(path).TrimEnd(Path.DirectorySeparatorChar); } catch { return path; }
	}
}
