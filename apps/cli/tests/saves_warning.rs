use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

fn data_dir(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("swofty-cli-saves-{name}-{}", std::process::id()));
    std::fs::remove_dir_all(&root).ok();
    std::fs::create_dir_all(&root).unwrap();
    for (id, saves) in [
        (
            "t.saves",
            r#","saves":{"affects":true,"message":"Retire drivers instead of removing the addon."}"#,
        ),
        ("t.plain", ""),
    ] {
        let dir = root.join("src").join(id);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("swofty.json"),
            format!(
                r#"{{"schema_version":1,"id":"{id}","name":"{id}","version":"1.0.0","author":"t","compat":{{"game":"f1_25"}}{saves}}}"#
            ),
        )
        .unwrap();
    }
    root
}

fn swofty(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_f1mp"))
        .args(args)
        .env("F1MP_DATA_DIR", root.join("data"))
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

fn ok(root: &Path, args: &[&str]) -> String {
    let out = swofty(root, args);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{args:?}: {stderr}");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn state(root: &Path, id: &str) -> Option<String> {
    ok(root, &["addons", "list"]).lines().find_map(|l| {
        let mut words = l.split_whitespace();
        (words.next() == Some(id)).then(|| words.nth(1).unwrap_or_default().to_owned())
    })
}

#[test]
fn turning_off_or_removing_an_addon_that_affects_saves_needs_yes() {
    let root = data_dir("yes");
    for id in ["t.saves", "t.plain"] {
        let src = root.join("src").join(id);
        ok(&root, &["addons", "install", src.to_str().unwrap()]);
        ok(&root, &["addons", "enable", id]);
    }
    assert_eq!(state(&root, "t.saves").as_deref(), Some("enabled"));

    for args in [
        &["addons", "disable", "t.saves"][..],
        &["addons", "remove", "t.saves"],
    ] {
        let out = swofty(&root, args);
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(!out.status.success(), "{args:?}");
        for text in [
            "Warning: t.saves (t.saves) has affected your saves.",
            "Careers you played with it may not load without it.",
            "Retire drivers instead of removing the addon.",
            "pass --yes to",
        ] {
            assert!(stderr.contains(text), "{args:?}: {stderr}");
        }
        assert_eq!(state(&root, "t.saves").as_deref(), Some("enabled"), "{args:?}");
    }

    ok(&root, &["addons", "disable", "t.plain"]);
    assert_eq!(state(&root, "t.plain").as_deref(), Some("disabled"));
    ok(&root, &["addons", "remove", "t.plain"]);
    assert_eq!(state(&root, "t.plain"), None);

    let out = ok(&root, &["addons", "disable", "--yes", "t.saves"]);
    assert!(out.contains("disabled t.saves"), "{out}");
    assert_eq!(state(&root, "t.saves").as_deref(), Some("disabled"));
    ok(&root, &["addons", "disable", "t.saves"]);

    let out = swofty(&root, &["addons", "remove", "t.saves"]);
    assert!(!out.status.success());
    assert_eq!(state(&root, "t.saves").as_deref(), Some("disabled"));
    let out = ok(&root, &["addons", "remove", "-y", "t.saves"]);
    assert!(out.contains("removed t.saves"), "{out}");
    assert_eq!(state(&root, "t.saves"), None);
    assert!(!root.join("data/addons/t.saves").exists());

    let out = swofty(&root, &["addons", "remove", "t.gone"]);
    assert!(String::from_utf8_lossy(&out.stderr).contains("t.gone is not installed"));
    std::fs::remove_dir_all(&root).ok();
}
