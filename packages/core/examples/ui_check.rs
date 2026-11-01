use std::path::Path;

use swofty_core::ui::check::{self, Level};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut pairs = Vec::new();
    let mut against = None;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--against" => against = it.next().cloned(),
            pair => {
                let (stock, built) = pair
                    .split_once('=')
                    .expect("usage: ui_check <stock.erp>=<built.erp>... [--against <stock.erp>=<other.erp>]");
                pairs.push((stock.to_owned(), built.to_owned()));
            }
        }
    }
    let run = |pairs: &[(String, String)]| {
        let named: Vec<(String, &Path, &Path)> = pairs
            .iter()
            .map(|(s, b)| {
                let name = Path::new(s).file_name().unwrap().to_string_lossy().into_owned();
                (name, Path::new(s.as_str()), Path::new(b.as_str()))
            })
            .collect();
        let refs: Vec<(&str, &Path, &Path)> = named.iter().map(|(n, s, b)| (n.as_str(), *s, *b)).collect();
        check::archive_files(&refs).expect("check the archives")
    };
    let report = run(&pairs);
    let base: Vec<String> = match &against {
        Some(p) => {
            let (s, b) = p.split_once('=').expect("--against <stock.erp>=<other.erp>");
            let r = run(&[(s.to_owned(), b.to_owned())]);
            r.hard.iter().chain(&r.warnings).map(|f| f.to_string()).collect()
        }
        None => Vec::new(),
    };
    let all: Vec<_> = report.hard.iter().chain(&report.warnings).collect();
    let mut shown = 0;
    for f in &all {
        let line = f.to_string();
        if base.contains(&line) {
            continue;
        }
        shown += 1;
        println!("{} {line}", if f.level == Level::Hard { "HARD" } else { "warn" });
    }
    eprintln!(
        "{} hard, {} warnings, {shown} not in the --against build",
        report.hard.len(),
        report.warnings.len()
    );
    std::process::exit(i32::from(!report.hard.is_empty()));
}
