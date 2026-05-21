using Avalonia;
using Avalonia.Controls.ApplicationLifetimes;
using GregModmanager.Avalonia.Services;
using GregModmanager.Localization;
using GregModmanager.Services;
using GregModmanager.Steam;
using Microsoft.Extensions.DependencyInjection;

namespace GregModmanager.Avalonia;

internal static class Program
{
    private const string LogCategory = "Avalonia.Program.Main";

    [STAThread]
    public static void Main(string[] args)
    {
        AppFileLog.StartSession();
        AppDomain.CurrentDomain.ProcessExit += (_, _) => AppFileLog.EndSession();
        AppFileLog.Info("Avalonia Program entry");

        // Eager Telemetry: Try to send old crash reports as early as possible
        try
        {
            var telemetry = new TelemetryService();
            _ = telemetry.ReportCrashesAsync();
        }
        catch (Exception ex)
        {
            AppFileLog.Error("Failed to start eager telemetry", ex);
        }

        DebugNdjsonSessionLog.Write("H1", LogCategory, "entry", new GregModmanager.Models.DebugLogPayload
        {
            BaseDir = AppContext.BaseDirectory,
            TempLog = DebugNdjsonSessionLog.LogPath,
            Args = Environment.GetCommandLineArgs(),
        });

        S.ApplySavedCulture();

        AppDomain.CurrentDomain.UnhandledException += (_, e) =>
        {
            var ex = e.ExceptionObject as Exception;
            AppFileLog.MarkCrash("AppDomain.UnhandledException", ex);
            AppFileLog.Error($"UnhandledException (terminating={e.IsTerminating})", ex);
            
            // Try to send the report immediately if we are terminating
            try { new TelemetryService().ReportCrashesAsync().GetAwaiter().GetResult(); } 
            catch { /* Ignore errors during emergency telemetry flush */ }
            
            DebugSessionLog.Write("H1", "Avalonia.UnhandledException", "unhandled", new
            {
                e.IsTerminating,
                message = ex?.Message,
                stack = ex?.StackTrace,
            });
        };
        TaskScheduler.UnobservedTaskException += (_, e) =>
        {
            AppFileLog.MarkCrash("TaskScheduler.UnobservedTaskException", e.Exception);
            AppFileLog.Error("UnobservedTaskException", e.Exception);
            DebugSessionLog.Write("H1", "Avalonia.UnobservedTaskException", "unobserved", new
            {
                e.Observed,
                message = e.Exception?.Message,
                stack = e.Exception?.StackTrace,
            });
            e.SetObserved();
        };

        try
        {
            var baseDir = AppContext.BaseDirectory;
            if (!string.IsNullOrEmpty(baseDir))
            {
                try
                {
                    if (IsDirectoryWritable(baseDir))
                    {
                        Directory.SetCurrentDirectory(baseDir);
                        AppFileLog.Info($"CurrentDirectory set to: {baseDir}");
                    }
                }
                catch { /* Optional optimization: fall back to default directory if not writable */ }
            }

            var steamOk = SteamApiNativeLoader.TryPreload();
            AppFileLog.Info($"SteamApiNativeLoader.TryPreload={steamOk}");
            
            // Note: Telemetry for steamOk will be tracked in App.axaml.cs after DI is ready

            if (GregModmanager.Services.Auth.ProtocolSingleInstance.ShouldForwardAndExitAsync(Environment.GetCommandLineArgs()).GetAwaiter().GetResult())
            {
                Environment.Exit(0);
            }

            if (HeadlessRunner.TryHandle(Environment.GetCommandLineArgs(), out var exitCode))
            {
                DebugNdjsonSessionLog.Write("H3", LogCategory, "headless_exit", new GregModmanager.Models.DebugLogPayload { ExitCode = exitCode });
                DebugSessionLog.Write("H4", LogCategory, "headless_exit", new { exitCode });
                Environment.Exit(exitCode);
                throw new InvalidOperationException("Unreachable: process should have exited.");
            }

            BuildAvaloniaApp().StartWithClassicDesktopLifetime(args);
        }
        catch (Exception ex)
        {
            AppFileLog.MarkCrash("Avalonia.Program.Main.Exception", ex);
            AppFileLog.Error("Avalonia Program exception", ex);
            DebugNdjsonSessionLog.Write("H1", LogCategory, "exception", new GregModmanager.Models.DebugLogPayload { Message = ex.Message, ExType = ex.GetType().FullName, StackTrace = ex.StackTrace });
            DebugSessionLog.Write("H1", LogCategory, "exception", new { ex.Message, ex.StackTrace });
            
            if (OperatingSystem.IsWindows() && !args.Contains("--headless"))
            {
                // Native fallback to inform user about startup crash
                Win32MessageBox(IntPtr.Zero, 
                    $"Der gregModmanager konnte nicht gestartet werden.\n\nFehler: {ex.Message}\n\nEin Crash-Bericht wurde unter %AppData%\\GregModmanager\\logs\\reports angelegt.", 
                    "gregModmanager - Startup Crash", 0x10);
            }
            throw;
        }
    }

    [System.Runtime.InteropServices.DllImport("user32.dll", EntryPoint = "MessageBoxW", CharSet = System.Runtime.InteropServices.CharSet.Unicode)]
    private static extern int Win32MessageBox(IntPtr hWnd, string text, string caption, uint type);

    public static AppBuilder BuildAvaloniaApp()
        => AppBuilder.Configure<App>()
            .UsePlatformDetect()
            .WithInterFont()
            .LogToTrace();

    public static IServiceProvider BuildServices()
    {
        var services = new ServiceCollection();

        services.AddSingleton<AppLogService>();
        services.AddSingleton<TelemetryService>();
        services.AddSingleton<ReproBundleService>();
        services.AddSingleton<SteamWorkshopService>();
        SteamApiNativeLoader.SetGameRoot(AppSettings.GetGameRootPath());
        services.AddSingleton<WorkspaceService>();
        services.AddSingleton<ModDependencyService>();
        services.AddSingleton<gregPluginChannelRegistry>(sp =>
        {
            var registry = new gregPluginChannelRegistry();
            registry.Register(new StablePluginSource(sp.GetRequiredService<ModDependencyService>()));
            registry.Register(new BetaPluginSource());
            registry.Register(new GitHubModSource());
            return registry;
        });
        services.AddSingleton<RalphSyncService>();
        services.AddSingleton<VdfGeneratorService>();
        services.AddSingleton<WorkshopDownloadService>();
        services.AddSingleton<ModsFolderSyncService>();
        services.AddSingleton<ModCollectionService>();
        services.AddSingleton<SubscriptionPoller>(sp =>
            new SubscriptionPoller(sp.GetRequiredService<SteamWorkshopService>()));
        services.AddSingleton<WorkshopSyncOrchestrator>();
        services.AddSingleton<GitVerificationService>();
        services.AddSingleton<BetterAuthService>();

        services.AddSingleton<GregModmanager.Services.Auth.IAuthApiClient, GregModmanager.Services.Auth.AuthApiClient>();
        services.AddSingleton<GregModmanager.Services.Auth.ISessionManager, GregModmanager.Services.Auth.SessionManager>();
        services.AddSingleton<GregModmanager.Services.Install.IInstallIntentClient, GregModmanager.Services.Install.InstallIntentClient>();
        services.AddSingleton<GregModmanager.Services.Install.IIntentSignatureVerifier, GregModmanager.Services.Install.DummyIntentSignatureVerifier>();

        services.AddSingleton<IDialogService, DialogService>();

        services.AddSingleton<MainWindow>();
        services.AddTransient<Views.ProjectsPage>();
        services.AddTransient<Views.NewProjectPage>();
        services.AddTransient<Views.MyUploadsPage>();
        services.AddTransient<Views.ModManagerPage>();
        services.AddTransient<Views.SettingsPage>();
        services.AddTransient<Views.EditorPage>();
        services.AddTransient<Views.ItemDetailPage>();

        return services.BuildServiceProvider();
    }

    private static bool IsDirectoryWritable(string directoryPath)
    {
        try
        {
            var testFile = Path.Combine(directoryPath, $".write-test-{Environment.ProcessId}.tmp");
            File.WriteAllText(testFile, "ok");
            File.Delete(testFile);
            return true;
        }
        catch
        {
            return false;
        }
    }
}
