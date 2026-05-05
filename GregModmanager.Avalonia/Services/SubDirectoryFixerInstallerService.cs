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

public sealed class SubDirectoryFixerInstallerService
{
    private const string PayloadRelativePath = "SubDirectoryFixer\\SubDirectoryFixer.dll";
    private const string TargetFileName = "SubDirectoryFixer.dll";
    private const string MarkerFileName = ".gregmodmanager-subdirfixer.sha256";

    public Task<SubDirectoryFixerInstallResult> EnsureInstalledAsync(string? gameRoot)
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

            var modsDir = Path.Combine(gameRoot, "Mods");
            Directory.CreateDirectory(modsDir);

            var targetPath = Path.Combine(modsDir, TargetFileName);
            var markerPath = Path.Combine(modsDir, MarkerFileName);

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