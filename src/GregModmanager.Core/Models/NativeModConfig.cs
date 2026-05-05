using System.Text.Json.Serialization;

namespace GregModmanager.Models;

/// <summary>Root object for <c>content/config.json</c> (Data Center native <see cref="ModLoader"/> format).</summary>
public sealed class NativeModConfig
{
	[JsonPropertyName("modName")]
	public string ModName { get; set; } = "";

	[JsonPropertyName("shopItems")]
	public List<NativeShopItem> ShopItems { get; set; } = new();

	[JsonPropertyName("staticItems")]
	public List<NativeStaticItem> StaticItems { get; set; } = new();

	[JsonPropertyName("dlls")]
	public List<NativeDllRef> Dlls { get; set; } = new();
}

public sealed class NativeShopItem
{
	[JsonPropertyName("itemName")]
	public string ItemName { get; set; } = "";

	[JsonPropertyName("price")]
	public int Price { get; set; }

	[JsonPropertyName("xpToUnlock")]
	public int XpToUnlock { get; set; }

	[JsonPropertyName("sizeInU")]
	public int SizeInU { get; set; }

	[JsonPropertyName("mass")]
	public double Mass { get; set; }

	[JsonPropertyName("modelScale")]
	public double ModelScale { get; set; } = 1.0;

	[JsonPropertyName("colliderSize")]
	public double[] ColliderSize { get; set; } = [0.5, 0.5, 0.5];

	[JsonPropertyName("colliderCenter")]
	public double[] ColliderCenter { get; set; } = [0, 0, 0];

	[JsonPropertyName("modelFile")]
	public string ModelFile { get; set; } = "";

	[JsonPropertyName("textureFile")]
	public string TextureFile { get; set; } = "";

	[JsonPropertyName("iconFile")]
	public string IconFile { get; set; } = "";

	[JsonPropertyName("objectType")]
	public int ObjectType { get; set; }
}

public sealed class NativeStaticItem
{
	[JsonPropertyName("itemName")]
	public string ItemName { get; set; } = "";

	[JsonPropertyName("modelFile")]
	public string ModelFile { get; set; } = "";

	[JsonPropertyName("textureFile")]
	public string TextureFile { get; set; } = "";

	[JsonPropertyName("modelScale")]
	public double ModelScale { get; set; } = 1.0;

	[JsonPropertyName("colliderSize")]
	public double[] ColliderSize { get; set; } = [0.5, 0.5, 0.5];

	[JsonPropertyName("colliderCenter")]
	public double[] ColliderCenter { get; set; } = [0, 0, 0];

	[JsonPropertyName("position")]
	public double[] Position { get; set; } = [0, 0, 0];

	[JsonPropertyName("rotation")]
	public double[] Rotation { get; set; } = [0, 0, 0];
}

public sealed class NativeDllRef
{
	[JsonPropertyName("fileName")]
	public string FileName { get; set; } = "";

	[JsonPropertyName("entryClass")]
	public string EntryClass { get; set; } = "";
}

