using GregModmanager.Models;

namespace GregModmanager.Tests;

public class ContentStatsTests
{
    [Fact]
    public void ContentStats_DefaultInitialization_HasCorrectDefaults()
    {
        // Act
        var stats = new ContentStats();

        // Assert
        Assert.False(stats.Exists);
        Assert.Equal(0, stats.TotalBytes);
        Assert.Empty(stats.TopEntries);
    }

    [Fact]
    public void ContentStats_CustomInitialization_StoresValuesCorrectly()
    {
        // Arrange
        var entries = new List<ContentFolderSize>
        {
            new("Folder1", 1000),
            new("Folder2", 2000)
        };

        // Act
        var stats = new ContentStats
        {
            Exists = true,
            TotalBytes = 3000,
            TopEntries = entries
        };

        // Assert
        Assert.True(stats.Exists);
        Assert.Equal(3000, stats.TotalBytes);
        Assert.Equal(2, stats.TopEntries.Count);
        Assert.Equal("Folder1", stats.TopEntries[0].Name);
        Assert.Equal(1000, stats.TopEntries[0].Bytes);
        Assert.Equal("Folder2", stats.TopEntries[1].Name);
        Assert.Equal(2000, stats.TopEntries[1].Bytes);
    }

    [Fact]
    public void ContentFolderSize_Initialization_StoresValuesCorrectly()
    {
        // Act
        var entry = new ContentFolderSize("TestFolder", 500);

        // Assert
        Assert.Equal("TestFolder", entry.Name);
        Assert.Equal(500, entry.Bytes);
    }
}
