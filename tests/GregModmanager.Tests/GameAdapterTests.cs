using GregModmanager.Services.GameAdapters;

namespace GregModmanager.Tests;

public sealed class GameAdapterTests : IDisposable
{
	private readonly string _root = Path.Combine(Path.GetTempPath(), "GregModmanager_GameAdapter_" + Guid.NewGuid().ToString("N"));

	public void Dispose()
	{
		if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true);
	}

	[Fact]
	public void DetectsDataCenterLayoutAndResolvesPortablePaths()
	{
		Directory.CreateDirectory(Path.Combine(_root, "Data Center_Data", "StreamingAssets"));
		var adapter = new DataCenterGameAdapter();

		var installation = adapter.Detect(_root);
		var paths = adapter.GetPaths(_root);

		Assert.NotNull(installation);
		Assert.Equal("datacenter", installation!.AdapterId);
		Assert.Equal(4170200u, installation.SteamAppId);
		Assert.Equal(Path.Combine(_root, "Data Center_Data", "StreamingAssets", "Mods"), paths.Workshop);
	}

	[Fact]
	public void PlanInstallRejectsPathTraversal()
	{
		var adapter = new DataCenterGameAdapter();

		Assert.Throws<InvalidDataException>(() => adapter.PlanInstall(_root, new[]
		{
			new GameFileInstallRequest("source.dll", "../Plugins/evil.dll", "test")
		}));
	}

	[Fact]
	public void RegistryDetectsRegisteredAdapter()
	{
		Directory.CreateDirectory(Path.Combine(_root, "Mods"));
		var registry = new GameAdapterRegistry(new IGameAdapter[] { new DataCenterGameAdapter() });

		var result = registry.Detect(_root);

		Assert.NotNull(result);
		Assert.Equal("datacenter", result!.Value.Adapter.Id);
	}
}
