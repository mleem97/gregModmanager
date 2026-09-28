namespace GregModmanager.Services;

public sealed record RuntimeDependencyInstallResult(
	bool Success,
	bool Changed,
	string Message,
	MelonLoaderInstallResult MelonLoader,
	SteamModfixInstallResult SteamModfix);

/// <summary>Runs the same idempotent runtime setup on every desktop platform.</summary>
public sealed class RuntimeDependencyInstallerService : IDisposable
{
	private readonly MelonLoaderInstallerService _melonLoader;
	private readonly SteamModfixInstallerService _steamModfix;
	private readonly SemaphoreSlim _gate = new(1, 1);
	private bool _disposed;

	public RuntimeDependencyInstallerService(
		MelonLoaderInstallerService melonLoader,
		SteamModfixInstallerService steamModfix)
	{
		_melonLoader = melonLoader;
		_steamModfix = steamModfix;
	}

	public async Task<RuntimeDependencyInstallResult> EnsureCurrentAsync(
		string gameRoot,
		IProgress<string>? progress = null,
		CancellationToken cancellationToken = default)
	{
		ObjectDisposedException.ThrowIf(_disposed, this);
		await _gate.WaitAsync(cancellationToken);
		try
		{
			if (string.IsNullOrWhiteSpace(gameRoot) || !Directory.Exists(gameRoot))
				return CreateFailure("Der Spielordner wurde nicht gefunden.");

			progress?.Report("Prüfe MelonLoader…");
			var melonLoader = await _melonLoader.EnsureCurrentAsync(gameRoot, progress, cancellationToken);
			progress?.Report("Prüfe SteamModfix…");
			var steamModfix = await _steamModfix.EnsureCurrentAsync(gameRoot, progress, cancellationToken);
			var success = melonLoader.Success && steamModfix.Success;
			var message = $"{melonLoader.Message} {steamModfix.Message}";
			return new(success, melonLoader.Changed || steamModfix.Changed, message, melonLoader, steamModfix);
		}
		finally
		{
			_gate.Release();
		}
	}

	public void Dispose()
	{
		if (_disposed) return;
		_disposed = true;
		_gate.Dispose();
	}

	private static RuntimeDependencyInstallResult CreateFailure(string message)
	{
		return new(
			false,
			false,
			message,
			new(false, false, message, null),
			new(false, false, message, null));
	}
}
