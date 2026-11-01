pub const KEPT: [&str; 2] = ["Summary", "Addons"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section<'a> {
    pub header: &'a str,
    pub title: &'a str,
    pub lines: Vec<&'a str>,
}

/// The `== <Title>` sections of a `perf.log`, each with its lines up to the
/// next header, trailing blank lines dropped.
pub fn sections(text: &str) -> Vec<Section<'_>> {
    let mut out: Vec<Section> = Vec::new();
    for line in text.lines() {
        let line = line.strip_suffix('\r').unwrap_or(line);
        if let Some(rest) = line.strip_prefix("== ") {
            out.push(Section {
                header: line,
                title: rest.trim().trim_end_matches('=').trim_end(),
                lines: Vec::new(),
            });
        } else if let Some(s) = out.last_mut() {
            s.lines.push(line);
        }
    }
    for s in &mut out {
        while s.lines.last().is_some_and(|l| l.trim().is_empty()) {
            s.lines.pop();
        }
    }
    out
}

/// The `Summary` and `Addons` sections verbatim, else the start of the
/// file, in at most `max` lines.
pub fn excerpt(text: &str, max: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for s in sections(text)
        .into_iter()
        .filter(|s| KEPT.iter().any(|k| s.title.eq_ignore_ascii_case(k)))
    {
        if !lines.is_empty() {
            lines.push(String::new());
        }
        lines.push(s.header.to_owned());
        lines.extend(s.lines.iter().map(|l| (*l).to_owned()));
    }
    if lines.is_empty() {
        lines = text
            .lines()
            .map(|l| l.strip_suffix('\r').unwrap_or(l).to_owned())
            .collect();
        while lines.last().is_some_and(|l| l.trim().is_empty()) {
            lines.pop();
        }
    }
    if lines.is_empty() {
        return vec!["(the report is empty)".into()];
    }
    if lines.len() > max {
        let more = lines.len() - max + 1;
        lines.truncate(max - 1);
        lines.push(format!("[{more} more lines in the file]"));
    }
    lines
}
