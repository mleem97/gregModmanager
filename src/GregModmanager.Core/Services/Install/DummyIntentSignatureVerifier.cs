using GregModmanager.Models.Install;

namespace GregModmanager.Services.Install;

public class DummyIntentSignatureVerifier : IIntentSignatureVerifier
{
    public bool VerifySignature(InstallIntentContext intent)
    {
        // For the Vertical Slice (Phase 3), we require the signature to at least match a mock known structure.
        return intent.Signature == "valid_dummy_sig" || intent.Signature.Length >= 32;
    }
}
