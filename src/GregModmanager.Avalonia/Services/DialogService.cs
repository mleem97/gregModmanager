using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.ApplicationLifetimes;
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

        var dialog = new Window
        {
            Title = title,
            Width = 420,
            Height = 200,
            CanResize = false,
            WindowStartupLocation = WindowStartupLocation.CenterOwner,
            Content = new StackPanel
            {
                Margin = new Thickness(20),
                Spacing = 10,
                Children =
                {
                    new TextBlock { Text = message, TextWrapping = TextWrapping.Wrap },
                    new StackPanel
                    {
                        Orientation = Orientation.Horizontal,
                        HorizontalAlignment = HorizontalAlignment.Right,
                        Spacing = 8,
                        Children =
                        {
                            new Button { Content = cancel, IsCancel = true },
                            new Button { Content = ok, IsDefault = true }
                        }
                    }
                }
            }
        };

        bool? result = null;
        var cancelBtn = ((StackPanel)((StackPanel)dialog.Content!).Children[1]).Children[0] as Button;
        var okBtn = ((StackPanel)((StackPanel)dialog.Content!).Children[1]).Children[1] as Button;
        cancelBtn!.Click += (_, _) => { result = false; dialog.Close(); };
        okBtn!.Click += (_, _) => { result = true; dialog.Close(); };

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
        var dialog = new Window
        {
            Title = title,
            Width = 620,
            Height = 260,
            CanResize = false,
            WindowStartupLocation = WindowStartupLocation.CenterOwner,
            Content = new StackPanel
            {
                Margin = new Thickness(20),
                Spacing = 14,
                Children =
                {
                    new TextBlock { Text = message, TextWrapping = TextWrapping.Wrap },
                    buttons
                }
            }
        };

        foreach (var choice in choices)
        {
            var button = new Button { Content = choice.Label, IsDefault = choice == choices[0] };
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
        var copyBtn = new Button { Content = S.Get("Error_Copy"), IsDefault = false };
        var okBtn = new Button { Content = ok, IsDefault = true };

        var dialog = new Window
        {
            Title = title,
            Width = 560,
            Height = 360,
            MinWidth = 420,
            MinHeight = 220,
            WindowStartupLocation = WindowStartupLocation.CenterOwner,
            Content = new StackPanel
            {
                Margin = new Thickness(20),
                Spacing = 10,
                Children =
                {
                    new ScrollViewer
                    {
                        MaxHeight = 230,
                        Content = new TextBlock { Text = message, TextWrapping = TextWrapping.Wrap }
                    },
                    status,
                    new StackPanel
                    {
                        Orientation = Orientation.Horizontal,
                        HorizontalAlignment = HorizontalAlignment.Right,
                        Spacing = 8,
                        Children =
                        {
                            copyBtn,
                            okBtn
                        }
                    }
                }
            }
        };

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
        string? result = null;

        var dialog = new Window
        {
            Title = title,
            Width = 460,
            Height = 220,
            CanResize = false,
            WindowStartupLocation = WindowStartupLocation.CenterOwner,
            Content = new StackPanel
            {
                Margin = new Thickness(20),
                Spacing = 10,
                Children =
                {
                    new TextBlock { Text = message },
                    textBox,
                    new StackPanel
                    {
                        Orientation = Orientation.Horizontal,
                        HorizontalAlignment = HorizontalAlignment.Right,
                        Spacing = 8,
                        Children =
                        {
                            new Button { Content = cancel, IsCancel = true },
                            new Button { Content = ok, IsDefault = true }
                        }
                    }
                }
            }
        };

        var cancelBtn = ((StackPanel)((StackPanel)dialog.Content!).Children[2]).Children[0] as Button;
        var okBtn = ((StackPanel)((StackPanel)dialog.Content!).Children[2]).Children[1] as Button;
        cancelBtn!.Click += (_, _) => { result = null; dialog.Close(); };
        okBtn!.Click += (_, _) => { result = textBox.Text; dialog.Close(); };

        await dialog.ShowDialog(window);
        return result;
    }
}
