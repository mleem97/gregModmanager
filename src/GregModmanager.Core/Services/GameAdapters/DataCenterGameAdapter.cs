using System.Runtime.InteropServices;

namespace GregModmanager.Services.GameAdapters;

/// <summary>Game adapter for the currently supported Data Center Steam title.</summary>
public sealed class DataCenterGameAdapter : IGameAdapter
{
	private static readonly IReadOnlySet<string> DeploymentMethods =
		new HashSet<string>(StringComparer.OrdinalIgnoreCase) { "copy", "hardlink", "symlink" };

	public string Id => "datacenter";
	public string DisplayName => "Data Center";
	public uint SteamAppId => GregModmanager.Steam.SteamConstants.DataCenterAppId;
	public GameAdapterCapabilities Capabilities { get; } = new(
		SupportsProfiles: false,
		SupportsLocalMods: true,
		SupportsWorkshop: true,
		SupportsLaunch: false,
		SupportsSaves: false,
		DeploymentMethods);

	public GameInstallation? Detect(string? candidateRoot = null)
	{
		var root = string.IsNullOrWhiteSpace(candidateRoot) ? AppSettings.TryDetectGameRoot() : candidateRoot;
		if (string.IsNullOrWhiteSpace(root) || !Directory.Exists(root)) return null;

		string fullRoot;
		try { fullRoot = Path.GetFullPath(root); }
		catch (Exception ex) when (ex is ArgumentException or IOException or NotSupportedException) { return null; }

		var executable = FindExecutable(fullRoot);
		var hasUnityData = Directory.Exists(Path.Combine(fullRoot, "Data Center_Data")) ||
			Directory.Exists(Path.Combine(fullRoot, "DataCenter_Data"));
		var hasKnownContent = Directory.Exists(Path.Combine(fullRoot, "MelonLoader")) ||
			Directory.Exists(Path.Combine(fullRoot, "Mods")) ||
			Directory.Exists(Path.Combine(fullRoot, "Plugins"));
		if (executable is null && !hasUnityData && !hasKnownContent) return null;

		return new(Id, fullRoot, SteamAppId, executable, null, true);
	}

	public GamePathSet GetPaths(string gameRoot)
	{
		var root = NormalizeRoot(gameRoot);
		var unityData = Directory.Exists(Path.Combine(root, "DataCenter_Data")) ? "DataCenter_Data" : "Data Center_Data";
		return new(root,
			Path.Combine(root, "Mods"),
			Path.Combine(root, "Plugins"),
			Path.Combine(root, "Plugins", "Dependencies"),
			Path.Combine(root, "UserData", "ModCfg"),
			Path.Combine(root, "UserData", "Saves"),
			Path.Combine(root, unityData, "StreamingAssets", "Mods"),
			FindExecutable(root));
	}

	public GameCompatibilityResult CheckCompatibility(string gameRoot)
	{
		var installation = Detect(gameRoot);
		if (installation is null)
			return new(false, "Data-Center-Installation nicht erkannt.", new[] { "Der Pfad enthält keine bekannte Data-Center-Struktur." });

		var reasons = new List<string>();
		if (!Capabilities.SupportsLaunch)
			reasons.Add("Direkter Start ist für diesen Adapter noch nicht implementiert; Steam bleibt der Startpunkt.");
		return new(reasons.Count == 0, reasons.Count == 0 ? "Installation kompatibel." : "Installation erkannt, aber mit Einschränkungen.", reasons);
	}

	public GameInstallPlan PlanInstall(string gameRoot, IEnumerable<GameFileInstallRequest> files)
	{
		var root = NormalizeRoot(gameRoot);
		var normalized = files.Select(file => file with
		{
			SourcePath = Path.GetFullPath(file.SourcePath),
			RelativeTargetPath = NormalizeRelativePath(file.RelativeTargetPath)
		}).ToList();
		var warnings = normalized.Where(file => !File.Exists(file.SourcePath))
			.Select(file => $"Quelle fehlt: {file.SourcePath}").ToList();
		return new(Id, root, normalized, warnings);
	}

	public GameOperationPlan PlanUninstall(string gameRoot, IEnumerable<string> ownedRelativePaths)
		=> new(Id, "uninstall", NormalizeRoot(gameRoot), null,
			ownedRelativePaths.Select(NormalizeRelativePath).ToArray(), null);

	public GameOperationPlan PlanLaunch(string gameRoot, IReadOnlyList<string>? arguments = null)
	{
		var root = NormalizeRoot(gameRoot);
		var executable = FindExecutable(root);
		return new(Id, "launch", root, executable, arguments ?? Array.Empty<string>(),
			executable is null ? "Kein Data-Center-Spielprogramm gefunden." : null);
	}

	private static string NormalizeRoot(string gameRoot)
	{
		if (string.IsNullOrWhiteSpace(gameRoot)) throw new ArgumentException("GameRoot darf nicht leer sein.", nameof(gameRoot));
		return Path.GetFullPath(gameRoot.Trim());
	}

	private static string NormalizeRelativePath(string path)
	{
		if (string.IsNullOrWhiteSpace(path)) throw new ArgumentException("Zielpfad darf nicht leer sein.", nameof(path));
		var normalized = path.Replace('\\', '/').TrimStart('/');
		if (normalized.Split('/').Any(part => part is ".." or "." || part.Contains('\0')))
			throw new InvalidDataException($"Unsicherer relativer Zielpfad: {path}");
		return normalized;
	}

	private static string? FindExecutable(string root)
	{
		var candidates = RuntimeInformation.IsOSPlatform(OSPlatform.Windows)
			? new[] { Path.Combine(root, "Data Center.exe"), Path.Combine(root, "DataCenter.exe") }
			: RuntimeInformation.IsOSPlatform(OSPlatform.OSX)
				? new[] { Path.Combine(root, "Data Center.app", "Contents", "MacOS", "Data Center"), Path.Combine(root, "DataCenter.app", "Contents", "MacOS", "DataCenter") }
				: new[] { Path.Combine(root, "Data Center.x86_64"), Path.Combine(root, "DataCenter.x86_64") };
		return candidates.FirstOrDefault(File.Exists);
	}
}
