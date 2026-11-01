//! Ranked search over the catalog. Every word must match a word of some field,
//! whole, as a prefix, inside it, or with one typo when it has four letters or
//! more; a match in the name counts most and one in the description least.
//! `author:` and `category:` narrow the results without ranking them.

use std::ops::Range;

use super::Addon;

const NAME: u32 = 10;
const ID: u32 = 8;
const AUTHOR: u32 = 7;
const PUBLISHER: u32 = 6;
const CATEGORY: u32 = 5;
const KIND: u32 = 4;
const SUMMARY: u32 = 3;
const DESCRIPTION: u32 = 1;
const TYPO_MIN: usize = 4;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Query {
    pub words: Vec<String>,
    pub author: Option<String>,
    pub category: Option<String>,
}

fn words(text: &str) -> impl Iterator<Item = (usize, &str)> {
    text.char_indices()
        .filter(|&(i, c)| {
            c.is_alphanumeric() && text[..i].chars().next_back().is_none_or(|p| !p.is_alphanumeric())
        })
        .map(move |(i, _)| {
            let end = text[i..]
                .find(|c: char| !c.is_alphanumeric())
                .map_or(text.len(), |n| i + n);
            (i, &text[i..end])
        })
}

/// Edits between `a` and `b`, a swap of neighbours counting as one, or
/// `limit + 1` once it is sure to be over `limit`.
fn distance(a: &[char], b: &[char], limit: usize) -> usize {
    if a.len().abs_diff(b.len()) > limit {
        return limit + 1;
    }
    let mut rows = vec![(0..=b.len()).collect::<Vec<_>>()];
    for i in 1..=a.len() {
        let mut row = vec![i; b.len() + 1];
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            row[j] = (rows[i - 1][j] + 1)
                .min(row[j - 1] + 1)
                .min(rows[i - 1][j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                row[j] = row[j].min(rows[i - 2][j - 2] + 1);
            }
        }
        rows.push(row);
    }
    rows[a.len()][b.len()].min(limit + 1)
}

/// How well `term` matches `word`, both lowercase: 4 whole, 3 prefix, 2 with
/// one typo, 1 inside it.
fn quality(term: &str, word: &str) -> u32 {
    if word == term {
        return 4;
    }
    if word.starts_with(term) {
        return 3;
    }
    let t: Vec<char> = term.chars().collect();
    if t.len() >= TYPO_MIN {
        let w: Vec<char> = word.chars().collect();
        let close = (t.len() - 1..=t.len() + 1)
            .filter(|&n| n <= w.len() && (n == w.len() || n >= TYPO_MIN))
            .any(|n| distance(&t, &w[..n], 1) <= 1);
        if close {
            return 2;
        }
    }
    u32::from(t.len() >= 3 && word.contains(term))
}

fn best(term: &str, text: &str) -> u32 {
    words(&text.to_lowercase())
        .map(|(_, w)| quality(term, w))
        .max()
        .unwrap_or(0)
}

fn narrows(wanted: &str, values: &[&str]) -> bool {
    values.iter().any(|v| {
        let v = v.to_lowercase();
        v.starts_with(wanted) || words(&v).any(|(_, w)| w.starts_with(wanted))
    })
}

impl Query {
    pub fn parse(text: &str) -> Self {
        let mut q = Self::default();
        for raw in text.split_whitespace() {
            let lower = raw.to_lowercase();
            if let Some((key, value)) = lower.split_once(':')
                && !value.is_empty()
            {
                match key {
                    "author" | "by" => {
                        q.author = Some(value.to_owned());
                        continue;
                    }
                    "category" | "cat" => {
                        q.category = Some(value.to_owned());
                        continue;
                    }
                    _ => {}
                }
            }
            q.words.extend(words(&lower).map(|(_, w)| w.to_owned()));
        }
        q
    }

    pub fn is_empty(&self) -> bool {
        self.words.is_empty() && self.author.is_none() && self.category.is_none()
    }

    /// How well `a` matches, or `None` when it doesn't.
    pub fn score(&self, a: &Addon) -> Option<u32> {
        let category = a.category_label().unwrap_or_else(|| "Other".to_owned());
        if let Some(author) = &self.author {
            let mut names = vec![a.author.as_str()];
            names.extend(
                a.owner()
                    .into_iter()
                    .chain(&a.publishers)
                    .map(|p| p.name.as_str()),
            );
            if !narrows(author, &names) {
                return None;
            }
        }
        if let Some(wanted) = &self.category {
            let raw = a.category.as_deref().unwrap_or("other");
            if !narrows(wanted, &[&category, raw]) {
                return None;
            }
        }
        let kinds = a.kind_labels().join(" ");
        let publishers = a
            .owner()
            .into_iter()
            .chain(&a.publishers)
            .map(|p| p.name.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        let fields = [
            (NAME, a.name.as_str()),
            (ID, a.id.as_str()),
            (AUTHOR, a.author.as_str()),
            (PUBLISHER, publishers.as_str()),
            (CATEGORY, category.as_str()),
            (KIND, kinds.as_str()),
            (SUMMARY, a.summary.as_str()),
            (DESCRIPTION, a.description.as_str()),
        ];
        let mut total = 1;
        for term in &self.words {
            let score = fields
                .iter()
                .map(|(weight, text)| weight * best(term, text))
                .max()
                .unwrap_or(0);
            if score == 0 {
                return None;
            }
            total += score;
        }
        let phrase = self.words.join(" ");
        if !phrase.is_empty() && a.name.to_lowercase().starts_with(&phrase) {
            total += NAME * 2;
        }
        Some(total)
    }

    /// How well a publisher's name matches, or `None` when it doesn't or the
    /// query only narrows by category.
    pub fn score_name(&self, name: &str) -> Option<u32> {
        if (self.words.is_empty() && self.author.is_none()) || self.category.is_some() {
            return None;
        }
        if let Some(author) = &self.author
            && !narrows(author, &[name])
        {
            return None;
        }
        let mut total = 1;
        for term in &self.words {
            match best(term, name) {
                0 => return None,
                q => total += q,
            }
        }
        Some(total)
    }

    /// The byte ranges of `text` that the query's words match, to highlight.
    pub fn marks(&self, text: &str) -> Vec<Range<usize>> {
        let lower = text.to_lowercase();
        if lower.len() != text.len() {
            return Vec::new();
        }
        let mut out = Vec::new();
        for (at, word) in words(&lower) {
            let longest = self
                .words
                .iter()
                .filter_map(|t| match quality(t, word) {
                    4 | 2 => Some(word.len()),
                    3 => Some(t.len()),
                    _ => None,
                })
                .max();
            if let Some(n) = longest {
                out.push(at..at + n);
            } else if let Some(t) = self.words.iter().find(|t| quality(t, word) == 1) {
                let inner = word.find(t.as_str()).unwrap_or(0);
                out.push(at + inner..at + inner + t.len());
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Publisher;

    fn addon(id: &str, name: &str, author: &str, category: Option<&str>, summary: &str) -> Addon {
        serde_json::from_value(serde_json::json!({
            "id": id, "name": name, "author": author, "summary": summary,
            "category": category, "kind": ["database"],
        }))
        .unwrap()
    }

    fn catalog() -> Vec<Addon> {
        let mut editor = addon(
            "swofty.career-editor",
            "Career Mode Editor",
            "F1 Multiplayer",
            None,
            "Edit your career live",
        );
        editor.kind = vec!["native".into()];
        editor.publishers = vec![Publisher {
            id: "1".into(),
            name: "jacob".into(),
            avatar: None,
        }];
        vec![
            editor,
            addon(
                "demo.monaco",
                "Monaco 1990",
                "Retro",
                Some("tracks"),
                "The old harbour",
            ),
            addon(
                "demo.tyres",
                "Tyre Wear Tweaks",
                "Pitwall",
                None,
                "Makes career tyres last longer",
            ),
            addon(
                "demo.lids",
                "Classic Helmets",
                "Retro",
                Some("helmets"),
                "Helmet pack",
            ),
        ]
    }

    fn ids(q: &str) -> Vec<String> {
        let q = Query::parse(q);
        let mut found: Vec<(u32, String)> = catalog()
            .iter()
            .filter_map(|a| q.score(a).map(|s| (s, a.id.clone())))
            .collect();
        found.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        found.into_iter().map(|(_, id)| id).collect()
    }

    #[test]
    fn ranks_the_name_above_the_summary() {
        assert_eq!(ids("career"), ["swofty.career-editor", "demo.tyres"]);
        assert_eq!(ids("tyres"), ["demo.tyres"]);
    }

    #[test]
    fn needs_every_word_and_matches_prefixes() {
        assert_eq!(ids("car edi"), ["swofty.career-editor"]);
        assert_eq!(ids("classic hel"), ["demo.lids"]);
        assert!(ids("career helmet").is_empty());
        assert_eq!(ids("").len(), 4);
    }

    #[test]
    fn forgives_one_typo_in_longer_words() {
        assert_eq!(ids("carrer"), ["swofty.career-editor", "demo.tyres"]);
        assert_eq!(ids("monoco"), ["demo.monaco"]);
        assert_eq!(ids("hemlets"), ["demo.lids"]);
        assert!(ids("lod").is_empty());
        assert!(ids("monxxo").is_empty());
    }

    #[test]
    fn searches_authors_publishers_categories_and_kinds() {
        assert_eq!(ids("jacob"), ["swofty.career-editor"]);
        assert_eq!(ids("code"), ["swofty.career-editor"]);
        assert_eq!(ids("retro"), ["demo.lids", "demo.monaco"]);
        assert_eq!(ids("tracks"), ["demo.monaco"]);
    }

    #[test]
    fn matches_publisher_names() {
        let name = |q: &str| Query::parse(q).score_name("Retro Grid");
        assert!(name("retro").is_some());
        assert!(name("grid ret").is_some());
        assert!(name("rerto").is_some());
        assert!(name("author:retro").is_some());
        assert!(name("retro liveries").is_none());
        assert!(name("author:pixel").is_none());
        assert!(name("category:cars").is_none());
        assert!(name("").is_none());
        assert!(name("retro grid") > name("ret"));
    }

    #[test]
    fn narrows_by_author_and_category() {
        assert_eq!(ids("author:retro"), ["demo.lids", "demo.monaco"]);
        assert_eq!(ids("author:retro monaco"), ["demo.monaco"]);
        assert_eq!(ids("author:jac"), ["swofty.career-editor"]);
        assert_eq!(ids("category:helmets"), ["demo.lids"]);
        assert_eq!(
            ids("category:other career"),
            ["swofty.career-editor", "demo.tyres"]
        );
        assert!(Query::parse("author:x").words.is_empty());
        assert_eq!(Query::parse("http://x").words, ["http", "x"]);
    }

    #[test]
    fn marks_the_matched_words() {
        let q = Query::parse("car edtor");
        assert_eq!(q.marks("Career Mode Editor"), [0..3, 12..18]);
        assert_eq!(Query::parse("ode").marks("Mode"), vec![(1..4)]);
        assert!(Query::parse("x").marks("Monaco").is_empty());
    }
}
