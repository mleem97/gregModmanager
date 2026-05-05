namespace GregModmanager.Services;

/// <summary>
/// Central registry for plugin distribution channels (stable, beta, …).
/// Injected as singleton; channels are registered at startup.
/// </summary>
public sealed class gregPluginChannelRegistry
{
	private readonly Dictionary<string, IgregPluginChannelSource> _sources = new(StringComparer.OrdinalIgnoreCase);

	public void Register(IgregPluginChannelSource source)
	{
		_sources[source.ChannelName] = source;
	}

	public IgregPluginChannelSource? GetSource(string channelName)
	{
		_sources.TryGetValue(channelName, out var source);
		return source;
	}

	public IReadOnlyList<string> AvailableChannels => _sources.Keys.ToList();
}

