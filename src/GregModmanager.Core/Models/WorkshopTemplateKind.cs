namespace GregModmanager.Models;

/// <summary>Scaffold layout under <c>content/</c> for a new Workshop project.</summary>
public enum WorkshopTemplateKind
{
	/// <summary>Game Object / Decoration style assets (vanilla Workshop delivery).</summary>
	VanillaObjectDecoration,

	/// <summary>MelonLoader mods — <c>content/Mods</c> mirrors <c>{GameRoot}/Mods</c> when linked from Workshop delivery.</summary>
	ModdedMelonLoader,

	/// <summary>gregCoreModFramework — <c>content/ModFramework/greg/Plugins</c> mirrors <c>{GameRoot}/greg/Plugins</c>.</summary>
	ModdedgregCoreModFramework,

	/// <summary>UXML UI Override — uses GregUxmlService to replace game interfaces.</summary>
	UxmlUiOverride,

	/// <summary>Standalone 3D Model (Asset Store / ModStore focused).</summary>
	Standalone3DModel,

	/// <summary>Standalone Texture/Material (Asset Store / ModStore focused).</summary>
	StandaloneTexture,

	/// <summary>Standalone Audio/Music (Asset Store / ModStore focused).</summary>
	StandaloneAudio,
}
