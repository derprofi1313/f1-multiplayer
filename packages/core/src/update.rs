use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::launcher::Progress;
use crate::{Error, Result, io_err};

/// F1 Multiplayer's own releases. Swofty's releases must never be offered as an update.
pub const LATEST_URL: &str = "https://api.github.com/repos/derprofi1313/f1-multiplayer/releases/latest";
pub const RELEASES_URL: &str = "https://github.com/derprofi1313/f1-multiplayer/releases";
pub const SUMS_FILE: &str = "SHA256SUMS";
pub const PRETEND_ENV: &str = "SWOFTY_UPDATE_PRETEND_VERSION";
pub const FIXTURE_ENV: &str = "SWOFTY_TEST_GITHUB";
pub const CHECK_TIMEOUT: Duration = Duration::from_secs(10);
pub const STAGING_DIR: &str = ".update";
/// Whether the launcher looks for releases of itself. Off until F1 Multiplayer publishes its own:
/// [`LATEST_URL`] still names Swofty's, which must never replace this app.
pub const SELF_UPDATE: bool = false;
/// The launcher's executable, without `.exe`.
pub const LAUNCHER_BIN: &str = "f1-multiplayer";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdateConfig {
    #[serde(default = "yes")]
    pub check: bool,
}

fn yes() -> bool {
    true
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self { check: true }
    }
}

impl UpdateConfig {
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }
}

pub fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

pub fn parse_version(tag: &str) -> Option<Version> {
    Version::parse(tag.trim().trim_start_matches('v')).ok()
}

pub fn current_version() -> Version {
    std::env::var(PRETEND_ENV)
        .ok()
        .and_then(|v| parse_version(&v))
        .unwrap_or_else(|| Version::parse(env!("CARGO_PKG_VERSION")).expect("the package version is semver"))
}

pub fn notes_url(tag: &str) -> String {
    format!("{RELEASES_URL}/tag/{tag}")
}

pub fn asset_name(tag: &str, os: &str, arch: &str) -> Option<String> {
    match (os, arch) {
        ("linux", "x86_64") => Some(format!("swofty-launcher-{tag}-linux-x86_64.tar.gz")),
        ("windows", "x86_64") => Some(format!("swofty-launcher-{tag}-windows-x86_64.zip")),
        _ => None,
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Release {
    pub tag_name: String,
    #[serde(default)]
    pub html_url: String,
    #[serde(default)]
    pub assets: Vec<Asset>,
    #[serde(default)]
    pub draft: bool,
    #[serde(default)]
    pub prerelease: bool,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Asset {
    pub name: String,
    pub size: u64,
    pub browser_download_url: String,
    #[serde(default)]
    pub digest: Option<String>,
}

impl Release {
    pub fn version(&self) -> Option<Version> {
        parse_version(&self.tag_name)
    }

    pub fn notes_url(&self) -> String {
        if self.html_url.is_empty() {
            notes_url(&self.tag_name)
        } else {
            self.html_url.clone()
        }
    }

    pub fn asset(&self, os: &str, arch: &str) -> Option<&Asset> {
        let name = asset_name(&self.tag_name, os, arch)?;
        self.assets.iter().find(|a| a.name == name)
    }

    pub fn platform_asset(&self) -> Result<&Asset> {
        let (os, arch) = (std::env::consts::OS, std::env::consts::ARCH);
        self.asset(os, arch).ok_or_else(|| {
            Error::Other(format!(
                "release {} has no download for {os} {arch}",
                self.tag_name
            ))
        })
    }

    pub fn sums(&self) -> Option<&Asset> {
        self.assets.iter().find(|a| a.name == SUMS_FILE)
    }
}

pub fn parse_sums(text: &str) -> BTreeMap<String, String> {
    text.lines()
        .filter_map(|line| {
            let (hash, name) = line.trim().split_once(char::is_whitespace)?;
            let name = name.trim_start().trim_start_matches('*');
            (hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit()) && !name.is_empty())
                .then(|| (name.to_owned(), hash.to_ascii_lowercase()))
        })
        .collect()
}

pub fn check_download(
    asset: &Asset,
    len: u64,
    sha256: &str,
    sums: Option<&BTreeMap<String, String>>,
) -> Result<()> {
    if len != asset.size {
        return Err(Error::Other(format!(
            "{}: downloaded {len} bytes, the release lists {}",
            asset.name, asset.size
        )));
    }
    let expected = match sums {
        Some(sums) => Some(
            sums.get(&asset.name)
                .cloned()
                .ok_or_else(|| Error::Other(format!("{SUMS_FILE} has no entry for {}", asset.name)))?,
        ),
        None => asset
            .digest
            .as_deref()
            .and_then(|d| d.strip_prefix("sha256:"))
            .map(str::to_ascii_lowercase),
    };
    match expected {
        Some(e) if e != sha256 => Err(Error::Other(format!(
            "{}: SHA-256 is {sha256}, expected {e}",
            asset.name
        ))),
        _ => Ok(()),
    }
}

fn user_agent() -> String {
    format!(
        "swofty-launcher/{} (+https://github.com/Swofty-Developments/SwoftyLauncher)",
        env!("CARGO_PKG_VERSION")
    )
}

pub fn endpoint(url: &str) -> String {
    #[cfg(debug_assertions)]
    if let Some(base) = std::env::var(FIXTURE_ENV).ok().filter(|b| !b.is_empty()) {
        let base = base.trim_end_matches('/');
        for host in ["https://api.github.com", "https://github.com"] {
            if let Some(rest) = url.strip_prefix(host) {
                return format!("{base}{rest}");
            }
        }
    }
    url.to_owned()
}

#[derive(Debug, Clone, PartialEq)]
pub enum Failure {
    Wait(u64, String),
    Failed(String),
}

impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Wait(_, text) | Self::Failed(text) => f.write_str(text),
        }
    }
}

impl From<Failure> for Error {
    fn from(f: Failure) -> Self {
        Error::Other(f.to_string())
    }
}

pub fn in_minutes(secs: u64) -> String {
    match secs.div_ceil(60) {
        0 | 1 => "a minute".into(),
        n => format!("{n} minutes"),
    }
}

pub fn refusal(
    url: &str,
    status: u16,
    retry_after: Option<&str>,
    remaining: Option<&str>,
    reset: Option<&str>,
    now: u64,
) -> Failure {
    let number = |v: Option<&str>| v.and_then(|v| v.trim().parse::<u64>().ok());
    let wait = number(retry_after).or_else(|| {
        (remaining.map(str::trim) == Some("0"))
            .then(|| number(reset).map(|at| at.saturating_sub(now)))
            .flatten()
    });
    match wait {
        Some(secs) => Failure::Wait(
            secs.max(1),
            format!(
                "{url}: GitHub's rate limit is reached; try again in {}",
                in_minutes(secs)
            ),
        ),
        None if status == 429 => Failure::Wait(
            60,
            format!("{url}: GitHub's rate limit is reached; try again in a minute"),
        ),
        None => Failure::Failed(format!("{url}: HTTP {status}")),
    }
}

pub fn request(
    url: &str,
    head: bool,
    timeout: Option<Duration>,
) -> std::result::Result<ureq::http::Response<ureq::Body>, Failure> {
    let target = endpoint(url);
    let builder = if head {
        ureq::head(&target)
    } else {
        ureq::get(&target)
    }
    .header("User-Agent", user_agent())
    .header("Accept", "application/vnd.github+json, application/octet-stream")
    .config()
    .timeout_global(timeout)
    .timeout_connect(Some(Duration::from_secs(15)))
    .timeout_recv_response(Some(Duration::from_secs(30)))
    .http_status_as_error(false);
    let config = if head { builder.max_redirects(0) } else { builder };
    let response = config
        .build()
        .call()
        .map_err(|e| Failure::Failed(format!("{url}: {e}")))?;
    let status = response.status().as_u16();
    if status < 400 {
        return Ok(response);
    }
    let header = |name: &str| response.headers().get(name).and_then(|v| v.to_str().ok());
    Err(refusal(
        url,
        status,
        header("retry-after"),
        header("x-ratelimit-remaining"),
        header("x-ratelimit-reset"),
        now_secs(),
    ))
}

pub fn get(url: &str, timeout: Option<Duration>) -> Result<ureq::Body> {
    Ok(request(url, false, timeout)?.into_body())
}

pub fn get_text(url: &str, timeout: Duration) -> Result<String> {
    get(url, Some(timeout))?
        .read_to_string()
        .map_err(|e| Error::Other(format!("{url}: {e}")))
}

pub fn fetch_json<T: serde::de::DeserializeOwned>(
    url: &str,
    timeout: Duration,
) -> std::result::Result<T, Failure> {
    let text = request(url, false, Some(timeout))?
        .into_body()
        .read_to_string()
        .map_err(|e| Failure::Failed(format!("{url}: {e}")))?;
    serde_json::from_str(&text).map_err(|e| Failure::Failed(format!("{url}: {e}")))
}

pub fn fetch_release(url: &str, timeout: Duration) -> std::result::Result<Release, Failure> {
    fetch_json(url, timeout)
}

pub fn fetch_latest(timeout: Duration) -> Result<Release> {
    Ok(fetch_release(LATEST_URL, timeout)?)
}

pub fn verifiable(release: &Release, asset: &Asset) -> bool {
    release.sums().is_some() || asset.digest.as_deref().is_some_and(|d| d.starts_with("sha256:"))
}

pub fn download(asset: &Asset, to: &Path, progress: &dyn Fn(u64)) -> Result<(u64, String)> {
    let url = &asset.browser_download_url;
    let mut reader = get(url, Some(Duration::from_secs(30 * 60)))?.into_reader();
    let mut file = std::fs::File::create(to).map_err(io_err(to))?;
    let mut hash = Sha256::new();
    let mut buf = vec![0; 64 * 1024];
    let mut len = 0u64;
    loop {
        let n = reader
            .read(&mut buf)
            .map_err(|e| Error::Other(format!("{url}: {e}")))?;
        if n == 0 {
            break;
        }
        len += n as u64;
        if len > asset.size {
            return Err(Error::Other(format!(
                "{}: larger than the {} bytes the release lists",
                asset.name, asset.size
            )));
        }
        hash.update(&buf[..n]);
        file.write_all(&buf[..n]).map_err(io_err(to))?;
        progress(len);
    }
    file.sync_all().map_err(io_err(to))?;
    let digest = hash.finalize().iter().map(|b| format!("{b:02x}")).collect();
    Ok((len, digest))
}

fn safe_path(name: &str) -> Option<PathBuf> {
    let mut out = PathBuf::new();
    for part in name.split(['/', '\\']) {
        match part {
            "" | "." => {}
            ".." => return None,
            p if p.contains(':') => return None,
            p => out.push(p),
        }
    }
    Some(out)
}

pub fn extract(archive: &Path, to: &Path) -> Result<()> {
    std::fs::create_dir_all(to).map_err(io_err(to))?;
    let file = std::fs::File::open(archive).map_err(io_err(archive))?;
    let name = archive.file_name().unwrap_or_default().to_string_lossy();
    if name.ends_with(".zip") {
        let mut zip = zip::ZipArchive::new(file)?;
        for i in 0..zip.len() {
            let mut entry = zip.by_index(i)?;
            let raw = entry.name()?.into_owned();
            let Some(rel) = safe_path(&raw) else {
                return Err(Error::Other(format!("{name}: unsafe path {raw}")));
            };
            if rel.as_os_str().is_empty() {
                continue;
            }
            let path = to.join(rel);
            if entry.is_dir() {
                std::fs::create_dir_all(&path).map_err(io_err(&path))?;
                continue;
            }
            write_entry(&mut entry, &path, None)?;
        }
        return Ok(());
    }
    if !name.ends_with(".tar.gz") {
        return Err(Error::Other(format!("{name}: not a .zip or .tar.gz archive")));
    }
    let mut tar = tar::Archive::new(flate2::read::GzDecoder::new(file));
    for entry in tar.entries().map_err(io_err(archive))? {
        let mut entry = entry.map_err(io_err(archive))?;
        let raw = entry
            .path()
            .map_err(io_err(archive))?
            .to_string_lossy()
            .into_owned();
        let Some(rel) = safe_path(&raw) else {
            return Err(Error::Other(format!("{name}: unsafe path {raw}")));
        };
        if rel.as_os_str().is_empty() {
            continue;
        }
        let path = to.join(rel);
        let kind = entry.header().entry_type();
        if kind.is_dir() {
            std::fs::create_dir_all(&path).map_err(io_err(&path))?;
        } else if kind.is_file() {
            let mode = entry.header().mode().ok();
            write_entry(&mut entry, &path, mode)?;
        }
    }
    Ok(())
}

fn write_entry(entry: &mut impl Read, path: &Path, mode: Option<u32>) -> Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(io_err(dir))?;
    }
    let mut out = std::fs::File::create(path).map_err(io_err(path))?;
    std::io::copy(entry, &mut out).map_err(io_err(path))?;
    #[cfg(unix)]
    if let Some(mode) = mode {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode & 0o755))
            .map_err(io_err(path))?;
    }
    #[cfg(not(unix))]
    let _ = mode;
    Ok(())
}

pub fn exe(name: &str) -> String {
    format!("{name}{}", std::env::consts::EXE_SUFFIX)
}

pub fn release_root(extracted: &Path) -> Result<PathBuf> {
    let complete = |dir: &Path| {
        dir.join("bin").join(exe(LAUNCHER_BIN)).is_file()
            && dir.join("bin").join(exe("swofty")).is_file()
            && crate::builtin::is_loader(&dir.join("loader"))
    };
    if complete(extracted) {
        return Ok(extracted.to_path_buf());
    }
    let dirs: Vec<PathBuf> = std::fs::read_dir(extracted)
        .map_err(io_err(extracted))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_dir())
        .collect();
    match dirs.as_slice() {
        [one] if complete(one) => Ok(one.clone()),
        _ => Err(Error::Other(
            "the archive does not hold bin/ with the launcher and the command line, and loader/".into(),
        )),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Install {
    pub root: PathBuf,
}

impl Install {
    pub fn current() -> std::result::Result<Self, String> {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        Self::locate(&exe)
    }

    pub fn locate(exe: &Path) -> std::result::Result<Self, String> {
        let bin = exe
            .parent()
            .filter(|b| b.file_name().is_some_and(|n| n == "bin"))
            .ok_or("this copy does not run from a release folder")?;
        let root = bin
            .parent()
            .ok_or("this copy does not run from a release folder")?;
        if root.starts_with("/nix/store") {
            return Err("this copy is installed with Nix; update it with Nix".into());
        }
        if !crate::builtin::is_loader(&root.join("loader")) {
            return Err("this copy has no loader folder next to bin".into());
        }
        for dir in [root, bin, &root.join("loader")] {
            let probe = dir.join(format!(".update-probe-{}", std::process::id()));
            std::fs::write(&probe, b"").map_err(|_| format!("no write access to {}", dir.display()))?;
            std::fs::remove_file(&probe).ok();
        }
        Ok(Self {
            root: root.to_path_buf(),
        })
    }

    pub fn launcher(&self) -> PathBuf {
        self.root.join("bin").join(exe(LAUNCHER_BIN))
    }

    pub fn staging(&self) -> PathBuf {
        self.root.join(STAGING_DIR)
    }

    pub fn cleanup(&self) -> bool {
        let dir = self.staging();
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for work in entries.flatten().map(|e| e.path()) {
                if work.join(APPLYING).is_file() {
                    recover(&work, &self.root);
                }
            }
        }
        std::fs::remove_dir_all(&dir).ok();
        !dir.exists()
    }

    pub fn update(&self, release: &Release, progress: &dyn Progress) -> Result<Version> {
        let version = release
            .version()
            .ok_or_else(|| Error::Other(format!("{} is not a version", release.tag_name)))?;
        if crate::process::game_running() {
            return Err(Error::Other("close F1 25 before updating".into()));
        }
        let asset = release.platform_asset()?;
        self.cleanup();
        let work = self
            .staging()
            .join(format!("{}-{}", std::process::id(), now_secs()));
        std::fs::create_dir_all(&work).map_err(io_err(&work))?;
        let result = self.stage_and_apply(release, asset, &work, progress);
        self.cleanup();
        result.map(|_| version)
    }

    fn stage_and_apply(
        &self,
        release: &Release,
        asset: &Asset,
        staging: &Path,
        progress: &dyn Progress,
    ) -> Result<()> {
        let sums = match release.sums() {
            Some(s) => Some(parse_sums(&get_text(
                &s.browser_download_url,
                Duration::from_secs(60),
            )?)),
            None => None,
        };
        let archive = staging.join(&asset.name);
        let total = asset.size.max(1) as f32;
        let text = format!("Downloading {}", release.tag_name);
        let (len, sha256) = download(asset, &archive, &|n| {
            progress.report(0.9 * n as f32 / total, &text);
        })?;
        check_download(asset, len, &sha256, sums.as_ref())?;
        progress.report(0.9, "Extracting");
        let extracted = staging.join("new");
        extract(&archive, &extracted)?;
        let new = release_root(&extracted)?;
        if crate::process::game_running() {
            return Err(Error::Other("close F1 25 before updating".into()));
        }
        progress.report(0.95, "Replacing files");
        apply(&new, &self.root, &staging.join("old"))
    }
}

struct Swap {
    target: PathBuf,
    saved: Option<PathBuf>,
}

fn swap(new: Option<&Path>, target: &Path, saved: &Path) -> std::io::Result<Swap> {
    let had = target.symlink_metadata().is_ok();
    if had {
        std::fs::create_dir_all(saved.parent().expect("under the staging folder"))?;
        if cfg!(windows) || new.is_none() {
            std::fs::rename(target, saved)?;
        } else if std::fs::hard_link(target, saved).is_err() {
            std::fs::copy(target, saved)?;
        }
    }
    let swap = Swap {
        target: target.to_path_buf(),
        saved: had.then(|| saved.to_path_buf()),
    };
    let Some(new) = new else {
        return Ok(swap);
    };
    let moved = target
        .parent()
        .map_or(Ok(()), std::fs::create_dir_all)
        .and_then(|_| std::fs::rename(new, target));
    if let Err(e) = moved {
        if had && cfg!(windows) {
            std::fs::rename(saved, target).ok();
        }
        return Err(e);
    }
    Ok(swap)
}

const APPLYING: &str = "applying";

fn recover(work: &Path, root: &Path) {
    let backup = work.join("old");
    if let Ok(files) = crate::builtin::loader_files(&backup) {
        for (rel, saved) in files {
            restore(&saved, &root.join(&rel), work);
        }
    }
    let added = std::fs::read_to_string(work.join(APPLYING)).unwrap_or_default();
    for rel in added.lines().filter(|l| !l.is_empty()) {
        std::fs::remove_file(root.join(rel)).ok();
    }
}

fn restore(saved: &Path, target: &Path, work: &Path) {
    if let Some(dir) = target.parent() {
        std::fs::create_dir_all(dir).ok();
    }
    if std::fs::rename(saved, target).is_ok() {
        return;
    }
    let aside = work.with_file_name(format!("aside-{}-{}", std::process::id(), now_secs()));
    let name = target.file_name().map(|n| aside.join(n));
    if let Some(name) = name
        && std::fs::create_dir_all(&aside).is_ok()
        && std::fs::rename(target, &name).is_ok()
    {
        std::fs::rename(saved, target).ok();
    }
}

fn roll_back(done: &[Swap]) {
    for s in done.iter().rev() {
        match &s.saved {
            Some(saved) => std::fs::rename(saved, &s.target).ok(),
            None => std::fs::remove_file(&s.target).ok(),
        };
    }
}

pub fn apply(new: &Path, root: &Path, backup: &Path) -> Result<()> {
    let files = crate::builtin::loader_files(new)?;
    let mut stale: Vec<String> = Vec::new();
    for sub in ["loader", "addons", crate::postfx::DXC_DIR] {
        let dir = root.join(sub);
        if dir.is_dir() {
            stale.extend(
                crate::builtin::loader_files(&dir)?
                    .into_keys()
                    .map(|rel| format!("{sub}/{rel}"))
                    .filter(|rel| !files.contains_key(rel)),
            );
        }
    }
    let steps = files
        .iter()
        .map(|(rel, src)| (rel.as_str(), Some(src.as_path())))
        .chain(stale.iter().map(|rel| (rel.as_str(), None)));
    let added: String = files
        .keys()
        .filter(|rel| root.join(rel).symlink_metadata().is_err())
        .map(|rel| format!("{rel}\n"))
        .collect();
    let journal = backup
        .parent()
        .map_or_else(|| backup.join(APPLYING), |w| w.join(APPLYING));
    std::fs::create_dir_all(backup).map_err(io_err(backup))?;
    std::fs::write(&journal, added).map_err(io_err(&journal))?;
    let mut done = Vec::new();
    for (rel, src) in steps {
        let target = root.join(rel);
        match swap(src, &target, &backup.join(rel)) {
            Ok(s) => done.push(s),
            Err(e) => {
                roll_back(&done);
                std::fs::remove_file(&journal).ok();
                return Err(Error::Io {
                    path: target,
                    source: e,
                });
            }
        }
    }
    std::fs::remove_file(&journal).map_err(io_err(&journal))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_launcher_only_looks_at_its_own_releases() {
        for url in [LATEST_URL, RELEASES_URL] {
            assert!(url.contains("derprofi1313/f1-multiplayer"), "{url}");
            assert!(!url.contains("Swofty-Developments"), "{url}");
        }
    }

    fn v(s: &str) -> Version {
        Version::parse(s).unwrap()
    }

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("swofty-update-{name}-{}", std::process::id()));
        std::fs::remove_dir_all(&dir).ok();
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write(path: &Path, text: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }

    fn read(path: &Path) -> String {
        std::fs::read_to_string(path).unwrap()
    }

    fn release_tree(root: &Path, tag: &str) {
        write(
            &root.join("bin").join(exe(LAUNCHER_BIN)),
            &format!("launcher {tag}"),
        );
        write(&root.join("bin").join(exe("swofty")), &format!("cli {tag}"));
        write(
            &root.join("loader").join(crate::install::PROXY),
            &format!("proxy {tag}"),
        );
        write(
            &root.join("loader").join(crate::builtin::CORE_DLL),
            &format!("core {tag}"),
        );
    }

    fn asset(name: &str, size: u64, digest: Option<&str>) -> Asset {
        Asset {
            name: name.into(),
            size,
            browser_download_url: format!("https://example.invalid/{name}"),
            digest: digest.map(Into::into),
        }
    }

    #[test]
    fn compares_versions_by_semver() {
        assert_eq!(parse_version("v0.2.1"), Some(v("0.2.1")));
        assert_eq!(parse_version("0.10.0"), Some(v("0.10.0")));
        assert_eq!(parse_version("latest"), None);
        assert!(parse_version("v0.10.0").unwrap() > v("0.9.9"));
        assert!(parse_version("v0.2.1-rc.1").unwrap() < v("0.2.1"));
    }

    #[test]
    fn old_update_settings_still_load() {
        let json = serde_json::to_value(UpdateConfig::default()).unwrap();
        assert_eq!(json, serde_json::json!({ "check": true }));
        let off: UpdateConfig = serde_json::from_str(r#"{"check":false}"#).unwrap();
        assert!(!off.check);
        let old = r#"{
            "data_version": 3,
            "updates": { "check": true, "skip": "1.0.7", "checked_at": 1790000000, "latest": "v1.0.7" },
            "managed": { "checked_at": 1790000000 },
            "sidebar_list": true
        }"#;
        let config: crate::session::Config = serde_json::from_str(old).unwrap();
        assert!(config.updates.is_default() && config.sidebar_list);
        assert_eq!(config.data_version, 3);
        let saved = serde_json::to_value(&config).unwrap();
        assert!(
            saved.get("updates").is_none() && saved.get("managed").is_none(),
            "{saved}"
        );
    }

    #[test]
    fn reads_rate_limits_from_the_response() {
        let url = "https://api.github.com/x";
        assert!(matches!(
            refusal(url, 403, Some("120"), None, None, 1000),
            Failure::Wait(120, _)
        ));
        assert!(matches!(
            refusal(url, 403, None, Some("0"), Some("1600"), 1000),
            Failure::Wait(600, _)
        ));
        assert!(matches!(
            refusal(url, 403, None, Some("0"), Some("900"), 1000),
            Failure::Wait(1, _)
        ));
        assert!(matches!(
            refusal(url, 429, None, None, None, 1000),
            Failure::Wait(60, _)
        ));
        assert!(matches!(
            refusal(url, 403, None, Some("12"), Some("1600"), 1000),
            Failure::Failed(_)
        ));
        assert_eq!(
            refusal(url, 404, None, None, None, 0),
            Failure::Failed(format!("{url}: HTTP 404"))
        );
        let text = refusal(url, 403, None, Some("0"), Some("1600"), 1000).to_string();
        assert!(
            text.contains("rate limit") && text.contains("10 minutes"),
            "{text}"
        );
    }

    #[test]
    fn a_release_offers_only_what_it_can_verify() {
        let name = "swofty-launcher-v0.2.1-linux-x86_64.tar.gz";
        let mut release = Release {
            tag_name: "v0.2.1".into(),
            html_url: String::new(),
            assets: vec![asset(name, 1, None)],
            ..Default::default()
        };
        assert!(!verifiable(&release, &release.assets[0]));
        release.assets[0].digest = Some(format!("sha256:{}", "a".repeat(64)));
        assert!(verifiable(&release, &release.assets[0]));
        release.assets[0].digest = None;
        release.assets.push(asset(SUMS_FILE, 3, None));
        assert!(verifiable(&release, &release.assets[0]));
    }

    #[test]
    fn selects_the_asset_for_each_platform() {
        let tag = "v0.2.1";
        let release = Release {
            tag_name: tag.into(),
            html_url: String::new(),
            assets: vec![
                asset("swofty-launcher-v0.2.1-linux-x86_64.tar.gz", 1, None),
                asset("swofty-launcher-v0.2.1-windows-x86_64.zip", 2, None),
                asset(SUMS_FILE, 3, None),
            ],
            ..Default::default()
        };
        assert_eq!(release.asset("linux", "x86_64").unwrap().size, 1);
        assert_eq!(release.asset("windows", "x86_64").unwrap().size, 2);
        assert!(release.asset("macos", "aarch64").is_none());
        assert!(release.asset("linux", "aarch64").is_none());
        assert_eq!(release.sums().unwrap().size, 3);
        assert_eq!(release.notes_url(), format!("{RELEASES_URL}/tag/{tag}"));
    }

    #[test]
    fn parses_sha256sums() {
        let a = "a".repeat(64);
        let b = "B".repeat(64);
        let sums = parse_sums(&format!(
            "{a}  swofty-launcher-v0.2.1-linux-x86_64.tar.gz\n{b} *swofty-launcher-v0.2.1-windows-x86_64.zip\nnot a line\n\n"
        ));
        assert_eq!(sums.len(), 2);
        assert_eq!(sums["swofty-launcher-v0.2.1-linux-x86_64.tar.gz"], a);
        assert_eq!(sums["swofty-launcher-v0.2.1-windows-x86_64.zip"], "b".repeat(64));
    }

    #[test]
    fn verifies_downloads_against_sums_digest_or_size() {
        let good = "c".repeat(64);
        let bad = "d".repeat(64);
        let a = asset("x.zip", 10, Some(&format!("sha256:{good}")));
        let sums = BTreeMap::from([("x.zip".to_owned(), good.clone())]);
        assert!(check_download(&a, 10, &good, Some(&sums)).is_ok());
        assert!(check_download(&a, 10, &bad, Some(&sums)).is_err());
        assert!(check_download(&a, 9, &good, Some(&sums)).is_err());
        assert!(check_download(&a, 10, &good, Some(&BTreeMap::new())).is_err());
        assert!(check_download(&a, 10, &good, None).is_ok());
        assert!(check_download(&a, 10, &bad, None).is_err());
        let size_only = asset("x.zip", 10, None);
        assert!(check_download(&size_only, 10, &bad, None).is_ok());
        assert!(check_download(&size_only, 11, &bad, None).is_err());
    }

    #[test]
    fn locates_only_release_layouts() {
        let dir = temp("locate");
        release_tree(&dir, "v1");
        let launcher = dir.join("bin").join(exe(LAUNCHER_BIN));
        assert_eq!(Install::locate(&launcher).unwrap().root, dir);
        assert!(Install::locate(&dir.join("target/debug/swofty-launcher")).is_err());
        std::fs::remove_dir_all(dir.join("loader")).unwrap();
        assert!(Install::locate(&launcher).is_err());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn extracts_both_archive_kinds() {
        let dir = temp("extract");
        let tree = dir.join("swofty-launcher-v2-x");
        release_tree(&tree, "v2");
        let tgz = dir.join("a.tar.gz");
        {
            let gz = flate2::write::GzEncoder::new(
                std::fs::File::create(&tgz).unwrap(),
                flate2::Compression::fast(),
            );
            let mut b = tar::Builder::new(gz);
            b.append_dir_all("swofty-launcher-v2-x", &tree).unwrap();
            b.into_inner().unwrap().finish().unwrap();
        }
        let zip_path = dir.join("a.zip");
        {
            let mut z = zip::ZipWriter::new(std::fs::File::create(&zip_path).unwrap());
            for (rel, path) in crate::builtin::loader_files(&tree).unwrap() {
                z.start_file(
                    format!("swofty-launcher-v2-x\\{}", rel.replace('/', "\\")),
                    zip::write::SimpleFileOptions::default(),
                )
                .unwrap();
                z.write_all(&std::fs::read(path).unwrap()).unwrap();
            }
            z.finish().unwrap();
        }
        for archive in [&tgz, &zip_path] {
            let out = dir.join(format!("out-{}", archive.extension().unwrap().to_string_lossy()));
            extract(archive, &out).unwrap();
            let root = release_root(&out).unwrap();
            assert_eq!(root, out.join("swofty-launcher-v2-x"));
            assert_eq!(read(&root.join("bin").join(exe("swofty"))), "cli v2");
        }
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    #[cfg(windows)]
    fn replaces_a_running_program() {
        let dir = temp("running");
        let (root, new) = (dir.join("install"), dir.join("new"));
        release_tree(&root, "v1");
        let running = root.join("bin").join(exe(LAUNCHER_BIN));
        let ping = PathBuf::from(std::env::var_os("SystemRoot").unwrap()).join("System32/PING.EXE");
        std::fs::copy(ping, &running).unwrap();
        let mut child = std::process::Command::new(&running)
            .args(["-n", "30", "127.0.0.1"])
            .stdout(std::process::Stdio::null())
            .spawn()
            .unwrap();
        release_tree(&new, "v2");
        let result = apply(&new, &root, &dir.join("old"));
        child.kill().ok();
        child.wait().ok();
        result.unwrap();
        assert_eq!(read(&running), "launcher v2");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn replaces_the_install_and_mirrors_the_loader() {
        let dir = temp("apply");
        let (root, new) = (dir.join("install"), dir.join("new"));
        release_tree(&root, "v1");
        write(&root.join("loader/stale.dll"), "stale");
        write(&root.join("addons/gone.swoftyaddon"), "gone");
        write(&root.join("addons/kept.swoftyaddon"), "kept v1");
        write(&root.join("dxc/lib/libold.so"), "old dxc");
        write(&root.join("bin/user-note.txt"), "mine");
        release_tree(&new, "v2");
        write(&new.join("dxc/bin").join(exe("dxc")), "dxc v2");
        write(&new.join("share/applications/a.desktop"), "desktop v2");
        write(&new.join("addons/kept.swoftyaddon"), "kept v2");
        apply(&new, &root, &dir.join("old")).unwrap();
        assert_eq!(read(&root.join("bin").join(exe(LAUNCHER_BIN))), "launcher v2");
        assert_eq!(read(&root.join("bin").join(exe("swofty"))), "cli v2");
        assert_eq!(
            read(&root.join("loader").join(crate::builtin::CORE_DLL)),
            "core v2"
        );
        assert_eq!(read(&root.join("share/applications/a.desktop")), "desktop v2");
        assert!(!root.join("loader/stale.dll").exists());
        assert!(!root.join("addons/gone.swoftyaddon").exists());
        assert_eq!(read(&root.join("addons/kept.swoftyaddon")), "kept v2");
        assert_eq!(read(&root.join("dxc/bin").join(exe("dxc"))), "dxc v2");
        assert!(!root.join("dxc/lib/libold.so").exists());
        assert_eq!(read(&root.join("bin/user-note.txt")), "mine");
        assert!(Install::locate(&root.join("bin").join(exe("swofty"))).is_ok());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn an_interrupted_update_is_rolled_back_at_the_next_cleanup() {
        let dir = temp("interrupted");
        let (root, new) = (dir.join("install"), dir.join("new"));
        release_tree(&root, "v1");
        release_tree(&new, "v2");
        write(&new.join("bin/added"), "v2");
        let work = root.join(STAGING_DIR).join("w1");
        apply(&new, &root, &work.join("old")).unwrap();
        assert!(!work.join(APPLYING).exists());
        assert_eq!(read(&root.join("bin").join(exe("swofty"))), "cli v2");
        write(&work.join(APPLYING), "bin/added\n");
        let install = Install { root: root.clone() };
        assert!(install.cleanup());
        assert_eq!(read(&root.join("bin").join(exe(LAUNCHER_BIN))), "launcher v1");
        assert_eq!(read(&root.join("bin").join(exe("swofty"))), "cli v1");
        assert_eq!(
            read(&root.join("loader").join(crate::builtin::CORE_DLL)),
            "core v1"
        );
        assert!(!root.join("bin/added").exists());
        assert!(!root.join(STAGING_DIR).exists());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_failed_replace_leaves_the_old_install() {
        let dir = temp("rollback");
        let (root, new) = (dir.join("install"), dir.join("new"));
        release_tree(&root, "v1");
        write(&root.join("loader/blocked"), "a file where v2 has a folder");
        release_tree(&new, "v2");
        write(&new.join("loader/blocked/inner.dll"), "v2");
        write(&new.join("bin/added"), "v2");
        assert!(apply(&new, &root, &dir.join("old")).is_err());
        assert_eq!(read(&root.join("bin").join(exe(LAUNCHER_BIN))), "launcher v1");
        assert_eq!(read(&root.join("bin").join(exe("swofty"))), "cli v1");
        assert_eq!(
            read(&root.join("loader").join(crate::builtin::CORE_DLL)),
            "core v1"
        );
        assert_eq!(read(&root.join("loader/blocked")), "a file where v2 has a folder");
        assert!(!root.join("bin/added").exists());
        std::fs::remove_dir_all(&dir).ok();
    }
}
