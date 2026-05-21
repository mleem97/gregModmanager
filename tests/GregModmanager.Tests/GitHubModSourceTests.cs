using System.Linq;
using Xunit;
using GregModmanager.Services;

namespace GregModmanager.Tests;

public class GitHubModSourceTests
{
    [Fact]
    public void ListPlugins_ReturnsExpectedHardcodedPlugins()
    {
        // Arrange
        var source = new GitHubModSource();

        // Act
        var plugins = source.ListPlugins();

        // Assert
        Assert.NotNull(plugins);
        Assert.Equal(4, plugins.Count);

        // Verify the generated PluginIds
        Assert.Equal("gregCore", plugins[0].PluginId);
        Assert.Equal("gregMod.IPAM", plugins[1].PluginId);
        Assert.Equal("gregMod.ResetSwitch", plugins[2].PluginId);
        Assert.Equal("SteamPlugin", plugins[3].PluginId); // Mapped from "-DataCenter-ModLoader"

        // Verify common properties for all returned plugins
        foreach (var plugin in plugins)
        {
            Assert.Equal("Latest (GitHub)", plugin.Version);
            Assert.Equal("github", plugin.Channel);
        }
    }
}
