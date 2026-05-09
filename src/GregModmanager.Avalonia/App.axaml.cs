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
        Services = Program.BuildServices();
        
        var telemetry = Services.GetRequiredService<TelemetryService>();
        _ = telemetry.ReportCrashesAsync();
        _ = telemetry.TrackEventAsync("startup", new 
        {
            steamActive = GregModmanager.Steam.SteamApiNativeLoader.IsLoaded,
            culture = System.Globalization.CultureInfo.CurrentCulture.Name,
            osDescription = System.Runtime.InteropServices.RuntimeInformation.OSDescription,
            dotNetVersion = System.Runtime.InteropServices.RuntimeInformation.FrameworkDescription
        });

        if (ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
        {
            var mainWindow = Services.GetRequiredService<MainWindow>();
            desktop.MainWindow = mainWindow;
        }

        base.OnFrameworkInitializationCompleted();
    }
}
