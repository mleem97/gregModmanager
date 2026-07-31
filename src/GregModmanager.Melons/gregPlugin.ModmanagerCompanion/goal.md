# Modmanager Companion Melon Plugin

This project will be a MelonLoader plugin that integrates a custom modloader into an IL2CPP-based Unity game. The goal is to provide a stable bootstrap layer that can initialize the modloader at the correct time in the game lifecycle, register its runtime services, and expose a clean interface for future mods.

## Intended purpose

The plugin should act as the bridge between the game and the modloader. In practice, this means:

- Loading the modloader during game startup.
- Ensuring the modloader is initialized after Unity has finished its core boot sequence.
- Providing a safe place to register internal systems, hooks, and commands.
- Avoiding early crashes caused by IL2CPP-specific loading behavior.

## Concrete implementation approach

For an IL2CPP Unity game, the most reliable approach is to build the plugin as a MelonMod that runs very early, but defers the actual modloader initialization until the first stable game frame.

### Recommended flow

1. On plugin startup, register basic Melon events.
2. Wait until the game has entered a usable runtime state.
3. Initialize the modloader from a deferred callback.
4. Register all required services and UI integrations.
5. Log success or failure clearly for debugging.

## Example Melon implementation skeleton

```csharp
using MelonLoader;
using UnityEngine;

namespace Greg.ModmanagerCompanion
{
    public class ModmanagerCompanion : MelonMod
    {
        private static bool _initialized;

        public override void OnApplicationStart()
        {
            MelonLogger.Msg("Modmanager Companion loaded.");
            MelonCoroutines.Start(InitializeRoutine());
        }

        private System.Collections.IEnumerator InitializeRoutine()
        {
            yield return new WaitForEndOfFrame();

            if (_initialized)
                yield break;

            try
            {
                InitializeModloader();
                _initialized = true;
                MelonLogger.Msg("Modmanager Companion initialized successfully.");
            }
            catch (System.Exception ex)
            {
                MelonLogger.Error($"Failed to initialize Modmanager Companion: {ex}");
            }
        }

        private void InitializeModloader()
        {
            // Example integration point:
            // - create runtime services
            // - register hooks
            // - initialize the modloader core
            // - attach UI or command handlers
        }
    }
}
```

## Notes for IL2CPP compatibility

Because this is an IL2CPP game, the implementation should:

- Avoid reflection-heavy code where possible.
- Prefer explicit, stable entry points.
- Keep initialization logic simple and fail gracefully.
- Use MelonLoader's logging heavily during early startup.
- Test startup behavior in both normal launch and modded launch scenarios.

## Expected result

When completed, this plugin should allow the game to load the modloader in a controlled and safe manner, making future mod integration much easier and more reliable.
