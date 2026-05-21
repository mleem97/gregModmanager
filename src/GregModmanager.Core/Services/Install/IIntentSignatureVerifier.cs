using GregModmanager.Models.Install;

namespace GregModmanager.Services.Install;

public interface IIntentSignatureVerifier
{
    bool VerifySignature(InstallIntentContext intent);
}
