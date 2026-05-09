#pragma warning disable CS8618

using Avalonia;
using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.Media;
using GregModmanager.Localization;
using GregModmanager.Models;
using GregModmanager.Services;
using Microsoft.Extensions.DependencyInjection;

namespace GregModmanager.Avalonia.Views;

public partial class ItemDetailPage : UserControl
{
    private readonly SteamWorkshopService _steam;
    private WorkshopItemDetailVm? _item;

    public ItemDetailPage(SteamWorkshopService steam)
    {
        InitializeComponent();
        _steam = steam;
    }

    public void LoadItem(ulong fileId)
    {
        _ = LoadItemSafeAsync(fileId);
    }

    private async Task LoadItemSafeAsync(ulong id)
    {
        try
        {
            await LoadItemAsync(id);
        }
        catch (Exception ex)
        {
            AppFileLog.Error($"ItemDetailPage load failed for fileId={id}", ex);
            var dialog = App.Services.GetRequiredService<Services.IDialogService>();
            await dialog.ShowMessageAsync(S.Get("Error"), ex.Message);
        }
    }

    private async Task LoadItemAsync(ulong fileId)
    {
        LoadingLabel.IsVisible = true;
        ContentPanel.IsVisible = false;

        _item = await _steam.GetItemDetailsAsync(fileId, CancellationToken.None);

        LoadingLabel.IsVisible = false;

        if (_item is null)
        {
            var dialog = App.Services.GetRequiredService<Services.IDialogService>();
            await dialog.ShowMessageAsync(S.Get("Error"), S.Get("Detail_CouldNotLoad"));
            return;
        }

        BindItem(_item);
        ContentPanel.IsVisible = true;
    }

    private void BindItem(WorkshopItemDetailVm item)
    {
        ItemTitle.Text = item.Title;
        AuthorLabel.Text = string.IsNullOrEmpty(item.OwnerName) ? $"Author: {item.OwnerSteamId}" : $"by {item.OwnerName}";
        SourceBadge.Text = item.SourceLabel;
        SourceBadge.Background = new SolidColorBrush(Color.Parse(item.SourceColor));
        SourceBadge.Foreground = new SolidColorBrush(Color.Parse("#001110"));
        VisibilityLabel.Text = item.Visibility;
        FileIdLabel.Text = item.PublishedFileId.ToString();
        PreviewUrlLabel.Text = item.PreviewImageUrl ?? "";

        SubsCount.Text = FormatNumber(item.NumSubscriptions);
        FavsCount.Text = FormatNumber(item.NumFavorites);
        VotesLabel.Text = $"+{item.VotesUp} / -{item.VotesDown}";
        SizeLabel.Text = WorkspaceService.FormatBytes(item.SizeBytes);
        ScoreLabel.Text = $"{item.Score:P0}";
        CommentsCount.Text = FormatNumber(item.NumComments);
        UpdatedLabel.Text = item.Updated.ToString("d");

        DescriptionLabel.Text = string.IsNullOrWhiteSpace(item.Description) ? S.Get("Detail_NoDescription") : item.Description;

        DependencyCard.IsVisible = item.HasDependencyHints;
        DependenciesBodyLabel.Text = item.DependencyHintBlock;

        TagsPanel.Children.Clear();
        foreach (var tag in item.Tags)
        {
            var border = new Border
            {
                Background = new SolidColorBrush(Color.Parse("#61F4D8")),
                Padding = new Thickness(10, 4),
                Margin = new Thickness(0, 0, 6, 6),
                CornerRadius = new CornerRadius(4)
            };
            border.Child = new TextBlock
            {
                Text = tag,
                FontSize = 11,
                Foreground = new SolidColorBrush(Color.Parse("#001110"))
            };
            TagsPanel.Children.Add(border);
        }

        UpdateSubscribeButton(SubscribeBtn, item.IsSubscribed);
        UpdateFavoriteButton(FavoriteBtn, false);

        if (item.IsBanned)
        {
            StatusLabel.Text = S.Get("Detail_ItemBanned");
            StatusLabel.Foreground = new SolidColorBrush(Color.Parse("#D7383B"));
        }
        else
        {
            StatusLabel.Text = "";
        }
    }

    private static void UpdateSubscribeButton(Button btn, bool isSubscribed)
    {
        btn.Content = isSubscribed ? S.Get("Detail_Unsubscribe") : S.Get("Detail_Subscribe");
    }

    private static void UpdateFavoriteButton(Button btn, bool isFavorited)
    {
        btn.Content = isFavorited ? S.Get("Detail_Unfavorite") : S.Get("Detail_Favorite");
    }

    private async void OnSubscribeToggle(object? sender, RoutedEventArgs e)
    {
        if (_item is null) return;

        StatusLabel.Text = S.Get("Working");
        bool success;
        var wasSubscribed = _item.IsSubscribed;
        if (wasSubscribed)
        {
            success = await _steam.UnsubscribeAsync(_item.PublishedFileId);
        }
        else
        {
            success = await _steam.SubscribeAsync(_item.PublishedFileId);
        }

        if (success)
        {
            var nowSubscribed = !wasSubscribed;
            UpdateSubscribeButton(SubscribeBtn, nowSubscribed);
            StatusLabel.Text = nowSubscribed ? S.Get("Detail_SubscribedMsg") : S.Get("Detail_UnsubscribedMsg");
            await ReloadItemAsync();
        }
        else
        {
            StatusLabel.Text = S.Get("Detail_ActionFailed");
        }
    }

    private async Task ReloadItemAsync()
    {
        if (_item is null) return;
        var refreshed = await _steam.GetItemDetailsAsync(_item.PublishedFileId, CancellationToken.None);
        if (refreshed is not null)
        {
            _item = refreshed;
        }
    }

    private async void OnFavoriteToggle(object? sender, RoutedEventArgs e)
    {
        if (_item is null) return;

        StatusLabel.Text = S.Get("Working");
        var isFav = FavoriteBtn.Content?.ToString() == S.Get("Detail_Unfavorite");
        bool success = isFav
            ? await _steam.RemoveFavoriteAsync(_item.PublishedFileId)
            : await _steam.AddFavoriteAsync(_item.PublishedFileId);

        if (success)
        {
            UpdateFavoriteButton(FavoriteBtn, !isFav);
            StatusLabel.Text = !isFav ? S.Get("Detail_AddedFavorites") : S.Get("Detail_RemovedFavorites");
        }
        else
        {
            StatusLabel.Text = S.Get("Detail_ActionFailed");
        }
    }

    public async void OnAddToCollection(object? sender, RoutedEventArgs e)
    {
        if (_item is null) return;

        var dialog = App.Services.GetRequiredService<Services.IDialogService>();
        var collectionName = await dialog.ShowPromptAsync(
            "Collection",
            "Collection name:",
            "Save",
            "Cancel",
            "My Mods");

        if (string.IsNullOrWhiteSpace(collectionName))
        {
            return;
        }

        var collections = App.Services.GetRequiredService<ModCollectionService>();
        var collection = collections.EnsureCollectionForItem(
            collectionName,
            _item.PublishedFileId,
            _item.Title,
            _item.SourceLabel,
            _item.IsGregFramework ? "MelonloaderPlugin" : "PlacableObject");

        await dialog.ShowMessageAsync(
            "Collection",
            $"Saved '{_item.Title}' to collection '{collection.Name}'.");
    }

    private async void OnVoteUp(object? sender, RoutedEventArgs e)
    {
        if (_item is null) return;
        StatusLabel.Text = S.Get("Detail_Voting");
        var ok = await _steam.VoteAsync(_item.PublishedFileId, true);
        StatusLabel.Text = ok ? S.Get("Detail_VotedUp") : S.Get("Detail_VoteFailed");
    }

    private async void OnVoteDown(object? sender, RoutedEventArgs e)
    {
        if (_item is null) return;
        StatusLabel.Text = S.Get("Detail_Voting");
        var ok = await _steam.VoteAsync(_item.PublishedFileId, false);
        StatusLabel.Text = ok ? S.Get("Detail_VotedDown") : S.Get("Detail_VoteFailed");
    }

    private void OnOpenInSteam(object? sender, RoutedEventArgs e)
    {
        if (_item is not null)
            _steam.OpenItemInBrowser(_item.PublishedFileId);
    }

    private void OnOpenChangelog(object? sender, RoutedEventArgs e)
    {
        if (_item is not null && !string.IsNullOrEmpty(_item.ChangelogUrl))
            _ = SafeProcess.OpenUrlAsync(_item.ChangelogUrl);
    }

    private void OnOpenComments(object? sender, RoutedEventArgs e)
    {
        if (_item is not null && !string.IsNullOrEmpty(_item.CommentsUrl))
            _ = SafeProcess.OpenUrlAsync(_item.CommentsUrl);
    }

    private static string FormatNumber(ulong n)
    {
        return n switch
        {
            >= 1_000_000 => $"{n / 1_000_000.0:0.#}M",
            >= 1_000 => $"{n / 1_000.0:0.#}K",
            _ => n.ToString(),
        };
    }
}
