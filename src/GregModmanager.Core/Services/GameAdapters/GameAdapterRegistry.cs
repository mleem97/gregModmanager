namespace GregModmanager.Services.GameAdapters;

public sealed class GameAdapterRegistry
{
	private readonly IReadOnlyList<IGameAdapter> _adapters;

	public GameAdapterRegistry(IEnumerable<IGameAdapter> adapters)
	{
		_adapters = adapters.DistinctBy(adapter => adapter.Id, StringComparer.OrdinalIgnoreCase).ToList();
	}

	public IReadOnlyList<IGameAdapter> Adapters => _adapters;

	public (IGameAdapter Adapter, GameInstallation Installation)? Detect(string? candidateRoot = null)
	{
		foreach (var adapter in _adapters)
		{
			var installation = adapter.Detect(candidateRoot);
			if (installation is not null) return (adapter, installation);
		}
		return null;
	}
}
