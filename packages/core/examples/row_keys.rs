use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::hash::{BuildHasherDefault, Hasher};
use std::io::{BufRead, Read, Write};
use std::path::{Path, PathBuf};

use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use swofty_core::ops::row_shape;
use swofty_formats::gvdb::Gvdb;
use swofty_formats::hash::{FNV64_BASIS, fnv1a64, fnv1a64_from};

const FULL_ROWS: usize = 64;
const MAX_NAME: usize = 128;
const MAX_ROWS: usize = 4096;
const EXPAND_ROUNDS: usize = 3;

#[derive(Default)]
struct Id(u64);

impl Hasher for Id {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, _: &[u8]) {
        unreachable!()
    }
    fn write_u64(&mut self, n: u64) {
        self.0 = n ^ (n >> 29);
    }
}

type Keys = HashSet<u64, BuildHasherDefault<Id>>;

struct Vocab {
    names: Vec<Vec<u8>>,
    threads: usize,
}

impl Vocab {
    fn probe(
        &self,
        keys: &Keys,
        prefix: &[u8],
        suffix: &[u8],
        subset: Option<&BTreeSet<Vec<u8>>>,
    ) -> Vec<Vec<u8>> {
        let h0 = fnv1a64_from(FNV64_BASIS, prefix);
        if let Some(subset) = subset {
            return subset
                .iter()
                .filter(|n| keys.contains(&fnv1a64_from(fnv1a64_from(h0, n), suffix)))
                .cloned()
                .collect();
        }
        let chunk = self.names.len().div_ceil(self.threads);
        std::thread::scope(|s| {
            let jobs: Vec<_> = self
                .names
                .chunks(chunk.max(1))
                .map(|names| {
                    s.spawn(move || {
                        names
                            .iter()
                            .filter(|n| keys.contains(&fnv1a64_from(fnv1a64_from(h0, n), suffix)))
                            .cloned()
                            .collect::<Vec<_>>()
                    })
                })
                .collect();
            jobs.into_iter()
                .flat_map(|j| j.join().expect("probe thread"))
                .collect()
        })
    }
}

impl Vocab {
    fn probe_row(
        &self,
        keys: &Keys,
        prefix: &[u8],
        subset: Option<&BTreeSet<Vec<u8>>>,
    ) -> (Vec<Vec<u8>>, Vec<Vec<u8>>) {
        if subset.is_some() {
            return (self.probe(keys, prefix, b"", subset), Vec::new());
        }
        let h0 = fnv1a64_from(FNV64_BASIS, prefix);
        let chunk = self.names.len().div_ceil(self.threads);
        std::thread::scope(|s| {
            let jobs: Vec<_> = self
                .names
                .chunks(chunk.max(1))
                .map(|names| {
                    s.spawn(move || {
                        let (mut leaves, mut rows) = (Vec::new(), Vec::new());
                        for n in names {
                            let h = fnv1a64_from(h0, n);
                            if keys.contains(&h) {
                                leaves.push(n.clone());
                            }
                            if keys.contains(&fnv1a64_from(h, b"_0._metadata_")) {
                                rows.push(n.clone());
                            }
                        }
                        (leaves, rows)
                    })
                })
                .collect();
            let (mut leaves, mut rows) = (Vec::new(), Vec::new());
            for j in jobs {
                let (l, r) = j.join().expect("probe thread");
                leaves.extend(l);
                rows.extend(r);
            }
            (leaves, rows)
        })
    }
}

#[derive(Default)]
struct Shape {
    rows: usize,
    leaves: BTreeSet<Vec<u8>>,
    counted: BTreeSet<Vec<u8>>,
    uncounted: BTreeSet<Vec<u8>>,
    entries: BTreeSet<String>,
}

struct Walker<'a> {
    db: &'a Gvdb,
    keys: Keys,
    vocab: Vocab,
    shapes: BTreeMap<String, Shape>,
    rows: usize,
    named: Keys,
}

fn text(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

fn count(db: &Gvdb, container: &str, word: &str) -> Option<usize> {
    db.instance_count(container, word)
}

impl Walker<'_> {
    fn has(&self, path: &str) -> bool {
        self.keys.contains(&fnv1a64(path))
    }

    fn table(&mut self, container: &str, word: &str) {
        let n = match count(self.db, container, word) {
            Some(n) => {
                self.named
                    .insert(fnv1a64(format!("{container}._{word}_instancecount_")));
                n
            }
            None => (0..MAX_ROWS)
                .take_while(|i| self.has(&format!("{container}.{word}_{i}._metadata_")))
                .count(),
        };
        for i in 0..n.min(MAX_ROWS) {
            self.row(&format!("{container}.{word}_{i}"));
        }
    }

    fn row(&mut self, path: &str) {
        self.rows += 1;
        let shape = row_shape(path);
        let full = self.shapes.get(&shape).is_none_or(|s| s.rows < FULL_ROWS);
        let first = self.shapes.get(&shape).is_none_or(|s| s.rows == 0);
        let (leaf_subset, counted_subset, uncounted_subset) = match self.shapes.get(&shape) {
            Some(s) if !full => (
                Some(s.leaves.clone()),
                Some(s.counted.clone()),
                Some(s.uncounted.clone()),
            ),
            _ => (None, None, None),
        };
        let dot = format!("{path}.");
        let (mut leaves, rows) = self
            .vocab
            .probe_row(&self.keys, dot.as_bytes(), leaf_subset.as_ref());
        let counted = self.vocab.probe(
            &self.keys,
            format!("{path}._").as_bytes(),
            b"_instancecount_",
            counted_subset.as_ref(),
        );
        let counted_set: BTreeSet<Vec<u8>> = counted.iter().cloned().collect();
        let uncounted: Vec<Vec<u8>> = match uncounted_subset {
            Some(subset) => self
                .vocab
                .probe(&self.keys, dot.as_bytes(), b"_0._metadata_", Some(&subset)),
            None => rows,
        }
        .into_iter()
        .filter(|n| !counted_set.contains(n))
        .collect();
        if first {
            leaves = self.expand(path, leaves);
        }
        let entry = self.shapes.entry(shape).or_default();
        entry.rows += 1;
        for l in &leaves {
            let name = text(l);
            if let Some(stem) = name.strip_suffix("_instancecount_")
                && let Some((head, word)) = stem.rsplit_once("._")
            {
                entry.entries.insert(format!("+{head}.{word}"));
            }
            entry.entries.insert(name);
        }
        for c in &counted {
            entry.entries.insert(format!("_{}_instancecount_", text(c)));
            entry.entries.insert(format!("+{}", text(c)));
        }
        for u in &uncounted {
            entry.entries.insert(format!("+{}", text(u)));
        }
        entry.leaves.extend(leaves.iter().cloned());
        entry.counted.extend(counted.iter().cloned());
        entry.uncounted.extend(uncounted.iter().cloned());
        for l in &leaves {
            self.named.insert(fnv1a64_from(fnv1a64(&dot), l));
        }
        let mut heads: Vec<(String, String)> = Vec::new();
        for l in &leaves {
            let name = text(l);
            if let Some(stem) = name.strip_suffix("_instancecount_")
                && let Some((head, word)) = stem.rsplit_once("._")
            {
                heads.push((format!("{path}.{head}"), word.to_owned()));
            }
        }
        for c in counted.iter().chain(&uncounted) {
            heads.push((path.to_owned(), text(c)));
        }
        for (container, word) in heads {
            self.table(&container, &word);
        }
    }

    fn expand(&self, path: &str, leaves: Vec<Vec<u8>>) -> Vec<Vec<u8>> {
        let mut found: BTreeSet<Vec<u8>> = leaves.into_iter().collect();
        let mut tried: HashSet<(Vec<u8>, Vec<u8>)> = HashSet::new();
        for _ in 0..EXPAND_ROUNDS {
            let mut patterns = Vec::new();
            for l in &found {
                let segs: Vec<&[u8]> = l.split(|&b| b == b'.').collect();
                if segs.len() < 2 {
                    continue;
                }
                let head = segs[..segs.len() - 1].join(&b'.');
                patterns.push(([head.as_slice(), b"."].concat(), Vec::new()));
                let pre = segs[..segs.len() - 2].join(&b'.');
                let pre = if pre.is_empty() {
                    pre
                } else {
                    [pre.as_slice(), b"."].concat()
                };
                patterns.push((pre, [b".", segs[segs.len() - 1]].concat()));
            }
            let mut new = BTreeSet::new();
            for (pre, post) in patterns {
                if !tried.insert((pre.clone(), post.clone())) {
                    continue;
                }
                let prefix = [format!("{path}.").as_bytes(), pre.as_slice()].concat();
                for n in self.vocab.probe(&self.keys, &prefix, &post, None) {
                    let full = [pre.as_slice(), n.as_slice(), post.as_slice()].concat();
                    if !found.contains(&full) {
                        new.insert(full);
                    }
                }
            }
            if new.is_empty() {
                break;
            }
            found.extend(new);
        }
        found.into_iter().collect()
    }
}

fn expand_braces(line: &str) -> Vec<String> {
    let Some(open) = line.find('{') else {
        return vec![line.to_owned()];
    };
    let close = open + line[open..].find('}').expect("brace group closes");
    let (head, tail) = (&line[..open], &line[close + 1..]);
    line[open + 1..close]
        .split(',')
        .flat_map(|choice| expand_braces(&format!("{head}{choice}{tail}")))
        .collect()
}

fn exe_names(exe: &Path, out: &mut BTreeSet<Vec<u8>>) {
    let data = std::fs::read(exe).expect("read the exe");
    exe_formats(&data, out);
    let ident = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
    let mut i = 0;
    while i < data.len() {
        if !(data[i].is_ascii_alphabetic() || data[i] == b'_') {
            i += 1;
            continue;
        }
        let mut j = i + 1;
        while j < data.len() && ident(data[j]) {
            j += 1;
        }
        let run = &data[i..j];
        for piece in [run, &run[..run.len().min(64)]] {
            if piece.len() >= 2 && piece.len() <= MAX_NAME && piece.iter().any(u8::is_ascii_lowercase) {
                out.insert(piece.to_vec());
            }
        }
        i = if run.len() > 64 { i + 64 } else { j };
    }
}

const FORMAT_TOP: u32 = 32;
const FORMAT_MAX_SPECS: usize = 2;

struct Spec {
    zero: bool,
    width: usize,
    hex: Option<bool>,
}

fn spec_at(b: &[u8]) -> Option<(Spec, usize)> {
    let mut i = 1;
    let mut zero = false;
    while i < b.len() && b"-+ #0".contains(&b[i]) {
        if !matches!(b[i], b'0') {
            return None;
        }
        zero = true;
        i += 1;
    }
    let digits = |i: &mut usize| {
        let start = *i;
        while *i < b.len() && b[*i].is_ascii_digit() {
            *i += 1;
        }
        std::str::from_utf8(&b[start..*i]).ok()?.parse::<usize>().ok()
    };
    let mut width = digits(&mut i).unwrap_or(0);
    if b.get(i) == Some(&b'.') {
        i += 1;
        width = width.max(digits(&mut i).unwrap_or(0));
        zero = true;
    }
    for m in [&b"I64"[..], b"I32", b"hh", b"ll", b"h", b"l", b"z", b"j", b"t"] {
        if b[i..].starts_with(m) {
            i += m.len();
            break;
        }
    }
    let hex = match b.get(i)? {
        b'd' | b'i' | b'u' => None,
        b'x' => Some(false),
        b'X' => Some(true),
        _ => return None,
    };
    if width > 0 && !zero {
        return None;
    }
    Some((Spec { zero, width, hex }, i + 1))
}

fn format_number(spec: &Spec, n: u32) -> String {
    let body = match spec.hex {
        None => n.to_string(),
        Some(false) => format!("{n:x}"),
        Some(true) => format!("{n:X}"),
    };
    match spec.zero {
        true => format!("{body:0>w$}", w = spec.width),
        false => body,
    }
}

// Identifiers the exe builds with printf, such as `colour_%u_default`: each integer
// conversion is expanded over 0..FORMAT_TOP.
fn exe_formats(data: &[u8], out: &mut BTreeSet<Vec<u8>>) {
    let ident = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
    let mut i = 0;
    while i < data.len() {
        if !(ident(data[i]) || data[i] == b'%') {
            i += 1;
            continue;
        }
        let mut parts: Vec<Result<&[u8], Spec>> = Vec::new();
        let mut j = i;
        let mut lit = j;
        while j < data.len() {
            if ident(data[j]) {
                j += 1;
                continue;
            }
            if data[j] != b'%' {
                break;
            }
            let Some((spec, len)) = spec_at(&data[j..]) else {
                break;
            };
            parts.push(Ok(&data[lit..j]));
            parts.push(Err(spec));
            j += len;
            lit = j;
        }
        parts.push(Ok(&data[lit..j]));
        let specs = parts.iter().filter(|p| p.is_err()).count();
        if (1..=FORMAT_MAX_SPECS).contains(&specs) && j - i <= MAX_NAME {
            let mut names: Vec<Vec<u8>> = vec![Vec::new()];
            for part in &parts {
                names = match part {
                    Ok(text) => names
                        .into_iter()
                        .map(|mut n| {
                            n.extend_from_slice(text);
                            n
                        })
                        .collect(),
                    Err(spec) => names
                        .iter()
                        .flat_map(|n| {
                            (0..FORMAT_TOP)
                                .map(move |k| [n.as_slice(), format_number(spec, k).as_bytes()].concat())
                        })
                        .collect(),
                };
            }
            out.extend(names.into_iter().filter(|n| {
                n.len() >= 2
                    && n.len() <= MAX_NAME
                    && (n[0].is_ascii_alphabetic() || n[0] == b'_')
                    && n.iter().any(u8::is_ascii_lowercase)
            }));
        }
        i = j.max(i + 1);
    }
}

fn lines(path: &Path) -> Vec<String> {
    let file = std::fs::File::open(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let reader: Box<dyn Read> = match path.extension().is_some_and(|e| e == "gz") {
        true => Box::new(GzDecoder::new(file)),
        false => Box::new(file),
    };
    std::io::BufReader::new(reader)
        .lines()
        .map(|l| l.expect("utf-8 line"))
        .collect()
}

fn arg(args: &[String], name: &str) -> Option<PathBuf> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let game = arg(&args, "--game").expect("--game <F1 25 folder>");
    let tools = arg(&args, "--tools").unwrap_or_else(|| PathBuf::from("tools/gamedb"));
    let out =
        arg(&args, "--out").unwrap_or_else(|| PathBuf::from("packages/core/src/activation/row_keys.txt.gz"));
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let db_path = game.join("game_package/gamedatabases/gamedatabases.2025.bin");
    let db = Gvdb::parse(&std::fs::read(&db_path).expect("read the database")).expect("parse the database");
    let keys: Keys = db.records.iter().map(|r| r.key).collect();

    let mut names = BTreeSet::new();
    exe_names(&game.join("F1_25.exe"), &mut names);
    let exe_count = names.len();
    let mut tables = BTreeSet::new();
    for l in lines(&tools.join("names.txt")) {
        let l = l.trim();
        if !l.is_empty() && !l.starts_with('#') {
            names.insert(l.as_bytes().to_vec());
        }
    }
    for l in lines(&tools.join("compositions.txt")) {
        let l = l.trim();
        if !l.is_empty() && !l.starts_with('#') {
            names.extend(expand_braces(l).into_iter().map(String::into_bytes));
        }
    }
    for l in lines(&tools.join("catalog/keys.tsv.gz")).into_iter().skip(1) {
        if let Some(field) = l.split('\t').nth(2) {
            names.insert(field.as_bytes().to_vec());
        }
    }
    for l in lines(&tools.join("catalog/tables.tsv")).into_iter().skip(1) {
        let mut cols = l.split('\t');
        let (Some(table), Some(word)) = (cols.next(), cols.next()) else {
            continue;
        };
        if !word.is_empty() {
            names.insert(word.as_bytes().to_vec());
        }
        if table.contains('{') {
            continue;
        }
        let segs: Vec<&str> = table.split('.').collect();
        if let Some(i) = segs.iter().position(|s| s.ends_with("_N")) {
            tables.insert((segs[..i].join("."), segs[i].trim_end_matches("_N").to_owned()));
        }
    }
    names.retain(|n| !n.is_empty() && n.len() <= MAX_NAME);
    eprintln!(
        "vocabulary: {} names ({exe_count} from the exe), {} top-level tables",
        names.len(),
        tables.len()
    );

    let mut w = Walker {
        db: &db,
        keys,
        vocab: Vocab {
            names: names.into_iter().collect(),
            threads,
        },
        shapes: BTreeMap::new(),
        rows: 0,
        named: Keys::default(),
    };
    let started = std::time::Instant::now();
    for (i, (container, word)) in tables.iter().enumerate() {
        w.table(container, word);
        if i % 50 == 0 {
            eprintln!(
                "{i}/{} tables, {} rows, {} shapes, {:.0?}",
                tables.len(),
                w.rows,
                w.shapes.len(),
                started.elapsed()
            );
        }
    }

    let mut stat_tables = Vec::new();
    for (container, word) in &tables {
        let Some(n) = count(&db, container, word) else {
            continue;
        };
        let ids: Vec<i64> = (0..n)
            .filter_map(|i| db.get_text(&format!("{container}.{word}_{i}.stat_id")))
            .filter_map(|v| v.trim().parse().ok())
            .collect();
        let unique: HashSet<i64> = ids.iter().copied().collect();
        if ids.len() > 1 {
            stat_tables.push((
                format!("{container}.{word}_N"),
                n,
                ids.len(),
                unique.len() == ids.len(),
                ids.iter().max().copied().unwrap_or(0),
            ));
        }
    }

    let mut body = String::new();
    body.push_str("# Row keys of gamedatabases.2025.bin per row shape, from F1_25.exe identifiers and format strings,\n");
    body.push_str(
        "# tools/gamedb/names.txt, tools/gamedb/compositions.txt and the key catalogue. `+word` is a nested table. Generated by\n",
    );
    body.push_str(
        "# `cargo run -p swofty-core --example row_keys -- --game <F1 25 folder>`. Do not edit by hand.\n",
    );
    let mut entries = 0;
    for (shape, s) in &w.shapes {
        if s.entries.is_empty() {
            continue;
        }
        body.push('@');
        body.push_str(shape);
        body.push('\n');
        for e in &s.entries {
            body.push_str(e);
            body.push('\n');
            entries += 1;
        }
    }
    let mut gz = GzEncoder::new(Vec::new(), Compression::best());
    gz.write_all(body.as_bytes()).expect("compress");
    std::fs::write(&out, gz.finish().expect("compress")).expect("write the catalogue");
    eprintln!(
        "{} shapes, {entries} entries, {} rows walked, {} keys named of {} records, {:.0?}",
        w.shapes.len(),
        w.rows,
        w.named.len(),
        db.records.len(),
        started.elapsed()
    );
    if let Some(path) = arg(&args, "--stat-ids") {
        let mut t = String::from("table\trows\twith_stat_id\tunique\tmax\n");
        for (table, rows, with, unique, max) in stat_tables {
            t.push_str(&format!("{table}\t{rows}\t{with}\t{unique}\t{max}\n"));
        }
        std::fs::write(path, t).expect("write the stat id table");
    }
}
