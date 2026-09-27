using System.Text.RegularExpressions;
using GregModmanager.Models;
using GregModmanager.Steam;

namespace GregModmanager.Services;

public static class UploadDependencyChecker
{
	public static List<UploadCheckResult> Check(string projectRoot, WorkshopMetadata metadata, string? changeLog = null)
	{
		var results = new List<UploadCheckResult>();

		CheckContentFolder(projectRoot, results);
		if (Directory.Exists(Path.Combine(projectRoot, "content")))
		{
			CheckNativeConfigJson(projectRoot, results);
		}
		CheckMetadataFields(metadata, results);
		CheckVersion(metadata, results);
		CheckProjectDocs(projectRoot, metadata, results);
		CheckPreviewImage(projectRoot, metadata, results);
		CheckTags(metadata, results);
		CheckContentSize(projectRoot, results);
		CheckChangelog(projectRoot, metadata, changeLog, results);
		CheckGregFrameworkDependency(projectRoot, metadata, results);

		return results;
	}

	public static bool IsReadyToUpload(List<UploadCheckResult> results)
	{
		return results.All(r => r.Severity != UploadCheckSeverity.Error);
	}

	private static void CheckNativeConfigJson(string projectRoot, List<UploadCheckResult> results)
	{
		var path = Path.Combine(projectRoot, "content", "config.json");
		if (File.Exists(path))
		{
			results.Add(new UploadCheckResult
			{
				Label = "config.json",
				Severity = UploadCheckSeverity.Ok,
				Detail = "Native mod definition present under content/.",
			});
			return;
		}

		results.Add(new UploadCheckResult
		{
			Label = "config.json",
			Severity = UploadCheckSeverity.Warning,
			Detail =
				"content/config.json is missing. It is required for native shop/static items when you ship vanilla assets. " +
				"Use the native config editor in the workshop project editor to create it.",
		});
	}

	private static void CheckContentFolder(string projectRoot, List<UploadCheckResult> results)
	{
		var contentDir = Path.Combine(projectRoot, "content");
		if (!Directory.Exists(contentDir))
		{
			results.Add(new UploadCheckResult
			{
				Label = "Content folder",
				Severity = UploadCheckSeverity.Error,
				Detail = "content/ folder is missing. Create it and add your files before uploading.",
			});
			return;
		}

		var fileCount = 0;
		foreach (var _ in Directory.EnumerateFiles(contentDir, "*", SearchOption.AllDirectories))
		{
			fileCount++;
			if (fileCount > 0) break;
		}

		if (fileCount == 0)
		{
			results.Add(new UploadCheckResult
			{
				Label = "Content folder",
				Severity = UploadCheckSeverity.Error,
				Detail = "content/ folder is empty. Add files to upload.",
			});
		}
		else
		{
			var total = Directory.EnumerateFiles(contentDir, "*", SearchOption.AllDirectories).Count();
			results.Add(new UploadCheckResult
			{
				Label = "Content folder",
				Severity = UploadCheckSeverity.Ok,
				Detail = $"content/ contains {total} file(s).",
			});
		}
	}

	private static void CheckMetadataFields(WorkshopMetadata metadata, List<UploadCheckResult> results)
	{
		if (string.IsNullOrWhiteSpace(metadata.Title))
		{
			results.Add(new UploadCheckResult
			{
				Label = "Title",
				Severity = UploadCheckSeverity.Error,
				Detail = "Title is empty. Steam requires a title.",
			});
		}
		else if (metadata.Title.Length > SteamConstants.MaxTitleLength)
		{
			results.Add(new UploadCheckResult
			{
				Label = "Title",
				Severity = UploadCheckSeverity.Error,
				Detail = $"Title exceeds {SteamConstants.MaxTitleLength} characters ({metadata.Title.Length}).",
			});
		}
		else
		{
			results.Add(new UploadCheckResult
			{
				Label = "Title",
				Severity = UploadCheckSeverity.Ok,
				Detail = $"\"{metadata.Title}\" ({metadata.Title.Length}/{SteamConstants.MaxTitleLength})",
			});
		}

		if (string.IsNullOrWhiteSpace(metadata.Description))
		{
			results.Add(new UploadCheckResult
			{
				Label = "Description",
				Severity = UploadCheckSeverity.Warning,
				Detail = "Description is empty. Recommended for discoverability.",
			});
		}
		else if (metadata.Description.Length > SteamConstants.MaxDescriptionLength)
		{
			results.Add(new UploadCheckResult
			{
				Label = "Description",
				Severity = UploadCheckSeverity.Error,
				Detail = $"Description exceeds {SteamConstants.MaxDescriptionLength} characters.",
			});
		}
		else
		{
			results.Add(new UploadCheckResult
			{
				Label = "Description",
				Severity = UploadCheckSeverity.Ok,
				Detail = $"{metadata.Description.Length}/{SteamConstants.MaxDescriptionLength} characters.",
			});
		}

		var validVisibilities = new[] { "Public", "FriendsOnly", "Private" };
		if (!validVisibilities.Contains(metadata.Visibility))
		{
			results.Add(new UploadCheckResult
			{
				Label = "Visibility",
				Severity = UploadCheckSeverity.Warning,
				Detail = $"Unknown visibility \"{metadata.Visibility}\". Expected: Public, FriendsOnly, or Private.",
			});
		}
		else
		{
			results.Add(new UploadCheckResult
			{
				Label = "Visibility",
				Severity = UploadCheckSeverity.Ok,
				Detail = metadata.Visibility,
			});
		}
	}

	private static void CheckPreviewImage(string projectRoot, WorkshopMetadata metadata, List<UploadCheckResult> results)
	{
		if (string.IsNullOrWhiteSpace(metadata.PreviewImageRelativePath))
		{
			results.Add(new UploadCheckResult
			{
				Label = "Preview image",
				Severity = UploadCheckSeverity.Error,
				Detail = "A preview image is required for every Workshop upload and update.",
			});
			return;
		}

		var previewPath = Path.Combine(projectRoot, metadata.PreviewImageRelativePath);
		if (!File.Exists(previewPath))
		{
			results.Add(new UploadCheckResult
			{
				Label = "Preview image",
				Severity = UploadCheckSeverity.Error,
				Detail = $"File not found: {metadata.PreviewImageRelativePath}. Select a preview image before uploading.",
			});
		}
		else
		{
			var size = new FileInfo(previewPath).Length;
			if (size > SteamConstants.MaxPreviewImageBytes)
			{
				results.Add(new UploadCheckResult
				{
					Label = "Preview image",
					Severity = UploadCheckSeverity.Error,
					Detail = $"Preview image is {WorkspaceService.FormatBytes(size)}. Steam allows at most {WorkspaceService.FormatBytes(SteamConstants.MaxPreviewImageBytes)}.",
				});
			}
			else
			{
				results.Add(new UploadCheckResult
				{
					Label = "Preview image",
					Severity = UploadCheckSeverity.Ok,
					Detail = $"{metadata.PreviewImageRelativePath} ({WorkspaceService.FormatBytes(size)})",
				});
			}
		}
	}

	private static void CheckTags(WorkshopMetadata metadata, List<UploadCheckResult> results)
	{
		if (metadata.Tags.Count == 0)
		{
			results.Add(new UploadCheckResult
			{
				Label = "Tags",
				Severity = UploadCheckSeverity.Warning,
				Detail = "No tags set. Tags help users find your content.",
			});
		}
		else
		{
			results.Add(new UploadCheckResult
			{
				Label = "Tags",
				Severity = UploadCheckSeverity.Ok,
				Detail = string.Join(", ", metadata.Tags),
			});
		}
	}

	private static void CheckContentSize(string projectRoot, List<UploadCheckResult> results)
	{
		var contentDir = Path.Combine(projectRoot, "content");
		if (!Directory.Exists(contentDir)) return;

		long totalBytes = 0;
		try
		{
			foreach (var f in Directory.EnumerateFiles(contentDir, "*", SearchOption.AllDirectories))
			{
				totalBytes += new FileInfo(f).Length;
			}
		}
		catch
		{
			return;
		}

		const long warnThreshold = 100L * 1024 * 1024;
		if (totalBytes > warnThreshold)
		{
			results.Add(new UploadCheckResult
			{
				Label = "Content size",
				Severity = UploadCheckSeverity.Warning,
				Detail = $"Total content size is {WorkspaceService.FormatBytes(totalBytes)}. Large uploads take longer.",
			});
		}
		else
		{
			results.Add(new UploadCheckResult
			{
				Label = "Content size",
				Severity = UploadCheckSeverity.Ok,
				Detail = WorkspaceService.FormatBytes(totalBytes),
			});
		}
	}

	private static void CheckVersion(WorkshopMetadata metadata, List<UploadCheckResult> results)
	{
		var version = (metadata.Version ?? "").Trim();
		if (string.IsNullOrWhiteSpace(version))
		{
			results.Add(new UploadCheckResult
			{
				Label = "Version",
				Severity = UploadCheckSeverity.Error,
				Detail = "Version is empty. Use Semantic Versioning (e.g. 1.0.0).",
			});
			return;
		}

		if (!KeepAChangelogParser.IsValidSemVersion(version))
		{
			results.Add(new UploadCheckResult
			{
				Label = "Version",
				Severity = UploadCheckSeverity.Error,
				Detail = $"\"{version}\" is not valid Semantic Versioning (expected MAJOR.MINOR.PATCH, e.g. 1.2.0).",
			});
			return;
		}

		results.Add(new UploadCheckResult
		{
			Label = "Version",
			Severity = UploadCheckSeverity.Ok,
			Detail = $"v{KeepAChangelogParser.NormalizeVersion(version)} (SemVer).",
		});
	}

	/// <summary>Reports README.md / CHANGELOG.md file sources used for Steam (BBCode) vs. Modstore (native Markdown).</summary>
	private static void CheckProjectDocs(string projectRoot, WorkshopMetadata metadata, List<UploadCheckResult> results)
	{
		var readme = ProjectDocsResolver.FindReadme(projectRoot);
		if (readme is not null)
		{
			var converted = ProjectDocsResolver.ResolveSteamDescription(projectRoot, metadata.Description).Text;
			var detail = $"README.md found ({Path.GetFileName(readme)}) — Steam uses converted BBCode ({converted.Length}/{SteamConstants.MaxDescriptionLength}). Modstore keeps native Markdown.";
			results.Add(new UploadCheckResult
			{
				Label = "README.md",
				Severity = converted.Length > SteamConstants.MaxDescriptionLength ? UploadCheckSeverity.Error : UploadCheckSeverity.Ok,
				Detail = converted.Length > SteamConstants.MaxDescriptionLength
					? detail + " Shorten the README: Steam allows at most 8000 characters."
					: detail,
			});
		}
		else
		{
			results.Add(new UploadCheckResult
			{
				Label = "README.md",
				Severity = UploadCheckSeverity.Warning,
				Detail = "No README.md in the project root — Steam uses the metadata description. Add README.md (Markdown) to share one source with the Modstore.",
			});
		}

		var changelogPath = ProjectDocsResolver.FindChangelog(projectRoot);
		if (changelogPath is null)
		{
			return;
		}

		string markdown;
		try
		{
			markdown = File.ReadAllText(changelogPath);
		}
		catch
		{
			results.Add(new UploadCheckResult
			{
				Label = "CHANGELOG.md",
				Severity = UploadCheckSeverity.Warning,
				Detail = $"Found {Path.GetFileName(changelogPath)} but could not read it.",
			});
			return;
		}

		if (!KeepAChangelogParser.TryParseEntries(markdown, out _, out var parseError))
		{
			results.Add(new UploadCheckResult
			{
				Label = "CHANGELOG.md",
				Severity = UploadCheckSeverity.Error,
				Detail = $"{Path.GetFileName(changelogPath)} is not Keep a Changelog format: {parseError}",
			});
			return;
		}

		var version = KeepAChangelogParser.NormalizeVersion(metadata.Version) ?? "1.0.0";
		if (KeepAChangelogParser.TryGetEntryForVersion(markdown, version, out var entry) && entry is not null)
		{
			var from = entry.Version == "Unreleased" ? "[Unreleased] (mentions this version)" : $"[{entry.Version}]";
			results.Add(new UploadCheckResult
			{
				Label = "CHANGELOG.md",
				Severity = UploadCheckSeverity.Ok,
				Detail = $"{Path.GetFileName(changelogPath)} {from} will be used for v{version} — no manual input needed.",
			});
		}
		else
		{
			results.Add(new UploadCheckResult
			{
				Label = "CHANGELOG.md",
				Severity = UploadCheckSeverity.Warning,
				Detail = $"{Path.GetFileName(changelogPath)} has no section for v{version}. Add '## [{version}] - YYYY-MM-DD' (Keep a Changelog) or enter notes manually.",
			});
		}
	}

	private static void CheckChangelog(string projectRoot, WorkshopMetadata metadata, string? changeLog, List<UploadCheckResult> results)
	{
		var isFirstPublish = metadata.PublishedFileId == 0;
		var resolved = ProjectDocsResolver.ResolveSteamChangelog(projectRoot, metadata.Version, changeLog);
		var hasChangelog = !string.IsNullOrWhiteSpace(resolved.Text);

		if (isFirstPublish && !hasChangelog)
		{
			results.Add(new UploadCheckResult
			{
				Label = "Changelog",
				Severity = UploadCheckSeverity.Error,
				Detail = "A version changelog is required for the first publish. Add '## [x.y.z]' to CHANGELOG.md (Keep a Changelog) or describe the initial release manually.",
			});
		}
		else if (!isFirstPublish && !hasChangelog)
		{
			results.Add(new UploadCheckResult
			{
				Label = "Changelog",
				Severity = UploadCheckSeverity.Warning,
				Detail = "No changelog provided. Recommended so subscribers know what changed.",
			});
		}
		else
		{
			var source = resolved.Source switch
			{
				ProjectDocsResolver.ChangelogSource.Manual => "manual input",
				ProjectDocsResolver.ChangelogSource.FileVersion => $"CHANGELOG.md [{resolved.EntryVersion}]",
				ProjectDocsResolver.ChangelogSource.FileUnreleased => "CHANGELOG.md [Unreleased]",
				_ => "manual input",
			};
			results.Add(new UploadCheckResult
			{
				Label = "Changelog",
				Severity = UploadCheckSeverity.Ok,
				Detail = $"{resolved.Text.Length} characters ({source}).",
			});
		}
	}

	/// <summary>Aligns <see cref="WorkshopMetadata.Needsgreg"/> with description/tags (gregCoreModFramework / GregFramework).</summary>
	private static void CheckGregFrameworkDependency(string projectRoot, WorkshopMetadata metadata, List<UploadCheckResult> results)
	{
		var desc = ProjectDocsResolver.ResolveSteamDescription(projectRoot, metadata.Description).Text;
		var descMentionsgreg = ContainsgregHint(desc);
		var tagsSuggestgreg = metadata.Tags.Any(t => TagSuggestsgreg(t));

		if (metadata.Needsgreg)
		{
			if (!descMentionsgreg)
			{
				results.Add(new UploadCheckResult
				{
					Label = "GregFramework (greg)",
					Severity = UploadCheckSeverity.Warning,
					Detail =
						"Marked as requiring gregCoreModFramework, but the description does not mention it yet. " +
						"A standard notice is still appended automatically on upload if missing.",
				});
			}
			else
			{
				results.Add(new UploadCheckResult
				{
					Label = "GregFramework (greg)",
					Severity = UploadCheckSeverity.Ok,
					Detail = "Requires gregCoreModFramework / GregFramework — description mentions the framework.",
				});
			}
		}
		else
		{
			if (tagsSuggestgreg || descMentionsgreg)
			{
				results.Add(new UploadCheckResult
				{
					Label = "GregFramework (greg)",
					Severity = UploadCheckSeverity.Warning,
					Detail =
						"Tags or description suggest greg — enable “Needs gregCoreModFramework” if players must install GregFramework.",
				});
			}
			else
			{
				results.Add(new UploadCheckResult
				{
					Label = "GregFramework (greg)",
					Severity = UploadCheckSeverity.Ok,
					Detail = "Not flagged as requiring gregCoreModFramework (GregFramework).",
				});
			}
		}
	}

	private static bool TagSuggestsgreg(string tag)
	{
		var t = tag.Trim().ToLowerInvariant();
		return t is "greg" or "gregCore-mod-framework" or "gregframework" or "gregCore" or "gregCoremodframework"
			|| t.Contains("gregCoremod", StringComparison.Ordinal)
			|| t.Contains("gregframework", StringComparison.Ordinal);
	}

	private static bool ContainsgregHint(string text)
	{
		if (string.IsNullOrWhiteSpace(text))
		{
			return false;
		}

		var s = text;
		return s.Contains("gregCoreModFramework", StringComparison.OrdinalIgnoreCase)
			|| s.Contains("GregFramework", StringComparison.OrdinalIgnoreCase)
			|| s.Contains("Greg Tools", StringComparison.OrdinalIgnoreCase)
			|| s.Contains("gregCore Mod Framework", StringComparison.OrdinalIgnoreCase)
			|| Regex.IsMatch(s, @"\bgreg\b", RegexOptions.IgnoreCase);
	}
}

