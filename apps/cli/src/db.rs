use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

use anyhow::{Result, anyhow, bail};
use clap::Subcommand;
use swofty_core::dbedit::package::{self, Package};
use swofty_core::dbedit::{Catalog, Edits, NodeId, NodeKind, RowCopy, deps, driver_manager, modules};
use swofty_core::manifest::Category;
use swofty_core::store::Store;

#[derive(Subcommand)]
pub enum DbCommand {
    #[command(
        about = "List DB Editor addons and whether each is on in the profile",
        alias = "packages"
    )]
    Addons,
    #[command(about = "Create an empty DB Editor addon")]
    New { name: String },
    #[command(about = "Delete a DB Editor addon from every profile")]
    Delete { id: String },
    #[command(about = "Write a DB Editor addon to a .zip to share it")]
    Export { id: String, file: PathBuf },
    #[command(
        about = "Add a DB Editor addon from a .zip, a DB Editor export, an EGO GameDatabases Editor export, or a modded gamedatabases.2025.bin or mod .zip holding one"
    )]
    Import { file: PathBuf },
    #[command(
        about = "Write the game database with DB Editor addons applied to a .zip players without the launcher extract into the game folder"
    )]
    ExportGame {
        file: PathBuf,
        #[arg(
            long = "addon",
            alias = "package",
            value_name = "ID",
            help = "An addon to apply, in order; repeat it (default: the DB Editor addons on in the profile, in load order)"
        )]
        addons: Vec<String>,
    },
    #[command(about = "Turn off every DB Editor addon that is on in the profile")]
    OffAll,
    #[command(about = "Show a DB Editor addon's details, or change them with the flags")]
    Details {
        id: String,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        version: Option<String>,
        #[arg(long)]
        author: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long, value_parser = category, help = "One of seasons, tracks, cars, helmets, career, ai, performance, utility, uncategorised")]
        category: Option<CategoryArg>,
    },
    #[command(about = "Search names, tables, keys, values and English text")]
    Search {
        query: String,
        #[arg(long, default_value_t = 30)]
        limit: usize,
    },
    #[command(
        about = "List what a copy of a row does not bring along: rows of other tables labelled with the row's label or a name derived from it, and values that pick the row by label or stat_id"
    )]
    Deps {
        #[arg(help = "The row, as a dotted or selector path")]
        row: String,
    },
    #[command(about = "Show a key's value, or what a table, row or container holds")]
    Get {
        #[arg(help = "A dotted or selector path, or 0x and 16 hex digits")]
        path: String,
        #[arg(
            long,
            alias = "package",
            help = "Show values with this addon's changes (default: stock)"
        )]
        addon: Option<String>,
    },
    #[command(about = "Change a key's value in a DB Editor addon")]
    Set {
        path: String,
        value: String,
        #[arg(long, alias = "package")]
        addon: String,
    },
    #[command(
        about = "Add a key the game database lacks; TYPE is the record type number (0 int, 1 float, 2 string)"
    )]
    Add {
        path: String,
        #[arg(value_name = "TYPE")]
        tag: u8,
        value: String,
        #[arg(long, alias = "package")]
        addon: String,
    },
    #[command(
        about = "Add a row to a table for an entity that has none: a copy of another entity's row, or of the game default row with FROM default"
    )]
    CopyRow {
        #[arg(help = "The table's rows, such as driver_statistics.driver_statistics.DriverStatisticsHolder")]
        table: String,
        #[arg(help = "The entity whose row to copy, or default")]
        from: String,
        #[arg(help = "The entity the new row belongs to")]
        to: String,
        #[arg(
            long,
            help = "The field naming a row's entity (default: the one the DB Editor pages use, else _metadata_)"
        )]
        field: Option<String>,
        #[arg(long, alias = "package")]
        addon: String,
    },
    #[command(about = "Remove a row copy-row added, with its changes")]
    RemoveRow {
        table: String,
        to: String,
        #[arg(long)]
        field: Option<String>,
        #[arg(long, alias = "package")]
        addon: String,
    },
    #[command(about = "Put a key back to its stock value in a DB Editor addon, or every key with --all")]
    Revert {
        path: Option<String>,
        #[arg(long)]
        all: bool,
        #[arg(long, alias = "package")]
        addon: String,
    },
    #[command(about = "List a DB Editor addon's database changes")]
    List {
        #[arg(long, alias = "package")]
        addon: String,
    },
}

/// The field naming a row's entity in `table`: `--field`, else the one a DB Editor page selects it by.
fn row_field(table: &str, field: Option<String>) -> String {
    field.unwrap_or_else(|| {
        modules()
            .iter()
            .flat_map(|m| m.row_specs())
            .find(|s| s.table == table)
            .map_or_else(|| "_metadata_".to_owned(), |s| s.field)
    })
}

fn row_line(r: &RowCopy) -> String {
    let from = r.from.as_deref().unwrap_or("the game default row");
    format!("{}: a copy of {from}", r.select())
}

#[derive(Debug, Clone, Copy)]
pub struct CategoryArg(Option<Category>);

fn category(s: &str) -> std::result::Result<CategoryArg, String> {
    if s == "uncategorised" {
        return Ok(CategoryArg(None));
    }
    match serde_json::from_value(serde_json::Value::String(s.to_owned())) {
        Ok(Category::Shaders) => Err("shaders is for shader packs, not DB Editor addons; see --help".into()),
        Ok(c) => Ok(CategoryArg(Some(c))),
        Err(_) => Err(format!("{s} is not a category; see --help")),
    }
}

/// Why `path` isn't in the game database when it names a Driver Manager driver's row.
fn driver_row(store: &Store, cat: &Catalog, path: &str) -> Option<String> {
    let label = path
        .split(['[', ']', '=', '.'])
        .find(|p| driver_manager::is_label(p))?;
    Some(
        match driver_manager::drivers(store, cat)
            .iter()
            .find(|d| d.label == label)
        {
            Some(d) => format!(
                "{path}: Driver Manager adds {} ({label}) when the profile builds; edit the driver on the Driver Manager page",
                d.name
            ),
            None => format!(
                "{path}: {label} is a Driver Manager label, and Driver Manager doesn't add that driver"
            ),
        },
    )
}

/// Notes on the Driver Manager drivers `value` picks: one Driver Manager doesn't add,
/// or any while it's off in `profile`.
fn driver_notes(store: &Store, cat: &Catalog, profile: &str, value: &str) -> Vec<String> {
    let labels: Vec<&str> = value.split(',').filter(|v| driver_manager::is_label(v)).collect();
    if labels.is_empty() {
        return Vec::new();
    }
    let drivers = driver_manager::drivers(store, cat);
    let on = store
        .profile(profile)
        .is_ok_and(|p| p.enabled().any(|id| id == driver_manager::ID));
    labels
        .into_iter()
        .filter_map(|l| match drivers.iter().find(|d| d.label == l) {
            Some(_) if on => None,
            Some(d) => Some(format!(
                "{l} is Driver Manager's {}, and Driver Manager is off in {profile}; {}",
                d.name,
                driver_manager::MISSING
            )),
            None => Some(format!(
                "Driver Manager doesn't add {l}; {}",
                driver_manager::MISSING
            )),
        })
        .collect()
}

fn open(store: &Store, id: &str) -> Result<Package> {
    if !package::is_package(store, id) {
        bail!("no DB Editor addon named {id}; f1mp db addons lists them");
    }
    Ok(Package::open(store, id)?)
}

/// Opens the addon, applies `change` and saves it; when another writer saved
/// the addon in between, starts over from its file.
fn edit<T>(
    store: &Store,
    cat: &Catalog,
    id: &str,
    mut change: impl FnMut(&mut Package) -> Result<T>,
) -> Result<T> {
    for _ in 0..5 {
        let stamp = package::modified(store, id);
        let mut p = open(store, id)?;
        let out = change(&mut p)?;
        if package::modified(store, id) == stamp {
            p.save(store, cat)?;
            return Ok(out);
        }
    }
    bail!("{id} kept changing while saving; try again")
}

pub fn command(store: &Store, profile: &str, game: Option<PathBuf>, cmd: DbCommand) -> Result<ExitCode> {
    let catalog = || -> Result<Arc<Catalog>> {
        let g = crate::build::game(store, game.clone())?;
        Ok(Catalog::open(store, &g)?)
    };
    match cmd {
        DbCommand::Addons => {
            let p = store.profile(profile)?;
            let on: Vec<&str> = p.enabled().collect();
            let package::Scan {
                packages: all,
                damaged,
            } = package::scan(store)?;
            for (id, e) in &damaged {
                println!("bad  {id}  {e}");
            }
            for i in &all {
                let state = if on.contains(&i.id.as_str()) { "on " } else { "off" };
                println!(
                    "{state}  {}  {} {} by {}, {} change(s)",
                    i.id, i.name, i.version, i.author, i.changes
                );
            }
            println!(
                "{} DB Editor addon(s); on/off is for {profile}",
                all.len() + damaged.len()
            );
        }
        DbCommand::New { name } => {
            let p = Package::create(store, &name)?;
            println!(
                "created {} ({}); turn it on with f1mp addons enable {}",
                p.id, p.manifest.name, p.id
            );
        }
        DbCommand::Delete { id } => {
            open(store, &id)?;
            Package::delete(store, &id)?;
            println!("deleted {id}");
        }
        DbCommand::Export { id, file } => {
            let p = open(store, &id)?;
            Package::export_zip(store, &id, &file)?;
            println!("wrote {} ({} change(s)) to {}", id, p.edits.len(), file.display());
        }
        DbCommand::Import { file } => {
            let got = package::import(store, &*catalog()?, &file)?;
            let (p, r) = (&got.package, &got.report);
            let counts = match r.format {
                "bin" => format!("{} change(s)", r.imported),
                _ => format!(
                    "{} imported, {} unchanged, {} skipped",
                    r.imported,
                    r.unchanged,
                    r.skipped.len()
                ),
            };
            println!(
                "{} ({}): {counts} into {} ({})",
                file.display(),
                r.format,
                p.id,
                p.name
            );
            if let Some(was) = &got.renamed_from {
                println!("  {was} is already installed; installed as {}", p.id);
            }
            let prefix = if r.format == "bin" { "" } else { "skipped " };
            for s in &r.skipped {
                println!("  {prefix}{s}");
            }
        }
        DbCommand::ExportGame { file, addons } => {
            let ids = match addons.is_empty() {
                true => package::enabled_packages(store, profile)?,
                false => addons,
            };
            if ids.is_empty() {
                bail!("no DB Editor addons are on in {profile}; pass --addon <id>");
            }
            for id in &ids {
                open(store, id)?;
            }
            let g = crate::build::game(store, game.clone())?;
            let got = package::export_game_files(store, &g, &ids, &file)?;
            println!(
                "wrote {} ({} change(s) from {}, {} bytes)",
                file.display(),
                got.changes,
                ids.join(", "),
                got.bytes
            );
            for s in got.skipped.iter().chain(&got.warnings) {
                println!("  {s}");
            }
        }
        DbCommand::OffAll => {
            let n = package::turn_off_all(store, profile)?;
            println!("turned off {n} DB Editor addon(s) in {profile}");
        }
        DbCommand::Details {
            id,
            name,
            version,
            author,
            description,
            category,
        } => {
            let mut p = open(store, &id)?;
            if let Some(CategoryArg(c)) = category {
                p.set_category(c);
            }
            if name.is_some()
                || version.is_some()
                || author.is_some()
                || description.is_some()
                || category.is_some()
            {
                let i = p.info();
                p.set_details(
                    name.as_deref().unwrap_or(&i.name),
                    version.as_deref().unwrap_or(&i.version),
                    author.as_deref().unwrap_or(&i.author),
                    description.as_deref().unwrap_or(&i.description),
                )?;
                p.write(store)?;
            }
            let i = p.info();
            println!(
                "{}\n  name: {}\n  version: {}\n  author: {}",
                i.id, i.name, i.version, i.author
            );
            println!("  description: {}\n  changes: {}", i.description, i.changes);
            if let Some(c) = p.manifest.category {
                println!("  category: {}", c.label());
            }
            if !i.game_build.is_empty() {
                println!("  database: {}", i.game_build);
            }
        }
        DbCommand::Search { query, limit } => {
            let cat = catalog()?;
            for h in cat.search_with(&query, limit, &driver_manager::drivers(store, &cat)) {
                println!("{:<10} {}  {}", format!("{:?}", h.kind), h.title, h.detail);
            }
        }
        DbCommand::Deps { row } => {
            let cat = catalog()?;
            let node = cat
                .find(&row)
                .ok_or_else(|| anyhow!("{row}: not in the game database"))?;
            if !matches!(cat.node(node).kind, NodeKind::Row { .. }) {
                bail!("{row}: not a row");
            }
            print_deps(&cat, node);
        }
        DbCommand::Get { path, addon } => {
            let edits = match &addon {
                Some(id) => open(store, id)?.edits,
                None => Edits::default(),
            };
            let cat = catalog()?;
            match (cat.find(&path), edits.copied_path(&cat, &path)) {
                (Some(node), _) => show(&cat, &edits, node),
                (None, Some(c)) => {
                    let value = edits.value(&cat, c.key).unwrap_or_default();
                    println!("{} (0x{:016x})", c.row.path(&c.rel), c.key);
                    println!("  value: {value}");
                    println!("  {}", row_line(c.row));
                    if edits.get(c.key).is_some() {
                        println!(
                            "  copied: {}",
                            cat.record_by_key(c.source).map_or("", |r| r.value)
                        );
                    }
                }
                (None, None) => match driver_row(store, &cat, &path) {
                    Some(why) => bail!("{why}"),
                    None => bail!("{path}: not in the game database"),
                },
            }
        }
        DbCommand::Set { path, value, addon } => {
            let cat = catalog()?;
            edit(store, &cat, &addon, |p| {
                let key = p
                    .edits
                    .resolve(&cat, &path)
                    .map_err(|e| anyhow!(driver_row(store, &cat, &path).unwrap_or(e)))?;
                p.edits.set(&cat, key, &value).map_err(|e| anyhow!("{path}: {e}"))
            })?;
            println!("{path} = {value} in {addon}");
            for note in driver_notes(store, &cat, profile, &value) {
                println!("  note: {note}");
            }
        }
        DbCommand::CopyRow {
            table,
            from,
            to,
            field,
            addon,
        } => {
            let cat = catalog()?;
            let field = row_field(&table, field);
            let source = (from != "default").then_some(from.as_str());
            let line = edit(store, &cat, &addon, |p| {
                p.edits
                    .copy_row(&cat, &table, &field, source, &to)
                    .map_err(|e| anyhow!("{e}"))?;
                Ok(row_line(p.edits.row(&table, &field, &to).expect("just added")))
            })?;
            println!("added {line} in {addon}");
        }
        DbCommand::RemoveRow {
            table,
            to,
            field,
            addon,
        } => {
            let cat = catalog()?;
            let field = row_field(&table, field);
            edit(store, &cat, &addon, |p| {
                match p.edits.remove_row(&table, &field, &to) {
                    true => Ok(()),
                    false => bail!("{addon} adds no row for {to} to {table}"),
                }
            })?;
            println!("removed the row for {to} from {table} in {addon}");
        }
        DbCommand::Add {
            path,
            tag,
            value,
            addon,
        } => {
            let cat = catalog()?;
            let key = edit(store, &cat, &addon, |p| {
                p.edits
                    .add(&cat, &path, tag, &value)
                    .map_err(|e| anyhow!("{path}: {e}"))
            })?;
            println!("added {path} (0x{key:016x}) = {value} in {addon}");
        }
        DbCommand::Revert { path, all, addon } => {
            let cat = catalog()?;
            match (path, all) {
                (_, true) => {
                    let n = edit(store, &cat, &addon, |p| Ok(p.clear_edits()))?;
                    println!("reverted {n} change(s) in {addon}");
                }
                (Some(path), false) => {
                    edit(store, &cat, &addon, |p| {
                        let key = p.edits.resolve(&cat, &path).map_err(|e| anyhow!("{e}"))?;
                        match p.edits.revert(key) {
                            true => Ok(()),
                            false => bail!("{path} has no change in {addon}"),
                        }
                    })?;
                    println!("reverted {path} in {addon}");
                }
                (None, false) => bail!("name a key to revert, or pass --all"),
            }
        }
        DbCommand::List { addon } => {
            let p = open(store, &addon)?;
            for r in &p.edits.rows {
                println!("{}", row_line(r));
            }
            for e in &p.edits.edits {
                let name = e.path.clone().unwrap_or_else(|| format!("0x{:016x}", e.key));
                let stock = match (&e.stock, e.is_copied()) {
                    (Some(s), _) => s.as_str(),
                    (None, true) => "(copied row)",
                    (None, false) => "(added)",
                };
                let value = e.value.as_deref().unwrap_or("(removed)");
                println!("{name}: {stock} -> {value}");
            }
            println!("{} change(s) in {addon}", p.edits.len());
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn print_deps(cat: &Catalog, row: NodeId) {
    let d = deps::deps(cat, row);
    let mut hows: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    for n in &d.names {
        hows.entry(n.value.as_str()).or_default().push(n.how());
    }
    println!("{}", cat.path(row));
    println!("names the game can derive from the row:");
    for (value, how) in &hows {
        println!("  {value}: {}", how.join(", "));
    }
    let tag = |area: Option<&str>| area.map(|a| format!(" [{a}]")).unwrap_or_default();
    if !d.rules.is_empty() {
        println!("rows the game is known to look up for the row:");
    }
    for (rule, name, found) in &d.rules {
        let by = match rule.field {
            None => format!("labelled {name} ({})", rule.how()),
            Some(field) => format!("whose {field} names {name}"),
        };
        let found = match found {
            Some(row) => cat.path(*row),
            None => "none".into(),
        };
        println!("  {} row {by}: {found}; {}", rule.table, rule.reader);
    }
    let mut keyed: BTreeMap<(bool, String, &str), (usize, Option<&str>)> = BTreeMap::new();
    for k in d.keyed.iter().filter(|k| k.rule.is_none()) {
        let tables = match k.tables.len() {
            1 => cat.path(k.tables[0]),
            _ => k.family.clone(),
        };
        let by_label = hows[k.name.value.as_str()]
            .iter()
            .any(|h| h.starts_with("the label"));
        let n = keyed
            .entry((!by_label, tables, k.name.value.as_str()))
            .or_insert((0, k.area));
        n.0 = n.0.max(k.tables.len());
    }
    let mut section = None;
    for ((by_field, tables, value), (n, area)) in &keyed {
        if section != Some(*by_field) {
            section = Some(*by_field);
            match by_field {
                false => println!("rows of other tables labelled with the label or a name built from it:"),
                true => println!(
                    "rows of other tables labelled with a value of the row (a copy that changes it needs its own):"
                ),
            }
        }
        let tables = match n {
            1 => tables.clone(),
            n => format!("{tables} ({n} tables)"),
        };
        println!("  {tables}: {value} ({}){}", hows[value].join(", "), tag(*area));
    }
    if d.keyed.is_empty() {
        println!("rows of other tables labelled with these names: none");
    }
    println!("values that pick the row:");
    if d.references.is_empty() {
        println!("  none");
    }
    for r in &d.references {
        let by = match r.by {
            deps::By::Label => "label",
            deps::By::StatId => "stat_id",
        };
        println!("  by {by}: {} ({}){}", r.field, r.count, tag(r.area));
    }
    if d.unnamed > 0 {
        println!(
            "  {} value(s) under keys the catalogue does not name hold the label",
            d.unnamed
        );
    }
    if d.keyed.iter().any(|k| k.area.is_some()) || d.references.iter().any(|r| r.area.is_some()) {
        println!(
            "[area]: a table of a game area an add-row copy does not reach by default; the build lists its findings only with f1mp build --side-data"
        );
    }
}

fn show(cat: &Catalog, edits: &Edits, node: NodeId) {
    let n = cat.node(node);
    if let Some(r) = n.record.map(|r| cat.record(r)) {
        let value = edits.value(cat, r.key).unwrap_or("(removed)");
        println!("{} (0x{:016x}, type {})", cat.path(node), r.key, r.tag);
        println!("  value: {value}");
        if edits.get(r.key).is_some() {
            println!("  stock: {}", r.value);
        }
        if let Some(text) = cat.text(value) {
            println!("  English: {text}");
        }
        if let Some(m) = cat.meta(r.key) {
            for (k, v) in [("meaning", &m.meaning), ("unit", &m.unit), ("range", &m.range)] {
                if !v.is_empty() {
                    println!("  {k}: {v}");
                }
            }
        }
        return;
    }
    println!("{}", cat.path(node));
    if let Some(m) = cat.table_meaning(node) {
        println!("  {m}");
    }
    for &c in cat.children(node) {
        let child = cat.node(c);
        let about = match &child.kind {
            NodeKind::Table { rows, .. } => format!("table, {rows} rows"),
            NodeKind::Row { .. } => [child.label.as_deref(), child.display.as_deref()]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join(" · "),
            NodeKind::Key => child
                .record
                .map(|r| {
                    edits
                        .value(cat, cat.record(r).key)
                        .unwrap_or("(removed)")
                        .to_owned()
                })
                .unwrap_or_default(),
            _ => String::new(),
        };
        println!("  {}  {about}", child.name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use swofty_formats::gvdb::{Gvdb, Record};
    use swofty_formats::hash::fnv1a64;

    #[test]
    fn keeps_a_change_another_window_saved_while_editing() {
        let root = std::env::temp_dir().join(format!("swofty-cli-db-edit-{}", std::process::id()));
        std::fs::remove_dir_all(&root).ok();
        let store = Store::new(root.clone());
        let records = vec![
            Record::new("entrants.teams._team_instancecount_", 0, "1"),
            Record::new("entrants.teams.team_0._metadata_", 2, "mclaren_26"),
            Record::new("entrants.teams.team_0.stat_id", 0, "100"),
            Record::new("entrants.teams.team_0.prestige", 0, "5"),
        ];
        let cat = Catalog::load(&Gvdb { records }.to_bytes()).unwrap();
        let id = Package::create(&store, "Mine").unwrap().id;
        let (stat, prestige) = (
            fnv1a64("entrants.teams.team_0.stat_id"),
            fnv1a64("entrants.teams.team_0.prestige"),
        );
        let mut calls = 0;
        edit(&store, &cat, &id, |p| {
            calls += 1;
            if calls == 1 {
                std::thread::sleep(std::time::Duration::from_millis(20));
                let mut other = Package::open(&store, &id)?;
                other.edits.set(&cat, stat, "7").map_err(|e| anyhow!(e))?;
                other.save(&store, &cat)?;
            }
            p.edits.set(&cat, prestige, "9").map_err(|e| anyhow!(e))
        })
        .unwrap();
        assert_eq!(calls, 2);
        let saved = Package::open(&store, &id).unwrap().edits;
        assert_eq!(saved.value(&cat, stat), Some("7"));
        assert_eq!(saved.value(&cat, prestige), Some("9"));
        std::fs::remove_dir_all(&root).ok();
    }
}
