using Avalonia;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Markup.Xaml;
using Microsoft.Extensions.DependencyInjection;
using GregModmanager.Services;

namespace GregModmanager.Avalonia;

public partial class App : Application
{
    public static IServiceProvider Services { get; private set; } = null!;

    public override void Initialize()
    {
        AvaloniaXamlLoader.Load(this);
    }

    public override void OnFrameworkInitializationCompleted()
    {
        try
        {
            Services = Program.BuildServices();
        }
        catch (Exception ex)
        {
            AppFileLog.MarkCrash("App.BuildServices", ex);
            AppFileLog.Error("Failed to build DI container", ex);
            ShowFatalError(ex);
            return;
        }

        try
        {
            var telemetry = Services.GetRequiredService<TelemetryService>();
            _ = telemetry.ReportCrashesAsync();
            _ = telemetry.TrackEventAsync("startup", new
            {
                steamActive = GregModmanager.Steam.SteamApiNativeLoader.IsLoaded,
                culture = System.Globalization.CultureInfo.CurrentCulture.Name,
                osDescription = System.Runtime.InteropServices.RuntimeInformation.OSDescription,
                dotNetVersion = System.Runtime.InteropServices.RuntimeInformation.FrameworkDescription
            });
        }
        catch (Exception ex)
        {
            AppFileLog.Error("Telemetry init failed (non-fatal)", ex);
        }

        if (ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
        {
            try
            {
                var mainWindow = Services.GetRequiredService<MainWindow>();
                desktop.MainWindow = mainWindow;
            }
            catch (Exception ex)
            {
                AppFileLog.MarkCrash("App.MainWindow", ex);
                AppFileLog.Error("Failed to create MainWindow", ex);
                ShowFatalError(ex);
                return;
            }
        }

        base.OnFrameworkInitializationCompleted();
    }

    private static void ShowFatalError(Exception ex)
    {
        if (OperatingSystem.IsWindows())
        {
            try
            {
                var logDir = Path.Combine(
                    Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData),
                    "GregModmanager", "logs");
                var msg = $"gregModmanager could not start.\n\nError: {ex.Message}\n\nLogs: {logDir}";
                Win32MessageBox(IntPtr.Zero, msg, "gregModmanager - Startup Error", 0x10);
            }
            catch
            {
                // Last resort: silent exit
            }
        }
    }

    [System.Runtime.InteropServices.DllImport("user32.dll", EntryPoint = "MessageBoxW", CharSet = System.Runtime.InteropServices.CharSet.Unicode)]
    private static extern int Win32MessageBox(IntPtr hWnd, string text, string caption, uint type);
}
