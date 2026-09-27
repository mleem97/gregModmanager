using System.Security.Cryptography;

namespace GregModmanager.Avalonia.Services;

public enum SubDirectoryFixerInstallStatus
{
    SkippedNoGameRoot,
    SkippedMissingPayload,
    AlreadyInstalled,
    Installed,
    Failed,
}

public sealed record SubDirectoryFixerInstallResult(SubDirectoryFixerInstallStatus Status, string Message);

public static class SubDirectoryFixerInstallerService
{
    private static readonly string PayloadRelativePath = Path.Combine("SubDirectoryFixer", "SubDirectoryFixer.dll");
    private const string TargetFileName = "SubDirectoryFixer.dll";
    private const string MarkerFileName = ".gregmodmanager-subdirfixer.sha256";

    public static Task<SubDirectoryFixerInstallResult> EnsureInstalledAsync(string? gameRoot)
    {
        if (string.IsNullOrWhiteSpace(gameRoot) || !Directory.Exists(gameRoot))
        {
            return Task.FromResult(new SubDirectoryFixerInstallResult(
                SubDirectoryFixerInstallStatus.SkippedNoGameRoot,
                "SubDirectoryFixer skipped: game root not configured."));
        }

        try
        {
            var payloadPath = Path.Combine(AppContext.BaseDirectory, PayloadRelativePath);
            if (!File.Exists(payloadPath))
            {
                return Task.FromResult(new SubDirectoryFixerInstallResult(
                    SubDirectoryFixerInstallStatus.SkippedMissingPayload,
                    $"SubDirectoryFixer payload missing at '{payloadPath}'."));
            }

            var pluginsDir = Path.Combine(Path.GetFullPath(gameRoot), "Plugins");
            Directory.CreateDirectory(pluginsDir);

            var targetPath = Path.Combine(pluginsDir, TargetFileName);
            var markerPath = Path.Combine(pluginsDir, MarkerFileName);

            var payloadHash = ComputeSha256(payloadPath);
            var previousHash = File.Exists(markerPath) ? File.ReadAllText(markerPath).Trim() : string.Empty;
            if (File.Exists(targetPath) && string.Equals(previousHash, payloadHash, StringComparison.OrdinalIgnoreCase))
            {
                return Task.FromResult(new SubDirectoryFixerInstallResult(
                    SubDirectoryFixerInstallStatus.AlreadyInstalled,
                    "SubDirectoryFixer already installed."));
            }

            File.Copy(payloadPath, targetPath, overwrite: true);
            File.WriteAllText(markerPath, payloadHash);

            return Task.FromResult(new SubDirectoryFixerInstallResult(
                SubDirectoryFixerInstallStatus.Installed,
                $"SubDirectoryFixer installed to '{targetPath}'."));
        }
        catch (Exception ex)
        {
            return Task.FromResult(new SubDirectoryFixerInstallResult(
                SubDirectoryFixerInstallStatus.Failed,
                $"SubDirectoryFixer install failed: {ex.Message}"));
        }
    }

    private static string ComputeSha256(string filePath)
    {
        using var stream = File.OpenRead(filePath);
        var hash = SHA256.HashData(stream);
        return Convert.ToHexString(hash);
    }
}