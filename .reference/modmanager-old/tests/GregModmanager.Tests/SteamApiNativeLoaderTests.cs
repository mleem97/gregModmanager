using System.Collections;
using System.Reflection;
using GregModmanager.Steam;

namespace GregModmanager.Tests;

public sealed class SteamApiNativeLoaderTests : IDisposable
{
    private readonly string _root = Path.Combine(Path.GetTempPath(), "GregModmanager_SteamVdf_" + Guid.NewGuid().ToString("N"));

    public void Dispose()
    {
        if (Directory.Exists(_root))
        {
            Directory.Delete(_root, recursive: true);
        }
    }

    [Fact]
    public void ParsesWhitespaceFormattedLibraryFolderPath()
    {
        var library = Path.Combine(_root, "Steam Library");
        Directory.CreateDirectory(library);
        var vdfPath = Path.Combine(_root, "libraryfolders.vdf");
        var escapedLibrary = library.Replace("\\", "\\\\", StringComparison.Ordinal);
        File.WriteAllText(vdfPath, $"\"path\"    \"{escapedLibrary}\"");

        var parser = typeof(SteamApiNativeLoader).GetMethod(
            "ParseLibraryFoldersVdf",
            BindingFlags.NonPublic | BindingFlags.Static);

        Assert.NotNull(parser);
        var parsedPaths = Assert.IsAssignableFrom<IEnumerable>(parser!.Invoke(null, [vdfPath]))
            .Cast<string>()
            .ToList();

        Assert.Contains(library, parsedPaths, StringComparer.Ordinal);
    }
}
