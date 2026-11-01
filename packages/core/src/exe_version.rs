use std::collections::HashMap;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;

type Stamp = (u64, Option<SystemTime>);
type Seen = HashMap<PathBuf, (Stamp, Option<String>)>;

static SEEN: Mutex<Option<Seen>> = Mutex::new(None);

/// The game's FileVersion from its PE VERSIONINFO resource, as `major.minor.build.revision`.
/// Reads the headers and the resource section only, and remembers the answer until the
/// file's size or modification time changes.
pub fn read(exe: &Path) -> Option<String> {
    let meta = std::fs::metadata(exe).ok()?;
    let stamp = (meta.len(), meta.modified().ok());
    let mut seen = SEEN.lock().unwrap_or_else(|e| e.into_inner());
    if let Some((s, version)) = seen.as_ref().and_then(|m| m.get(exe))
        && *s == stamp
    {
        return version.clone();
    }
    let version = std::fs::File::open(exe).ok().and_then(|f| {
        swofty_formats::pe::file_version(&mut BufReader::new(f))
            .ok()
            .flatten()
    });
    seen.get_or_insert_with(HashMap::new)
        .insert(exe.to_path_buf(), (stamp, version.clone()));
    version
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exe(version: (u32, u32), pad: usize) -> Vec<u8> {
        let mut out = vec![0; 0x200];
        out[..2].copy_from_slice(b"MZ");
        out[0x3c..0x40].copy_from_slice(&0x80u32.to_le_bytes());
        out[0x80..0x84].copy_from_slice(b"PE\0\0");
        out[0x86..0x88].copy_from_slice(&1u16.to_le_bytes());
        out[0x94..0x96].copy_from_slice(&0xf0u16.to_le_bytes());
        out[0x98..0x9a].copy_from_slice(&0x20bu16.to_le_bytes());
        let dir = 0x98 + 112 + 2 * 8;
        out[dir..dir + 4].copy_from_slice(&0x1000u32.to_le_bytes());
        out[dir + 4..dir + 8].copy_from_slice(&0x100u32.to_le_bytes());
        let section = 0x98 + 0xf0;
        out[section + 8..section + 12].copy_from_slice(&0x100u32.to_le_bytes());
        out[section + 12..section + 16].copy_from_slice(&0x1000u32.to_le_bytes());
        out[section + 16..section + 20].copy_from_slice(&0x100u32.to_le_bytes());
        out[section + 20..section + 24].copy_from_slice(&0x200u32.to_le_bytes());
        let mut res = vec![0; 0x100];
        res[0x20..0x24].copy_from_slice(&0xFEEF_04BDu32.to_le_bytes());
        res[0x28..0x2c].copy_from_slice(&version.0.to_le_bytes());
        res[0x2c..0x30].copy_from_slice(&version.1.to_le_bytes());
        out.extend(res);
        out.extend(std::iter::repeat_n(0, pad));
        out
    }

    fn dir(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("swofty-exever-{name}-{}", std::process::id()));
        std::fs::remove_dir_all(&d).ok();
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn a_non_pe_file_has_no_version() {
        let dir = dir("none");
        let f = dir.join("not.exe");
        std::fs::write(&f, b"hello").unwrap();
        assert_eq!(read(&f), None);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn reads_the_version_again_when_the_exe_changes() {
        let dir = dir("change");
        let f = dir.join("F1_25.exe");
        std::fs::write(&f, exe((1 << 16, (141 << 16) | 7865), 0)).unwrap();
        assert_eq!(read(&f).as_deref(), Some("1.0.141.7865"));
        assert_eq!(read(&f).as_deref(), Some("1.0.141.7865"));
        std::fs::write(&f, exe((1 << 16, (142 << 16) | 1), 16)).unwrap();
        assert_eq!(read(&f).as_deref(), Some("1.0.142.1"));
        std::fs::remove_dir_all(&dir).ok();
    }
}
