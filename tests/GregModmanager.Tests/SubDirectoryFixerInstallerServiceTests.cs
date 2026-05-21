using GregModmanager.Avalonia.Services;
using System.Security.Cryptography;

namespace GregModmanager.Tests;

public class SubDirectoryFixerInstallerServiceTests : IDisposable
{
    private readonly string _tempGameRoot;

    public SubDirectoryFixerInstallerServiceTests()
    {
        _tempGameRoot = Path.Combine(Path.GetTempPath(), "GregModmanager_Tests_" + Guid.NewGuid().ToString());

        // Note: The service uses Path.Combine(AppContext.BaseDirectory, "SubDirectoryFixer\\SubDirectoryFixer.dll").
        // On Linux, Path.Combine does not translate '\' to '/', so it attempts to find a file literally named
        // "SubDirectoryFixer\SubDirectoryFixer.dll" in the BaseDirectory.
        // We create it here to ensure the tests pass on Linux when run in the CI/headless environment.
        var payloadPath = Path.Combine(AppContext.BaseDirectory, "SubDirectoryFixer\\SubDirectoryFixer.dll");
        var dirPath = Path.GetDirectoryName(payloadPath);
        if (!string.IsNullOrEmpty(dirPath))
        {
            Directory.CreateDirectory(dirPath);
        }
        if (!File.Exists(payloadPath))
        {
            File.WriteAllText(payloadPath, "dummy-payload-content");
        }
    }

    public void Dispose()
    {
        if (Directory.Exists(_tempGameRoot))
        {
            try
            {
                Directory.Delete(_tempGameRoot, true);
            }
            catch
            {
                // Ignore cleanup errors
            }
        }
    }

    [Theory]
    [InlineData(null)]
    [InlineData("")]
    [InlineData("   ")]
    public async Task EnsureInstalledAsync_WithInvalidGameRoot_ReturnsSkippedNoGameRoot(string? gameRoot)
    {
        var result = await SubDirectoryFixerInstallerService.EnsureInstalledAsync(gameRoot);

        Assert.Equal(SubDirectoryFixerInstallStatus.SkippedNoGameRoot, result.Status);
    }

    [Fact]
    public async Task EnsureInstalledAsync_WithNonExistentGameRoot_ReturnsSkippedNoGameRoot()
    {
        var result = await SubDirectoryFixerInstallerService.EnsureInstalledAsync(_tempGameRoot);

        Assert.Equal(SubDirectoryFixerInstallStatus.SkippedNoGameRoot, result.Status);
    }

    [Fact]
    public async Task EnsureInstalledAsync_WithValidGameRoot_InstallsSuccessfully()
    {
        // Arrange
        Directory.CreateDirectory(_tempGameRoot);

        // Act
        var result = await SubDirectoryFixerInstallerService.EnsureInstalledAsync(_tempGameRoot);

        // Assert
        Assert.Equal(SubDirectoryFixerInstallStatus.Installed, result.Status);

        var pluginsDir = Path.Combine(_tempGameRoot, "Plugins");
        var targetFile = Path.Combine(pluginsDir, "SubDirectoryFixer.dll");
        var markerFile = Path.Combine(pluginsDir, ".gregmodmanager-subdirfixer.sha256");

        Assert.True(File.Exists(targetFile));
        Assert.True(File.Exists(markerFile));
    }

    [Fact]
    public async Task EnsureInstalledAsync_AlreadyInstalled_ReturnsAlreadyInstalled()
    {
        // Arrange
        Directory.CreateDirectory(_tempGameRoot);

        // First install
        await SubDirectoryFixerInstallerService.EnsureInstalledAsync(_tempGameRoot);

        // Act - Second install
        var result = await SubDirectoryFixerInstallerService.EnsureInstalledAsync(_tempGameRoot);

        // Assert
        Assert.Equal(SubDirectoryFixerInstallStatus.AlreadyInstalled, result.Status);
    }

    [Fact]
    public async Task EnsureInstalledAsync_MarkerHashMismatch_UpdatesSuccessfully()
    {
        // Arrange
        Directory.CreateDirectory(_tempGameRoot);

        // First install
        await SubDirectoryFixerInstallerService.EnsureInstalledAsync(_tempGameRoot);

        // Tamper with marker file
        var pluginsDir = Path.Combine(_tempGameRoot, "Plugins");
        var markerFile = Path.Combine(pluginsDir, ".gregmodmanager-subdirfixer.sha256");

        // Make sure Plugins exists in case the first call failed
        Directory.CreateDirectory(pluginsDir);
        await File.WriteAllTextAsync(markerFile, "invalid-hash");

        // Act - Update
        var result = await SubDirectoryFixerInstallerService.EnsureInstalledAsync(_tempGameRoot);

        // Assert
        Assert.Equal(SubDirectoryFixerInstallStatus.Installed, result.Status);
        var newMarkerHash = await File.ReadAllTextAsync(markerFile);
        Assert.NotEqual("invalid-hash", newMarkerHash);
    }

    [Fact]
    public async Task EnsureInstalledAsync_FileLocked_ReturnsFailedStatus()
    {
        // Arrange
        Directory.CreateDirectory(_tempGameRoot);

        // Create the directory structure where it will be installed
        var pluginsDir = Path.Combine(_tempGameRoot, "Plugins");
        Directory.CreateDirectory(pluginsDir);

        // Create a dummy file and lock it
        var targetFile = Path.Combine(pluginsDir, "SubDirectoryFixer.dll");
        await File.WriteAllTextAsync(targetFile, "dummy");

        // Lock the file for exclusive access
        using var stream = new FileStream(targetFile, FileMode.Open, FileAccess.ReadWrite, FileShare.None);

        // Act
        var result = await SubDirectoryFixerInstallerService.EnsureInstalledAsync(_tempGameRoot);

        // Assert
        Assert.Equal(SubDirectoryFixerInstallStatus.Failed, result.Status);
        Assert.Contains("install failed", result.Message, StringComparison.OrdinalIgnoreCase);
    }
}
