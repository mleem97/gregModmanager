#pragma warning disable CS8618

using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.Threading;
using Avalonia.VisualTree;
using GregModmanager.Localization;
using GregModmanager.Models;
using GregModmanager.Services;
using Microsoft.Extensions.DependencyInjection;
using System.Collections.ObjectModel;

namespace GregModmanager.Avalonia.Views;

public partial class MyUploadsPage : UserControl
{
    private const string ErrorKey = "Error";

    private readonly SteamWorkshopService _steam;
    private readonly WorkspaceService _workspace;
    private readonly AppLogService _log;
    private readonly ObservableCollection<WorkshopItemDetailVm> _items = new();

    private int _page = 1;
    private bool _hasMore;

    public MyUploadsPage() => InitializeComponent();

    public MyUploadsPage(SteamWorkshopService steam, WorkspaceService workspace, AppLogService log)
    {
        InitializeComponent();
        _steam = steam;
        _workspace = workspace;
        _log = log;
        UploadsList.ItemsSource = _items;
        Loaded += (_, _) => _ = LoadListAsync();
    }

    private void OnUploadsSelectionChanged(object? sender, SelectionChangedEventArgs e) => UpdateSelectionCount();

    private void UpdateSelectionCount() => UpdateSelectionCount(UploadsList, SelectionCountLabel);

    private static void UpdateSelectionCount(ListBox list, TextBlock label)
    {
        var n = list.SelectedItems?.Count ?? 0;
        label.Text = S.Format("Uploads_Selected", n);
    }

    private async void OnRefresh(object? sender, RoutedEventArgs e) => await LoadListAsync();

    private async Task LoadListAsync()
    {
        try
        {
            var result = await _steam.ListMyPublishedPagedAsync(_page, CancellationToken.None);

            _items.Clear();
            foreach (var x in result.Items) _items.Add(x);

            _hasMore = result.HasMorePages;
            PrevBtn.IsEnabled = _page > 1;
            NextBtn.IsEnabled = _hasMore;
            PageLabel.Text = S.Format("PageWithTotal", _page, result.TotalResults);

            UploadsList.SelectedItems?.Clear();
            UpdateSelectionCount();
            _log.Append($"Workshop uploads: {result.TotalResults} item(s), page {_page}.");
        }
        catch (Exception ex)
        {
            var dialog = App.Services.GetRequiredService<Services.IDialogService>();
            await dialog.ShowMessageAsync(S.Get(ErrorKey), ex.Message);
        }
    }

    private void OnPrev(object? sender, RoutedEventArgs e)
    {
        if (_page > 1) { _page--; _ = LoadListAsync(); }
    }

    private void OnNext(object? sender, RoutedEventArgs e)
    {
        if (_hasMore) { _page++; _ = LoadListAsync(); }
    }

    private async void OnImportSelected(object? sender, RoutedEventArgs e)
    {
        var selected = UploadsList.SelectedItems?.OfType<WorkshopItemDetailVm>().ToList() ?? new List<WorkshopItemDetailVm>();
        if (selected.Count == 0)
        {
            var dialog = App.Services.GetRequiredService<Services.IDialogService>();
            await dialog.ShowMessageAsync(S.Get("Import"), S.Get("Uploads_SelectFirst"));
            return;
        }

        var dialogSvc = App.Services.GetRequiredService<Services.IDialogService>();
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
                try
                {
                    var editor = App.Services.GetRequiredService<EditorPage>();
                    editor.LoadProject(lastPath);
                    if (this.GetVisualRoot() is MainWindow mw) mw.NavigateTo(editor);
                }
                catch (Exception navEx)
                {
                    AppFileLog.Error("MyUploadsPage navigation failed after bulk import", navEx);
                    await dialogSvc.ShowMessageAsync(S.Get(ErrorKey), navEx.Message);
                }
            }
        }
        catch (Exception ex)
        {
            await dialogSvc.ShowMessageAsync(S.Get(ErrorKey), ex.Message);
        }
    }

    private async void OnDownloadClicked(object? sender, RoutedEventArgs e)
    {
        if (sender is not Button { DataContext: WorkshopItemDetailVm vm }) return;

        var suggested = $"{vm.Title}_{vm.PublishedFileId}";
        var dialog = App.Services.GetRequiredService<Services.IDialogService>();
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
            try
            {
                var editor = App.Services.GetRequiredService<EditorPage>();
                editor.LoadProject(outcome.ProjectRoot!);
                if (this.GetVisualRoot() is MainWindow mw) mw.NavigateTo(editor);
            }
            catch (Exception navEx)
            {
                AppFileLog.Error("MyUploadsPage navigation failed after single import", navEx);
                await dialog.ShowMessageAsync(S.Get(ErrorKey), navEx.Message);
            }
        }
        catch (Exception ex)
        {
            await dialog.ShowMessageAsync(ErrorKey, ex.Message);
        }
    }

    private void OnViewOnSteam(object? sender, RoutedEventArgs e)
    {
        if (sender is Button { DataContext: WorkshopItemDetailVm vm })
        {
            _steam.OpenItemInBrowser(vm.PublishedFileId);
        }
    }
}
