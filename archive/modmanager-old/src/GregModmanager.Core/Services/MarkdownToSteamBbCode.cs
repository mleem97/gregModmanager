using System.Text;
using System.Text.RegularExpressions;

namespace GregModmanager.Services;

/// <summary>
/// Converts project <c>README.md</c> (Markdown) to Steam Workshop BBCode.
/// Steam understands only its BBCode subset (see <see cref="SteamBbCodePreview"/>);
/// the Modstore keeps native Markdown, so this conversion is Steam-only.
/// </summary>
public static partial class MarkdownToSteamBbCode
{
	[GeneratedRegex(@"`([^`\n]+)`")]
	private static partial Regex InlineCodeRegex();

	[GeneratedRegex(@"!\[([^\]]*)\]\(([^)\s]+)(?:\s+""[^""]*"")?\)")]
	private static partial Regex ImageRegex();

	[GeneratedRegex(@"\[([^\]]+)\]\(([^)\s]+)(?:\s+""[^""]*"")?\)")]
	private static partial Regex InlineLinkRegex();

	[GeneratedRegex(@"\[([^\]]+)\]\[([^\]]*)\]")]
	private static partial Regex ReferenceLinkRegex();

	[GeneratedRegex(@"(?m)^\s*\[([^\]]+)\]:\s*(\S+).*$")]
	private static partial Regex LinkDefinitionRegex();

	[GeneratedRegex(@"\*\*([^*]+)\*\*|__([^_]+)__")]
	private static partial Regex BoldRegex();

	[GeneratedRegex(@"~~([^~]+)~~")]
	private static partial Regex StrikeRegex();

	[GeneratedRegex(@"<((?:https?|ftp):[^<>\s]+)>")]
	private static partial Regex AutolinkRegex();

	/// <summary>Full Markdown document to Steam BBCode.</summary>
	public static string Convert(string? markdown)
	{
		if (string.IsNullOrWhiteSpace(markdown))
		{
			return string.Empty;
		}

		var text = markdown.Replace("\r\n", "\n").Replace("\r", "\n");

		// Collect reference link definitions first so [text][ref] can resolve.
		var references = new Dictionary<string, string>(StringComparer.OrdinalIgnoreCase);
		foreach (Match m in LinkDefinitionRegex().Matches(text))
		{
			references[m.Groups[1].Value.Trim()] = m.Groups[2].Value.Trim();
		}

		text = LinkDefinitionRegex().Replace(text, "");

		var sb = new StringBuilder(text.Length);
		var lines = text.Split('\n');

		var inCodeBlock = false;
		var codeBuffer = new StringBuilder();
		var listKind = ListKind.None;
		var listItems = new List<string>();
		var quoteBuffer = new List<string>();
		var tableRows = new List<string[]>();
		var tableHasHeader = false;

		void FlushList()
		{
			if (listKind == ListKind.None || listItems.Count == 0)
			{
				listItems.Clear();
				listKind = ListKind.None;
				return;
			}

			sb.Append(listKind == ListKind.Ordered ? "[olist]\n" : "[list]\n");
			foreach (var item in listItems)
			{
				sb.Append("[*] ").Append(item).Append('\n');
			}

			sb.Append(listKind == ListKind.Ordered ? "[/olist]\n" : "[/list]\n");
			listItems.Clear();
			listKind = ListKind.None;
		}

		void FlushQuote()
		{
			if (quoteBuffer.Count == 0)
			{
				return;
			}

			sb.Append("[quote]\n");
			foreach (var q in quoteBuffer)
			{
				sb.Append(ConvertInline(q, references)).Append('\n');
			}

			sb.Append("[/quote]\n");
			quoteBuffer.Clear();
		}

		void FlushTable()
		{
			if (tableRows.Count == 0)
			{
				return;
			}

			sb.Append("[table]\n");
			for (var r = 0; r < tableRows.Count; r++)
			{
				var row = tableRows[r];
				var isHeader = r == 0 && tableHasHeader;
				sb.Append("[tr]\n");
				foreach (var cell in row)
				{
					var tag = isHeader ? "th" : "td";
					sb.Append('[').Append(tag).Append(']')
						.Append(ConvertInline(cell.Trim(), references))
						.Append("[/").Append(tag).Append("]\n");
				}

				sb.Append("[/tr]\n");
			}

			sb.Append("[/table]\n");
			tableRows.Clear();
			tableHasHeader = false;
		}

		for (var i = 0; i < lines.Length; i++)
		{
			var line = lines[i];

			// Fenced code blocks -> [code]...[/code]
			if (line.TrimStart().StartsWith("```", StringComparison.Ordinal))
			{
				if (!inCodeBlock)
				{
					FlushList();
					FlushQuote();
					FlushTable();
					inCodeBlock = true;
					codeBuffer.Clear();
				}
				else
				{
					inCodeBlock = false;
					sb.Append("[code]").Append(codeBuffer.ToString().Trim('\n')).Append("[/code]\n");
					codeBuffer.Clear();
				}

				continue;
			}

			if (inCodeBlock)
			{
				codeBuffer.Append(line).Append('\n');
				continue;
			}

			// Indented code block (4 spaces) — treat consecutive ones as code.
			if (line.StartsWith("    ", StringComparison.Ordinal) && line.Trim().Length > 0)
			{
				FlushList();
				FlushQuote();
				FlushTable();
				sb.Append("[code]").Append(line.Trim()).Append("[/code]\n");
				continue;
			}

			var trimmed = line.Trim();

			// Table rows: | a | b | (header separator | --- | marks header table)
			if (trimmed.StartsWith('|') && trimmed.EndsWith('|') && trimmed.Contains('|'))
			{
				var cells = trimmed.Trim('|').Split('|').Select(c => c.Trim()).ToArray();
				var isSeparator = cells.Length > 0 && cells.All(c => c.Length > 0 && c.All(ch => ch is '-' or ':' or ' '));
				if (isSeparator)
				{
					tableHasHeader = tableRows.Count > 0;
					continue;
				}

				// A table starts when the next line is a separator row.
				if (tableRows.Count == 0)
				{
					var next = i + 1 < lines.Length ? lines[i + 1].Trim() : "";
					var nextCells = next.Trim('|').Split('|').Select(c => c.Trim()).ToArray();
					var nextIsSep = next.StartsWith('|') && nextCells.Length > 0 && nextCells.All(c => c.Length > 0 && c.All(ch => ch is '-' or ':' or ' '));
					if (!nextIsSep)
					{
						// Not a table — inline fallback with pipes preserved.
						FlushList();
						FlushQuote();
						sb.Append(ConvertInline(line.Trim(), references)).Append("\n");
						continue;
					}

					FlushList();
					FlushQuote();
				}

				tableRows.Add(cells);
				continue;
			}

			if (tableRows.Count > 0)
			{
				FlushTable();
			}

			// Horizontal rule
			if (trimmed is "---" or "***" or "___" || (trimmed.Length >= 3 && trimmed.All(c => c is '-' or '*' or '_' or ' ') && trimmed.Distinct().Count() <= 2 && trimmed.Trim(' ', '-', '*', '_').Length == 0))
			{
				FlushList();
				FlushQuote();
				sb.Append("[hr][/hr]\n");
				continue;
			}

			// ATX headings
			if (trimmed.StartsWith('#'))
			{
				FlushList();
				FlushQuote();
				var level = 0;
				while (level < trimmed.Length && trimmed[level] == '#')
				{
					level++;
				}

				var heading = trimmed[level..].Trim().TrimEnd('#').Trim();
				var inline = ConvertInline(heading, references);
				sb.Append(level switch
				{
					1 => $"[h1]{inline}[/h1]\n",
					2 => $"[h2]{inline}[/h2]\n",
					3 => $"[h3]{inline}[/h3]\n",
					_ => $"[b]{inline}[/b]\n",
				});
				continue;
			}

			// Blockquote (consecutive > lines form one block)
			if (trimmed.StartsWith("> "))
			{
				FlushList();
				quoteBuffer.Add(trimmed[2..]);
				var next = i + 1 < lines.Length ? lines[i + 1].Trim() : null;
				if (next is null || !next.StartsWith("> "))
				{
					FlushQuote();
				}

				continue;
			}

			if (trimmed == ">")
			{
				quoteBuffer.Add("");
				continue;
			}

			if (quoteBuffer.Count > 0)
			{
				FlushQuote();
			}

			// Unordered list items (-, *, +), task lists included.
			if (TryParseListItem(trimmed, out var ordered, out var itemText))
			{
				var kind = ordered ? ListKind.Ordered : ListKind.Unordered;
				if (listKind != ListKind.None && listKind != kind)
				{
					FlushList();
				}

				listKind = kind;
				listItems.Add(ConvertInline(itemText, references));
				var next = i + 1 < lines.Length ? lines[i + 1].Trim() : null;
				if (next is null || !TryParseListItem(next, out _, out _))
				{
					FlushList();
				}

				continue;
			}

			if (listKind != ListKind.None)
			{
				FlushList();
			}

			// Blank line = paragraph break (collapse multiples later).
			if (trimmed.Length == 0)
			{
				sb.Append('\n');
				continue;
			}

			sb.Append(ConvertInline(line.Trim(), references)).Append('\n');
		}

		FlushList();
		FlushQuote();
		FlushTable();
		if (inCodeBlock)
		{
			sb.Append("[code]").Append(codeBuffer.ToString().Trim('\n')).Append("[/code]\n");
		}

		// Collapse 3+ newlines to 2, trim edges.
		var result = Regex.Replace(sb.ToString().Replace("\r", ""), @"\n{3,}", "\n\n").Trim();
		return result;
	}

	private enum ListKind
	{
		None,
		Unordered,
		Ordered,
	}

	private static bool TryParseListItem(string trimmed, out bool ordered, out string text)
	{
		ordered = false;
		text = "";
		if (trimmed.Length < 2)
		{
			return false;
		}

		// Unordered: - item, * item, + item
		if ((trimmed[0] is '-' or '*' or '+') && char.IsWhiteSpace(trimmed[1]))
		{
			text = StripTaskMarker(trimmed[1..].Trim());
			return true;
		}

		// Ordered: 1. item, 1) item
		var dot = trimmed.IndexOf('.');
		var paren = trimmed.IndexOf(')');
		var sep = dot >= 0 && paren >= 0 ? Math.Min(dot, paren) : Math.Max(dot, paren);
		if (sep > 0 && sep < 5 && trimmed[..sep].All(char.IsDigit))
		{
			var rest = trimmed[(sep + 1)..];
			if (rest.Length > 0 && char.IsWhiteSpace(rest[0]))
			{
				ordered = true;
				text = StripTaskMarker(rest.Trim());
				return true;
			}
		}

		return false;
	}

	private static string StripTaskMarker(string text)
	{
		if (text.StartsWith("[ ] ", StringComparison.Ordinal) || text.StartsWith("[x] ", StringComparison.OrdinalIgnoreCase))
		{
			return text[4..];
		}

		return text;
	}

	/// <summary>Inline Markdown (bold, italic, code, links, images) to BBCode.</summary>
	public static string ConvertInline(string? text, Dictionary<string, string>? references = null)
	{
		if (string.IsNullOrEmpty(text))
		{
			return string.Empty;
		}

		// Protect inline code spans with placeholders.
		var codeSpans = new List<string>();
		var s = InlineCodeRegex().Replace(text, m =>
		{
			codeSpans.Add(m.Groups[1].Value);
			return $"\u0000CODE{codeSpans.Count - 1}\u0000";
		});

		s = ImageRegex().Replace(s, m => $"[img]{m.Groups[2].Value.Trim()}[/img]");

		s = InlineLinkRegex().Replace(s, m => $"[url={m.Groups[2].Value.Trim()}]{m.Groups[1].Value.Trim()}[/url]");

		if (references is not null)
		{
			s = ReferenceLinkRegex().Replace(s, m =>
			{
				var label = string.IsNullOrEmpty(m.Groups[2].Value) ? m.Groups[1].Value : m.Groups[2].Value;
				return references.TryGetValue(label.Trim(), out var url)
					? $"[url={url}]{m.Groups[1].Value.Trim()}[/url]"
					: m.Groups[1].Value;
			});
		}

		s = AutolinkRegex().Replace(s, m => $"[url={m.Groups[1].Value}]{m.Groups[1].Value}[/url]");

		s = BoldRegex().Replace(s, m => $"[b]{(m.Groups[1].Success ? m.Groups[1].Value : m.Groups[2].Value)}[/b]");
		s = StrikeRegex().Replace(s, m => $"[strike]{m.Groups[1].Value}[/strike]");

		// Italic after bold: *x* and _x_ (avoid list artifacts and urls).
		s = ConvertItalics(s);

		for (var i = 0; i < codeSpans.Count; i++)
		{
			s = s.Replace($"\u0000CODE{i}\u0000", $"[code]{codeSpans[i]}[/code]");
		}

		return s;
	}

	private static string ConvertItalics(string s)
	{
		// *italic* — skip ** (already handled) and [*] list markers.
		s = Regex.Replace(s, @"(?<!\*)\*(?!\*|\s)([^*\n]+?)(?<!\s)\*(?!\*)", "[i]$1[/i]");
		// _italic_ — skip __ and words_like_this.
		s = Regex.Replace(s, @"(?<![\w_])_(?!_|\s)([^_\n]+?)(?<!\s)_(?![\w_])", "[i]$1[/i]");
		return s;
	}
}
