using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Interactivity;
using Avalonia.Platform.Storage;
using Avalonia.VisualTree;
using GregModmanager.Localization;
using GregModmanager.Services;
using Microsoft.Extensions.DependencyInjection;

namespace GregModmanager.Avalonia.Views;

public partial class SettingsPage : UserControl
{
    private readonly WorkspaceService _workspace = null!;
    private readonly ReproBundleService _reproBundle = null!;
    private const string CurrentPathKey = "Settings_CurrentPath";

    public SettingsPage() => InitializeComponent();

    public SettingsPage(WorkspaceService workspace, ReproBundleService reproBundle)
    {
        InitializeComponent();
        _workspace = workspace;
        _reproBundle = reproBundle;

        LanguagePicker.ItemsSource = S.SupportedLanguages.Select(l => l.DisplayName).ToList();
        var savedCode = S.GetSavedLanguage();
        var idx = Array.FindIndex(S.SupportedLanguages, l => l.Code == savedCode);
        LanguagePicker.SelectedIndex = idx >= 0 ? idx : 0;

        ModStoreSwitch.IsChecked = AppSettings.IsModStoreEnabled();
        GameRootEntry.Text = AppSettings.GetGameRootPath();
        UpdateGameRootLabel();
        CustomPathEntry.Text = S.Preferences.GetString(WorkspaceService.CustomWorkspacePathKey, "");
        CurrentPathLabel.Text = S.Format(CurrentPathKey, _workspace.WorkspaceRoot);
        
        TelemetrySwitch.IsChecked = AppSettings.IsTelemetryEnabled();
    }

    private static void OnTelemetryToggled(object? sender, RoutedEventArgs e)
    {
        if (sender is CheckBox toggle)
        {
            var enabled = toggle.IsChecked ?? true;
            S.Preferences.SetBool(AppSettings.TelemetryEnabledKey, enabled);
        }
    }

    private void UpdateGameRootLabel() => UpdateGameRootLabel(this.CurrentGameRootLabel);

    private static void UpdateGameRootLabel(TextBlock label)
    {
        var path = AppSettings.GetGameRootPath();
        label.Text = string.IsNullOrEmpty(path)
            ? S.Get("Settings_GameRootNotSet")
            : S.Format(CurrentPathKey, path);
    }

    private void OnBrowseGameRoot(object? sender, RoutedEventArgs e)
    {
        var topLevel = TopLevel.GetTopLevel(this);
        if (topLevel == null) return;
        _ = BrowseGameRootAsync(topLevel);
    }

    private async Task BrowseGameRootAsync(TopLevel topLevel)
    {
        var folders = await topLevel.StorageProvider.OpenFolderPickerAsync(new FolderPickerOpenOptions { Title = "Select Game Root" });
        if (folders.Count > 0 && folders[0].TryGetLocalPath() is string path)
            GameRootEntry.Text = path;
    }

    private void OnApplyGameRoot(object? sender, RoutedEventArgs e)
    {
        var path = GameRootEntry.Text?.Trim() ?? "";
        if (!string.IsNullOrEmpty(path) && !Directory.Exists(path))
        {
            GameRootHint.Text = S.Get("Settings_GameRootNotFound");
            return;
        }
        S.Preferences.SetString(AppSettings.GameRootPathKey, path);
        this.UpdateGameRootLabel();
        GameRootHint.Text = S.Get("Settings_PathUpdated");
    }

    private void OnResetGameRoot(object? sender, RoutedEventArgs e)
    {
        S.Preferences.Remove(AppSettings.GameRootPathKey);
        GameRootEntry.Text = "";
        this.UpdateGameRootLabel();
        GameRootHint.Text = S.Get("Settings_PathReset");
    }

    private void OnBrowseWorkspace(object? sender, RoutedEventArgs e)
    {
        var topLevel = TopLevel.GetTopLevel(this);
        if (topLevel == null) return;
        _ = BrowseWorkspaceAsync(topLevel);
    }

    private async Task BrowseWorkspaceAsync(TopLevel topLevel)
    {
        var folders = await topLevel.StorageProvider.OpenFolderPickerAsync(new FolderPickerOpenOptions { Title = "Select Workspace" });
        if (folders.Count > 0 && folders[0].TryGetLocalPath() is string path)
            CustomPathEntry.Text = path;
    }

    private void OnApplyPath(object? sender, RoutedEventArgs e)
    {
        var path = CustomPathEntry.Text?.Trim() ?? "";
        if (!string.IsNullOrEmpty(path) && !Directory.Exists(path))
        {
            try { Directory.CreateDirectory(path); }
            catch (Exception ex)
            {
                PathHint.Text = S.Format("Settings_CannotCreate", ex.Message);
                return;
            }
        }
        S.Preferences.SetString(WorkspaceService.CustomWorkspacePathKey, path);
        _workspace.InvalidateCache();
        CurrentPathLabel.Text = S.Format(CurrentPathKey, _workspace.WorkspaceRoot);
        PathHint.Text = S.Get("Settings_PathUpdated");
    }

    private void OnResetPath(object? sender, RoutedEventArgs e)
    {
        S.Preferences.Remove(WorkspaceService.CustomWorkspacePathKey);
        CustomPathEntry.Text = "";
        _workspace.InvalidateCache();
        CurrentPathLabel.Text = S.Format(CurrentPathKey, _workspace.WorkspaceRoot);
        PathHint.Text = S.Get("Settings_PathReset");
    }

    private static void OnLanguageChanged(object? sender, SelectionChangedEventArgs e)
    {
        if (sender is ComboBox picker && picker.FindAncestorOfType<SettingsPage>() is SettingsPage page)
        {
            var idx = picker.SelectedIndex;
            if (idx < 0 || idx >= S.SupportedLanguages.Length) return;
            var code = S.SupportedLanguages[idx].Code;
            S.SetLanguage(code);
            page.LanguageHint.Text = S.Get("Settings_LanguageRestart");
        }
    }

    private static void OnModStoreToggled(object? sender, RoutedEventArgs e)
    {
        if (sender is CheckBox toggle)
        {
            var enabled = toggle.IsChecked ?? false;
            S.Preferences.SetBool(AppSettings.ModStoreEnabledKey, enabled);
            // Note: Hint update would require instance access, 
            // but we can rely on the XAML-bound control for static purely if needed.
        }
    }

    private static async void OnOpenLogs(object? sender, RoutedEventArgs e)
    {
        try
        {
            var logPath = AppFileLog.LogPath;
            var dir = Path.GetDirectoryName(logPath);
            if (!string.IsNullOrWhiteSpace(dir))
            {
                Directory.CreateDirectory(dir);
                SafeProcess.OpenFolder(dir);
            }
        }
        catch (Exception ex)
        {
            AppFileLog.Error("Failed to open logs", ex);
            var dialog = App.Services.GetRequiredService<Services.IDialogService>();
            await dialog.ShowMessageAsync(S.Get("Error"), S.Format("Settings_OpenLogsFailed", ex.Message));
        }
    }

    private void OnCreateReproBundle(object? sender, RoutedEventArgs e)
    {
        _ = CreateReproBundleAsync();
    }

    private async Task CreateReproBundleAsync()
    {
        try
        {
            var zipPath = await _reproBundle.CreateBundleAsync();
            SafeProcess.OpenExplorerAndSelect(zipPath);
            var dialog = App.Services.GetRequiredService<Services.IDialogService>();
            await dialog.ShowMessageAsync(S.Get("OK"), S.Format("Settings_ReproBundleCreated", zipPath));
        }
        catch (Exception ex)
        {
            AppFileLog.Error("Failed to create repro bundle", ex);
            var dialog = App.Services.GetRequiredService<Services.IDialogService>();
            await dialog.ShowMessageAsync(S.Get("Error"), S.Format("Settings_ReproBundleCreateFailed", ex.Message));
        }
    }

    private static void OnRestartApp(object? sender, RoutedEventArgs e)
    {
        var exe = Environment.ProcessPath;
        if (string.IsNullOrEmpty(exe)) return;
        SafeProcess.LaunchApp(exe);
        if (Application.Current?.ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
            desktop.Shutdown();
    }
}
