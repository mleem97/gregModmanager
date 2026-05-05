using GregModmanager.Models;
using System.Text.Json;

namespace GregModmanager.Services;

/// <summary>
/// Fetches mod packages directly from GitHub Releases.
/// Enabled via "github" channel in settings.
/// </summary>
public sealed class GitHubModSource : IgregPluginChannelSource
{
    private readonly HttpClient _http = new();
    public string ChannelName => "github";

    public IReadOnlyList<PluginPackageInfo> ListPlugins()
    {
        var repos = new[] { 
            "mleem97/gregCore", 
            "mleem97/gregMod.IPAM",
            "mleem97/gregMod.ResetSwitch",
            "ASavageSwan/-DataCenter-ModLoader"
        };

        var list = new List<PluginPackageInfo>();
        
        foreach (var repo in repos)
        {
            string id = repo.Split('/').Last();
            if (id == "-DataCenter-ModLoader") id = "SteamPlugin"; // Friendly name for the external plugin

            list.Add(new PluginPackageInfo
            {
                PluginId = id,
                Version = "Latest (GitHub)",
                Channel = ChannelName,
            });
        }

        return list;
    }

    public async Task InstallAsync(PluginPackageInfo plugin, string targetDir)
    {
        // logic to download from https://github.com/{owner}/{repo}/releases/latest/download/{plugin.PluginId}.dll
    }
}

