#pragma warning disable CS8618

using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Interactivity;
using Avalonia.Media;
using Avalonia.Threading;
using Avalonia.VisualTree;
using GregModmanager.Localization;
using GregModmanager.Models;
using GregModmanager.Services;
using Microsoft.Extensions.DependencyInjection;
using System.Collections.ObjectModel;

namespace GregModmanager.Avalonia.Views;

public partial class ModManagerPage : UserControl
{
    private readonly SteamWorkshopService _steam;
    private readonly ModDependencyService _deps;
    private readonly gregPluginChannelRegistry _channels;
    private readonly AppLogService _log;
    private readonly WorkshopSyncOrchestrator _syncOrchestrator;

    private readonly ObservableCollection<DependencyCheckResult> _checks = new();
    private readonly ObservableCollection<PluginPackageInfo> _plugins = new();
    private readonly ObservableCollection<WorkshopItemDetailVm> _storeItems = new();
    private readonly ObservableCollection<WorkshopItemDetailVm> _installedItems = new();
    private readonly ObservableCollection<WorkshopItemDetailVm> _favoritesItems = new();

    private const string TabStoreKey = "store";
    private const string TabInstalledKey = "installed";
    private const string TabFavoritesKey = "favorites";
    private const string TabHealthKey = "health";

    private int _storePage = 1;
    private bool _storeHasMore;
    private int _installedPage = 1;
    private bool _installedHasMore;
    private int _favoritesPage = 1;
    private bool _favoritesHasMore;
    private string _currentTab = TabStoreKey;

    public ModManagerPage() => InitializeComponent();

    public ModManagerPage(
        SteamWorkshopService steam,
        ModDependencyService deps,
        gregPluginChannelRegistry channels,
        AppLogService log,
        WorkshopSyncOrchestrator syncOrchestrator)
    {
        InitializeComponent();
        _steam = steam;
        _deps = deps;
        _channels = channels;
        _log = log;
        _syncOrchestrator = syncOrchestrator;

        ChecksList.ItemsSource = _checks;
        PluginsList.ItemsSource = _plugins;
        StoreList.ItemsSource = _storeItems;
        InstalledList.ItemsSource = _installedItems;
        FavoritesList.ItemsSource = _favoritesItems;

        SortPicker.ItemsSource = new[] { "Update Date", "Creation Date", "Vote Score", "Trending", "Subscriptions", "Title A-Z" };
        TagFilter.ItemsSource = new[] { "All", "Mod", "Map", "Tool", "Audio", "Texture" };
        ChannelPicker.ItemsSource = new[] { "stable", "beta" };

        SortPicker.SelectedIndex = 0;
        TagFilter.SelectedIndex = 0;
        ChannelPicker.SelectedIndex = 0;

        _syncOrchestrator.StatusChanged += OnSyncStatusChanged;
        Loaded += (_, _) =>
        {
            _syncOrchestrator.Start();
            _ = LoadStoreAsync();
        };
    }

    private static void OnSyncStatusChanged(WorkshopSyncEvent evt)
    {
        if (Application.Current?.ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop 
            && desktop.MainWindow?.Content is ModManagerPage page)
        {
            OnSyncStatusChanged(evt, page.SyncStatusBar, page.SyncStatusLabel);
        }
    }

    private static void OnSyncStatusChanged(WorkshopSyncEvent evt, Border bar, TextBlock label)
    {
        Dispatcher.UIThread.Post(() =>
        {
            bar.IsVisible = true;
            label.Text = evt.Message;
            bar.Background = evt.Kind switch
            {
                "warning" => new SolidColorBrush(Color.Parse("#2A1A08")),
                "complete" or "removed" => new SolidColorBrush(Color.Parse("#0D3835")),
                _ => new SolidColorBrush(Color.Parse("#001E1C")),
            };
        });
    }

    private void OnTabStore(object? sender, RoutedEventArgs e) => SwitchToTab(TabStoreKey);
    private void OnTabInstalled(object? sender, RoutedEventArgs e) => SwitchToTab(TabInstalledKey);
    private void OnTabFavorites(object? sender, RoutedEventArgs e) => SwitchToTab(TabFavoritesKey);
    private void OnTabHealth(object? sender, RoutedEventArgs e) => SwitchToTab(TabHealthKey);

    private void OnRefreshCurrentTab(object? sender, RoutedEventArgs e)
    {
        switch (_currentTab)
        {
            case TabStoreKey: _ = LoadStoreAsync(); break;
            case TabInstalledKey: _ = LoadInstalledAsync(); break;
            case TabFavoritesKey: _ = LoadFavoritesAsync(); break;
            case TabHealthKey: RefreshChecks(); RefreshPluginList(); break;
        }
    }

    private void SwitchToTab(string tab)
    {
        _currentTab = tab;
        StorePanel.IsVisible = tab == TabStoreKey;
        InstalledPanel.IsVisible = tab == TabInstalledKey;
        FavoritesPanel.IsVisible = tab == TabFavoritesKey;
        HealthPanel.IsVisible = tab == TabHealthKey;

        SetTabActive(TabStore, tab == TabStoreKey);
        SetTabActive(TabInstalled, tab == TabInstalledKey);
        SetTabActive(TabFavorites, tab == TabFavoritesKey);
        SetTabActive(TabHealth, tab == TabHealthKey);

        switch (tab)
        {
            case TabStoreKey: _ = LoadStoreAsync(); break;
            case TabInstalledKey: _ = LoadInstalledAsync(); break;
            case TabFavoritesKey: _ = LoadFavoritesAsync(); break;
            case TabHealthKey: RefreshChecks(); RefreshPluginList(); break;
        }
    }

    private static void SetTabActive(Button btn, bool active)
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

    private async Task LoadStoreAsync()
    {
        StoreStatusLabel.Text = S.Get("Loading");

        var searchText = SearchEntry.Text;
        WorkshopBrowseResultVm result;

        if (!string.IsNullOrWhiteSpace(searchText))
        {
            result = await _steam.SearchAsync(searchText, _storePage, CancellationToken.None);
        }
        else
        {
            var sort = GetSelectedSort(SortPicker);
            var tag = GetSelectedTag(TagFilter);
            result = await _steam.BrowseAsync(_storePage, sort, tag, CancellationToken.None);
        }

        _storeItems.Clear();
        foreach (var item in result.Items) _storeItems.Add(item);

        _storeHasMore = result.HasMorePages;
        StorePrevBtn.IsEnabled = _storePage > 1;
        StoreNextBtn.IsEnabled = _storeHasMore;
        StorePageLabel.Text = S.Format("PageWithTotal", _storePage, result.TotalResults);
        StoreStatusLabel.Text = result.Items.Count == 0 ? S.Get("Mod_NoItems") : "";
    }

    private void OnSearchSubmit(object? sender, RoutedEventArgs e)
    {
        _storePage = 1;
        _ = LoadStoreAsync();
    }

    private void OnFilterChanged(object? sender, SelectionChangedEventArgs e)
    {
        _storePage = 1;
        _ = LoadStoreAsync();
    }

    private void OnStorePrev(object? sender, RoutedEventArgs e)
    {
        if (_storePage > 1) { _storePage--; _ = LoadStoreAsync(); }
    }

    private void OnStoreNext(object? sender, RoutedEventArgs e)
    {
        if (_storeHasMore) { _storePage++; _ = LoadStoreAsync(); }
    }

    private async void OnQuickSubscribe(object? sender, RoutedEventArgs e)
    {
        if (sender is not Button { DataContext: WorkshopItemDetailVm vm }) return;
        var ok = await _steam.SubscribeAsync(vm.PublishedFileId);
        if (ok)
        {
            _log.Append($"Subscribed to {vm.Title}");
            if (sender is Button btn) btn.Content = S.Get("Mod_Subscribed");
        }
    }

    private void OnStoreItemTapped(object? sender, RoutedEventArgs e)
    {
        if (sender is not Button { DataContext: WorkshopItemDetailVm vm }) return;
        var detail = App.Services.GetRequiredService<ItemDetailPage>();
        detail.LoadItem(vm.PublishedFileId);
        if (this.GetVisualRoot() is MainWindow mw) mw.NavigateTo(detail);
    }

    private static WorkshopSortMode GetSelectedSort(ComboBox picker)
    {
        return picker.SelectedIndex switch
        {
            1 => WorkshopSortMode.CreationDate,
            2 => WorkshopSortMode.VoteScore,
            3 => WorkshopSortMode.Trending,
            4 => WorkshopSortMode.Subscriptions,
            5 => WorkshopSortMode.TitleAsc,
            _ => WorkshopSortMode.UpdateDate,
        };
    }

    private static string? GetSelectedTag(ComboBox picker)
    {
        if (picker.SelectedIndex <= 0) return null;
        return picker.SelectedItem as string;
    }

    private async Task LoadInstalledAsync()
    {
        InstalledStatusLabel.Text = S.Get("Mod_LoadingSubscribed");
        var result = await _steam.ListSubscribedAsync(_installedPage, CancellationToken.None);

        _installedItems.Clear();
        foreach (var item in result.Items) _installedItems.Add(item);

        _installedHasMore = result.HasMorePages;
        InstalledPrevBtn.IsEnabled = _installedPage > 1;
        InstalledNextBtn.IsEnabled = _installedHasMore;
        InstalledPageLabel.Text = S.Format("PageWithTotal", _installedPage, result.TotalResults);
        InstalledStatusLabel.Text = result.Items.Count == 0 ? S.Get("Mod_NoSubscribed") : S.Format("Mod_SubscribedCount", result.TotalResults);
    }

    private void OnInstalledPrev(object? sender, RoutedEventArgs e)
    {
        if (_installedPage > 1) { _installedPage--; _ = LoadInstalledAsync(); }
    }

    private void OnInstalledNext(object? sender, RoutedEventArgs e)
    {
        if (_installedHasMore) { _installedPage++; _ = LoadInstalledAsync(); }
    }

    private async void OnQuickUnsubscribe(object? sender, RoutedEventArgs e)
    {
        if (sender is not Button { DataContext: WorkshopItemDetailVm vm }) return;
        var ok = await _steam.UnsubscribeAsync(vm.PublishedFileId);
        if (ok)
        {
            _log.Append($"Unsubscribed from {vm.Title}");
            _installedItems.Remove(vm);
        }
    }

    private async Task LoadFavoritesAsync()
    {
        FavoritesStatusLabel.Text = S.Get("Mod_LoadingFavorites");
        var result = await _steam.ListFavoritedAsync(_favoritesPage, CancellationToken.None);

        _favoritesItems.Clear();
        foreach (var item in result.Items) _favoritesItems.Add(item);

        _favoritesHasMore = result.HasMorePages;
        FavoritesPrevBtn.IsEnabled = _favoritesPage > 1;
        FavoritesNextBtn.IsEnabled = _favoritesHasMore;
        FavoritesPageLabel.Text = S.Format("PageWithTotal", _favoritesPage, result.TotalResults);
        FavoritesStatusLabel.Text = result.Items.Count == 0 ? S.Get("Mod_NoFavorites") : S.Format("Mod_FavoritedCount", result.TotalResults);
    }

    private void OnFavoritesPrev(object? sender, RoutedEventArgs e)
    {
        if (_favoritesPage > 1) { _favoritesPage--; _ = LoadFavoritesAsync(); }
    }

    private void OnFavoritesNext(object? sender, RoutedEventArgs e)
    {
        if (_favoritesHasMore) { _favoritesPage++; _ = LoadFavoritesAsync(); }
    }

    private async void OnQuickUnfavorite(object? sender, RoutedEventArgs e)
    {
        if (sender is not Button { DataContext: WorkshopItemDetailVm vm }) return;
        var ok = await _steam.RemoveFavoriteAsync(vm.PublishedFileId);
        if (ok)
        {
            _log.Append($"Removed {vm.Title} from favorites");
            _favoritesItems.Remove(vm);
        }
    }

    private void OnRefreshChecks(object? sender, RoutedEventArgs e)
    {
        _deps.InvalidateCache();
        RefreshChecks();
    }

    private void RefreshChecks()
    {
        _checks.Clear();
        var results = _deps.RunChecks();
        foreach (var r in results) _checks.Add(r);

        UpdateMelonStatus();
    }

    private void UpdateMelonStatus()
    {
        var mlCheck = _checks.FirstOrDefault(c => c.Label == "MelonLoader");
        if (mlCheck is null)
        {
            MelonStatusLabel.Text = S.Get("Mod_MelonUnknown");
            return;
        }

        MelonStatusLabel.Text = mlCheck.Status switch
        {
            DependencyStatus.Ok => S.Format("Mod_MelonInstalled", mlCheck.Detail),
            _ => mlCheck.Detail,
        };
    }

    private static void OnOpenFolder(object? sender, RoutedEventArgs e)
    {
        var path = (sender as Button)?.CommandParameter as string;
        if (string.IsNullOrEmpty(path)) return;

        if (File.Exists(path))
            path = Path.GetDirectoryName(path) ?? path;
        if (!Directory.Exists(path))
        {
            var parent = Path.GetDirectoryName(path);
            if (!string.IsNullOrEmpty(parent) && Directory.Exists(parent))
                path = parent;
        }

        SafeProcess.OpenFolder(path);
    }

    private static void OnMelonLoaderDownload(object? sender, RoutedEventArgs e)
    {
        _ = SafeProcess.OpenUrlAsync(AppSettings.MelonLoaderReleasesUrl);
    }

    private void OnOpenGameFolder(object? sender, RoutedEventArgs e)
    {
        var root = _deps.GameRoot;
        if (string.IsNullOrEmpty(root) || !Directory.Exists(root)) return;

        SafeProcess.OpenFolder(root);
    }

    private void OnChannelChanged(object? sender, SelectionChangedEventArgs e) => RefreshPluginList();

    private void RefreshPluginList()
    {
        _plugins.Clear();
        var channelName = ChannelPicker.SelectedIndex == 1 ? "beta" : "stable";
        var source = _channels.GetSource(channelName);

        if (source is null)
        {
            ChannelInfoLabel.Text = channelName == "beta"
                ? S.Get("Mod_BetaNotConfigured")
                : S.Get("Mod_StableInfo");
            return;
        }

        ChannelInfoLabel.Text = channelName == "beta"
            ? S.Get("Mod_BetaInfo")
            : S.Get("Mod_StableInfo");

        try
        {
            var list = source.ListPlugins();
            foreach (var p in list)
                _plugins.Add(p);
        }
        catch (NotImplementedException)
        {
            ChannelInfoLabel.Text = S.Format("Mod_ChannelNotImpl", channelName);
        }
        catch (Exception ex)
        {
            ChannelInfoLabel.Text = $"Error: {ex.Message}";
        }
    }
}
