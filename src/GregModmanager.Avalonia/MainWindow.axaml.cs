using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.Media;
using Avalonia.Platform.Storage;
using Avalonia.Threading;
using GregModmanager.Avalonia.Services;
using GregModmanager.Avalonia.Views;
using GregModmanager.Localization;
using GregModmanager.Models.Auth;
using GregModmanager.Services;
using GregModmanager.Services.Auth;
using Microsoft.Extensions.DependencyInjection;

namespace GregModmanager.Avalonia;

public partial class MainWindow : Window
{
    private readonly IServiceProvider _services;
    private readonly SteamWorkshopService _steam;
    private readonly ISessionManager _session;
    private readonly WorkspaceService _workspace;
    private Control? _currentPage;
    private System.Timers.Timer? _statusTimer;

    public MainWindow(
        IServiceProvider services,
        SteamWorkshopService steam,
        ISessionManager session,
        WorkspaceService workspace)
    {
        InitializeComponent();
        _services = services;
        _steam = steam;
        _session = session;
        _workspace = workspace;

        if (AppSettings.IsModStoreEnabled())
            BtnModStore.IsVisible = true;

        _session.StateChanged += () => Dispatcher.UIThread.Post(UpdateStatusIndicators);
        _session.ProtocolInvoked += uri => Dispatcher.UIThread.Post(async () => await HandleProtocolUriAsync(uri));
        Loaded += OnLoaded;
    }

    private async void OnLoaded(object? sender, RoutedEventArgs e)
    {
        try
        {
            var gameRoot = AppSettings.GetGameRootPath();
            if (!string.IsNullOrEmpty(gameRoot))
            {
                try
                {
                    var installResult = await SubDirectoryFixerInstallerService.EnsureInstalledAsync(gameRoot);
                    if (installResult.Status is SubDirectoryFixerInstallStatus.Installed or SubDirectoryFixerInstallStatus.Failed)
                    {
                        AppFileLog.Info(installResult.Message);
                    }
                }
                catch (Exception ex)
                {
                    AppFileLog.Error("SubDirectoryFixer install failed (non-fatal)", ex);
                }
            }

            await _session.InitializeAsync();

            if (!WorkspaceService.HasUserConfiguredWorkspace)
            {
                if (!await SelectWorkspaceOnFirstStartAsync())
                {
                    AppFileLog.Warn("Workspace selection was cancelled on first start; shutting down.");
                    if (Application.Current?.ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
                        desktop.Shutdown();
                    return;
                }
            }

            if (!AppSettings.HasAppModeChoice())
            {
                var dialog = _services.GetRequiredService<IDialogService>();
                var mode = await dialog.ShowChoiceAsync(
                    S.Get("AppMode_SelectTitle"),
                    S.Get("AppMode_SelectMessage"),
                    new[]
                    {
                        (AppSettings.AppModeFull, S.Get("AppMode_Full")),
                        (AppSettings.AppModeModManagerOnly, S.Get("AppMode_ModManagerOnly")),
                        (AppSettings.AppModeDecideLater, S.Get("AppMode_DecideLater"))
                });
                AppSettings.SetAppMode(mode ?? AppSettings.AppModeDecideLater);
            }
            BtnModStore.IsVisible = AppSettings.IsModStoreEnabled();

            var args = Environment.GetCommandLineArgs();
            foreach (var arg in args)
            {
                if (arg.StartsWith("greg://", StringComparison.OrdinalIgnoreCase))
                {
                    await HandleProtocolUriAsync(arg);
                }
            }

            var timer = new System.Timers.Timer(2000);
            timer.Elapsed += (_, _) =>
            {
                try
                {
                    Dispatcher.UIThread.Post(UpdateStatusIndicators);
                }
                catch (Exception ex)
                {
                    AppFileLog.Error("Status timer tick failed", ex);
                }
            };
            timer.AutoReset = true;
            timer.Start();
            _statusTimer = timer;

            NavigateTo<ProjectsPage>();
        }
        catch (Exception ex)
        {
            AppFileLog.MarkCrash("MainWindow.OnLoaded", ex);
            AppFileLog.Error("MainWindow.OnLoaded failed", ex);
        }
    }

    private async Task<bool> SelectWorkspaceOnFirstStartAsync()
    {
        var folders = await StorageProvider.OpenFolderPickerAsync(new FolderPickerOpenOptions
        {
            Title = S.Get("Workspace_SelectTitle"),
            AllowMultiple = false
        });
        var path = folders.FirstOrDefault()?.TryGetLocalPath();
        if (string.IsNullOrWhiteSpace(path))
            return false;

        S.Preferences.SetString(WorkspaceService.CustomWorkspacePathKey, Path.GetFullPath(path));
        _workspace.InvalidateCache();
        AppFileLog.Info($"Workspace selected on first start: {path}");
        return true;
    }

    private async Task HandleProtocolUriAsync(string arg)
    {
        try
        {
            if (arg.Contains("/auth/callback"))
                await _session.HandleProtocolCallbackAsync(arg);
            else if (arg.Contains("/install/intent"))
            {
                var installClient = _services.GetRequiredService<GregModmanager.Services.Install.IInstallIntentClient>();
                await installClient.HandleIntentAsync(arg);
            }
            else if (arg.Contains("/install?modId=") || arg.Contains("?modId="))
            {
                var uri = new Uri(arg);
                var query = System.Web.HttpUtility.ParseQueryString(uri.Query);
                var modId = query["modId"];
                if (!string.IsNullOrEmpty(modId))
                {
                    var dialog = _services.GetRequiredService<IDialogService>();
                    await dialog.ShowMessageAsync("Mod Manager", $"Installing Mod ID: {modId}...");
                    NavigateTo<ModManagerPage>();
                }
            }
        }
        catch (Exception ex)
        {
            AppFileLog.Error($"Protocol handler failed for: {arg}", ex);
        }
    }

    private void UpdateStatusIndicators()
    {
        if (_steam.TryGetSteamReady(out var userName))
        {
            SteamStatusLed.Fill = new SolidColorBrush(Color.Parse("#61F4D8"));
            SteamStatusText.Text = "Steam Connected";
        }
        else
        {
            SteamStatusLed.Fill = new SolidColorBrush(Color.Parse("#D7383B"));
            SteamStatusText.Text = "Steam Disconnected";
        }

        if (_session.State == SessionState.Authenticated && _session.CurrentSession != null)
        {
            GregApiStatusLed.Fill = new SolidColorBrush(Color.Parse("#61F4D8"));
            GregApiStatusText.Text = "gregAPI Online";
            AuthStatusLed.Fill = new SolidColorBrush(Color.Parse("#61F4D8"));
            AuthStatusText.Text = $"Logged in as {_session.CurrentSession.User?.DisplayName ?? "User"}";
        }
        else
        {
            GregApiStatusLed.Fill = new SolidColorBrush(Color.Parse("#D7383B"));
            GregApiStatusText.Text = AppSettings.IsLocalBuild ? "gregAPI Local" : "gregAPI Offline";
            AuthStatusLed.Fill = new SolidColorBrush(Color.Parse("#D7383B"));
            AuthStatusText.Text = AppSettings.IsLocalBuild ? "Login To Localhost" : "Login To Datacentermods.com";
        }
    }

    private void SetNavActive(Button active)
    {
        foreach (var child in NavPanel.Children)
        {
            if (child is Button btn)
                btn.Classes.Remove("active");
        }
        active.Classes.Add("active");
    }

    private void OnNavProjects(object? sender, RoutedEventArgs e)
    {
        SetNavActive(BtnProjects);
        NavigateTo<ProjectsPage>();
    }

    private void OnNavNewProject(object? sender, RoutedEventArgs e)
    {
        SetNavActive(BtnNewProject);
        NavigateTo<NewProjectPage>();
    }

    private void OnNavMyUploads(object? sender, RoutedEventArgs e)
    {
        SetNavActive(BtnMyUploads);
        NavigateTo<MyUploadsPage>();
    }

    private void OnNavModStore(object? sender, RoutedEventArgs e)
    {
        SetNavActive(BtnModStore);
        NavigateTo<ModManagerPage>();
    }

    private void OnNavSettings(object? sender, RoutedEventArgs e)
    {
        SetNavActive(BtnSettings);
        NavigateTo<SettingsPage>();
    }

    public void NavigateTo<T>() where T : Control
    {
        _currentPage = _services.GetRequiredService<T>();
        MainContent.Content = _currentPage;
    }

    public void NavigateTo(Control page)
    {
        _currentPage = page;
        MainContent.Content = page;
    }

    private void OnTitleBarPointerPressed(object? sender, PointerPressedEventArgs e)
    {
        if (!e.GetCurrentPoint(this).Properties.IsLeftButtonPressed)
            return;

        if (e.ClickCount >= 2)
        {
            ToggleMaximize();
            e.Handled = true;
            return;
        }

        if (WindowState != WindowState.Maximized)
            BeginMoveDrag(e);
    }

    private void OnMaximizeClicked(object? sender, RoutedEventArgs e) => ToggleMaximize();

    private void ToggleMaximize()
    {
        WindowState = WindowState == WindowState.Maximized
            ? WindowState.Normal
            : WindowState.Maximized;
    }

    private void OnMinimizeClicked(object? sender, RoutedEventArgs e)
    {
        WindowState = WindowState.Minimized;
    }

    private void OnCloseClicked(object? sender, RoutedEventArgs e)
    {
        _statusTimer?.Stop();
        _statusTimer?.Dispose();
        _statusTimer = null;
        Close();
    }
}
