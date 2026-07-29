using System;
using System.Diagnostics;
using System.IO;
using System.Runtime.InteropServices;
using GregModmanager.Services;

namespace GregModmanager.Core.Services;

public static class ProtocolRegistryService
{
    private const string ProtocolScheme = "greg";
    private const string ProtocolDescription = "gregModmanager Protocol";

    public static void RegisterProtocol()
    {
        try
        {
            var appPath = Process.GetCurrentProcess().MainModule?.FileName;
            if (string.IsNullOrEmpty(appPath)) return;

            if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
            {
                RegisterWindowsProtocol(appPath);
            }
            else if (RuntimeInformation.IsOSPlatform(OSPlatform.Linux))
            {
                RegisterLinuxProtocol(appPath);
            }
        }
        catch (Exception ex)
        {
            AppFileLog.Error("Failed to register protocol", ex);
        }
    }

    private static void RegisterWindowsProtocol(string appPath)
    {
#pragma warning disable CA1416 // Validate platform compatibility
        try
        {
            using var key = Microsoft.Win32.Registry.CurrentUser.CreateSubKey($@"Software\Classes\{ProtocolScheme}");
            key.SetValue(string.Empty, $"URL:{ProtocolDescription}");
            key.SetValue("URL Protocol", string.Empty);

            using var defaultIcon = key.CreateSubKey("DefaultIcon");
            defaultIcon.SetValue(string.Empty, $"{appPath},1");

            using var commandKey = key.CreateSubKey(@"shell\open\command");
            commandKey.SetValue(string.Empty, $"\"{appPath}\" \"%1\"");
        }
        catch (Exception ex)
        {
            AppFileLog.Error("Failed to register Windows protocol via Registry", ex);
        }
#pragma warning restore CA1416
    }

    private static void RegisterLinuxProtocol(string appPath)
    {
        try
        {
            var desktopFileContent = $@"
[Desktop Entry]
Name=gregModmanager
Exec={appPath} %u
Type=Application
Terminal=false
MimeType=x-scheme-handler/{ProtocolScheme};
";
            var desktopFilePath = Path.Combine(
                Environment.GetFolderPath(Environment.SpecialFolder.UserProfile),
                ".local/share/applications/gregmodmanager.desktop");

            Directory.CreateDirectory(Path.GetDirectoryName(desktopFilePath)!);
            File.WriteAllText(desktopFilePath, desktopFileContent.Trim());

            // Update mime database
            var psi = new ProcessStartInfo("update-desktop-database", "~/.local/share/applications")
            {
                UseShellExecute = true,
                WindowStyle = ProcessWindowStyle.Hidden
            };
            Process.Start(psi)?.WaitForExit();
        }
        catch (Exception ex)
        {
            AppFileLog.Error("Failed to register Linux protocol via .desktop file", ex);
        }
    }
}
