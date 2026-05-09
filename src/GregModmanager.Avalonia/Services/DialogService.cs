using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Layout;
using Avalonia.Media;
using System.Threading.Tasks;

namespace GregModmanager.Avalonia.Services;

public interface IDialogService
{
    Task<bool> ShowConfirmAsync(string title, string message, string ok = "OK", string cancel = "Cancel");
    Task ShowMessageAsync(string title, string message, string ok = "OK");
    Task<string?> ShowPromptAsync(string title, string message, string ok = "OK", string cancel = "Cancel", string initialValue = "");
}

public sealed class DialogService : IDialogService
{
    private static Window? GetTopLevel()
    {
        if (Application.Current?.ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
            return desktop.MainWindow;
        return null;
    }

    public async Task<bool> ShowConfirmAsync(string title, string message, string ok = "OK", string cancel = "Cancel")
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

    public async Task ShowMessageAsync(string title, string message, string ok = "OK")
    {
        var window = GetTopLevel();
        if (window == null) return;

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
                            new Button { Content = ok, IsDefault = true }
                        }
                    }
                }
            }
        };

        var okBtn = ((StackPanel)((StackPanel)dialog.Content!).Children[1]).Children[0] as Button;
        okBtn!.Click += (_, _) => dialog.Close();

        await dialog.ShowDialog(window);
    }

    public async Task<string?> ShowPromptAsync(string title, string message, string ok = "OK", string cancel = "Cancel", string initialValue = "")
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
