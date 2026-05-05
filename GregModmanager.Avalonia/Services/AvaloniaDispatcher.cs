using Avalonia.Threading;

namespace GregModmanager.Avalonia.Services;

public sealed class AvaloniaDispatcher
{
    public void Post(Action action) => Dispatcher.UIThread.Post(action);
    public T Invoke<T>(Func<T> func) => Dispatcher.UIThread.Invoke(func);
    public void Invoke(Action action) => Dispatcher.UIThread.Invoke(action);
}
