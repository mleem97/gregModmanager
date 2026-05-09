using System;
using System.IO;
using System.IO.Pipes;
using System.Threading;
using System.Threading.Tasks;
using GregModmanager.Services;

namespace GregModmanager.Services.Auth;

public static class ProtocolSingleInstance
{
    private const string PipeName = "GregModmanagerProtocolPipe";

    public static async Task<bool> ShouldForwardAndExitAsync(string[] args)
    {
        var uri = args.FirstOrDefault(arg => arg.StartsWith("greg://", StringComparison.OrdinalIgnoreCase));

        if (string.IsNullOrEmpty(uri)) return false;

        try
        {
            using var client = new NamedPipeClientStream(".", PipeName, PipeDirection.Out, PipeOptions.Asynchronous);
            await client.ConnectAsync(500); // 500ms timeout

            using var writer = new StreamWriter(client);
            await writer.WriteLineAsync(uri);
            return true; // Sent successfully, we can exit this instance
        }
        catch (TimeoutException)
        {
            return false; // Nobody is listening, we are the first instance
        }
        catch
        {
            return false; // Same
        }
    }

    public static void StartListening(Action<string> onProtocolInvoked)
    {
        Task.Run(async () =>
        {
            while (true)
            {
                try
                {
#pragma warning disable CA1416 // Validate platform compatibility
                    using var server = new NamedPipeServerStream(PipeName, PipeDirection.In, 1, PipeTransmissionMode.Message, PipeOptions.Asynchronous);
#pragma warning restore CA1416
                    await server.WaitForConnectionAsync();

                    using var reader = new StreamReader(server);
                    var line = await reader.ReadLineAsync();

                    if (!string.IsNullOrEmpty(line) && line.StartsWith("greg://", StringComparison.OrdinalIgnoreCase))
                    {
                        onProtocolInvoked(line);
                    }
                }
                catch
                {
                    // Ignore errors and restart pipe listener
                    await Task.Delay(100);
                }
            }
        });
    }
}

