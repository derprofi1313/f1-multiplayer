//! The volumes an install writes to: which one a folder is on, and how much
//! room it has left.

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Volume {
    /// Tells two volumes apart.
    pub id: String,
    /// The name players know it by: `C:` on Windows, its mount point elsewhere.
    pub name: String,
}

/// The volume `path` is on, measured at its nearest existing folder.
pub fn volume(path: &Path) -> Option<Volume> {
    sys::volume(existing(path)?)
}

/// The bytes this process may still write on the volume `path` is on.
pub fn free_space(path: &Path) -> Option<u64> {
    sys::free_space(existing(path)?)
}

/// Whether `e` says the volume is full.
pub fn is_full(e: &std::io::Error) -> bool {
    const ERROR_HANDLE_DISK_FULL: i32 = 39;
    const ERROR_DISK_FULL: i32 = 112;
    e.kind() == std::io::ErrorKind::StorageFull
        || (cfg!(windows) && matches!(e.raw_os_error(), Some(ERROR_HANDLE_DISK_FULL | ERROR_DISK_FULL)))
}

/// `bytes` in the binary gigabytes Windows shows, to one decimal.
pub fn gb(bytes: u64) -> String {
    format!("{:.1}", bytes as f64 / (1u64 << 30) as f64)
}

fn existing(path: &Path) -> Option<&Path> {
    path.ancestors().find(|a| !a.as_os_str().is_empty() && a.exists())
}

#[cfg(windows)]
mod sys {
    use std::os::windows::ffi::OsStrExt;
    use std::path::Path;

    use super::Volume;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetDiskFreeSpaceExW(dir: *const u16, available: *mut u64, total: *mut u64, free: *mut u64) -> i32;
        fn GetVolumePathNameW(path: *const u16, out: *mut u16, len: u32) -> i32;
    }

    fn wide(path: &Path) -> Vec<u16> {
        path.as_os_str().encode_wide().chain(std::iter::once(0)).collect()
    }

    pub fn free_space(path: &Path) -> Option<u64> {
        let path = wide(path);
        let mut available = 0u64;
        // SAFETY: the path is NUL-terminated and outlives the call; the
        // outputs left null are optional.
        let ok = unsafe {
            GetDiskFreeSpaceExW(
                path.as_ptr(),
                &mut available,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        } != 0;
        ok.then_some(available)
    }

    pub fn volume(path: &Path) -> Option<Volume> {
        let path = wide(path);
        let mut out = vec![0u16; 1024];
        // SAFETY: both buffers outlive the call, and `out`'s length is passed.
        if unsafe { GetVolumePathNameW(path.as_ptr(), out.as_mut_ptr(), out.len() as u32) } == 0 {
            return None;
        }
        let end = out.iter().position(|&c| c == 0).unwrap_or(out.len());
        let root = String::from_utf16_lossy(&out[..end]);
        Some(Volume {
            id: root.to_lowercase(),
            name: root.trim_end_matches('\\').to_owned(),
        })
    }
}

#[cfg(target_os = "linux")]
mod sys {
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::fs::MetadataExt;
    use std::path::Path;

    use super::Volume;

    pub fn free_space(path: &Path) -> Option<u64> {
        let path = std::ffi::CString::new(path.as_os_str().as_bytes()).ok()?;
        // SAFETY: statvfs fills the zeroed struct, and the path is
        // NUL-terminated and outlives the call.
        let mut s: libc::statvfs = unsafe { std::mem::zeroed() };
        (unsafe { libc::statvfs(path.as_ptr(), &mut s) } == 0).then(|| s.f_bavail.saturating_mul(s.f_frsize))
    }

    pub fn volume(path: &Path) -> Option<Volume> {
        let dev = std::fs::metadata(path).ok()?.dev();
        let mount = path
            .ancestors()
            .take_while(|a| std::fs::metadata(a).is_ok_and(|m| m.dev() == dev))
            .last()?;
        Some(Volume {
            id: dev.to_string(),
            name: mount.display().to_string(),
        })
    }
}

#[cfg(not(any(windows, target_os = "linux")))]
mod sys {
    use std::path::Path;

    use super::Volume;

    pub fn free_space(_: &Path) -> Option<u64> {
        None
    }

    pub fn volume(_: &Path) -> Option<Volume> {
        None
    }
}

/// The folders an install writes to, each with the volume it is on.
pub(crate) fn volumes<'a>(dirs: impl IntoIterator<Item = (&'a Path, u64)>) -> Vec<(PathBuf, Volume, u64)> {
    let mut out: Vec<(PathBuf, Volume, u64)> = Vec::new();
    for (dir, bytes) in dirs {
        let Some(v) = volume(dir) else { continue };
        match out.iter_mut().find(|(_, o, _)| o.id == v.id) {
            Some((_, _, total)) => *total += bytes,
            None => out.push((dir.to_path_buf(), v, bytes)),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tells_a_full_disk_from_other_errors() {
        assert!(is_full(&std::io::Error::from(std::io::ErrorKind::StorageFull)));
        assert!(!is_full(&std::io::Error::from(
            std::io::ErrorKind::PermissionDenied
        )));
        #[cfg(unix)]
        assert!(is_full(&std::io::Error::from_raw_os_error(28)));
        #[cfg(windows)]
        assert!(
            is_full(&std::io::Error::from_raw_os_error(112))
                && is_full(&std::io::Error::from_raw_os_error(39))
        );
    }

    #[cfg(any(windows, target_os = "linux"))]
    #[test]
    fn measures_the_volume_of_a_folder_not_made_yet() {
        let dir = std::env::temp_dir();
        let v = volume(&dir).unwrap();
        assert_eq!(volume(&dir.join("not/made/yet")), Some(v.clone()));
        assert!(free_space(&dir.join("not/made/yet")).is_some());
        let both = volumes([(dir.as_path(), 5), (dir.join("x").as_path(), 7)]);
        assert_eq!(both.len(), 1);
        assert_eq!((&both[0].1, both[0].2), (&v, 12));
    }

    #[test]
    fn shows_gigabytes_as_windows_does() {
        assert_eq!(gb(3 << 30), "3.0");
        assert_eq!(gb(1288490189), "1.2");
    }
}
