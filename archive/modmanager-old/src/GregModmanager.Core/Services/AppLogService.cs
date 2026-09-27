using System.Collections.ObjectModel;

namespace GregModmanager.Services;

public sealed class AppLogService
{
	private const int MaxLines = 500;

	public ObservableCollection<string> Lines { get; } = new();

	public event EventHandler? LineAppended;

	public void Append(string message)
	{
		var line = $"{DateTime.Now:HH:mm:ss} {message}";
		AppFileLog.Info(message);

		// Caller must ensure UI thread access when modifying ObservableCollection.
		// In Avalonia, use Dispatcher.UIThread.Post() before calling Append() if needed.
		Lines.Add(line);
		while (Lines.Count > MaxLines)
		{
			Lines.RemoveAt(0);
		}

		LineAppended?.Invoke(this, EventArgs.Empty);
	}
}

