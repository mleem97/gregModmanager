//! Converts project `README.md` (Markdown) to Steam Workshop BBCode.
//!
//! Steam understands only its BBCode subset; the Modstore keeps native
//! Markdown, so this conversion is Steam-only. Port of
//! `MarkdownToSteamBbCode`.

use std::collections::HashMap;

/// Full Markdown document to Steam BBCode.
pub fn convert(markdown: &str) -> String {
    if markdown.trim().is_empty() {
        return String::new();
    }
    let text = markdown.replace("\r\n", "\n").replace('\r', "\n");

    // Collect `[ref]: url` definitions first so `[text][ref]` can resolve.
    let mut references: HashMap<String, String> = HashMap::new();
    let mut lines: Vec<String> = Vec::new();
    for line in text.lines() {
        match parse_link_definition(line) {
            Some((label, url)) => {
                references.entry(label).or_insert(url);
            }
            None => lines.push(line.to_string()),
        }
    }

    let mut out = String::with_capacity(text.len());
    let mut in_code_block = false;
    let mut code_buffer = String::new();
    let mut list_kind = ListKind::None;
    let mut list_items: Vec<String> = Vec::new();
    let mut quote_buffer: Vec<String> = Vec::new();
    let mut table_rows: Vec<Vec<String>> = Vec::new();
    let mut table_has_header = false;

    let flush_list = |out: &mut String, kind: &mut ListKind, items: &mut Vec<String>| {
        if *kind != ListKind::None && !items.is_empty() {
            out.push_str(if *kind == ListKind::Ordered {
                "[olist]\n"
            } else {
                "[list]\n"
            });
            for item in items.iter() {
                out.push_str("[*] ");
                out.push_str(item);
                out.push('\n');
            }
            out.push_str(if *kind == ListKind::Ordered {
                "[/olist]\n"
            } else {
                "[/list]\n"
            });
        }
        items.clear();
        *kind = ListKind::None;
    };

    let flush_quote = |out: &mut String, quote: &mut Vec<String>| {
        if quote.is_empty() {
            return;
        }
        out.push_str("[quote]\n");
        for q in quote.iter() {
            out.push_str(&convert_inline(q, &references));
            out.push('\n');
        }
        out.push_str("[/quote]\n");
        quote.clear();
    };

    let flush_table = |out: &mut String, rows: &mut Vec<Vec<String>>, has_header: &mut bool| {
        if rows.is_empty() {
            return;
        }
        out.push_str("[table]\n");
        for (r, row) in rows.iter().enumerate() {
            let header = r == 0 && *has_header;
            let tag = if header { "th" } else { "td" };
            out.push_str("[tr]\n");
            for cell in row {
                out.push_str(&format!(
                    "[{tag}]{inner}[/{tag}]\n",
                    inner = convert_inline(cell.trim(), &references)
                ));
            }
            out.push_str("[/tr]\n");
        }
        out.push_str("[/table]\n");
        rows.clear();
        *has_header = false;
    };

    let n = lines.len();
    for (i, line) in lines.iter().enumerate() {
        // Fenced code blocks -> [code]...[/code]
        if line.trim_start().starts_with("```") {
            if !in_code_block {
                flush_list(&mut out, &mut list_kind, &mut list_items);
                flush_quote(&mut out, &mut quote_buffer);
                flush_table(&mut out, &mut table_rows, &mut table_has_header);
                in_code_block = true;
                code_buffer.clear();
            } else {
                in_code_block = false;
                out.push_str("[code]");
                out.push_str(code_buffer.trim_matches('\n'));
                out.push_str("[/code]\n");
                code_buffer.clear();
            }
            continue;
        }
        if in_code_block {
            code_buffer.push_str(line);
            code_buffer.push('\n');
            continue;
        }

        // Indented code block (4 spaces).
        if line.starts_with("    ") && !line.trim().is_empty() {
            flush_list(&mut out, &mut list_kind, &mut list_items);
            flush_quote(&mut out, &mut quote_buffer);
            flush_table(&mut out, &mut table_rows, &mut table_has_header);
            out.push_str("[code]");
            out.push_str(line.trim());
            out.push_str("[/code]\n");
            continue;
        }

        let trimmed = line.trim();

        // Table rows: | a | b |
        if is_table_row(trimmed) {
            let cells = split_row(trimmed);
            if is_separator_row(&cells) {
                table_has_header = !table_rows.is_empty();
                continue;
            }
            if table_rows.is_empty() {
                let next_is_sep = i + 1 < n
                    && is_table_row(lines[i + 1].trim())
                    && is_separator_row(&split_row(lines[i + 1].trim()));
                if !next_is_sep {
                    flush_list(&mut out, &mut list_kind, &mut list_items);
                    flush_quote(&mut out, &mut quote_buffer);
                    out.push_str(&convert_inline(line.trim(), &references));
                    out.push('\n');
                    continue;
                }
                flush_list(&mut out, &mut list_kind, &mut list_items);
                flush_quote(&mut out, &mut quote_buffer);
            }
            table_rows.push(cells);
            continue;
        }
        if !table_rows.is_empty() {
            flush_table(&mut out, &mut table_rows, &mut table_has_header);
        }

        // Horizontal rule.
        if is_horizontal_rule(trimmed) {
            flush_list(&mut out, &mut list_kind, &mut list_items);
            flush_quote(&mut out, &mut quote_buffer);
            out.push_str("[hr][/hr]\n");
            continue;
        }

        // ATX headings.
        if let Some((level, heading)) = parse_heading(trimmed) {
            flush_list(&mut out, &mut list_kind, &mut list_items);
            flush_quote(&mut out, &mut quote_buffer);
            let inline = convert_inline(&heading, &references);
            match level {
                1 => out.push_str(&format!("[h1]{inline}[/h1]\n")),
                2 => out.push_str(&format!("[h2]{inline}[/h2]\n")),
                3 => out.push_str(&format!("[h3]{inline}[/h3]\n")),
                _ => out.push_str(&format!("[b]{inline}[/b]\n")),
            }
            continue;
        }

        // Blockquote (consecutive `>` lines form one block).
        if let Some(stripped) = trimmed.strip_prefix("> ") {
            flush_list(&mut out, &mut list_kind, &mut list_items);
            quote_buffer.push(stripped.to_string());
            let next_is_quote =
                i + 1 < n && (lines[i + 1].trim().starts_with("> ") || lines[i + 1].trim() == ">");
            if !next_is_quote {
                flush_quote(&mut out, &mut quote_buffer);
            }
            continue;
        }
        if trimmed == ">" {
            quote_buffer.push(String::new());
            continue;
        }
        if !quote_buffer.is_empty() {
            flush_quote(&mut out, &mut quote_buffer);
        }

        // List items.
        if let Some((ordered, item)) = parse_list_item(trimmed) {
            let kind = if ordered {
                ListKind::Ordered
            } else {
                ListKind::Unordered
            };
            if list_kind != ListKind::None && list_kind != kind {
                flush_list(&mut out, &mut list_kind, &mut list_items);
            }
            list_kind = kind;
            list_items.push(convert_inline(&item, &references));
            let next_is_item = i + 1 < n && parse_list_item(lines[i + 1].trim()).is_some();
            if !next_is_item {
                flush_list(&mut out, &mut list_kind, &mut list_items);
            }
            continue;
        }
        if list_kind != ListKind::None {
            flush_list(&mut out, &mut list_kind, &mut list_items);
        }

        // Blank line = paragraph break.
        if trimmed.is_empty() {
            out.push('\n');
            continue;
        }

        out.push_str(&convert_inline(line.trim(), &references));
        out.push('\n');
    }

    flush_list(&mut out, &mut list_kind, &mut list_items);
    flush_quote(&mut out, &mut quote_buffer);
    flush_table(&mut out, &mut table_rows, &mut table_has_header);
    if in_code_block {
        out.push_str("[code]");
        out.push_str(code_buffer.trim_matches('\n'));
        out.push_str("[/code]\n");
    }

    collapse_newlines(&out)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ListKind {
    None,
    Unordered,
    Ordered,
}

/// Parses `[label]: url` reference definitions.
fn parse_link_definition(line: &str) -> Option<(String, String)> {
    let t = line.trim_start();
    let inner = t.strip_prefix('[')?;
    let end = inner.find(']')?;
    let label = inner[..end].trim();
    if label.is_empty() {
        return None;
    }
    let after = inner[end + 1..].trim_start();
    let after = after.strip_prefix(':')?.trim_start();
    let url: String = after.split_whitespace().next()?.to_string();
    if url.is_empty() {
        return None;
    }
    Some((label.to_lowercase(), url))
}

fn is_table_row(trimmed: &str) -> bool {
    trimmed.starts_with('|') && trimmed.ends_with('|') && trimmed.len() > 1
}

fn split_row(trimmed: &str) -> Vec<String> {
    trimmed
        .trim_matches('|')
        .split('|')
        .map(|c| c.trim().to_string())
        .collect()
}

fn is_separator_row(cells: &[String]) -> bool {
    !cells.is_empty()
        && cells
            .iter()
            .all(|c| !c.is_empty() && c.chars().all(|ch| ch == '-' || ch == ':' || ch == ' '))
}

fn is_horizontal_rule(trimmed: &str) -> bool {
    if trimmed.len() < 3 {
        return false;
    }
    if !trimmed
        .chars()
        .all(|c| c == '-' || c == '*' || c == '_' || c == ' ')
    {
        return false;
    }
    let distinct: std::collections::HashSet<char> = trimmed.chars().collect();
    distinct.len() <= 2
}

/// Parses `#…` headings into `(level, text)`.
fn parse_heading(trimmed: &str) -> Option<(usize, String)> {
    if !trimmed.starts_with('#') {
        return None;
    }
    let level = trimmed.chars().take_while(|c| *c == '#').count();
    let mut heading = trimmed[level..].trim().to_string();
    while heading.ends_with('#') {
        heading.pop();
    }
    Some((level, heading.trim().to_string()))
}

/// Parses `- item` / `1. item` into `(ordered, text)`.
fn parse_list_item(trimmed: &str) -> Option<(bool, String)> {
    if trimmed.len() < 2 {
        return None;
    }
    let bytes = trimmed.as_bytes();
    if (bytes[0] == b'-' || bytes[0] == b'*' || bytes[0] == b'+') && bytes[1].is_ascii_whitespace()
    {
        return Some((false, strip_task_marker(trimmed[1..].trim())));
    }
    let sep = trimmed.find('.').or_else(|| trimmed.find(')'))?;
    if sep == 0 || sep > 4 {
        return None;
    }
    if !trimmed[..sep].chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let rest = &trimmed[sep + 1..];
    if rest.is_empty() || !rest.as_bytes()[0].is_ascii_whitespace() {
        return None;
    }
    Some((true, strip_task_marker(rest.trim())))
}

fn strip_task_marker(text: &str) -> String {
    if text.starts_with("[ ] ") || text.to_lowercase().starts_with("[x] ") {
        return text[4..].to_string();
    }
    text.to_string()
}

fn collapse_newlines(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut blanks = 0;
    for line in s.replace('\r', "").split('\n') {
        if line.trim().is_empty() {
            blanks += 1;
            if blanks <= 1 {
                out.push('\n');
            }
        } else {
            blanks = 0;
            out.push_str(line);
            out.push('\n');
        }
    }
    // Collapse runs of 3+ newlines to 2 and trim edges.
    let mut result = String::with_capacity(out.len());
    let mut run = 0;
    for c in out.chars() {
        if c == '\n' {
            run += 1;
            if run <= 2 {
                result.push(c);
            }
        } else {
            run = 0;
            result.push(c);
        }
    }
    result.trim().to_string()
}

/// Inline Markdown (bold, italic, code, links, images) to BBCode.
pub fn convert_inline(text: &str, refs: &HashMap<String, String>) -> String {
    // Protect code spans with placeholders.
    let mut spans: Vec<String> = Vec::new();
    let mut s = protect_code_spans(text, &mut spans);
    s = convert_images(&s);
    s = convert_inline_links(&s);
    s = convert_reference_links(&s, refs);
    s = convert_autolinks(&s);
    s = convert_delimited(&s, "**", "b", '*');
    s = convert_delimited(&s, "__", "b", '_');
    s = convert_delimited(&s, "~~", "strike", '~');
    s = convert_italics_star(&s);
    s = convert_italics_underscore(&s);
    for (i, span) in spans.iter().enumerate() {
        s = s.replace(
            &format!("\u{E000}{i}\u{E001}"),
            &format!("[code]{span}[/code]"),
        );
    }
    s
}

fn protect_code_spans(text: &str, spans: &mut Vec<String>) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(start) = rest.find('`') {
        if let Some(rel_end) = rest[start + 1..].find('`') {
            let inner = &rest[start + 1..start + 1 + rel_end];
            if inner.contains('\n') || inner.is_empty() {
                out.push_str(&rest[..=start]);
                rest = &rest[start + 1..];
                continue;
            }
            out.push_str(&rest[..start]);
            let idx = spans.len();
            spans.push(inner.to_string());
            out.push_str(&format!("\u{E000}{idx}\u{E001}"));
            rest = &rest[start + 1 + rel_end + 1..];
        } else {
            break;
        }
    }
    out.push_str(rest);
    out
}

/// Splits `url` from an optional `"title"` suffix.
fn split_destination(s: &str) -> Option<(String, usize)> {
    let s = s.strip_prefix('(')?;
    let mut url = String::new();
    for (i, c) in s.char_indices() {
        if c == ')' {
            return Some((url, i + 2));
        }
        if c.is_whitespace() {
            // Optional "title" then ')'.
            let tail = &s[i..];
            let tail = tail.trim_start();
            if let Some(quoted) = tail.strip_prefix('"') {
                if let Some(end) = quoted.find('"') {
                    let after = quoted[end + 1..].trim_start();
                    if after.starts_with(')') {
                        return Some((url, s.len() - after.len() + 1));
                    }
                }
            }
            return None;
        }
        url.push(c);
    }
    None
}

fn convert_images(s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(bang) = rest.find("![") {
        let after_bang = &rest[bang + 1..];
        if let Some(close) = after_bang.find(']') {
            let dest = &after_bang[close + 1..];
            if let Some((url, consumed)) = split_destination(dest) {
                out.push_str(&rest[..bang]);
                out.push_str(&format!("[img]{url}[/img]"));
                rest = &after_bang[close + 1 + consumed..];
                continue;
            }
        }
        out.push_str(&rest[..=bang]);
        rest = &rest[bang + 1..];
    }
    out.push_str(rest);
    out
}

fn convert_inline_links(s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(open) = rest.find('[') {
        // Skip images (handled before) and placeholders.
        if open > 0 && rest.as_bytes()[open - 1] == b'!' {
            out.push_str(&rest[..=open]);
            rest = &rest[open + 1..];
            continue;
        }
        let after_open = &rest[open + 1..];
        if let Some(close) = after_open.find(']') {
            let label = &after_open[..close];
            if label.contains('\n') || label.contains('[') {
                out.push_str(&rest[..=open]);
                rest = &rest[open + 1..];
                continue;
            }
            let dest = &after_open[close + 1..];
            if let Some((url, consumed)) = split_destination(dest) {
                out.push_str(&rest[..open]);
                out.push_str(&format!("[url={url}]{label}[/url]"));
                rest = &after_open[close + 1 + consumed..];
                continue;
            }
        }
        out.push_str(&rest[..=open]);
        rest = &rest[open + 1..];
    }
    out.push_str(rest);
    out
}

fn convert_reference_links(s: &str, refs: &HashMap<String, String>) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(open) = rest.find('[') {
        let after_open = &rest[open + 1..];
        if let Some(close) = after_open.find(']') {
            let label = &after_open[..close];
            let after_close = &after_open[close + 1..];
            if let Some(tail) = after_close.strip_prefix('[') {
                if let Some(end) = tail.find(']') {
                    let given = tail[..end].trim();
                    let key = if given.is_empty() {
                        label.to_lowercase()
                    } else {
                        given.to_lowercase()
                    };
                    if let Some(url) = refs.get(&key) {
                        out.push_str(&rest[..open]);
                        out.push_str(&format!("[url={url}]{label}[/url]"));
                        rest = &tail[end + 1..];
                        continue;
                    }
                }
            }
        }
        out.push_str(&rest[..=open]);
        rest = &rest[open + 1..];
    }
    out.push_str(rest);
    out
}

fn convert_autolinks(s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(open) = rest.find('<') {
        let after = &rest[open + 1..];
        if let Some(close) = after.find('>') {
            let url = &after[..close];
            if !url.contains([' ', '<', '\n'])
                && (url.starts_with("http://")
                    || url.starts_with("https://")
                    || url.starts_with("ftp://"))
            {
                out.push_str(&rest[..open]);
                out.push_str(&format!("[url={url}]{url}[/url]"));
                rest = &after[close + 1..];
                continue;
            }
        }
        out.push_str(&rest[..=open]);
        rest = &rest[open + 1..];
    }
    out.push_str(rest);
    out
}

/// Converts paired delimiters (`**`, `__`, `~~`) whose inner text contains
/// no delimiter character.
fn convert_delimited(s: &str, delim: &str, tag: &str, inner_forbidden: char) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(start) = rest.find(delim) {
        let after = &rest[start + delim.len()..];
        match after.find(delim) {
            Some(rel) => {
                let inner = &after[..rel];
                if inner.is_empty() || inner.contains(inner_forbidden) {
                    out.push_str(&rest[..start + delim.len()]);
                    rest = after;
                    continue;
                }
                out.push_str(&rest[..start]);
                out.push_str(&format!("[{tag}]{inner}[/{tag}]"));
                rest = &after[rel + delim.len()..];
            }
            None => break,
        }
    }
    out.push_str(rest);
    out
}

fn convert_italics_star(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '*'
            && prev_is(&chars, i, |c| c != '*')
            && next_is(&chars, i, |c| c != '*' && !c.is_whitespace())
        {
            if let Some(j) = find_close_star(&chars, i + 1) {
                out.push_str("[i]");
                out.extend(chars[i + 1..j].iter());
                out.push_str("[/i]");
                i = j + 1;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

fn find_close_star(chars: &[char], from: usize) -> Option<usize> {
    let mut j = from;
    while j < chars.len() {
        if chars[j] == '*' {
            let prev_ok = j > 0 && !chars[j - 1].is_whitespace();
            let next_ok = j + 1 >= chars.len() || chars[j + 1] != '*';
            if prev_ok && next_ok {
                // Inner text must not contain another '*'.
                if !chars[from..j].contains(&'*') {
                    return Some(j);
                }
                return None;
            }
            return None;
        }
        j += 1;
    }
    None
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn convert_italics_underscore(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '_'
            && prev_is(&chars, i, |c| !is_word_char(c))
            && next_is(&chars, i, |c| c != '_' && !c.is_whitespace())
        {
            let mut j = i + 1;
            let mut found = None;
            while j < chars.len() {
                if chars[j] == '_' {
                    let prev_ok = !chars[j - 1].is_whitespace();
                    let next_ok = j + 1 >= chars.len() || !is_word_char(chars[j + 1]);
                    if prev_ok && next_ok && !chars[i + 1..j].contains(&'_') {
                        found = Some(j);
                    }
                    break;
                }
                j += 1;
            }
            if let Some(j) = found {
                out.push_str("[i]");
                out.extend(chars[i + 1..j].iter());
                out.push_str("[/i]");
                i = j + 1;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

fn prev_is(chars: &[char], i: usize, f: impl Fn(char) -> bool) -> bool {
    i == 0 || f(chars[i - 1])
}

fn next_is(chars: &[char], i: usize, f: impl Fn(char) -> bool) -> bool {
    i + 1 >= chars.len() || f(chars[i + 1])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn inline(s: &str) -> String {
        convert_inline(s, &HashMap::new())
    }

    #[test]
    fn headings() {
        let bb = convert("# Title\n## Sub\n### Sub3\n#### Deep");
        assert!(bb.contains("[h1]Title[/h1]"), "{bb}");
        assert!(bb.contains("[h2]Sub[/h2]"), "{bb}");
        assert!(bb.contains("[h3]Sub3[/h3]"), "{bb}");
        assert!(bb.contains("[b]Deep[/b]"), "{bb}");
    }

    #[test]
    fn inline_styles() {
        let bb = inline("**bold** and *italic* and `code` and ~~strike~~");
        assert!(bb.contains("[b]bold[/b]"), "{bb}");
        assert!(bb.contains("[i]italic[/i]"), "{bb}");
        assert!(bb.contains("[code]code[/code]"), "{bb}");
        assert!(bb.contains("[strike]strike[/strike]"), "{bb}");
    }

    #[test]
    fn links_and_images() {
        let bb = inline("[Docs](https://example.com) and ![shot](https://example.com/a.png)");
        assert!(bb.contains("[url=https://example.com]Docs[/url]"), "{bb}");
        assert!(bb.contains("[img]https://example.com/a.png[/img]"), "{bb}");
    }

    #[test]
    fn lists() {
        let bb = convert("- Item 1\n- Item 2");
        assert!(bb.contains("[list]"), "{bb}");
        assert!(bb.contains("[*] Item 1"), "{bb}");
        assert!(bb.contains("[/list]"), "{bb}");
        let bb = convert("1. First\n2. Second");
        assert!(bb.contains("[olist]"), "{bb}");
        assert!(bb.contains("[*] First"), "{bb}");
        assert!(bb.contains("[/olist]"), "{bb}");
    }

    #[test]
    fn code_quote_hr() {
        let bb = convert("```\nsome **raw** code\n```\n\n> quoted\n\n---");
        assert!(bb.contains("[code]some **raw** code[/code]"), "{bb}");
        assert!(bb.contains("[quote]"), "{bb}");
        assert!(bb.contains("[hr][/hr]"), "{bb}");
    }

    #[test]
    fn tables() {
        let bb = convert("| A | B |\n| --- | --- |\n| 1 | 2 |");
        assert!(bb.contains("[table]"), "{bb}");
        assert!(bb.contains("[th]A[/th]"), "{bb}");
        assert!(bb.contains("[td]1[/td]"), "{bb}");
    }

    #[test]
    fn empty() {
        assert_eq!(convert(""), "");
        assert_eq!(convert("   "), "");
    }
}
