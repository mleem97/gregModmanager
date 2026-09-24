using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.Media;
using Avalonia.VisualTree;
using GregModmanager.Avalonia.Services;
using GregModmanager.Localization;
using GregModmanager.Models;
using GregModmanager.Services;
using Microsoft.Extensions.DependencyInjection;
using System.Collections.ObjectModel;

namespace GregModmanager.Avalonia.Views;

public partial class ProjectsPage : UserControl
{
    private const string ErrorKey = "Error";

    private readonly WorkspaceService _workspace = null!;
    private readonly SteamWorkshopService _steam = null!;
    private readonly AppLogService _log = null!;
    private readonly List<ProjectItemVm> _allProjects = new();
    private readonly ObservableCollection<WorkshopItemDetailVm> _uploads = new();
    private string _searchQuery = "";
    private int _uploadsPage = 1;
    private bool _uploadsHasMore;
    private bool _uploadsLoaded;

    public ProjectsPage() => InitializeComponent();

    public ProjectsPage(WorkspaceService workspace, SteamWorkshopService steam, AppLogService log)
    {
        InitializeComponent();
        _workspace = workspace;
        _steam = steam;
        _log = log;
        _workspace.EnsureWorkspaceStructure();

        var migrated = _workspace.MigrateLegacyProjects();
        if (migrated > 0) _log.Append(S.Format("Projects_Migrated", migrated));

        TabBtnLocal.Content = S.Get("Projects_TabLocal");
        TabBtnWorkshop.Content = S.Get("Projects_TabWorkshop");
        UploadsList.ItemsSource = _uploads;

        WorkspacePathLabel.Text = _workspace.WorkspaceRoot;
        _log.LineAppended += OnLogAppended;
        ReloadProjects();
        SetProjectsTab(0);
        _log.Append(S.Get("Projects_Ready"));
    }

    private void OnLogAppended(object? sender, EventArgs e)
    {
        LogText.Text = string.Join(Environment.NewLine, _log.Lines);
    }

    #region Tabs

    private void OnTabLocal(object? sender, RoutedEventArgs e) => SetProjectsTab(0);

    private void OnTabWorkshop(object? sender, RoutedEventArgs e)
    {
        SetProjectsTab(1);
        if (!_uploadsLoaded)
        {
            _uploadsLoaded = true;
            _ = LoadUploadsAsync();
        }
    }

    private void SetProjectsTab(int index)
    {
        LocalPanel.IsVisible = index == 0;
        UploadsPanel.IsVisible = index == 1;
        TabBtnLocal.Classes.Set("active", index == 0);
        TabBtnWorkshop.Classes.Set("active", index == 1);
    }

    #endregion

    #region Local projects

    private void ReloadProjects()
    {
        _allProjects.Clear();
        foreach (var p in _workspace.ScanProjects())
        {
            _allProjects.Add(new ProjectItemVm(p, _workspace));
        }
        ApplySearchFilter();
    }

    private void OnSearchTextChanged(object? sender, TextChangedEventArgs e)
    {
        _searchQuery = SearchBox.Text ?? "";
        ApplySearchFilter();
    }

    private void ApplySearchFilter()
    {
        var q = _searchQuery.Trim();
        var filtered = _allProjects.Where(vm =>
            string.IsNullOrEmpty(q) ||
            vm.Name.Contains(q, StringComparison.OrdinalIgnoreCase) ||
            vm.RootPath.Contains(q, StringComparison.OrdinalIgnoreCase) ||
            vm.Title.Contains(q, StringComparison.OrdinalIgnoreCase) ||
            (!string.IsNullOrEmpty(vm.Tags) && vm.Tags.Contains(q, StringComparison.OrdinalIgnoreCase))
        ).ToList();
        ProjectList.ItemsSource = filtered;
    }

    private void OnRefreshProjects(object? sender, RoutedEventArgs e)
    {
        ReloadProjects();
        _log.Append(S.Get("Projects_Refreshed"));
    }

    private void OnProjectTapped(object? sender, PointerPressedEventArgs e)
    {
        // Clicks on the Edit button are handled by OnEditProjectClicked;
        // ignore the bubbled press so the editor is not opened twice.
        if (e.Source is Button)
        {
            return;
        }

        if (sender is Border border && border.DataContext is ProjectItemVm vm)
        {
            OpenInEditor(vm.RootPath);
        }
    }

    private void OnEditProjectClicked(object? sender, RoutedEventArgs e)
    {
        var rootPath = (sender as Button)?.DataContext is ProjectItemVm vm
            ? vm.RootPath
            : null;
        if (string.IsNullOrEmpty(rootPath))
        {
            return;
        }

        e.Handled = true;
        OpenInEditor(rootPath);
    }

    private void OpenInEditor(string rootPath)
    {
        try
        {
            var editor = App.Services.GetRequiredService<EditorPage>();
            editor.LoadProject(rootPath);
            var mw = this.VisualRoot as MainWindow
                ?? App.Services.GetService<MainWindow>();
            if (mw is not null)
            {
                mw.NavigateTo(editor);
                _log.Append($"Opened project: {rootPath}");
            }
            else
            {
                _log.Append($"Could not navigate: no main window for {rootPath} (visualRoot={this.VisualRoot?.GetType().FullName ?? "null"})");
            }
        }
        catch (Exception ex)
        {
            _log.Append($"Failed to open project {rootPath}: {ex.Message}");
            var dialog = App.Services.GetRequiredService<IDialogService>();
            _ = dialog.ShowErrorAsync(S.Get(ErrorKey), S.Get("Projects_OpenFailed"), ex);
        }
    }

    #endregion

    #region Workshop uploads

    private void OnUploadsSelectionChanged(object? sender, SelectionChangedEventArgs e) => UpdateSelectionCount();

    private void UpdateSelectionCount()
    {
        var n = UploadsList.SelectedItems?.Count ?? 0;
        SelectionCountLabel.Text = S.Format("Uploads_Selected", n);
    }

    private async void OnRefreshUploads(object? sender, RoutedEventArgs e) => await LoadUploadsAsync();

    private async Task LoadUploadsAsync()
    {
        try
        {
            var result = await _steam.ListMyPublishedPagedAsync(_uploadsPage, CancellationToken.None);

            _uploads.Clear();
            foreach (var x in result.Items) _uploads.Add(x);

            _uploadsHasMore = result.HasMorePages;
            PrevBtn.IsEnabled = _uploadsPage > 1;
            NextBtn.IsEnabled = _uploadsHasMore;
            PageLabel.Text = S.Format("PageWithTotal", _uploadsPage, result.TotalResults);

            UploadsList.SelectedItems?.Clear();
            UpdateSelectionCount();
            _log.Append($"Workshop uploads: {result.TotalResults} item(s), page {_uploadsPage}.");
        }
        catch (Exception ex)
        {
            var dialog = App.Services.GetRequiredService<IDialogService>();
            await dialog.ShowErrorAsync(S.Get(ErrorKey), S.Get("Uploads_ActionFailed"), ex);
        }
    }

    private void OnPrevUploads(object? sender, RoutedEventArgs e)
    {
        if (_uploadsPage > 1) { _uploadsPage--; _ = LoadUploadsAsync(); }
    }

    private void OnNextUploads(object? sender, RoutedEventArgs e)
    {
        if (_uploadsHasMore) { _uploadsPage++; _ = LoadUploadsAsync(); }
    }

    private async void OnImportSelected(object? sender, RoutedEventArgs e)
    {
        var selected = UploadsList.SelectedItems?.OfType<WorkshopItemDetailVm>().ToList() ?? new List<WorkshopItemDetailVm>();
        if (selected.Count == 0)
        {
            var dialog = App.Services.GetRequiredService<IDialogService>();
            await dialog.ShowMessageAsync(S.Get("Import"), S.Get("Uploads_SelectFirst"));
            return;
        }

        var dialogSvc = App.Services.GetRequiredService<IDialogService>();
        var ok = await dialogSvc.ShowConfirmAsync(
            S.Get("Uploads_BulkImport"),
            S.Format("Uploads_BulkImportMsg", selected.Count, _workspace.WorkspaceRoot),
            S.Get("Import"), S.Get("Cancel"));
        if (!ok) return;

        var lastPath = "";
        try
        {
            foreach (var vm in selected)
            {
                var progress = new Progress<float>(p => _log.Append($"[{vm.PublishedFileId}] {p:P0}"));
                var log = new Progress<string>(s => _log.Append($"[{vm.PublishedFileId}] {s}"));
                var outcome = await _steam.ImportPublishedToWorkspaceAsync(
                    vm.PublishedFileId, null, _workspace, log, progress, CancellationToken.None);

                if (!outcome.Success)
                {
                    await dialogSvc.ShowMessageAsync(S.Get("Uploads_ImportFailed"), $"{vm.Title}: {outcome.Message}");
                    return;
                }

                lastPath = outcome.ProjectRoot ?? "";
            }

            await dialogSvc.ShowMessageAsync(S.Get("Uploads_Imported"), $"{selected.Count} project(s) under:\n{_workspace.WorkspaceRoot}");
            if (!string.IsNullOrEmpty(lastPath))
            {
                OpenInEditor(lastPath);
                ReloadProjects();
            }
        }
        catch (Exception ex)
        {
            await dialogSvc.ShowErrorAsync(S.Get(ErrorKey), S.Get("Uploads_ActionFailed"), ex);
        }
    }

    private async void OnDownloadClicked(object? sender, RoutedEventArgs e)
    {
        if (sender is not Button { DataContext: WorkshopItemDetailVm vm }) return;

        var suggested = $"{vm.Title}_{vm.PublishedFileId}";
        var dialog = App.Services.GetRequiredService<IDialogService>();
        var folder = await dialog.ShowPromptAsync(
            S.Get("Uploads_FolderName"),
            S.Format("Uploads_FolderPrompt", _workspace.WorkspaceRoot),
            S.Get("Import"), S.Get("Cancel"), suggested);
        if (folder is null) return;

        try
        {
            var progress = new Progress<float>(p => _log.Append($"Download {p:P0}"));
            var log = new Progress<string>(s => _log.Append(s));
            var outcome = await _steam.ImportPublishedToWorkspaceAsync(
                vm.PublishedFileId,
                string.IsNullOrWhiteSpace(folder) ? null : folder,
                _workspace, log, progress, CancellationToken.None);

            if (!outcome.Success)
            {
                await dialog.ShowMessageAsync(S.Get("Uploads_ImportFailed"), outcome.Message);
                return;
            }

            await dialog.ShowMessageAsync(S.Get("Uploads_Imported"), outcome.ProjectRoot ?? "");
            if (!string.IsNullOrEmpty(outcome.ProjectRoot))
            {
                OpenInEditor(outcome.ProjectRoot);
                ReloadProjects();
            }
        }
        catch (Exception ex)
        {
            await dialog.ShowErrorAsync(S.Get(ErrorKey), S.Get("Uploads_ActionFailed"), ex);
        }
    }

    private void OnViewOnSteam(object? sender, RoutedEventArgs e)
    {
        if (sender is Button { DataContext: WorkshopItemDetailVm vm })
        {
            _steam.OpenItemInBrowser(vm.PublishedFileId);
        }
    }

    private void OnAddUpdateClicked(object? sender, RoutedEventArgs e)
    {
        // Add Update uses the same editable project and PublishedFileId as Edit.
        // The editor's Save & Upload action then calls Steam's update workflow.
        OnEditUploadClicked(sender, e);
    }

    private async void OnEditUploadClicked(object? sender, RoutedEventArgs e)
    {
        if (sender is not Button { DataContext: WorkshopItemDetailVm vm }) return;

        try
        {
            var localProject = _workspace.FindProjectByPublishedFileId(vm.PublishedFileId);
            if (localProject is not null)
            {
                OpenInEditor(localProject.RootPath);
                return;
            }

            var log = new Progress<string>(s => _log.Append(s));
            var progress = new Progress<float>(p => _log.Append($"Edit import {p:P0}"));
            var outcome = await _steam.ImportPublishedToWorkspaceAsync(
                vm.PublishedFileId, null, _workspace, log, progress, CancellationToken.None);
            if (!outcome.Success)
            {
                var dialog = App.Services.GetRequiredService<IDialogService>();
                await dialog.ShowMessageAsync(S.Get("Uploads_ImportFailed"), outcome.Message);
                return;
            }

            if (!string.IsNullOrEmpty(outcome.ProjectRoot))
            {
                OpenInEditor(outcome.ProjectRoot);
                ReloadProjects();
            }
        }
        catch (Exception ex)
        {
            var dialog = App.Services.GetRequiredService<IDialogService>();
            await dialog.ShowErrorAsync(S.Get(ErrorKey), S.Get("Uploads_ActionFailed"), ex);
        }
    }

    #endregion
}

public sealed class ProjectItemVm
{
    public ProjectItemVm(WorkshopProject project, WorkspaceService workspace)
    {
        Name = project.Name;
        RootPath = project.RootPath;
        var meta = workspace.LoadMetadata(project.RootPath);
        Title = string.IsNullOrWhiteSpace(meta.Title) ? project.Name : meta.Title;
        Tags = meta.Tags.Count > 0 ? string.Join(", ", meta.Tags) : "";
        HasTags = meta.Tags.Count > 0;
        IsPublished = meta.PublishedFileId != 0;
        PublishedStatus = IsPublished ? "Published" : "";

        var checks = UploadDependencyChecker.Check(project.RootPath, meta);
        var errors = checks.Count(c => c.Severity == UploadCheckSeverity.Error);
        var warnings = checks.Count(c => c.Severity == UploadCheckSeverity.Warning);

        if (errors > 0)
            ReadinessText = $"{errors} error(s)";
        else if (warnings > 0)
            ReadinessText = $"Ready ({warnings} warning(s))";
        else
            ReadinessText = "Ready";

        if (errors > 0)
            ReadinessColor = "#D7383B";
        else if (warnings > 0)
            ReadinessColor = "#D7A23B";
        else
            ReadinessColor = "#61F4D8";

        // Local changes since the last successful publish ("UPDATED!").
        // Hash-based, no network. Only when a publish hash was recorded.
        HasUpdates = workspace.GetSyncState(project.RootPath) == WorkspaceService.ProjectSyncState.Modified;
    }

    public string Name { get; }
    public string RootPath { get; }
    public string Title { get; }
    public string Tags { get; }
    public bool HasTags { get; }
    public bool IsPublished { get; }
    public string PublishedStatus { get; }
    public string ReadinessText { get; }
    public string ReadinessColor { get; }
    public string EditLabel { get; } = S.Get("Projects_Edit");
    public bool HasUpdates { get; }
    public string UpdatedLabel { get; } = S.Get("Projects_UpdatedBadge");
}
