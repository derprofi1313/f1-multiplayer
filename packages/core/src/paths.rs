use std::path::PathBuf;

/// Overrides the data folder.
pub const DATA_DIR_ENV: &str = "F1MP_DATA_DIR";

pub fn data_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os(DATA_DIR_ENV) {
        return PathBuf::from(dir);
    }
    platform_data_dir()
}

#[cfg(windows)]
pub fn platform_data_dir() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("F1Multiplayer")
}

#[cfg(not(windows))]
pub fn platform_data_dir() -> PathBuf {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let flatpak = home.join(".var/app/com.valvesoftware.Steam/data");
    if flatpak.is_dir() && !home.join(".local/share/Steam").exists() {
        return flatpak.join("f1-multiplayer");
    }
    std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".local/share"))
        .join("f1-multiplayer")
}

pub fn game_folder_view(path: &std::path::Path) -> String {
    if cfg!(windows) {
        return path.display().to_string();
    }
    format!("Z:{}", path.display().to_string().replace('/', "\\"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proton_view_uses_z_drive() {
        if cfg!(windows) {
            return;
        }
        assert_eq!(
            game_folder_view(std::path::Path::new("/home/a/b")),
            "Z:\\home\\a\\b"
        );
    }
}
