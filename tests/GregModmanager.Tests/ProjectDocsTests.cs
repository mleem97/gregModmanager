using GregModmanager.Services;

namespace GregModmanager.Tests;

public class KeepAChangelogParserTests
{
	private const string Sample = """
		# Changelog

		The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
		and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

		## [Unreleased]

		### Added

		- Planned feature.

		## [1.1.0] - 2026-09-27

		### Fixed

		- Fixed crash on startup.

		## [1.0.0] - 2026-09-01

		### Added

		- Initial release.

		[1.1.0]: https://example.com/compare/1.0.0...1.1.0
		[1.0.0]: https://example.com/releases/1.0.0
		""";

	[Theory]
	[InlineData("1.0.0", true)]
	[InlineData("v1.0.0", true)]
	[InlineData("1.2.3-beta.1", true)]
	[InlineData("1.0.0+build.5", true)]
	[InlineData("0.0.1", true)]
	[InlineData("", false)]
	[InlineData("1.0", false)]
	[InlineData("1.0.0.0", false)]
	[InlineData("01.0.0", false)]
	[InlineData("latest", false)]
	public void IsValidSemVersion_Theory(string version, bool expected)
	{
		Assert.Equal(expected, KeepAChangelogParser.IsValidSemVersion(version));
	}

	[Fact]
	public void TryParseEntries_FindsAllSections()
	{
		Assert.True(KeepAChangelogParser.TryParseEntries(Sample, out var entries, out var error));
		Assert.Null(error);
		Assert.Equal(3, entries.Count);
		Assert.Equal("Unreleased", entries[0].Version);
		Assert.Equal("1.1.0", entries[1].Version);
		Assert.Equal("2026-09-27", entries[1].Date);
		Assert.Equal("1.0.0", entries[2].Version);
		// Link definitions at the bottom must not leak into the body.
		Assert.DoesNotContain("example.com", entries[2].Body);
		Assert.Contains("Initial release", entries[2].Body);
	}

	[Fact]
	public void TryGetEntryForVersion_ExactMatch()
	{
		Assert.True(KeepAChangelogParser.TryGetEntryForVersion(Sample, "1.1.0", out var entry));
		Assert.NotNull(entry);
		Assert.Equal("1.1.0", entry!.Version);
		Assert.Contains("crash", entry.Body);
	}

	[Fact]
	public void TryGetEntryForVersion_StripsLeadingV()
	{
		Assert.True(KeepAChangelogParser.TryGetEntryForVersion(Sample, "v1.0.0", out var entry));
		Assert.NotNull(entry);
		Assert.Equal("1.0.0", entry!.Version);
	}

	[Fact]
	public void TryGetEntryForVersion_MissingVersion_ReturnsFalse()
	{
		Assert.False(KeepAChangelogParser.TryGetEntryForVersion(Sample, "9.9.9", out _));
	}

	[Fact]
	public void TryGetLatestReleaseEntry_PicksHighestSemVer()
	{
		Assert.True(KeepAChangelogParser.TryGetLatestReleaseEntry(Sample, out var entry));
		Assert.NotNull(entry);
		Assert.Equal("1.1.0", entry!.Version);
	}

	[Fact]
	public void TryParseEntries_DuplicateVersion_Fails()
	{
		var dup = Sample + "\n## [1.0.0] - 2026-10-01\n\n### Fixed\n\n- Dup.\n";
		Assert.False(KeepAChangelogParser.TryParseEntries(dup, out _, out var error));
		Assert.Contains("Duplicate", error);
	}

	[Fact]
	public void TryParseEntries_NoSections_Fails()
	{
		Assert.False(KeepAChangelogParser.TryParseEntries("# Just a readme\n\nNo sections.", out _, out _));
	}

	[Fact]
	public void CompareSemVer_ReleaseBeatsPrerelease()
	{
		Assert.True(KeepAChangelogParser.CompareSemVer("1.0.0", "1.0.0-beta.1") > 0);
		Assert.True(KeepAChangelogParser.CompareSemVer("1.0.1", "1.0.0") > 0);
		Assert.Equal(0, KeepAChangelogParser.CompareSemVer("v2.0.0", "2.0.0"));
	}
}

public class MarkdownToSteamBbCodeTests
{
	[Fact]
	public void Convert_Headings()
	{
		var bb = MarkdownToSteamBbCode.Convert("# Title\n## Sub\n### Sub3\n#### Deep");
		Assert.Contains("[h1]Title[/h1]", bb);
		Assert.Contains("[h2]Sub[/h2]", bb);
		Assert.Contains("[h3]Sub3[/h3]", bb);
		Assert.Contains("[b]Deep[/b]", bb);
	}

	[Fact]
	public void Convert_InlineStyles()
	{
		var bb = MarkdownToSteamBbCode.Convert("**bold** and *italic* and `code` and ~~strike~~");
		Assert.Contains("[b]bold[/b]", bb);
		Assert.Contains("[i]italic[/i]", bb);
		Assert.Contains("[code]code[/code]", bb);
		Assert.Contains("[strike]strike[/strike]", bb);
	}

	[Fact]
	public void Convert_LinksAndImages()
	{
		var bb = MarkdownToSteamBbCode.Convert("[Docs](https://example.com) and ![shot](https://example.com/a.png)");
		Assert.Contains("[url=https://example.com]Docs[/url]", bb);
		Assert.Contains("[img]https://example.com/a.png[/img]", bb);
	}

	[Fact]
	public void Convert_UnorderedList()
	{
		var bb = MarkdownToSteamBbCode.Convert("- Item 1\n- Item 2");
		Assert.Contains("[list]", bb);
		Assert.Contains("[*] Item 1", bb);
		Assert.Contains("[/list]", bb);
	}

	[Fact]
	public void Convert_OrderedList()
	{
		var bb = MarkdownToSteamBbCode.Convert("1. First\n2. Second");
		Assert.Contains("[olist]", bb);
		Assert.Contains("[*] First", bb);
		Assert.Contains("[/olist]", bb);
	}

	[Fact]
	public void Convert_CodeBlock_Quote_Hr()
	{
		var bb = MarkdownToSteamBbCode.Convert("```\nsome **raw** code\n```\n\n> quoted\n\n---");
		Assert.Contains("[code]some **raw** code[/code]", bb);
		Assert.Contains("[quote]", bb);
		Assert.Contains("[hr][/hr]", bb);
	}

	[Fact]
	public void Convert_Table()
	{
		var bb = MarkdownToSteamBbCode.Convert("| A | B |\n| --- | --- |\n| 1 | 2 |");
		Assert.Contains("[table]", bb);
		Assert.Contains("[th]A[/th]", bb);
		Assert.Contains("[td]1[/td]", bb);
	}

	[Fact]
	public void Convert_Empty_ReturnsEmpty()
	{
		Assert.Equal(string.Empty, MarkdownToSteamBbCode.Convert(null));
		Assert.Equal(string.Empty, MarkdownToSteamBbCode.Convert("   "));
	}
}

public class ProjectDocsResolverTests : IDisposable
{
	private readonly string _root = Path.Combine(Path.GetTempPath(), "gm-docs-" + Guid.NewGuid().ToString("N"));

	public ProjectDocsResolverTests()
	{
		Directory.CreateDirectory(_root);
	}

	public void Dispose()
	{
		try { Directory.Delete(_root, true); } catch { /* best effort */ }
	}

	[Fact]
	public void ResolveSteamDescription_PrefersReadmeAsBbCode()
	{
		File.WriteAllText(Path.Combine(_root, "README.md"), "# My Mod\n\n**Bold** feature.");
		var resolved = ProjectDocsResolver.ResolveSteamDescription(_root, "fallback");
		Assert.True(resolved.FromFile);
		Assert.Contains("[h1]My Mod[/h1]", resolved.Text);
		Assert.Contains("[b]Bold[/b]", resolved.Text);
		Assert.DoesNotContain("fallback", resolved.Text);
	}

	[Fact]
	public void ResolveModstoreDescription_KeepsNativeMarkdown()
	{
		File.WriteAllText(Path.Combine(_root, "README.md"), "# My Mod\n\n**Bold** feature.");
		var resolved = ProjectDocsResolver.ResolveModstoreDescription(_root, "fallback");
		Assert.True(resolved.FromFile);
		Assert.Contains("**Bold**", resolved.Text);
		Assert.DoesNotContain("[b]", resolved.Text);
	}

	[Fact]
	public void ResolveSteamDescription_FallsBackWithoutFile()
	{
		var resolved = ProjectDocsResolver.ResolveSteamDescription(_root, "fallback desc");
		Assert.False(resolved.FromFile);
		Assert.Equal("fallback desc", resolved.Text);
	}

	[Fact]
	public void ResolveSteamChangelog_ManualWins()
	{
		File.WriteAllText(Path.Combine(_root, "CHANGELOG.md"), "# Changelog\n\n## [1.1.0] - 2026-09-27\n\n### Fixed\n\n- From file.\n");
		var resolved = ProjectDocsResolver.ResolveSteamChangelog(_root, "1.1.0", "Manual note");
		Assert.Equal(ProjectDocsResolver.ChangelogSource.Manual, resolved.Source);
		Assert.Equal("Manual note", resolved.Text);
	}

	[Fact]
	public void ResolveSteamChangelog_ReadsVersionSection()
	{
		File.WriteAllText(Path.Combine(_root, "CHANGELOG.md"), "# Changelog\n\n## [1.1.0] - 2026-09-27\n\n### Fixed\n\n- From file.\n");
		var resolved = ProjectDocsResolver.ResolveSteamChangelog(_root, "1.1.0", manualChangelog: null);
		Assert.Equal(ProjectDocsResolver.ChangelogSource.FileVersion, resolved.Source);
		Assert.Contains("From file", resolved.Text);
	}

	[Fact]
	public void ResolveSteamChangelog_FallsBackToUnreleased()
	{
		File.WriteAllText(Path.Combine(_root, "CHANGELOG.md"), "# Changelog\n\n## [Unreleased]\n\n### Added\n\n- Next thing.\n");
		var resolved = ProjectDocsResolver.ResolveSteamChangelog(_root, "2.0.0", manualChangelog: null);
		Assert.Equal(ProjectDocsResolver.ChangelogSource.FileUnreleased, resolved.Source);
	}

	[Fact]
	public void ResolveSteamChangelog_NoneWithoutFile()
	{
		var resolved = ProjectDocsResolver.ResolveSteamChangelog(_root, "1.0.0", manualChangelog: null);
		Assert.Equal(ProjectDocsResolver.ChangelogSource.None, resolved.Source);
		Assert.Equal("", resolved.Text);
	}

	[Fact]
	public void FormatSteamChangeNote_PrefixesVersion()
	{
		Assert.Equal("v1.1.0: Fixed X", ProjectDocsResolver.FormatSteamChangeNote("1.1.0", "Fixed X"));
		Assert.Equal("v1.1.0: Fixed X", ProjectDocsResolver.FormatSteamChangeNote("1.1.0", "v1.1.0: Fixed X"));
		Assert.Equal("v1.0.0", ProjectDocsResolver.FormatSteamChangeNote("1.0.0", ""));
	}

	[Fact]
	public void FindReadme_FindsLowercaseVariant()
	{
		File.WriteAllText(Path.Combine(_root, "readme.md"), "hi");
		Assert.NotNull(ProjectDocsResolver.FindReadme(_root));
	}
}
