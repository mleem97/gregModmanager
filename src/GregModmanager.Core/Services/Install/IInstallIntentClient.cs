using System.Threading.Tasks;

namespace GregModmanager.Services.Install;

public interface IInstallIntentClient
{
    Task HandleIntentAsync(string rawUri);
}
