use std::path::Path;

// F1 Multiplayer has no website yet: posts link nowhere and carry their images inline or by full URL.
const SITE: &str = "";

fn front_matter(text: &str) -> Vec<(String, String)> {
    let mut lines = text.lines();
    if lines.next().map(str::trim) != Some("---") {
        return Vec::new();
    }
    lines
        .take_while(|l| l.trim() != "---")
        .filter_map(|l| l.split_once(':'))
        .map(|(k, v)| {
            let v = v.trim();
            let v = v
                .strip_prefix('"')
                .and_then(|v| v.strip_suffix('"'))
                .or_else(|| v.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')))
                .unwrap_or(v);
            (k.trim().to_owned(), v.replace("\\\"", "\""))
        })
        .collect()
}

fn body(text: &str) -> String {
    let mut lines: Vec<&str> = text.lines().collect();
    if lines.first().map(|l| l.trim()) == Some("---") {
        let end = lines[1..].iter().position(|l| l.trim() == "---");
        lines.drain(..end.map_or(lines.len(), |e| e + 2));
    }
    let start = lines
        .iter()
        .position(|l| !l.trim().is_empty())
        .unwrap_or(lines.len());
    let mut lines = &lines[start..];
    if lines.first().is_some_and(|l| l.starts_with("# ")) {
        lines = &lines[1..];
    }
    lines.join("\n").trim().to_owned()
}

// Mirrors docs/scripts/news.mjs, which publishes the same feed as news.json.
fn posts(dir: &Path, kind: &str, out: &mut Vec<serde_json::Value>) {
    println!("cargo:rerun-if-changed={}", dir.display());
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for path in entries.flatten().map(|e| e.path()) {
        let Some(slug) = path
            .file_stem()
            .and_then(|s| s.to_str())
            .filter(|s| *s != "index" && path.extension().is_some_and(|e| e == "md"))
        else {
            continue;
        };
        let text = std::fs::read_to_string(&path).unwrap_or_default();
        let fm = front_matter(&text);
        let get = |key: &str| fm.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone());
        if get("draft").as_deref() == Some("true") {
            continue;
        }
        let title = get("title").unwrap_or_else(|| slug.to_owned());
        let title = if kind == "release" && title.starts_with(|c: char| c.is_ascii_digit()) {
            format!("F1 Multiplayer {title}")
        } else {
            title
        };
        let image = get("image").map(|i| {
            if i.starts_with('/') {
                format!("{SITE}{i}")
            } else {
                i
            }
        });
        let mut post = serde_json::json!({
            "id": format!("{kind}/{slug}"),
            "kind": kind,
            "title": title,
            "date": get("date").unwrap_or_default(),
            "summary": get("description").unwrap_or_default(),
            "image": image,
            "url": (!SITE.is_empty())
                .then(|| format!("{SITE}/{}/{slug}", if kind == "release" { "releases" } else { "news" })),
        });
        if kind == "news" {
            post["body"] = body(&text).into();
        }
        out.push(post);
    }
}

fn tools_art(out: &Path) {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/tools");
    println!("cargo:rerun-if-changed={}", dir.display());
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .map(|d| d.flatten().map(|e| e.path()).filter(|p| p.is_file()).collect())
        .unwrap_or_default();
    files.sort();
    let mut code = String::from("pub const ART: &[(&str, &[u8])] = &[\n");
    for path in files {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default();
        code += &format!(
            "    ({name:?}, include_bytes!({:?})),\n",
            path.display().to_string()
        );
    }
    code += "];\n";
    std::fs::write(out.join("tools_art.rs"), code).expect("failed to write the tools art table");
}

fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");
    let docs = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs");
    let mut feed = Vec::new();
    // F1 Multiplayer's own posts; docs/news and docs/releases are Swofty Launcher's history.
    posts(&docs.join("f1mp/news"), "news", &mut feed);
    posts(&docs.join("f1mp/releases"), "release", &mut feed);
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    let json = serde_json::json!({ "posts": feed });
    std::fs::write(out.join("news.json"), json.to_string()).expect("failed to write the embedded news feed");
    tools_art(&out);
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico")
            .set("FileDescription", "F1 Multiplayer 2016-2026")
            .set("ProductName", "F1 Multiplayer 2016-2026")
            .set("OriginalFilename", "f1-multiplayer.exe");
        res.compile().expect("failed to embed the launcher icon");
    }
}
