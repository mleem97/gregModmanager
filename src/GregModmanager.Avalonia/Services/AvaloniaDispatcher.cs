using Avalonia.Threading;

namespace GregModmanager.Avalonia.Services;

public sealed class AvaloniaDispatcher
{
    public static void Post(Action action) => Dispatcher.UIThread.Post(action);
    public static T Invoke<T>(Func<T> func) => Dispatcher.UIThread.Invoke(func);
    public static void Invoke(Action action) => Dispatcher.UIThread.Invoke(action);
}
