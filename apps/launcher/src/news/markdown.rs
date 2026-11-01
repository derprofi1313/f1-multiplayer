//! The small Markdown subset news posts are written in: headings, paragraphs,
//! lists, quotes, code, rules, and inline bold, italics, code and links. HTML
//! and images are dropped, and only `https:`, `mailto:` and site-relative links
//! are kept.

pub const SITE: &str = "https://f1.swofty.net";

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Span {
    pub text: String,
    pub strong: bool,
    pub emphasis: bool,
    pub code: bool,
    pub link: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Block {
    Heading(usize, Vec<Span>),
    Paragraph(Vec<Span>),
    Item {
        depth: usize,
        number: Option<u32>,
        spans: Vec<Span>,
    },
    Quote(Vec<Span>),
    Code(String),
    Rule,
}

pub fn safe_url(url: &str) -> Option<String> {
    let url = url.trim();
    if url.is_empty() || url.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return None;
    }
    let scheme = |s: &str| url.get(..s.len()).is_some_and(|h| h.eq_ignore_ascii_case(s));
    if scheme("https://") && url.len() > 8 || scheme("mailto:") && url.len() > 7 {
        return Some(url.to_owned());
    }
    if url.starts_with('/') && !url.starts_with("//") {
        let at = url.find(['#', '?']).unwrap_or(url.len());
        let (path, rest) = url.split_at(at);
        let path = path.strip_suffix(".md").unwrap_or(path);
        return Some(format!("{SITE}{path}{rest}"));
    }
    None
}

enum Pending {
    None,
    Paragraph(String),
    Item(usize, Option<u32>, String),
    Quote(String),
}

impl Pending {
    fn append(&mut self, line: &str) -> bool {
        match self {
            Self::None => false,
            Self::Paragraph(text) | Self::Item(_, _, text) | Self::Quote(text) => {
                text.push(' ');
                text.push_str(line);
                true
            }
        }
    }

    fn flush(&mut self, blocks: &mut Vec<Block>) {
        match std::mem::replace(self, Self::None) {
            Self::None => {}
            Self::Paragraph(text) => blocks.push(Block::Paragraph(inline(&text))),
            Self::Item(depth, number, text) => blocks.push(Block::Item {
                depth,
                number,
                spans: inline(&text),
            }),
            Self::Quote(text) => blocks.push(Block::Quote(inline(&text))),
        }
    }
}

fn heading(line: &str) -> Option<(usize, &str)> {
    let level = line.bytes().take_while(|b| *b == b'#').count();
    let rest = &line[level..];
    ((1..=6).contains(&level) && (rest.is_empty() || rest.starts_with(' ')))
        .then(|| (level, rest.trim().trim_end_matches('#').trim_end()))
}

fn rule(line: &str) -> bool {
    let marks: String = line.chars().filter(|c| !c.is_whitespace()).collect();
    marks.len() >= 3 && ['-', '*', '_'].iter().any(|m| marks.chars().all(|c| c == *m))
}

fn list_item(line: &str) -> Option<(Option<u32>, &str)> {
    if let Some(rest) = line.strip_prefix(['-', '*', '+']).filter(|r| r.starts_with(' ')) {
        return Some((None, rest.trim_start()));
    }
    let digits = line.bytes().take_while(u8::is_ascii_digit).count();
    let rest = line[digits..].strip_prefix(['.', ')'])?;
    if digits == 0 || digits > 9 || !rest.starts_with(' ') {
        return None;
    }
    Some((line[..digits].parse().ok(), rest.trim_start()))
}

fn html(line: &str) -> bool {
    line.starts_with('<')
        && line[1..]
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '/' || c == '!')
}

fn table_row(line: &str) -> Option<String> {
    let cells: Vec<&str> = line
        .trim_matches('|')
        .split('|')
        .map(str::trim)
        .filter(|c| !c.is_empty())
        .collect();
    let separator = cells.iter().all(|c| c.chars().all(|ch| matches!(ch, '-' | ':')));
    (!separator).then(|| cells.join(" · "))
}

pub fn parse(text: &str) -> Vec<Block> {
    let mut lines = text.lines().peekable();
    if lines.peek().map(|l| l.trim()) == Some("---") {
        lines.next();
        lines.by_ref().find(|l| l.trim() == "---");
    }
    let mut blocks = Vec::new();
    let mut pending = Pending::None;
    let mut fence: Option<(String, String)> = None;
    for raw in lines {
        let line = raw.trim();
        if let Some((marker, code)) = &mut fence {
            if line.starts_with(marker.as_str()) {
                blocks.push(Block::Code(std::mem::take(code)));
                fence = None;
            } else {
                if !code.is_empty() {
                    code.push('\n');
                }
                code.push_str(raw.trim_end());
            }
            continue;
        }
        if line.starts_with("```") || line.starts_with("~~~") {
            pending.flush(&mut blocks);
            fence = Some((line[..3].to_owned(), String::new()));
            continue;
        }
        if line.is_empty() || line.starts_with(":::") || html(line) {
            pending.flush(&mut blocks);
            continue;
        }
        if let Some((level, title)) = heading(line) {
            pending.flush(&mut blocks);
            blocks.push(Block::Heading(level, inline(title)));
        } else if rule(line) {
            pending.flush(&mut blocks);
            blocks.push(Block::Rule);
        } else if let Some((number, rest)) = list_item(line) {
            pending.flush(&mut blocks);
            let indent = raw.len() - raw.trim_start().len();
            pending = Pending::Item((indent / 2).min(3), number, rest.to_owned());
        } else if let Some(rest) = line.strip_prefix('>') {
            if !matches!(pending, Pending::Quote(_)) {
                pending.flush(&mut blocks);
                pending = Pending::Quote(rest.trim().to_owned());
            } else {
                pending.append(rest.trim());
            }
        } else if line.starts_with('|') {
            pending.flush(&mut blocks);
            if let Some(row) = table_row(line) {
                blocks.push(Block::Paragraph(inline(&row)));
            }
        } else if !pending.append(line) {
            pending = Pending::Paragraph(line.to_owned());
        }
    }
    pending.flush(&mut blocks);
    if let Some((_, code)) = fence {
        blocks.push(Block::Code(code));
    }
    blocks
}

#[derive(Clone, Default)]
struct Style {
    strong: bool,
    emphasis: bool,
    link: Option<String>,
}

fn push(out: &mut Vec<Span>, text: &str, style: &Style, code: bool) {
    if text.is_empty() {
        return;
    }
    if let Some(last) = out.last_mut()
        && last.strong == style.strong
        && last.emphasis == style.emphasis
        && last.code == code
        && last.link == style.link
    {
        last.text.push_str(text);
        return;
    }
    out.push(Span {
        text: text.to_owned(),
        strong: style.strong,
        emphasis: style.emphasis,
        code,
        link: style.link.clone(),
    });
}

fn closing(rest: &str, marker: &str) -> Option<usize> {
    let mut from = 0;
    while let Some(at) = rest[from..].find(marker).map(|i| i + from) {
        let after = rest[at + marker.len()..].chars().next();
        let before = rest[..at].chars().next_back();
        let doubled = marker.len() == 1 && (after == Some('*') || before == Some('*'));
        let inside_word = marker.starts_with('_') && after.is_some_and(char::is_alphanumeric);
        if at > 0 && !before.is_some_and(char::is_whitespace) && !doubled && !inside_word {
            return Some(at);
        }
        from = at + marker.len();
    }
    None
}

fn bracketed(rest: &str) -> Option<(&str, &str, usize)> {
    let mut depth = 0;
    let close = rest.char_indices().find_map(|(i, c)| {
        match c {
            '[' => depth += 1,
            ']' => depth -= 1,
            _ => {}
        }
        (depth == 0).then_some(i)
    })?;
    let after = rest[close + 1..].strip_prefix('(')?;
    let end = after.find(')')?;
    let target = after[..end].split_whitespace().next().unwrap_or_default();
    let target = target
        .strip_prefix('<')
        .and_then(|t| t.strip_suffix('>'))
        .unwrap_or(target);
    Some((&rest[1..close], target, close + 2 + end + 1))
}

fn inline_into(text: &str, style: &Style, out: &mut Vec<Span>) {
    let mut plain = String::new();
    let mut i = 0;
    while let Some(c) = text[i..].chars().next() {
        let rest = &text[i..];
        let prev = text[..i].chars().next_back();
        match c {
            '\\' if rest[1..].starts_with(|n: char| n.is_ascii_punctuation()) => {
                plain.push_str(&rest[1..2]);
                i += 2;
                continue;
            }
            '`' => {
                let ticks = rest.bytes().take_while(|b| *b == b'`').count();
                let fence = &rest[..ticks];
                if let Some(end) = rest[ticks..].find(fence) {
                    push(out, &std::mem::take(&mut plain), style, false);
                    let code = &rest[ticks..ticks + end];
                    push(
                        out,
                        code.strip_prefix(' ')
                            .and_then(|c| c.strip_suffix(' '))
                            .unwrap_or(code),
                        style,
                        true,
                    );
                    i += 2 * ticks + end;
                    continue;
                }
            }
            '*' | '_' if c == '*' || !prev.is_some_and(char::is_alphanumeric) => {
                let marker = if rest[1..].starts_with(c) {
                    &rest[..2]
                } else {
                    &rest[..1]
                };
                let inner = &rest[marker.len()..];
                if !inner.starts_with(char::is_whitespace)
                    && let Some(end) = closing(inner, marker)
                {
                    push(out, &std::mem::take(&mut plain), style, false);
                    let mut nested = style.clone();
                    if marker.len() == 2 {
                        nested.strong = true;
                    } else {
                        nested.emphasis = true;
                    }
                    inline_into(&inner[..end], &nested, out);
                    i += 2 * marker.len() + end;
                    continue;
                }
            }
            '!' if rest.starts_with("![") => {
                if let Some((_, _, len)) = bracketed(&rest[1..]) {
                    i += 1 + len;
                    continue;
                }
            }
            '[' => {
                if let Some((label, target, len)) = bracketed(rest) {
                    push(out, &std::mem::take(&mut plain), style, false);
                    let nested = Style {
                        link: safe_url(target),
                        ..style.clone()
                    };
                    inline_into(label, &nested, out);
                    i += len;
                    continue;
                }
            }
            '<' => {
                if let Some(end) = rest.find('>') {
                    let inside = &rest[1..end];
                    if let Some(url) = safe_url(inside).filter(|_| !inside.starts_with('/')) {
                        push(out, &std::mem::take(&mut plain), style, false);
                        let nested = Style {
                            link: Some(url),
                            ..style.clone()
                        };
                        push(out, inside, &nested, false);
                        i += end + 1;
                        continue;
                    }
                    if html(rest) && !inside.contains('<') {
                        i += end + 1;
                        continue;
                    }
                }
            }
            _ => {}
        }
        plain.push(c);
        i += c.len_utf8();
    }
    push(out, &plain, style, false);
}

pub fn inline(text: &str) -> Vec<Span> {
    let mut out = Vec::new();
    inline_into(text, &Style::default(), &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::{Block, Span, inline, parse, safe_url};

    fn texts(spans: &[Span]) -> Vec<(&str, bool, bool, Option<&str>)> {
        spans
            .iter()
            .map(|s| (s.text.as_str(), s.strong, s.code, s.link.as_deref()))
            .collect()
    }

    #[test]
    fn keeps_only_https_mailto_and_site_links() {
        assert_eq!(
            safe_url("https://f1.swofty.net/guide/").as_deref(),
            Some("https://f1.swofty.net/guide/")
        );
        assert_eq!(
            safe_url("HTTPS://discord.gg/x").as_deref(),
            Some("HTTPS://discord.gg/x")
        );
        assert_eq!(
            safe_url("mailto:support@example.com").as_deref(),
            Some("mailto:support@example.com")
        );
        assert_eq!(
            safe_url("/using/addons.md#order").as_deref(),
            Some("https://f1.swofty.net/using/addons#order")
        );
        for bad in [
            "http://f1.swofty.net",
            "javascript:alert(1)",
            "file:///etc/passwd",
            "//evil.example",
            "steam://run/1",
            "https://",
            "relative/page",
            "https://a b",
        ] {
            assert_eq!(safe_url(bad), None, "{bad}");
        }
    }

    #[test]
    fn reads_bold_code_and_links_inline() {
        let spans = inline(
            "**Mod Importer.** Drop a `.serps` on [the docs](/using/mods) or [x](javascript:alert(1)).",
        );
        assert_eq!(
            texts(&spans),
            [
                ("Mod Importer.", true, false, None),
                (" Drop a ", false, false, None),
                (".serps", false, true, None),
                (" on ", false, false, None),
                ("the docs", false, false, Some("https://f1.swofty.net/using/mods")),
                (" or x).", false, false, None),
            ]
        );
    }

    #[test]
    fn leaves_lone_markers_snake_case_and_html_alone() {
        let spans = inline("2 * 3 and snake_case_name <b>bold</b> ![pic](https://x/y.png) 5 * 6");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].text, "2 * 3 and snake_case_name bold  5 * 6");
    }

    #[test]
    fn parses_the_blocks_of_a_post() {
        let blocks = parse(
            "---\ntitle: x\n---\n\nIntro line\ncontinues.\n\n## What's new\n\n- **A.** one\n  more\n- two\n  - nested\n1. first\n\n> quoted\n\n```\nlet x = 1;\n  indented\n```\n\n---\n\n| a | b |\n|---|---|\n| c | d |\n\n::: tip\n<div>raw</div>\n:::\n",
        );
        let kinds: Vec<String> = blocks
            .iter()
            .map(|b| match b {
                Block::Heading(l, s) => format!("h{l}:{}", s[0].text),
                Block::Paragraph(s) => format!("p:{}", s.iter().map(|s| s.text.as_str()).collect::<String>()),
                Block::Item { depth, number, spans } => format!(
                    "li{depth}{number:?}:{}",
                    spans.iter().map(|s| s.text.as_str()).collect::<String>()
                ),
                Block::Quote(s) => format!("q:{}", s[0].text),
                Block::Code(c) => format!("code:{c}"),
                Block::Rule => "hr".into(),
            })
            .collect();
        assert_eq!(
            kinds,
            [
                "p:Intro line continues.",
                "h2:What's new",
                "li0None:A. one more",
                "li0None:two",
                "li1None:nested",
                "li0Some(1):first",
                "q:quoted",
                "code:let x = 1;\n  indented",
                "hr",
                "p:a · b",
                "p:c · d",
            ]
        );
    }
}
