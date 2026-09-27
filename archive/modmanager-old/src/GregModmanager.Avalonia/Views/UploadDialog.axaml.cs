#pragma warning disable CS8618

using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.Threading;
using GregModmanager.Localization;
using System.Collections.ObjectModel;

namespace GregModmanager.Avalonia.Views;

/// <summary>
/// Modal Steam Workshop upload dialog: live status, progress bar, scrolling
/// log, and a prominently placed Cancel button wired to a
/// <see cref="CancellationTokenSource"/>. Closing the window cancels too.
/// </summary>
public partial class UploadDialog : Window
{
    private readonly CancellationTokenSource _cts = new();
    private readonly ObservableCollection<string> _lines = new();
    private bool _running = true;

    public UploadDialog() => InitializeComponent();

    public UploadDialog(string title) : this()
    {
        TitleLabel.Text = title;
        Title = title;
        StatusLabel.Text = S.Get("UploadDialog_StatusStarting");
        CancelButton.Content = S.Get("UploadDialog_Cancel");
        CloseButton.Content = S.Get("UploadDialog_Close");
        LogList.ItemsSource = _lines;
        Closing += OnClosing;
    }

    public CancellationToken Token => _cts.Token;

    public bool WasCancelled => _cts.IsCancellationRequested;

    public void ReportStatus(string status)
    {
        Dispatcher.UIThread.Post(() => StatusLabel.Text = status);
    }

    public void ReportProgress(float fraction)
    {
        Dispatcher.UIThread.Post(() =>
        {
            ProgressBar.IsIndeterminate = false;
            ProgressBar.Value = Math.Clamp(fraction, 0f, 1f);
        });
    }

    public void ReportLog(string line)
    {
        Dispatcher.UIThread.Post(() =>
        {
            _lines.Add(line);
            while (_lines.Count > 300)
                _lines.RemoveAt(0);
            LogScroller.ScrollToEnd();
        });
    }

    public void SetFinished(bool success, string message)
    {
        Dispatcher.UIThread.Post(() =>
        {
            _running = false;
            StatusLabel.Text = message;
            ProgressBar.Value = success ? 1 : ProgressBar.Value;
            CancelButton.IsVisible = false;
            CloseButton.IsVisible = true;
            CloseButton.Focus();
        });
    }

    private void OnCancelClicked(object? sender, RoutedEventArgs e) => RequestCancel();

    private void OnCloseClicked(object? sender, RoutedEventArgs e) => Close();

    private void OnClosing(object? sender, WindowClosingEventArgs e) => RequestCancel();

    private void RequestCancel()
    {
        if (!_running || _cts.IsCancellationRequested) return;
        try { _cts.Cancel(); } catch { /* already disposed */ }
        Dispatcher.UIThread.Post(() =>
        {
            StatusLabel.Text = S.Get("UploadDialog_StatusCancelling");
            CancelButton.IsEnabled = false;
        });
    }
}
