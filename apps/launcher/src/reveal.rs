use std::path::{Path, PathBuf};
use std::process::Command;

use gpui_kit::App;

pub const FILE_MANAGER: &str = if cfg!(windows) {
    "File Explorer"
} else {
    "your file manager"
};

/// Opens the file manager on `path`'s folder with `path` selected. Linux asks
/// `org.freedesktop.FileManager1` first; when no file manager answers, or
/// Explorer can't start, GPUI's reveal takes over (the desktop portal, then
/// `xdg-open` on the folder).
pub fn show(path: &Path, cx: &App) {
    let path = real_path(path);
    cx.spawn(async move |cx| {
        let target = path.clone();
        let shown = cx
            .background_executor()
            .spawn(async move { select(&target) })
            .await;
        if !shown {
            cx.update(|cx| cx.reveal_path(&path));
        }
    })
    .detach();
}

/// `path` as the desktop sees it: symlinks resolved, and outside the Steam
/// Runtime's container, which mounts the host's files under `/run/host`.
pub fn real_path(path: &Path) -> PathBuf {
    if cfg!(windows) {
        return path.to_path_buf();
    }
    let real = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    outside_container(&real)
}

fn outside_container(path: &Path) -> PathBuf {
    match path.strip_prefix("/run/host") {
        Ok(rest) if path.has_root() => Path::new("/").join(rest),
        _ => path.to_path_buf(),
    }
}

#[cfg(windows)]
fn select(path: &Path) -> bool {
    use std::os::windows::process::CommandExt;
    Command::new("explorer.exe")
        .raw_arg(explorer_arg(path))
        .spawn()
        .is_ok()
}

#[cfg_attr(not(windows), allow(dead_code))]
fn explorer_arg(path: &Path) -> String {
    format!("/select,\"{}\"", path.display().to_string().replace('/', "\\"))
}

#[cfg(not(windows))]
fn select(path: &Path) -> bool {
    let uri = file_uri(path);
    show_items(&uri).into_iter().any(|mut c| {
        c.stdin(std::process::Stdio::null())
            .output()
            .is_ok_and(|o| o.status.success())
    })
}

/// `gdbus` or `dbus-send` calling `org.freedesktop.FileManager1.ShowItems`,
/// without the Steam overlay and Steam Runtime libraries a launch from Steam
/// passes on.
#[cfg_attr(windows, allow(dead_code))]
fn show_items(uri: &str) -> [Command; 2] {
    let host = |program: &str| {
        let mut c = Command::new(program);
        c.env_remove("LD_PRELOAD");
        if std::env::var("LD_LIBRARY_PATH").is_ok_and(|p| p.to_ascii_lowercase().contains("steam")) {
            c.env_remove("LD_LIBRARY_PATH");
        }
        c
    };
    let mut gdbus = host("gdbus");
    gdbus.args([
        "call",
        "--session",
        "--timeout",
        "10",
        "--dest",
        "org.freedesktop.FileManager1",
        "--object-path",
        "/org/freedesktop/FileManager1",
        "--method",
        "org.freedesktop.FileManager1.ShowItems",
        &format!("['{uri}']"),
        "''",
    ]);
    let mut dbus_send = host("dbus-send");
    dbus_send.args([
        "--session",
        "--print-reply",
        "--reply-timeout=10000",
        "--dest=org.freedesktop.FileManager1",
        "--type=method_call",
        "/org/freedesktop/FileManager1",
        "org.freedesktop.FileManager1.ShowItems",
        &format!("array:string:{uri}"),
        "string:",
    ]);
    [gdbus, dbus_send]
}

/// A `file://` URI with every byte outside the unreserved set and `/`
/// percent-encoded, so it is safe inside GVariant text and `dbus-send`'s
/// comma-separated arrays.
#[cfg_attr(windows, allow(dead_code))]
fn file_uri(path: &Path) -> String {
    #[cfg(unix)]
    let bytes = std::os::unix::ffi::OsStrExt::as_bytes(path.as_os_str()).to_vec();
    #[cfg(not(unix))]
    let bytes = path.to_string_lossy().into_owned().into_bytes();
    let mut uri = String::from("file://");
    for b in bytes {
        if b.is_ascii_alphanumeric() || b"-._~/".contains(&b) {
            uri.push(b as char);
        } else {
            uri.push_str(&format!("%{b:02X}"));
        }
    }
    uri
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_uris_escape_everything_but_plain_path_characters() {
        assert_eq!(
            file_uri(Path::new(
                "/home/a b/.local/share/swofty/logs/swofty-logs-Naël,o'k-20261023-101010.zip"
            )),
            "file:///home/a%20b/.local/share/swofty/logs/swofty-logs-Na%C3%ABl%2Co%27k-20261023-101010.zip"
        );
    }

    #[test]
    fn leaves_the_steam_runtime_container() {
        assert_eq!(
            outside_container(Path::new("/run/host/home/p/.local/share/swofty/logs/x.zip")),
            PathBuf::from("/home/p/.local/share/swofty/logs/x.zip")
        );
        assert_eq!(
            outside_container(Path::new("/home/p/run/host/x.zip")),
            PathBuf::from("/home/p/run/host/x.zip")
        );
        assert_eq!(
            outside_container(Path::new("/run/hostile/x.zip")),
            PathBuf::from("/run/hostile/x.zip")
        );
    }

    #[cfg(unix)]
    #[test]
    fn resolves_symlinks_to_the_real_file() {
        let root = std::env::temp_dir().join(format!("swofty-reveal-{}", std::process::id()));
        std::fs::remove_dir_all(&root).ok();
        std::fs::create_dir_all(root.join("real/logs")).unwrap();
        std::fs::write(root.join("real/logs/x.zip"), "zip").unwrap();
        std::os::unix::fs::symlink(root.join("real"), root.join("link")).unwrap();
        assert_eq!(
            real_path(&root.join("link/logs/x.zip")),
            std::fs::canonicalize(root.join("real/logs/x.zip")).unwrap()
        );
        let missing = root.join("link/logs/gone.zip");
        assert_eq!(real_path(&missing), missing);
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn explorer_selects_the_quoted_path() {
        assert_eq!(
            explorer_arg(Path::new(r"C:\Users\Naël Smith\AppData\Local\Swofty\logs\x.zip")),
            r#"/select,"C:\Users\Naël Smith\AppData\Local\Swofty\logs\x.zip""#
        );
        assert_eq!(
            explorer_arg(Path::new("D:/Swofty data/logs/x.zip")),
            r#"/select,"D:\Swofty data\logs\x.zip""#
        );
    }

    #[test]
    fn asks_the_file_manager_to_show_the_item() {
        let [gdbus, dbus_send] = show_items("file:///home/p/x.zip");
        let args =
            |c: &Command| -> Vec<String> { c.get_args().map(|a| a.to_string_lossy().into_owned()).collect() };
        assert_eq!(gdbus.get_program(), "gdbus");
        assert!(args(&gdbus).ends_with(&[
            "--method".into(),
            "org.freedesktop.FileManager1.ShowItems".into(),
            "['file:///home/p/x.zip']".into(),
            "''".into(),
        ]));
        assert_eq!(dbus_send.get_program(), "dbus-send");
        assert!(args(&dbus_send).ends_with(&[
            "org.freedesktop.FileManager1.ShowItems".into(),
            "array:string:file:///home/p/x.zip".into(),
            "string:".into(),
        ]));
        for c in [&gdbus, &dbus_send] {
            assert!(
                c.get_envs().any(|(k, v)| k == "LD_PRELOAD" && v.is_none()),
                "the Steam overlay stays out of the file manager"
            );
        }
    }
}
