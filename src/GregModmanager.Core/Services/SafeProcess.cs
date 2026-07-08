using System;
using System.Diagnostics;
using System.Diagnostics.CodeAnalysis;
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
            if (Uri.TryCreate(url, UriKind.Absolute, out var uri) && IsAllowedBrowserUri(uri))
            {
                StartShellOpen(uri.AbsoluteUri);
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
            var fullPath = Path.GetFullPath(path);
            if (!Directory.Exists(fullPath))
            {
                AppFileLog.Warn($"Blocked attempt to open missing folder: {path}");
                return;
            }

            StartShellOpen(fullPath);
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
            var fullPath = Path.GetFullPath(filePath);
            if (!File.Exists(fullPath) && !Directory.Exists(fullPath))
            {
                AppFileLog.Warn($"Blocked attempt to select missing file system entry: {filePath}");
                return;
            }

            StartProcess("explorer.exe", new[] { $"/select,{fullPath}" });
        }
        catch (Exception ex)
        {
            AppFileLog.Error($"Failed to open explorer and select: {filePath}", ex);
        }
    }

    /// <summary>
    /// Launches a local executable with UseShellExecute = false.
    /// </summary>
    public static void LaunchApp(string exePath)
    {
        LaunchApp(exePath, Array.Empty<string>());
    }

    /// <summary>
    /// Launches a local executable with explicit arguments and UseShellExecute = false.
    /// </summary>
    [SuppressMessage("Security", "S2076:OS commands should not be vulnerable to command injection", Justification = "The executable path is normalized, required to exist locally, and arguments are passed through ArgumentList instead of a shell string.")]
    public static void LaunchApp(string exePath, IReadOnlyList<string> arguments)
    {
        if (string.IsNullOrWhiteSpace(exePath)) return;

        try
        {
            var fullPath = Path.GetFullPath(exePath);
            if (!File.Exists(fullPath))
            {
                AppFileLog.Warn($"Blocked attempt to launch missing executable: {exePath}");
                return;
            }

            StartProcess(fullPath, arguments);
        }
        catch (Exception ex)
        {
            AppFileLog.Error($"Failed to launch app: {exePath}", ex);
        }
    }

    private static bool IsAllowedBrowserUri(Uri uri)
    {
        return uri.Scheme == Uri.UriSchemeHttp || uri.Scheme == Uri.UriSchemeHttps;
    }

    private static void StartShellOpen(string target)
    {
        if (OperatingSystem.IsWindows())
        {
            StartProcess("rundll32.exe", new[] { "url.dll,FileProtocolHandler", target });
            return;
        }

        if (OperatingSystem.IsMacOS())
        {
            StartProcess("open", new[] { target });
            return;
        }

        StartProcess("xdg-open", new[] { target });
    }

    private static void StartProcess(string fileName, IEnumerable<string> arguments)
    {
        var info = new ProcessStartInfo
        {
            FileName = fileName,
            UseShellExecute = false
        };

        foreach (var argument in arguments)
        {
            info.ArgumentList.Add(argument);
        }

        Process.Start(info);
    }
}
