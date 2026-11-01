use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn data_dir(name: &str, profiles: &[&str]) -> PathBuf {
    let root = std::env::temp_dir().join(format!("swofty-cli-profile-{name}-{}", std::process::id()));
    std::fs::remove_dir_all(&root).ok();
    std::fs::create_dir_all(root.join("profiles")).unwrap();
    std::fs::write(
        root.join("config.json"),
        r#"{"data_version":3,"active_profile":"grid"}"#,
    )
    .unwrap();
    for p in profiles {
        std::fs::write(
            root.join(format!("profiles/{p}.json")),
            format!(r#"{{"schema_version":2,"name":"{p}","addons":[]}}"#),
        )
        .unwrap();
    }
    root
}

fn swofty(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_f1mp"))
        .args(args)
        .env("F1MP_DATA_DIR", root)
        .output()
        .unwrap()
}

fn text(out: &Output) -> (String, String) {
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn lists_profiles_and_refuses_the_ones_that_must_stay() {
    let root = data_dir("refuse", &["grid"]);
    let out = swofty(&root, &["profile", "list"]);
    let (stdout, stderr) = text(&out);
    assert!(out.status.success(), "{stderr}");
    let lines: Vec<String> = stdout
        .lines()
        .map(|l| l.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect();
    assert_eq!(lines, ["vanilla built in", "grid active"]);

    for (args, error) in [
        (
            &["profile", "delete", "-y", "grid"][..],
            "grid is the only profile",
        ),
        (
            &["profile", "delete", "--yes", "vanilla"][..],
            "vanilla profile is built in",
        ),
        (
            &["profile", "delete", "--yes", "nope"][..],
            "no profile named nope",
        ),
    ] {
        let out = swofty(&root, args);
        let (_, stderr) = text(&out);
        assert!(!out.status.success());
        assert!(stderr.contains(error), "{args:?}: {stderr}");
    }

    std::fs::write(
        root.join("profiles/main.json"),
        r#"{"schema_version":2,"name":"main","addons":[]}"#,
    )
    .unwrap();
    std::fs::create_dir_all(root.join("state/journal")).unwrap();
    std::fs::write(
        root.join("state/journal/j1.json"),
        r#"{"schema_version":1,"id":"j1","profile":"grid","build_id":"b","entries":[],"committed":true}"#,
    )
    .unwrap();
    let out = swofty(&root, &["profile", "delete", "grid", "--yes"]);
    let (_, stderr) = text(&out);
    assert!(!out.status.success());
    assert!(
        stderr.contains("grid is installed in the game folder (journal j1)"),
        "{stderr}"
    );
    assert!(root.join("profiles/grid.json").is_file());
    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn deletes_a_profile_only_when_told_to() {
    if swofty_core::process::game_running() {
        eprintln!("skipped: F1 25 is running, and profiles are not deleted while it runs");
        return;
    }
    let root = data_dir("delete", &["grid", "main"]);
    std::fs::create_dir_all(root.join("cache/overlay/grid/h1")).unwrap();

    let out = swofty(&root, &["profile", "delete", "grid"]);
    let (_, stderr) = text(&out);
    assert!(!out.status.success());
    assert!(stderr.contains("pass --yes to delete grid"), "{stderr}");
    assert!(root.join("profiles/grid.json").is_file());

    let out = swofty(&root, &["profile", "delete", "grid", "--yes"]);
    let (stdout, stderr) = text(&out);
    assert!(out.status.success(), "{stderr}");
    assert!(stdout.contains("main is now the active profile"), "{stdout}");
    assert!(stdout.contains("deleted profile grid"), "{stdout}");
    assert!(!root.join("profiles/grid.json").exists());
    assert!(!root.join("cache/overlay/grid").exists());
    assert!(root.join("profiles/main.json").is_file());
    let config: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("config.json")).unwrap()).unwrap();
    assert_eq!(config["active_profile"], "main");
    std::fs::remove_dir_all(&root).ok();
}
