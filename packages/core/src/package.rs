use std::io::Read;
use std::path::Path;

use crate::manifest::{AddonManifest, MANIFEST_FILE};
use crate::{Error, Result, io_err};

pub const EXT: &str = "swoftyaddon";

pub fn manifest(pkg: &Path) -> Result<AddonManifest> {
    let file = std::fs::File::open(pkg).map_err(io_err(pkg))?;
    let mut zip = zip::ZipArchive::new(file)?;
    let mut entry = zip
        .by_name(MANIFEST_FILE)
        .map_err(|_| Error::Other(format!("{}: no {MANIFEST_FILE} at the root", pkg.display())))?;
    let mut text = String::new();
    entry.read_to_string(&mut text).map_err(io_err(pkg))?;
    Ok(AddonManifest::parse(&text)?)
}
