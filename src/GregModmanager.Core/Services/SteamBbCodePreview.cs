using System.Net;
using System.Text;

namespace GregModmanager.Services;

/// <summary>
/// Renders Steam Workshop BBCode to HTML for a live preview (subset of
/// <see href="https://steamcommunity.com/comment/forum/formattinghelp">Steam formatting</see>).
/// </summary>
public static class SteamBbCodePreview
{
	private const int MaxInputLength = 400_000;

	public static string ToHtmlDocument(string? bbcode)
	{
		var body = ConvertToHtml(bbcode);
		return
			"<!DOCTYPE html><html><head><meta charset=\"utf-8\"/><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"/>" +
			"<style>" + Css + "</style></head><body>" + body + "</body></html>";
	}

	private static string ConvertToHtml(string? bbcode)
	{
		if (string.IsNullOrEmpty(bbcode))
		{
			return "<p class=\"empty\">—</p>";
		}

		var s = bbcode.Length > MaxInputLength ? bbcode[..MaxInputLength] : bbcode;
		var sb = new StringBuilder(s.Length * 2);
		var i = 0;
		ParseContent(sb, s, ref i, null, false);
		return sb.Length == 0 ? "<p class=\"empty\">—</p>" : sb.ToString();
	}

	private static void ParseContent(StringBuilder sb, string s, ref int i, string? closeUntil, bool listItemInner)
	{
		while (i < s.Length)
		{
			if (listItemInner && s[i] == '[' && TryPeekTag(s, i, out var boundary))
			{
				if (!boundary.IsClose && boundary.Name == "*")
				{
					return;
				}

				if (boundary.IsClose && string.Equals(boundary.Name, "list", StringComparison.OrdinalIgnoreCase))
				{
					return;
				}

				if (boundary.IsClose && string.Equals(boundary.Name, "olist", StringComparison.OrdinalIgnoreCase))
				{
					return;
				}
			}

			if (s[i] != '[')
			{
				var start = i;
				while (i < s.Length && s[i] != '[')
				{
					i++;
				}

				AppendEncoded(sb, s.AsSpan(start, i - start));
				continue;
			}

			var tagStart = i;
			if (!TryReadTag(s, ref i, out var tag))
			{
				sb.Append(HtmlEncodeChar(s[tagStart]));
				i = tagStart + 1;
				continue;
			}

			if (tag.IsClose)
			{
				if (closeUntil != null && string.Equals(tag.Name, closeUntil, StringComparison.OrdinalIgnoreCase))
				{
					return;
				}

				AppendEncoded(sb, s.AsSpan(tagStart, i - tagStart));
				continue;
			}

			var li = listItemInner;
			switch (tag.Name)
			{
				case "b":
					sb.Append("<strong>");
					ParseContent(sb, s, ref i, "b", li);
					sb.Append("</strong>");
					break;
				case "i":
					sb.Append("<em>");
					ParseContent(sb, s, ref i, "i", li);
					sb.Append("</em>");
					break;
				case "u":
					sb.Append("<u>");
					ParseContent(sb, s, ref i, "u", li);
					sb.Append("</u>");
					break;
				case "strike":
					sb.Append("<s>");
					ParseContent(sb, s, ref i, "strike", li);
					sb.Append("</s>");
					break;
				case "h1":
					sb.Append("<h1>");
					ParseContent(sb, s, ref i, "h1", li);
					sb.Append("</h1>");
					break;
				case "h2":
					sb.Append("<h2>");
					ParseContent(sb, s, ref i, "h2", li);
					sb.Append("</h2>");
					break;
				case "h3":
					sb.Append("<h3>");
					ParseContent(sb, s, ref i, "h3", li);
					sb.Append("</h3>");
					break;
				case "code":
					sb.Append("<code>");
					ParseContent(sb, s, ref i, "code", li);
					sb.Append("</code>");
					break;
				case "quote":
					sb.Append("<blockquote>");
					ParseContent(sb, s, ref i, "quote", li);
					sb.Append("</blockquote>");
					break;
				case "spoiler":
					sb.Append("<details class=\"spoiler\"><summary>Spoiler</summary><div>");
					ParseContent(sb, s, ref i, "spoiler", li);
					sb.Append("</div></details>");
					break;
				case "noparse":
				{
					var raw = ReadRawUntilClose(s, ref i, "noparse");
					AppendEncoded(sb, raw.AsSpan());
					break;
				}
				case "url":
					if (!string.IsNullOrEmpty(tag.Value))
					{
						sb.Append("<a href=\"");
						AppendAttr(sb, tag.Value);
						sb.Append("\" target=\"_blank\" rel=\"noopener noreferrer\">");
						ParseContent(sb, s, ref i, "url", li);
						sb.Append("</a>");
					}
					else
					{
						var inner = ReadRawUntilClose(s, ref i, "url").Trim();
						sb.Append("<a href=\"");
						AppendAttr(sb, inner);
						sb.Append("\" target=\"_blank\" rel=\"noopener noreferrer\">");
						AppendEncoded(sb, inner.AsSpan());
						sb.Append("</a>");
					}

					break;
				case "img":
				{
					var inner = ReadRawUntilClose(s, ref i, "img").Trim();
					if (inner.Length > 0)
					{
						sb.Append("<img src=\"");
						AppendAttr(sb, inner);
						sb.Append("\" alt=\"\" loading=\"lazy\"/>");
					}

					break;
				}
				case "hr":
					if (PeekClose(s, i, "hr"))
					{
						SkipClose(s, ref i, "hr");
					}

					sb.Append("<hr/>");
					break;
				case "list":
					ParseList(sb, s, ref i, ordered: false);
					break;
				case "olist":
					ParseList(sb, s, ref i, ordered: true);
					break;
				case "table":
					sb.Append("<table>");
					ParseContent(sb, s, ref i, "table", li);
					sb.Append("</table>");
					break;
				case "tr":
					sb.Append("<tr>");
					ParseContent(sb, s, ref i, "tr", li);
					sb.Append("</tr>");
					break;
				case "th":
					sb.Append("<th>");
					ParseContent(sb, s, ref i, "th", li);
					sb.Append("</th>");
					break;
				case "td":
					sb.Append("<td>");
					ParseContent(sb, s, ref i, "td", li);
					sb.Append("</td>");
					break;
				default:
					AppendEncoded(sb, s.AsSpan(tagStart, i - tagStart));
					break;
			}
		}
	}

	private static string ReadRawUntilClose(string s, ref int i, string closeName)
	{
		var close = "[/" + closeName + "]";
		var idx = s.IndexOf(close, i, StringComparison.OrdinalIgnoreCase);
		if (idx < 0)
		{
			var rest = s[i..];
			i = s.Length;
			return rest;
		}

		var seg = s.Substring(i, idx - i);
		i = idx + close.Length;
		return seg;
	}

	private static bool PeekClose(string s, int i, string name)
	{
		var t = i;
		return TryReadTag(s, ref t, out var tag) && tag.IsClose &&
		       string.Equals(tag.Name, name, StringComparison.OrdinalIgnoreCase);
	}

	private static void SkipClose(string s, ref int i, string name)
	{
		if (PeekClose(s, i, name))
		{
			TryReadTag(s, ref i, out _);
		}
	}

	private static void ParseList(StringBuilder sb, string s, ref int i, bool ordered)
	{
		var closeName = ordered ? "olist" : "list";
		sb.Append(ordered ? "<ol>" : "<ul>");
		SkipWhitespace(s, ref i);
		while (i < s.Length)
		{
			if (!TryPeekTag(s, i, out var peek))
			{
				break;
			}

			if (peek.IsClose && string.Equals(peek.Name, closeName, StringComparison.OrdinalIgnoreCase))
			{
				TryReadTag(s, ref i, out _);
				sb.Append(ordered ? "</ol>" : "</ul>");
				return;
			}

			if (!peek.IsClose && peek.Name == "*")
			{
				TryReadTag(s, ref i, out _);
				sb.Append("<li>");
				ParseContent(sb, s, ref i, null, true);
				sb.Append("</li>");
				SkipWhitespace(s, ref i);
				continue;
			}

			break;
		}

		sb.Append(ordered ? "</ol>" : "</ul>");
	}

	private static void SkipWhitespace(string s, ref int i)
	{
		while (i < s.Length && char.IsWhiteSpace(s[i]))
		{
			i++;
		}
	}

	private static bool TryPeekTag(string s, int pos, out TagInfo tag)
	{
		var j = pos;
		return TryReadTag(s, ref j, out tag);
	}

	private static bool TryReadTag(string s, ref int pos, out TagInfo tag)
	{
		tag = default;
		if (pos >= s.Length || s[pos] != '[')
		{
			return false;
		}

		var end = s.IndexOf(']', pos + 1);
		if (end < 0)
		{
			return false;
		}

		var inner = s.Substring(pos + 1, end - pos - 1);
		pos = end + 1;

		if (string.IsNullOrWhiteSpace(inner))
		{
			return false;
		}

		inner = inner.Trim();
		if (inner.Length == 0)
		{
			return false;
		}

		if (inner[0] == '/')
		{
			tag = new TagInfo(true, inner[1..].Trim().ToLowerInvariant(), null);
			return tag.Name.Length > 0;
		}

		var eq = inner.IndexOf('=');
		if (eq > 0)
		{
			var name = inner[..eq].Trim().ToLowerInvariant();
			var val = inner[(eq + 1)..].Trim();
			tag = new TagInfo(false, name, val);
			return name.Length > 0;
		}

		var n = inner.ToLowerInvariant();
		tag = new TagInfo(false, n, null);
		return true;
	}

	private static void AppendEncoded(StringBuilder sb, ReadOnlySpan<char> text)
	{
		foreach (var c in text)
		{
			sb.Append(HtmlEncodeChar(c));
		}
	}

	private static string HtmlEncodeChar(char c) => c switch
	{
		'&' => "&amp;",
		'<' => "&lt;",
		'>' => "&gt;",
		'"' => "&quot;",
		_ => c.ToString(),
	};

	private static void AppendAttr(StringBuilder sb, string value) => sb.Append(WebUtility.HtmlEncode(value));

	private const string Css = """
		body{font-family:Segoe UI,system-ui,sans-serif;font-size:14px;line-height:1.5;color:#C0FCF6;background:#001E1C;margin:0;padding:10px 12px;word-wrap:break-word;}
		.empty{color:#5A9E96;font-style:italic;}
		h1{font-size:1.45em;color:#61F4D8;margin:0.4em 0;font-weight:700;}
		h2{font-size:1.25em;color:#61F4D8;margin:0.45em 0;font-weight:700;}
		h3{font-size:1.1em;color:#7FBFB8;margin:0.5em 0;font-weight:700;}
		a{color:#61F4D8;}
		code{font-family:Consolas,monospace;background:#0D3835;padding:2px 6px;border-radius:4px;font-size:0.92em;white-space:pre-wrap;}
		blockquote{border-left:3px solid #61F4D8;margin:8px 0;padding:4px 0 4px 12px;color:#7FBFB8;}
		ul,ol{margin:6px 0;padding-left:1.4em;}
		li{margin:3px 0;}
		hr{border:none;border-top:1px solid #0D3835;margin:12px 0;}
		table{border-collapse:collapse;width:100%;margin:8px 0;font-size:0.95em;}
		th,td{border:1px solid #0D3835;padding:6px 8px;text-align:left;}
		th{background:#001715;color:#61F4D8;}
		img{max-width:100%;height:auto;border-radius:4px;margin:6px 0;}
		details.spoiler{margin:8px 0;background:#001715;border-radius:6px;padding:8px;}
		details.spoiler summary{cursor:pointer;color:#7FBFB8;font-size:0.9em;}
		""";

	private readonly struct TagInfo(bool isClose, string name, string? value)
	{
		public bool IsClose { get; } = isClose;
		public string Name { get; } = name;
		public string? Value { get; } = value;
	}
}

