use std::process::ExitCode;
use std::sync::Mutex;

use anyhow::{Result, bail};
use swofty_core::launcher::Progress;
use swofty_core::update::{self, Install};

struct Printer(Mutex<(i32, String)>);

impl Progress for Printer {
    fn report(&self, done: f32, text: &str) {
        let step = (done * 10.0) as i32;
        if let Ok(mut last) = self.0.lock()
            && (last.0 != step || last.1 != text)
        {
            *last = (step, text.to_owned());
            eprintln!("swofty: {text} {:.0}%", done * 100.0);
        }
    }
}

pub fn update(check: bool) -> Result<ExitCode> {
    let current = update::current_version();
    let release = update::fetch_latest(update::CHECK_TIMEOUT)?;
    let Some(latest) = release.version().filter(|v| *v > current) else {
        println!("swofty {current} is up to date");
        return Ok(ExitCode::SUCCESS);
    };
    let notes = release.notes_url();
    if check {
        println!("swofty {latest} is available (this is {current}): {notes}");
        return Ok(ExitCode::SUCCESS);
    }
    let install = match Install::current() {
        Ok(install) => install,
        Err(reason) => bail!("{reason}; download {latest} from {notes}"),
    };
    install.cleanup();
    println!("updating {} from {current} to {latest}", install.root.display());
    let version = install.update(&release, &Printer(Mutex::default()))?;
    println!("updated to {version}; restart the launcher to use it");
    Ok(ExitCode::SUCCESS)
}
