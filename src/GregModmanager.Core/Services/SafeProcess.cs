using System;
using System.Diagnostics;
using System.Threading.Tasks;

namespace GregModmanager.Services;

public static class SafeProcess
{
    /// <summary>
    /// Opens a URL in the default browser safely, ensuring only http and https schemes are allowed.
    /// </summary>
    public static Task OpenUrlAsync(string url)
    {
        if (string.IsNullOrWhiteSpace(url)) return Task.CompletedTask;

        try
        {
            if (Uri.TryCreate(url, UriKind.Absolute, out var uri) &&
                (uri.Scheme == Uri.UriSchemeHttp || uri.Scheme == Uri.UriSchemeHttps))
            {
                Process.Start(new ProcessStartInfo
                {
                    FileName = uri.ToString(),
                    UseShellExecute = true
                });
            }
            else
            {
                AppFileLog.Warn($"Blocked attempt to open insecure or invalid URL: {url}");
            }
        }
        catch (Exception ex)
        {
            AppFileLog.Error($"Failed to open URL: {url}", ex);
        }

        return Task.CompletedTask;
    }

    /// <summary>
    /// Opens a folder in the system's file explorer.
    /// </summary>
    public static void OpenFolder(string path)
    {
        if (string.IsNullOrWhiteSpace(path)) return;

        try
        {
            if (OperatingSystem.IsWindows())
            {
                Process.Start(new ProcessStartInfo
                {
                    FileName = "explorer.exe",
                    Arguments = $"\"{path}\"",
                    UseShellExecute = false
                });
            }
            else
            {
                // For other OS, we might still need UseShellExecute for some scenarios,
                // but we should be careful. MAUI doesn't have a direct "OpenFolder" that works everywhere.
                Process.Start(new ProcessStartInfo
                {
                    FileName = path,
                    UseShellExecute = true
                });
            }
        }
        catch (Exception ex)
        {
            AppFileLog.Error($"Failed to open folder: {path}", ex);
        }
    }

    /// <summary>
    /// Specifically for Windows, opens explorer and selects a file.
    /// </summary>
    public static void OpenExplorerAndSelect(string filePath)
    {
        if (!OperatingSystem.IsWindows() || string.IsNullOrWhiteSpace(filePath)) return;

        try
        {
            Process.Start(new ProcessStartInfo
            {
                FileName = "explorer.exe",
                Arguments = $"/select,\"{filePath}\"",
                UseShellExecute = false
            });
        }
        catch (Exception ex)
        {
            AppFileLog.Error($"Failed to open explorer and select: {filePath}", ex);
        }
    }

    /// <summary>
    /// Launches an executable with UseShellExecute = false.
    /// </summary>
    public static void LaunchApp(string exePath, string arguments = "")
    {
        if (string.IsNullOrWhiteSpace(exePath)) return;

        try
        {
            Process.Start(new ProcessStartInfo
            {
                FileName = exePath,
                Arguments = arguments,
                UseShellExecute = false
            });
        }
        catch (Exception ex)
        {
            AppFileLog.Error($"Failed to launch app: {exePath}", ex);
        }
    }
}
