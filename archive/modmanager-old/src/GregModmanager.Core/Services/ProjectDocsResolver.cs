using GregModmanager.Models;

namespace GregModmanager.Services;

/// <summary>
/// File-based project docs: <c>README.md</c> is the description source,
/// <c>CHANGELOG.md</c> (Keep a Changelog + SemVer) is the changelog source.
/// Steam gets converted BBCode/plain text, the Modstore keeps native Markdown.
/// </summary>
public static class ProjectDocsResolver
{
	public const long MaxDocsFileBytes = 400_000;

	public enum ChangelogSource
	{
		None,
		Manual,
		FileVersion,
		FileUnreleased,
	}

	public sealed record ResolvedDescription(string Text, bool FromFile, string? FilePath);

	public sealed record ResolvedChangelog(string Text, ChangelogSource Source, string? FilePath, string? EntryVersion);

	private static readonly string[] ReadmeCandidates = ["README.md", "readme.md", "README.markdown", "readme.markdown"];
	private static readonly string[] ChangelogCandidates = ["CHANGELOG.md", "changelog.md", "CHANGELOG.markdown", "changelog.markdown"];

	/// <summary>Finds <c>README.md</c> in the project root (case variants included).</summary>
	public static string? FindReadme(string projectRoot)
	{
		return FindFirstExisting(projectRoot, ReadmeCandidates);
	}

	/// <summary>Finds <c>CHANGELOG.md</c> in the project root (case variants included).</summary>
	public static string? FindChangelog(string projectRoot)
	{
		return FindFirstExisting(projectRoot, ChangelogCandidates);
	}

	private static string? FindFirstExisting(string projectRoot, string[] candidates)
	{
		if (string.IsNullOrWhiteSpace(projectRoot) || !Directory.Exists(projectRoot))
		{
			return null;
		}

		foreach (var name in candidates)
		{
			try
			{
				var path = Path.Combine(projectRoot, name);
				if (File.Exists(path))
				{
					return path;
				}
			}
			catch
			{
				// ignore bad paths
			}
		}

		return null;
	}

	private static string? ReadDocsFile(string path)
	{
		try
		{
			var info = new FileInfo(path);
			if (!info.Exists || info.Length == 0 || info.Length > MaxDocsFileBytes)
			{
				return null;
			}

			return File.ReadAllText(path);
		}
		catch
		{
			return null;
		}
	}

	/// <summary>
	/// Steam description: <c>README.md</c> converted to BBCode when present,
	/// otherwise the metadata fallback.
	/// </summary>
	public static ResolvedDescription ResolveSteamDescription(string projectRoot, string? fallbackDescription)
	{
		var path = FindReadme(projectRoot);
		if (path is not null)
		{
			var markdown = ReadDocsFile(path);
			if (!string.IsNullOrWhiteSpace(markdown))
			{
				var bbcode = MarkdownToSteamBbCode.Convert(markdown).Trim();
				if (!string.IsNullOrWhiteSpace(bbcode))
				{
					return new ResolvedDescription(bbcode, true, path);
				}
			}
		}

		return new ResolvedDescription((fallbackDescription ?? "").Trim(), false, null);
	}

	/// <summary>Modstore description: native Markdown, no BBCode conversion.</summary>
	public static ResolvedDescription ResolveModstoreDescription(string projectRoot, string? fallbackDescription)
	{
		var path = FindReadme(projectRoot);
		if (path is not null)
		{
			var markdown = ReadDocsFile(path)?.Trim();
			if (!string.IsNullOrWhiteSpace(markdown))
			{
				return new ResolvedDescription(markdown, true, path);
			}
		}

		return new ResolvedDescription((fallbackDescription ?? "").Trim(), false, null);
	}

	/// <summary>
	/// Steam changelog priority: manual input &gt; <c>CHANGELOG.md</c> section for
	/// <paramref name="version"/> &gt; <c>[Unreleased]</c> section. Returns empty
	/// with <see cref="ChangelogSource.None"/> when nothing usable exists.
	/// </summary>
	public static ResolvedChangelog ResolveSteamChangelog(string projectRoot, string? version, string? manualChangelog)
	{
		if (!string.IsNullOrWhiteSpace(manualChangelog))
		{
			return new ResolvedChangelog(manualChangelog.Trim(), ChangelogSource.Manual, null, null);
		}

		var path = FindChangelog(projectRoot);
		if (path is null)
		{
			return new ResolvedChangelog("", ChangelogSource.None, null, null);
		}

		var markdown = ReadDocsFile(path);
		if (string.IsNullOrWhiteSpace(markdown))
		{
			return new ResolvedChangelog("", ChangelogSource.None, path, null);
		}

		var wanted = KeepAChangelogParser.NormalizeVersion(version) ?? "1.0.0";
		if (KeepAChangelogParser.TryGetEntryForVersion(markdown, wanted, out var entry) && entry is not null)
		{
			var source = entry.Version == "Unreleased" ? ChangelogSource.FileUnreleased : ChangelogSource.FileVersion;
			return new ResolvedChangelog(entry.Body.Trim(), source, path, entry.Version);
		}

		if (KeepAChangelogParser.TryGetUnreleasedEntry(markdown, out var unreleased) && unreleased is not null)
		{
			return new ResolvedChangelog(unreleased.Body.Trim(), ChangelogSource.FileUnreleased, path, unreleased.Version);
		}

		return new ResolvedChangelog("", ChangelogSource.None, path, null);
	}

	/// <summary>Modstore changelog: same resolution, raw Markdown body (no BBCode).</summary>
	public static ResolvedChangelog ResolveModstoreChangelog(string projectRoot, string? version, string? manualChangelog)
	{
		return ResolveSteamChangelog(projectRoot, version, manualChangelog);
	}

	/// <summary>Prefixes the Steam change note with the version unless already present.</summary>
	public static string FormatSteamChangeNote(string? version, string body)
	{
		var v = (version ?? "1.0.0").Trim();
		if (string.IsNullOrWhiteSpace(v))
		{
			v = "1.0.0";
		}

		var b = (body ?? "").Trim();
		if (string.IsNullOrEmpty(b))
		{
			return $"v{v}";
		}

		if (b.StartsWith($"v{v}", StringComparison.OrdinalIgnoreCase) || b.StartsWith(v, StringComparison.OrdinalIgnoreCase))
		{
			return b;
		}

		return $"v{v}: {b}";
	}

	/// <summary>
	/// Effective Steam description including auto-appended requirement notices.
	/// Single place so publish, hash, and checks never disagree.
	/// </summary>
	public static string BuildEffectiveSteamDescription(string projectRoot, WorkshopMetadata meta)
	{
		var resolved = ResolveSteamDescription(projectRoot, meta.Description);
		var desc = resolved.Text;
		if (meta.NeedsMelonLoader && !desc.Contains("MelonLoader", StringComparison.OrdinalIgnoreCase))
		{
			desc += "\n\n---\n" + Localization.S.Get("Editor_MelonLoaderNotice");
		}

		if (meta.Needsgreg && !desc.Contains("gregCoreModFramework", StringComparison.OrdinalIgnoreCase))
		{
			desc += "\n\n---\n" + Localization.S.Get("Editor_gregNotice");
		}

		return desc.Trim();
	}
}
