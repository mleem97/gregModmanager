#pragma warning disable CS8618

using Avalonia;
using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.Layout;
using Avalonia.Media;
using Avalonia.Platform.Storage;
using Avalonia.Threading;
using GregModmanager.Localization;
using GregModmanager.Models;
using GregModmanager.Services;
using GregModmanager.Steam;
using Microsoft.Extensions.DependencyInjection;
using System.Collections.ObjectModel;
using System.Globalization;
using System.IO.Compression;
using System.Text;

namespace GregModmanager.Avalonia.Views;

public partial class EditorPage : UserControl
{
    private const int MaxWorkshopTags = 20;

    private readonly WorkspaceService _workspace;
    private readonly SteamWorkshopService _steam;
    private readonly AppLogService _log;
    private readonly ObservableCollection<UploadCheckResult> _checkResults = new();

    private string _projectRoot = "";
    private WorkshopMetadata _metadata = new();

    public EditorPage(WorkspaceService workspace, SteamWorkshopService steam, AppLogService log)
    {
        InitializeComponent();
        _workspace = workspace;
        _steam = steam;
        _log = log;

        VisibilityPicker.ItemsSource = new[] { "Public", "FriendsOnly", "Private" };
        NativeProfilePicker.ItemsSource = new[] { "decoration", "code" };
        CheckResultsList.ItemsSource = _checkResults;
        SetEditorTab(0);
    }

    public void LoadProject(string rootPath)
    {
        _projectRoot = rootPath;
        _ = LoadAsyncSafe();
    }

    private async Task LoadAsyncSafe()
    {
        try
        {
            await LoadAsync();
        }
        catch (Exception ex)
        {
            _log.Append($"Failed to open project: {ex.Message}");
            var dialog = App.Services.GetRequiredService<Services.IDialogService>();
            await dialog.ShowMessageAsync(S.Get("Error"), $"Could not open project. {ex.Message}");
        }
    }

    private async Task LoadAsync()
    {
        if (string.IsNullOrEmpty(_projectRoot)) return;

        Dispatcher.UIThread.Post(() =>
        {
            TitleLabel.Text = Path.GetFileName(_projectRoot.TrimEnd(Path.DirectorySeparatorChar));
            if (string.IsNullOrEmpty(TitleLabel.Text)) TitleLabel.Text = "Workshop item";
            PathLabel.Text = _projectRoot;
        });

        _metadata = _workspace.LoadMetadata(_projectRoot);

        var localSnapshot = new WorkshopMetadata
        {
            Needsgreg = _metadata.Needsgreg,
            NeedsMelonLoader = _metadata.NeedsMelonLoader,
            NativeConfigProfile = _metadata.NativeConfigProfile,
            PreviewImageRelativePath = _metadata.PreviewImageRelativePath,
            AdditionalPreviews = new List<string>(_metadata.AdditionalPreviews),
            WorkshopDependencyIds = new List<ulong>(_metadata.WorkshopDependencyIds),
        };

        if (_metadata.PublishedFileId != 0)
        {
            Dispatcher.UIThread.Post(() => SyncStatusLabel.Text = S.Get("Editor_LoadingFromSteam"));

            var steam = await _steam.GetItemDetailsAsync(_metadata.PublishedFileId, CancellationToken.None);

            if (steam is not null)
            {
                SteamWorkshopService.ApplySteamWorkshopToMetadata(steam, _metadata, localSnapshot, MaxWorkshopTags);
                try
                {
                    _workspace.SaveMetadata(_projectRoot, _metadata);
                    Dispatcher.UIThread.Post(() => SyncStatusLabel.Text = S.Get("Editor_LoadedFromSteam"));
                }
                catch (Exception ex)
                {
                    Dispatcher.UIThread.Post(() => SyncStatusLabel.Text = S.Format("Editor_SteamSaveFailed", ex.Message));
                }
            }
            else
            {
                Dispatcher.UIThread.Post(() => SyncStatusLabel.Text = S.Get("Editor_SteamRefreshFailed"));
            }
        }
        else
        {
            Dispatcher.UIThread.Post(() => SyncStatusLabel.Text = "");
        }

        Dispatcher.UIThread.Post(BindEditorFromMetadata);
    }

    private void BindEditorFromMetadata()
    {
        TitleEntry.Text = _metadata.Title;
        DescriptionEditor.Text = _metadata.Description;
        VisibilityPicker.SelectedItem = _metadata.Visibility is "Public" or "FriendsOnly" or "Private"
            ? _metadata.Visibility
            : "Public";
        TagsEntry.Text = string.Join(", ", _metadata.Tags);
        NeedsgregSwitch.IsChecked = _metadata.Needsgreg;
        NeedsMelonLoaderSwitch.IsChecked = _metadata.NeedsMelonLoader;
        var profile = string.IsNullOrWhiteSpace(_metadata.NativeConfigProfile)
            ? "decoration"
            : _metadata.NativeConfigProfile.Trim().ToLowerInvariant();
        NativeProfilePicker.SelectedItem = profile == "code" ? "code" : "decoration";
        _metadata.WorkshopDependencyIds = _metadata.WorkshopDependencyIds.Where(x => x > 0).Distinct().ToList();
        PreviewPathLabel.Text = Path.Combine(_projectRoot, _metadata.PreviewImageRelativePath);

        var isUpdate = _metadata.PublishedFileId != 0;
        PublishedIdLabel.Text = isUpdate
            ? S.Format("Editor_FileId", _metadata.PublishedFileId)
            : S.Get("Editor_NotPublished");

        ChangeLogHintLabel.Text = isUpdate
            ? S.Get("Editor_ChangeNotesHint")
            : S.Get("Editor_ChangelogRequiredHint");
        ViewOnSteamBtn.IsVisible = isUpdate;

        UpdateContentSizeUi();
        UpdateCounts();
        UpdateTagsHint();
        RebuildScreenshotGallery();
        RunUploadCheck();
        RebuildWorkshopDepRows();
        _ = EnrichWorkshopDependencyTitlesAsync();
    }

    private void UpdateContentSizeUi()
    {
        var content = Path.Combine(_projectRoot, "content");
        if (!Directory.Exists(content))
        {
            ContentStatusLabel.Text = S.Get("Editor_ContentMissing");
            ContentSizeBody.Text = "";
            return;
        }

        var st = _workspace.GetContentStats(_projectRoot);
        if (!st.Exists)
        {
            ContentStatusLabel.Text = "content/ — could not analyze.";
            ContentSizeBody.Text = "";
            return;
        }

        var fileCount = CountFilesQuick(content);
        ContentStatusLabel.Text = S.Format("Editor_ContentFiles", fileCount, WorkspaceService.FormatBytes(st.TotalBytes));

        var sb = new StringBuilder();
        foreach (var entry in st.TopEntries.Take(6))
        {
            sb.AppendLine($"  {entry.Name}  {WorkspaceService.FormatBytes(entry.Bytes)}");
        }

        ContentSizeBody.Text = sb.ToString().TrimEnd();
    }

    private static int CountFilesQuick(string dir)
    {
        var n = 0;
        foreach (var _ in Directory.EnumerateFiles(dir, "*", SearchOption.AllDirectories))
        {
            if (++n >= 5_000_000) return 5_000_000;
        }
        return n;
    }

    #region Workshop dependencies

    private void RebuildWorkshopDepRows()
    {
        WorkshopDepStack.Children.Clear();
        foreach (var id in _metadata.WorkshopDependencyIds)
        {
            var grid = new Grid { ColumnDefinitions = new ColumnDefinitions("*,Auto") };
            var lbl = new TextBlock
            {
                Text = FormatWorkshopDepLabel(id),
                FontSize = 12,
                Foreground = new SolidColorBrush(Color.Parse("#C0FCF6")),
                VerticalAlignment = VerticalAlignment.Center,
                TextTrimming = TextTrimming.CharacterEllipsis,
            };
            Grid.SetColumn(lbl, 0);
            grid.Children.Add(lbl);
            var rm = new Button
            {
                Content = S.Get("Editor_WorkshopDepRemove"),
                CommandParameter = id,
            };
            rm.Click += OnRemoveWorkshopDepClicked;
            Grid.SetColumn(rm, 1);
            grid.Children.Add(rm);
            WorkshopDepStack.Children.Add(grid);
        }
    }

    private string FormatWorkshopDepLabel(ulong id)
    {
        return id.ToString(CultureInfo.InvariantCulture);
    }

    private async Task EnrichWorkshopDependencyTitlesAsync()
    {
        foreach (var id in _metadata.WorkshopDependencyIds.ToList())
        {
            var item = await _steam.GetItemDetailsAsync(id, CancellationToken.None).ConfigureAwait(false);
            if (item is null) continue;

            Dispatcher.UIThread.Post(RebuildWorkshopDepRows);
        }
    }

    private async void OnWorkshopDepAddById(object? sender, RoutedEventArgs e)
    {
        var s = WorkshopDepIdEntry.Text?.Trim() ?? "";
        if (!ulong.TryParse(s, NumberStyles.Integer, CultureInfo.InvariantCulture, out var id) || id == 0)
        {
            var dialog = App.Services.GetRequiredService<Services.IDialogService>();
            await dialog.ShowMessageAsync(S.Get("Error"), S.Get("Editor_WorkshopDepInvalidId"));
            return;
        }

        if (!await TryAddWorkshopDependencyAsync(id, null))
        {
            return;
        }

        WorkshopDepIdEntry.Text = "";
    }

    private async Task<bool> TryAddWorkshopDependencyAsync(ulong id, string? titleHint)
    {
        if (_metadata.PublishedFileId != 0 && id == _metadata.PublishedFileId)
        {
            var dialog = App.Services.GetRequiredService<Services.IDialogService>();
            await dialog.ShowMessageAsync(S.Get("Error"), S.Get("Editor_WorkshopDepSelf"));
            return false;
        }

        if (_metadata.WorkshopDependencyIds.Contains(id))
        {
            var dialog = App.Services.GetRequiredService<Services.IDialogService>();
            await dialog.ShowMessageAsync(S.Get("Error"), S.Get("Editor_WorkshopDepDuplicate"));
            return false;
        }

        _metadata.WorkshopDependencyIds.Add(id);
        RebuildWorkshopDepRows();
        RunUploadCheck();
        return true;
    }

    private void OnRemoveWorkshopDepClicked(object? sender, RoutedEventArgs e)
    {
        if (sender is not Button b || b.CommandParameter is null) return;

        var id = Convert.ToUInt64(b.CommandParameter, CultureInfo.InvariantCulture);
        _metadata.WorkshopDependencyIds.Remove(id);
        RebuildWorkshopDepRows();
        RunUploadCheck();
    }

    #endregion

    #region Upload Dependency Checker

    private void RunUploadCheck()
    {
        ApplyMetadataFromUi();
        var results = UploadDependencyChecker.Check(_projectRoot, _metadata, ChangeLogEditor.Text);
        _checkResults.Clear();
        foreach (var r in results) _checkResults.Add(r);

        var ready = UploadDependencyChecker.IsReadyToUpload(results);
        ReadinessStatusLabel.Text = ready ? S.Get("Editor_ReadyToUpload") : S.Get("Editor_IssuesFound");
        ReadinessStatusLabel.Foreground = ready
            ? new SolidColorBrush(Color.Parse("#61F4D8"))
            : new SolidColorBrush(Color.Parse("#D7383B"));
    }

    private void OnRunUploadCheck(object? sender, RoutedEventArgs e) => RunUploadCheck();

    #endregion

    #region Editor sub-tabs

    private void OnTabDetails(object? sender, RoutedEventArgs e) => SetEditorTab(0);
    private void OnTabAssets(object? sender, RoutedEventArgs e) => SetEditorTab(1);
    private void OnTabPublish(object? sender, RoutedEventArgs e) => SetEditorTab(2);

    private void SetEditorTab(int index)
    {
        PanelDetails.IsVisible = index == 0;
        PanelAssets.IsVisible = index == 1;
        PanelPublish.IsVisible = index == 2;
        SetEditorTabStyle(TabBtnDetails, index == 0);
        SetEditorTabStyle(TabBtnAssets, index == 1);
        SetEditorTabStyle(TabBtnPublish, index == 2);
    }

    private static void SetEditorTabStyle(Button btn, bool active)
    {
        if (active)
        {
            btn.Background = new SolidColorBrush(Color.Parse("#61F4D8"));
            btn.Foreground = new SolidColorBrush(Color.Parse("#001110"));
        }
        else
        {
            btn.Background = Brushes.Transparent;
            btn.Foreground = new SolidColorBrush(Color.Parse("#61F4D8"));
        }
    }

    private void OnNeedsgregToggled(object? sender, RoutedEventArgs e) => RunUploadCheck();
    private void OnNeedsMelonLoaderToggled(object? sender, RoutedEventArgs e) => RunUploadCheck();
    private void OnNativeProfileChanged(object? sender, SelectionChangedEventArgs e) => RunUploadCheck();
    private void OnChangeLogTextChanged(object? sender, TextChangedEventArgs e) => RunUploadCheck();

    #endregion

    #region Field events

    private void OnTitleChanged(object? sender, TextChangedEventArgs e) => UpdateCounts();

    private void OnDescriptionChanged(object? sender, TextChangedEventArgs e)
    {
        UpdateCounts();
    }

    private void OnTagsChanged(object? sender, TextChangedEventArgs e) => UpdateTagsHint();

    private void UpdateCounts()
    {
        var t = TitleEntry.Text?.Length ?? 0;
        var d = DescriptionEditor.Text?.Length ?? 0;
        TitleCountLabel.Text = $"{t} / {SteamConstants.MaxTitleLength}";
        DescriptionCountLabel.Text = $"{d} / {SteamConstants.MaxDescriptionLength}";
    }

    private void UpdateTagsHint()
    {
        var raw = TagsEntry.Text ?? "";
        var count = ParseTags(raw).Count;
        TagsHintLabel.Text = $"{count} / {MaxWorkshopTags} tags";
    }

    private static List<string> ParseTags(string? raw)
    {
        if (string.IsNullOrWhiteSpace(raw)) return new List<string>();
        return raw
            .Split(',', StringSplitOptions.RemoveEmptyEntries)
            .Select(s => s.Trim())
            .Where(s => s.Length > 0)
            .Take(MaxWorkshopTags)
            .ToList();
    }

    #endregion

    #region BBCode formatting

    private void OnBbBold(object? s, RoutedEventArgs e) => InsertBbTag("b");
    private void OnBbItalic(object? s, RoutedEventArgs e) => InsertBbTag("i");
    private void OnBbUnderline(object? s, RoutedEventArgs e) => InsertBbTag("u");
    private void OnBbStrike(object? s, RoutedEventArgs e) => InsertBbTag("strike");
    private void OnBbH1(object? s, RoutedEventArgs e) => InsertBbTag("h1");
    private void OnBbH2(object? s, RoutedEventArgs e) => InsertBbTag("h2");
    private void OnBbH3(object? s, RoutedEventArgs e) => InsertBbTag("h3");
    private void OnBbCode(object? s, RoutedEventArgs e) => InsertBbTag("code");
    private void OnBbQuote(object? s, RoutedEventArgs e) => InsertBbTag("quote");
    private void OnBbSpoiler(object? s, RoutedEventArgs e) => InsertBbTag("spoiler");

    private void OnBbUrl(object? s, RoutedEventArgs e)
    {
        var text = DescriptionEditor.Text ?? "";
        var start = DescriptionEditor.SelectionStart;
        var end = DescriptionEditor.SelectionEnd;
        var selLen = Math.Abs(end - start);
        var cursor = Math.Min(start, end);

        if (selLen > 0 && cursor + selLen <= text.Length)
        {
            var selected = text.Substring(cursor, selLen);
            var insert = $"[url={selected}]{selected}[/url]";
            DescriptionEditor.Text = text.Remove(cursor, selLen).Insert(cursor, insert);
            DescriptionEditor.CaretIndex = cursor + insert.Length;
        }
        else
        {
            var insert = "[url=https://]link text[/url]";
            DescriptionEditor.Text = text.Insert(cursor, insert);
            DescriptionEditor.CaretIndex = cursor + 5;
        }
    }

    private void OnBbImg(object? s, RoutedEventArgs e)
    {
        var text = DescriptionEditor.Text ?? "";
        var cursor = DescriptionEditor.CaretIndex;
        var insert = "[img]https://[/img]";
        DescriptionEditor.Text = text.Insert(cursor, insert);
        DescriptionEditor.CaretIndex = cursor + 5;
    }

    private void OnBbList(object? s, RoutedEventArgs e)
    {
        var text = DescriptionEditor.Text ?? "";
        var cursor = DescriptionEditor.CaretIndex;
        var insert = "[list]\n[*] Item 1\n[*] Item 2\n[/list]";
        DescriptionEditor.Text = text.Insert(cursor, insert);
        DescriptionEditor.CaretIndex = cursor + insert.Length;
    }

    private void OnBbHr(object? s, RoutedEventArgs e)
    {
        var text = DescriptionEditor.Text ?? "";
        var cursor = DescriptionEditor.CaretIndex;
        var insert = "[hr][/hr]";
        DescriptionEditor.Text = text.Insert(cursor, insert);
        DescriptionEditor.CaretIndex = cursor + insert.Length;
    }

    private void OnBbTable(object? s, RoutedEventArgs e)
    {
        var text = DescriptionEditor.Text ?? "";
        var cursor = DescriptionEditor.CaretIndex;
        var insert = "[table]\n[tr]\n[th]Header[/th]\n[th]Header[/th]\n[/tr]\n[tr]\n[td]Cell[/td]\n[td]Cell[/td]\n[/tr]\n[/table]";
        DescriptionEditor.Text = text.Insert(cursor, insert);
        DescriptionEditor.CaretIndex = cursor + insert.Length;
    }

    private void InsertBbTag(string tag)
    {
        var text = DescriptionEditor.Text ?? "";
        var start = DescriptionEditor.SelectionStart;
        var end = DescriptionEditor.SelectionEnd;
        var selLen = Math.Abs(end - start);
        var cursor = Math.Min(start, end);

        if (selLen > 0 && cursor + selLen <= text.Length)
        {
            var selected = text.Substring(cursor, selLen);
            var wrapped = $"[{tag}]{selected}[/{tag}]";
            DescriptionEditor.Text = text.Remove(cursor, selLen).Insert(cursor, wrapped);
            DescriptionEditor.CaretIndex = cursor + wrapped.Length;
        }
        else
        {
            var open = $"[{tag}]";
            var close = $"[/{tag}]";
            DescriptionEditor.Text = text.Insert(cursor, open + close);
            DescriptionEditor.CaretIndex = cursor + open.Length;
        }
    }

    #endregion

    #region Screenshots

    private void RebuildScreenshotGallery()
    {
        ScreenshotsGallery.Children.Clear();
        foreach (var relPath in _metadata.AdditionalPreviews)
        {
            var absPath = Path.Combine(_projectRoot, relPath);
            var grid = new Grid { ColumnDefinitions = new ColumnDefinitions("*,Auto") };
            var lbl = new TextBlock
            {
                Text = relPath,
                FontSize = 12,
                Foreground = new SolidColorBrush(Color.Parse("#C0FCF6")),
                VerticalAlignment = VerticalAlignment.Center,
                TextTrimming = TextTrimming.CharacterEllipsis,
            };
            Grid.SetColumn(lbl, 0);
            grid.Children.Add(lbl);
            var rm = new Button
            {
                Content = "Remove",
                CommandParameter = relPath,
            };
            rm.Click += OnRemoveScreenshot;
            Grid.SetColumn(rm, 1);
            grid.Children.Add(rm);
            ScreenshotsGallery.Children.Add(grid);
        }
        UpdateScreenshotCount();
    }

    private void UpdateScreenshotCount()
    {
        ScreenshotCountLabel.Text = $"{_metadata.AdditionalPreviews.Count} / 9";
    }

    private async void OnAddScreenshot(object? sender, RoutedEventArgs e)
    {
        if (_metadata.AdditionalPreviews.Count >= 9)
        {
            var dialog = App.Services.GetRequiredService<Services.IDialogService>();
            await dialog.ShowMessageAsync(S.Get("Editor_Screenshots"), S.Get("Editor_MaxScreenshots"));
            return;
        }

        var topLevel = TopLevel.GetTopLevel(this);
        if (topLevel == null) return;

        var files = await topLevel.StorageProvider.OpenFilePickerAsync(new FilePickerOpenOptions
        {
            Title = S.Get("Editor_ScreenshotPicker"),
            AllowMultiple = false,
            FileTypeFilter = new[]
            {
                new FilePickerFileType("Images")
                {
                    Patterns = new[] { "*.png", "*.jpg", "*.jpeg", "*.gif", "*.webp" }
                }
            }
        });

        if (files.Count == 0) return;
        var path = files[0].TryGetLocalPath();
        if (path is null) return;

        var ext = Path.GetExtension(path)?.ToLowerInvariant() ?? ".png";
        if (ext is not (".png" or ".jpg" or ".jpeg" or ".gif" or ".webp")) ext = ".png";

        var screenshotsDir = Path.Combine(_projectRoot, "screenshots");
        Directory.CreateDirectory(screenshotsDir);

        var fileName = $"screenshot_{_metadata.AdditionalPreviews.Count + 1}_{DateTime.Now:HHmmss}{ext}";
        var dest = Path.Combine(screenshotsDir, fileName);
        File.Copy(path, dest, true);

        var relPath = Path.Combine("screenshots", fileName);
        _metadata.AdditionalPreviews.Add(relPath);

        RebuildScreenshotGallery();
        _log.Append($"Added screenshot: {relPath}");
    }

    private void OnRemoveScreenshot(object? sender, RoutedEventArgs e)
    {
        if (sender is not Button { CommandParameter: string relPath }) return;

        _metadata.AdditionalPreviews.Remove(relPath);
        RebuildScreenshotGallery();
        _log.Append($"Removed screenshot: {relPath}");
    }

    #endregion

    #region Actions

    private async void OnReloadFromDisk(object? sender, RoutedEventArgs e)
    {
        await LoadAsync();
        _log.Append($"Reloaded from disk: {_projectRoot}");
    }

    private void OnOpenInExplorer(object? sender, RoutedEventArgs e)
    {
        if (string.IsNullOrEmpty(_projectRoot)) return;
        SafeProcess.OpenFolder(_projectRoot);
    }

    private async void OnExportContentZip(object? sender, RoutedEventArgs e)
    {
        var content = Path.Combine(_projectRoot, "content");
        if (!Directory.Exists(content))
        {
            var dialog = App.Services.GetRequiredService<Services.IDialogService>();
            await dialog.ShowMessageAsync(S.Get("Editor_ExportZip"), S.Get("Editor_ContentMissing"));
            return;
        }

        try
        {
            var zipName = $"content-export-{DateTime.Now:yyyyMMdd-HHmmss}.zip";
            var zipPath = Path.Combine(_projectRoot, zipName);
            if (File.Exists(zipPath)) File.Delete(zipPath);

            ZipFile.CreateFromDirectory(content, zipPath, CompressionLevel.Fastest, false);
            _log.Append($"Exported {zipPath}");
            var dialog = App.Services.GetRequiredService<Services.IDialogService>();
            await dialog.ShowMessageAsync(S.Get("Editor_Exported"), zipPath);
        }
        catch (Exception ex)
        {
            var dialog = App.Services.GetRequiredService<Services.IDialogService>();
            await dialog.ShowMessageAsync(S.Get("Editor_ExportFailed"), ex.Message);
        }
    }

    private async void OnPickPreview(object? sender, RoutedEventArgs e)
    {
        var topLevel = TopLevel.GetTopLevel(this);
        if (topLevel == null) return;

        var files = await topLevel.StorageProvider.OpenFilePickerAsync(new FilePickerOpenOptions
        {
            Title = S.Get("Editor_PreviewPicker"),
            AllowMultiple = false,
            FileTypeFilter = new[]
            {
                new FilePickerFileType("Images")
                {
                    Patterns = new[] { "*.png", "*.jpg", "*.jpeg", "*.gif", "*.webp" }
                }
            }
        });

        if (files.Count == 0) return;
        var path = files[0].TryGetLocalPath();
        if (path is null) return;

        var ext = Path.GetExtension(path)?.ToLowerInvariant() ?? ".png";
        if (ext is not (".png" or ".jpg" or ".jpeg" or ".gif" or ".webp")) ext = ".png";

        var fileName = $"preview{ext}";
        var dest = Path.Combine(_projectRoot, fileName);
        File.Copy(path, dest, true);

        _metadata.PreviewImageRelativePath = fileName;
        PreviewPathLabel.Text = dest;
        _log.Append($"Preview saved: {dest}");
        RunUploadCheck();
    }

    private void ApplyMetadataFromUi()
    {
        _metadata.Title = TitleEntry.Text ?? "";
        _metadata.Description = DescriptionEditor.Text ?? "";
        _metadata.Visibility = VisibilityPicker.SelectedItem as string ?? "Public";
        _metadata.Tags = ParseTags(TagsEntry.Text);
        _metadata.Needsgreg = NeedsgregSwitch.IsChecked == true;
        _metadata.NeedsMelonLoader = NeedsMelonLoaderSwitch.IsChecked == true;
        _metadata.NativeConfigProfile = NativeProfilePicker.SelectedItem as string ?? "decoration";
        _metadata.WorkshopDependencyIds = _metadata.WorkshopDependencyIds.Where(x => x > 0).Distinct().ToList();
    }

    private static string BuildUploadDescription(WorkshopMetadata meta)
    {
        var desc = meta.Description ?? "";
        if (meta.NeedsMelonLoader && !desc.Contains("MelonLoader", StringComparison.OrdinalIgnoreCase))
        {
            desc += "\n\n---\n" + S.Get("Editor_MelonLoaderNotice");
        }

        if (meta.Needsgreg && !desc.Contains("gregCoreModFramework", StringComparison.OrdinalIgnoreCase))
        {
            desc += "\n\n---\n" + S.Get("Editor_gregNotice");
        }

        return desc;
    }

    private async void OnSave(object? sender, RoutedEventArgs e)
    {
        try
        {
            ApplyMetadataFromUi();
            _workspace.SaveMetadata(_projectRoot, _metadata);
            _log.Append($"Saved metadata for {_projectRoot}");
            RunUploadCheck();
            var dialog = App.Services.GetRequiredService<Services.IDialogService>();
            await dialog.ShowMessageAsync(S.Get("Editor_Saved"), S.Get("Editor_MetaUpdated"));
        }
        catch (Exception ex)
        {
            var dialog = App.Services.GetRequiredService<Services.IDialogService>();
            await dialog.ShowMessageAsync(S.Get("Error"), ex.Message);
        }
    }

    private async void OnSaveAndUpload(object? sender, RoutedEventArgs e)
    {
        try
        {
            ApplyMetadataFromUi();
            RunUploadCheck();

            var checks = UploadDependencyChecker.Check(_projectRoot, _metadata, ChangeLogEditor.Text);
            if (!UploadDependencyChecker.IsReadyToUpload(checks))
            {
                var dialog = App.Services.GetRequiredService<Services.IDialogService>();
                await dialog.ShowMessageAsync(S.Get("Editor_NotReady"), S.Get("Editor_NotReadyMsg"));
                return;
            }

            _workspace.SaveMetadata(_projectRoot, _metadata);

            var content = Path.Combine(_projectRoot, "content");
            SyncStatusLabel.Text = S.Get("Editor_Uploading");
            var changeLog = ChangeLogEditor.Text;

            var originalDesc = _metadata.Description;
            _metadata.Description = BuildUploadDescription(_metadata);

            var upload = new Progress<float>(p =>
            {
                Dispatcher.UIThread.Post(() =>
                    SyncStatusLabel.Text = S.Format("Editor_UploadProgress", p));
            });
            var log = new Progress<string>(s => _log.Append(s));

            var outcome = await _steam.PublishAsync(
                _projectRoot, _metadata, content, changeLog,
                upload, log, CancellationToken.None);

            _metadata.Description = originalDesc;

            if (!outcome.Success)
            {
                SyncStatusLabel.Text = S.Format("Editor_PublishFailed", outcome.Message);
                var dialog = App.Services.GetRequiredService<Services.IDialogService>();
                await dialog.ShowMessageAsync(S.Get("Error"), outcome.Message);
                return;
            }

            _workspace.SaveMetadata(_projectRoot, _metadata);
            PublishedIdLabel.Text = S.Format("Editor_FileId", _metadata.PublishedFileId);
            ChangeLogHintLabel.Text = S.Get("Editor_ChangeNotesHint");
            ViewOnSteamBtn.IsVisible = true;

            if (_metadata.AdditionalPreviews.Count > 0 && SteamUgcPreviews.IsAvailable)
            {
                SyncStatusLabel.Text = S.Get("Editor_UploadingScreenshots");
                var absPaths = _metadata.AdditionalPreviews
                    .Select(p => Path.Combine(_projectRoot, p))
                    .ToList();
                await SteamUgcPreviews.UploadAdditionalPreviewsAsync(
                    _metadata.PublishedFileId, absPaths, log, CancellationToken.None);
            }

            SyncStatusLabel.Text = S.Get("Editor_PublishedSyncing");

            var synced = await _steam.SyncAfterPublishAsync(
                _metadata.PublishedFileId, _projectRoot, _metadata, _workspace, log, CancellationToken.None);

            SyncStatusLabel.Text = synced
                ? S.Get("Editor_SyncComplete")
                : S.Get("Editor_SyncIncomplete");

            PreviewPathLabel.Text = Path.Combine(_projectRoot, _metadata.PreviewImageRelativePath);
            RebuildScreenshotGallery();
            UpdateContentSizeUi();
            RunUploadCheck();

            var dialog2 = App.Services.GetRequiredService<Services.IDialogService>();
            await dialog2.ShowMessageAsync(S.Get("Editor_Publish"),
                S.Format("Editor_FileId", _metadata.PublishedFileId) + "\n\n" +
                (synced ? S.Get("Editor_SyncComplete") : S.Get("Editor_SyncIncomplete")));
        }
        catch (Exception ex)
        {
            SyncStatusLabel.Text = "";
            var dialog = App.Services.GetRequiredService<Services.IDialogService>();
            await dialog.ShowMessageAsync(S.Get("Error"), ex.Message);
        }
    }

    private void OnViewOnSteam(object? sender, RoutedEventArgs e)
    {
        if (_metadata.PublishedFileId != 0)
            _steam.OpenItemInBrowser(_metadata.PublishedFileId);
    }

    #endregion
}
