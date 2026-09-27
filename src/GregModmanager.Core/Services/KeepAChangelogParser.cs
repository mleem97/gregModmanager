using System.Text.RegularExpressions;

namespace GregModmanager.Services;

/// <summary>
/// Minimal <see href="https://keepachangelog.com/en/1.1.0/">Keep a Changelog</see>
/// parser for project <c>CHANGELOG.md</c> files plus
/// <see href="https://semver.org/spec/v2.0.0.html">Semantic Versioning</see>
/// validation. Lets the manager read the newest changelog entry for an update
/// without manual input in the app.
/// </summary>
public static partial class KeepAChangelogParser
{
	/// <summary>One <c>## [x.y.z]</c> section (or <c>## [Unreleased]</c>).</summary>
	public sealed record ChangelogEntry(string Version, string? Date, string Body);

	[GeneratedRegex(@"^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?$")]
	private static partial Regex SemVerRegex();

	[GeneratedRegex(@"^##\s+(?:\[([^\]]+)\]|(\S+))(?:\s*-\s*(.+))?\s*$")]
	private static partial Regex SectionHeadingRegex();

	[GeneratedRegex(@"(?m)^\[[^\]]+\]:\s*\S+.*$")]
	private static partial Regex LinkDefinitionRegex();

	/// <summary>Strict SemVer 2.0.0 check. A leading <c>v</c> is accepted and ignored.</summary>
	public static bool IsValidSemVersion(string? version)
	{
		var v = NormalizeVersion(version);
		return v is not null && SemVerRegex().IsMatch(v);
	}

	/// <summary>Trims whitespace and a single leading <c>v</c>. Returns null when empty.</summary>
	public static string? NormalizeVersion(string? version)
	{
		if (string.IsNullOrWhiteSpace(version))
		{
			return null;
		}

		var v = version.Trim();
		if (v.Length > 1 && (v[0] == 'v' || v[0] == 'V'))
		{
			v = v[1..].Trim();
		}

		return string.IsNullOrEmpty(v) ? null : v;
	}

	/// <summary>Parses all <c>##</c> sections. Returns false when no usable section exists.</summary>
	public static bool TryParseEntries(string? markdown, out List<ChangelogEntry> entries, out string? error)
	{
		var list = new List<ChangelogEntry>();
		entries = list;
		error = null;
		if (string.IsNullOrWhiteSpace(markdown))
		{
			error = "Changelog is empty.";
			return false;
		}

		string? currentVersion = null;
		string? currentDate = null;
		var body = new List<string>();
		var seen = new HashSet<string>(StringComparer.OrdinalIgnoreCase);

		void Flush()
		{
			if (currentVersion is null)
			{
				return;
			}

			var text = StripLinkDefinitions(string.Join("\n", body).Trim());
			list.Add(new ChangelogEntry(currentVersion, currentDate, text));
			body.Clear();
		}

		foreach (var rawLine in markdown.Replace("\r\n", "\n").Split('\n'))
		{
			var line = rawLine.TrimEnd();
			var match = SectionHeadingRegex().Match(line.Trim());
			if (!match.Success)
			{
				if (currentVersion is not null)
				{
					body.Add(rawLine);
				}

				continue;
			}

			var token = (match.Groups[1].Success ? match.Groups[1].Value : match.Groups[2].Value).Trim();
			var date = match.Groups[3].Success ? match.Groups[3].Value.Trim() : null;
			if (string.IsNullOrWhiteSpace(date))
			{
				date = null;
			}

			Flush();

			if (string.Equals(token, "Unreleased", StringComparison.OrdinalIgnoreCase))
			{
				currentVersion = "Unreleased";
				currentDate = null;
			}
			else
			{
				var normalized = NormalizeVersion(token);
				if (normalized is null || !SemVerRegex().IsMatch(normalized))
				{
					// Not a version section (e.g. a "Compare" heading) — skip until next ##.
					currentVersion = null;
					currentDate = null;
					continue;
				}

				if (!seen.Add(normalized))
				{
					error = $"Duplicate changelog section for version {normalized}.";
					list.Clear();
					return false;
				}

				currentVersion = normalized;
				currentDate = date;
			}
		}

		Flush();

		if (list.Count == 0)
		{
			error = "No '## [x.y.z]' or '## [Unreleased]' section found (Keep a Changelog format).";
			return false;
		}

		return true;
	}

	/// <summary>Section body for an exact SemVer version. Falls back to Unreleased when it names that version.</summary>
	public static bool TryGetEntryForVersion(string? markdown, string? version, out ChangelogEntry? entry)
	{
		entry = null;
		var wanted = NormalizeVersion(version);
		if (wanted is null || !TryParseEntries(markdown, out var entries, out _))
		{
			return false;
		}

		foreach (var e in entries)
		{
			if (string.Equals(e.Version, wanted, StringComparison.OrdinalIgnoreCase) && !string.IsNullOrWhiteSpace(e.Body))
			{
				entry = e;
				return true;
			}
		}

		// Unreleased often already describes the next version about to be published.
		var unreleased = entries.FirstOrDefault(e => e.Version == "Unreleased" && !string.IsNullOrWhiteSpace(e.Body));
		if (unreleased is not null && MentionsVersion(unreleased.Body, wanted))
		{
			entry = unreleased;
			return true;
		}

		return false;
	}

	/// <summary>Newest versioned section by SemVer precedence (ignores Unreleased and empty bodies).</summary>
	public static bool TryGetLatestReleaseEntry(string? markdown, out ChangelogEntry? entry)
	{
		entry = null;
		if (!TryParseEntries(markdown, out var entries, out _))
		{
			return false;
		}

		ChangelogEntry? best = null;
		foreach (var e in entries)
		{
			if (e.Version == "Unreleased" || string.IsNullOrWhiteSpace(e.Body))
			{
				continue;
			}

			if (best is null || CompareSemVer(e.Version, best.Version) > 0)
			{
				best = e;
			}
		}

		entry = best;
		return best is not null;
	}

	public static bool TryGetUnreleasedEntry(string? markdown, out ChangelogEntry? entry)
	{
		entry = null;
		if (!TryParseEntries(markdown, out var entries, out _))
		{
			return false;
		}

		entry = entries.FirstOrDefault(e => e.Version == "Unreleased" && !string.IsNullOrWhiteSpace(e.Body));
		return entry is not null;
	}

	private static string StripLinkDefinitions(string body)
	{
		if (string.IsNullOrEmpty(body))
		{
			return body;
		}

		return LinkDefinitionRegex().Replace(body, "").Trim();
	}

	private static bool MentionsVersion(string body, string version)
	{
		return body.Contains(version, StringComparison.OrdinalIgnoreCase)
			|| body.Contains("v" + version, StringComparison.OrdinalIgnoreCase);
	}

	/// <summary>Compares two normalized SemVer strings. Release &gt; prerelease.</summary>
	public static int CompareSemVer(string a, string b)
	{
		var pa = ParseSemVer(a);
		var pb = ParseSemVer(b);
		for (var i = 0; i < 3; i++)
		{
			var c = pa.Numbers[i].CompareTo(pb.Numbers[i]);
			if (c != 0)
			{
				return c;
			}
		}

		var aPre = pa.Prerelease is null;
		var bPre = pb.Prerelease is null;
		if (aPre && bPre)
		{
			return 0;
		}

		if (aPre)
		{
			return 1;
		}

		if (bPre)
		{
			return -1;
		}

		return ComparePrerelease(pa.Prerelease!, pb.Prerelease!);
	}

	private static (int[] Numbers, string? Prerelease) ParseSemVer(string version)
	{
		var v = NormalizeVersion(version) ?? version;
		var plus = v.IndexOf('+');
		if (plus >= 0)
		{
			v = v[..plus];
		}

		string? pre = null;
		var dash = v.IndexOf('-');
		if (dash >= 0)
		{
			pre = v[(dash + 1)..];
			v = v[..dash];
		}

		var parts = v.Split('.');
		var nums = new int[3];
		for (var i = 0; i < 3 && i < parts.Length; i++)
		{
			int.TryParse(parts[i], out nums[i]);
		}

		return (nums, pre);
	}

	private static int ComparePrerelease(string a, string b)
	{
		var pa = a.Split('.');
		var pb = b.Split('.');
		var n = Math.Max(pa.Length, pb.Length);
		for (var i = 0; i < n; i++)
		{
			if (i >= pa.Length)
			{
				return -1;
			}

			if (i >= pb.Length)
			{
				return 1;
			}

			var isNumA = int.TryParse(pa[i], out var na);
			var isNumB = int.TryParse(pb[i], out var nb);
			if (isNumA && isNumB)
			{
				var c = na.CompareTo(nb);
				if (c != 0)
				{
					return c;
				}
			}
			else if (isNumA)
			{
				return -1;
			}
			else if (isNumB)
			{
				return 1;
			}
			else
			{
				var c = string.Compare(pa[i], pb[i], StringComparison.Ordinal);
				if (c != 0)
				{
					return c;
				}
			}
		}

		return 0;
	}
}
