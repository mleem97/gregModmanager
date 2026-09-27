using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Controls.Primitives;
using Avalonia.Input.Platform;
using Avalonia.Layout;
using Avalonia.Media;
using GregModmanager.Localization;
using GregModmanager.Services;
using System.Globalization;
using System.Threading.Tasks;

namespace GregModmanager.Avalonia.Services;

public interface IDialogService
{
    Task<bool> ShowConfirmAsync(string title, string message);
    Task<bool> ShowConfirmAsync(string title, string message, string ok);
    Task<bool> ShowConfirmAsync(string title, string message, string ok, string cancel);
    Task ShowMessageAsync(string title, string message);
    Task ShowMessageAsync(string title, string message, string ok);
    Task ShowErrorAsync(string title, string message, Exception exception);
    Task<string?> ShowChoiceAsync(string title, string message, IReadOnlyList<(string Id, string Label)> choices);
    Task<string?> ShowPromptAsync(string title, string message);
    Task<string?> ShowPromptAsync(string title, string message, string ok);
    Task<string?> ShowPromptAsync(string title, string message, string ok, string cancel);
    Task<string?> ShowPromptAsync(string title, string message, string ok, string cancel, string initialValue);
}

public sealed class DialogService : IDialogService
{
    private static Window? GetTopLevel()
    {
        if (Application.Current?.ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
            return desktop.MainWindow;
        return null;
    }

    private static string GetAppVersion()
        => typeof(DialogService).Assembly.GetName().Version?.ToString() ?? "unknown";

    private static Window CreateDialogWindow(string title, double width, double height, Control body, bool canResize = false)
    {
        var dialog = new Window
        {
            Title = title,
            Width = width,
            Height = height,
            MinWidth = Math.Min(width, 420),
            MinHeight = Math.Min(height, 220),
            CanResize = canResize,
            WindowDecorations = WindowDecorations.None,
            WindowStartupLocation = WindowStartupLocation.CenterOwner
        };
        dialog.Classes.Add("dialog-window");

        var titleLabel = new TextBlock
        {
            Text = $"DIALOG // {title.ToUpperInvariant()}",
            VerticalAlignment = VerticalAlignment.Center
        };
        titleLabel.Classes.Add("micro-label");
        titleLabel.Classes.Add("cyan");

        var closeButton = new Button
        {
            Content = "✕",
            Width = 34,
            Height = 28,
            HorizontalContentAlignment = HorizontalAlignment.Center,
            VerticalContentAlignment = VerticalAlignment.Center
        };
        closeButton.Classes.Add("window-btn");
        closeButton.Classes.Add("close");
        closeButton.Click += (_, _) => dialog.Close();

        var titlebar = new Border { Child = new Grid { ColumnDefinitions = new ColumnDefinitions("*,Auto") } };
        titlebar.Classes.Add("titlebar");
        var titleGrid = (Grid)titlebar.Child;
        titleGrid.Children.Add(new StackPanel
        {
            Orientation = Orientation.Horizontal,
            VerticalAlignment = VerticalAlignment.Center,
            Spacing = 8,
            Children =
            {
                new Border { Width = 12, Height = 12, CornerRadius = new CornerRadius(2), Classes = { "accent-box" } },
                titleLabel
            }
        });
        Grid.SetColumn(closeButton, 1);
        titleGrid.Children.Add(closeButton);

        body.Classes.Add("dialog-body");
        var root = new Grid { RowDefinitions = new RowDefinitions("42,*") };
        root.Children.Add(titlebar);
        Grid.SetRow(body, 1);
        root.Children.Add(body);
        dialog.Content = root;
        return dialog;
    }

    private static StackPanel CreateDialogBody(params Control[] controls)
    {
        var body = new StackPanel { Margin = new Thickness(18), Spacing = 12 };
        foreach (var control in controls)
            body.Children.Add(control);
        return body;
    }

    private static Button CreateActionButton(string content, bool primary = false)
    {
        var button = new Button { Content = content, IsDefault = primary };
        if (primary)
            button.Classes.Add("btn-primary");
        else
            button.Classes.Add("dialog-secondary");
        return button;
    }

    public Task<bool> ShowConfirmAsync(string title, string message)
    {
        return ShowConfirmAsync(title, message, "OK", "Cancel");
    }

    public Task<bool> ShowConfirmAsync(string title, string message, string ok)
    {
        return ShowConfirmAsync(title, message, ok, "Cancel");
    }

    public async Task<bool> ShowConfirmAsync(string title, string message, string ok, string cancel)
    {
        var window = GetTopLevel();
        if (window == null) return false;

        var cancelBtn = CreateActionButton(cancel);
        cancelBtn.IsCancel = true;
        var okBtn = CreateActionButton(ok, primary: true);
        var actions = new StackPanel
        {
            Orientation = Orientation.Horizontal,
            HorizontalAlignment = HorizontalAlignment.Right,
            Spacing = 8,
            Children = { cancelBtn, okBtn }
        };
        var dialog = CreateDialogWindow(title, 420, 200,
            CreateDialogBody(
                new TextBlock { Text = message, TextWrapping = TextWrapping.Wrap, Classes = { "dialog-message" } },
                actions));

        bool? result = null;
        cancelBtn.Click += (_, _) => { result = false; dialog.Close(); };
        okBtn.Click += (_, _) => { result = true; dialog.Close(); };

        await dialog.ShowDialog<bool?>(window);
        return result == true;
    }

    public Task ShowMessageAsync(string title, string message)
    {
        return ShowMessageAsync(title, message, "OK");
    }

    public Task ShowErrorAsync(string title, string message, Exception exception)
    {
        var details = string.Join(Environment.NewLine,
            $"{title}: {message}",
            $"{S.Get("Error_AppVersion")}: {GetAppVersion()}",
            $"{S.Get("Error_OperatingSystem")}: {Environment.OSVersion}",
            $"{S.Get("Error_Language")}: {CultureInfo.CurrentUICulture.Name}",
            $"{S.Get("Error_LogPath")}: {AppFileLog.LogPath}",
            string.Empty,
            S.Get("Error_TechnicalDetails"),
            exception.ToString());

        return ShowMessageCoreAsync(title, message, S.Get("OK"), details);
    }

    public async Task<string?> ShowChoiceAsync(string title, string message, IReadOnlyList<(string Id, string Label)> choices)
    {
        var window = GetTopLevel();
        if (window is null || choices.Count == 0) return null;

        var result = new TaskCompletionSource<string?>(TaskCreationOptions.RunContinuationsAsynchronously);
        var buttons = new StackPanel
        {
            Orientation = Orientation.Horizontal,
            HorizontalAlignment = HorizontalAlignment.Right,
            Spacing = 8
        };
        var dialog = CreateDialogWindow(title, 620, 260,
            CreateDialogBody(
                new TextBlock { Text = message, TextWrapping = TextWrapping.Wrap, Classes = { "dialog-message" } },
                buttons));

        foreach (var choice in choices)
        {
            var button = CreateActionButton(choice.Label, primary: choice == choices[0]);
            button.Click += (_, _) =>
            {
                result.TrySetResult(choice.Id);
                dialog.Close();
            };
            buttons.Children.Add(button);
        }

        dialog.Closed += (_, _) => result.TrySetResult(null);
        await dialog.ShowDialog(window);
        return await result.Task;
    }

    public Task ShowMessageAsync(string title, string message, string ok)
        => ShowMessageCoreAsync(title, message, ok, $"{title}: {message}");

    private async Task ShowMessageCoreAsync(string title, string message, string ok, string copyText)
    {
        var window = GetTopLevel();
        if (window == null) return;

        var status = new TextBlock
        {
            Foreground = Brushes.CadetBlue,
            FontSize = 11,
            TextWrapping = TextWrapping.Wrap
        };
        var copyBtn = CreateActionButton(S.Get("Error_Copy"));
        var okBtn = CreateActionButton(ok, primary: true);
        var dialog = CreateDialogWindow(title, 560, 360,
            CreateDialogBody(
                new ScrollViewer
                {
                    MaxHeight = 230,
                    VerticalScrollBarVisibility = ScrollBarVisibility.Auto,
                    Content = new TextBlock { Text = message, TextWrapping = TextWrapping.Wrap, Classes = { "dialog-message" } }
                },
                status,
                new StackPanel
                {
                    Orientation = Orientation.Horizontal,
                    HorizontalAlignment = HorizontalAlignment.Right,
                    Spacing = 8,
                    Children = { copyBtn, okBtn }
                }));

        copyBtn.Click += async (_, _) =>
        {
            try
            {
                var clipboard = TopLevel.GetTopLevel(dialog)?.Clipboard;
                if (clipboard is null)
                    throw new InvalidOperationException(S.Get("Error_ClipboardUnavailable"));

                await clipboard.SetTextAsync(copyText);
                status.Text = S.Get("Error_Copied");
            }
            catch (Exception ex)
            {
                AppFileLog.Warn($"Could not copy dialog message: {ex.Message}");
                status.Text = S.Get("Error_CopyFailed");
            }
        };
        okBtn.Click += (_, _) => dialog.Close();

        await dialog.ShowDialog(window);
    }

    public Task<string?> ShowPromptAsync(string title, string message)
    {
        return ShowPromptAsync(title, message, "OK", "Cancel", string.Empty);
    }

    public Task<string?> ShowPromptAsync(string title, string message, string ok)
    {
        return ShowPromptAsync(title, message, ok, "Cancel", string.Empty);
    }

    public Task<string?> ShowPromptAsync(string title, string message, string ok, string cancel)
    {
        return ShowPromptAsync(title, message, ok, cancel, string.Empty);
    }

    public async Task<string?> ShowPromptAsync(string title, string message, string ok, string cancel, string initialValue)
    {
        var window = GetTopLevel();
        if (window == null) return null;

        var textBox = new TextBox { Text = initialValue };
        textBox.Classes.Add("input-terminal");
        string? result = null;

        var cancelBtn = CreateActionButton(cancel);
        cancelBtn.IsCancel = true;
        var okBtn = CreateActionButton(ok, primary: true);
        var dialog = CreateDialogWindow(title, 460, 220,
            CreateDialogBody(
                new TextBlock { Text = message, TextWrapping = TextWrapping.Wrap, Classes = { "dialog-message" } },
                textBox,
                new StackPanel
                {
                    Orientation = Orientation.Horizontal,
                    HorizontalAlignment = HorizontalAlignment.Right,
                    Spacing = 8,
                    Children = { cancelBtn, okBtn }
                }));

        cancelBtn.Click += (_, _) => { result = null; dialog.Close(); };
        okBtn.Click += (_, _) => { result = textBox.Text; dialog.Close(); };

        await dialog.ShowDialog(window);
        return result;
    }
}
