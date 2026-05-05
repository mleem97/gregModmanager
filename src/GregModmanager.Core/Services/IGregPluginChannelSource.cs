using GregModmanager.Models;

namespace GregModmanager.Services;

/// <summary>
/// Provides a list of available greg plugin packages from a specific distribution channel.
/// Implementations: <see cref="StablePluginSource"/> (local scan) and <see cref="BetaPluginSource"/> (server, TODO).
/// </summary>
public interface IgregPluginChannelSource
{
	string ChannelName { get; }
	IReadOnlyList<PluginPackageInfo> ListPlugins();
}

