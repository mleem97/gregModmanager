namespace GregModmanager.Services.GameAdapters;

public sealed record GameAdapterCapabilities(
	bool SupportsProfiles,
	bool SupportsLocalMods,
	bool SupportsWorkshop,
	bool SupportsLaunch,
	bool SupportsSaves,
	IReadOnlySet<string> DeploymentMethods);

public sealed record GameInstallation(
	string AdapterId,
	string RootPath,
	uint? SteamAppId,
	string? ExecutablePath,
	string? Version,
	bool IsValid);

public sealed record GamePathSet(
	string Root,
	string Mods,
	string Plugins,
	string UserLibraries,
	string Config,
	string Saves,
	string Workshop,
	string? Executable);

public sealed record GameCompatibilityResult(
	bool Compatible,
	string Summary,
	IReadOnlyList<string> Reasons);

public sealed record GameFileInstallRequest(
	string SourcePath,
	string RelativeTargetPath,
	string PackageId);

public sealed record GameInstallPlan(
	string AdapterId,
	string GameRoot,
	IReadOnlyList<GameFileInstallRequest> Files,
	IReadOnlyList<string> Warnings);

public sealed record GameOperationPlan(
	string AdapterId,
	string Operation,
	string GameRoot,
	string? ExecutablePath,
	IReadOnlyList<string> Arguments,
	string? Reason);

/// <summary>Game-specific paths, capabilities and planning rules. It does not mutate the game.</summary>
public interface IGameAdapter
{
	string Id { get; }
	string DisplayName { get; }
	uint SteamAppId { get; }
	GameAdapterCapabilities Capabilities { get; }
	GameInstallation? Detect(string? candidateRoot = null);
	GamePathSet GetPaths(string gameRoot);
	GameCompatibilityResult CheckCompatibility(string gameRoot);
	GameInstallPlan PlanInstall(string gameRoot, IEnumerable<GameFileInstallRequest> files);
	GameOperationPlan PlanUninstall(string gameRoot, IEnumerable<string> ownedRelativePaths);
	GameOperationPlan PlanLaunch(string gameRoot, IReadOnlyList<string>? arguments = null);
}
