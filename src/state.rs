use anyhow::{self, bail};
use log::{debug, error};
use std::{fs::OpenOptions, io::Read, io::Write};

#[derive(Debug)]
pub struct State {
    filename: String,
}

impl State {
    pub fn new(filename: &str) -> Self {
        Self {
            filename: filename.to_owned(),
        }
    }

    pub fn get(&self) -> anyhow::Result<String> {
        debug!("Opening state file {}", self.filename);
        let mut f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&self.filename)?;
        let mut res = String::new();
        let _ = f.read_to_string(&mut res);
        let stripped = res.strip_suffix("\n").unwrap_or(res.as_str());
        Ok(stripped.to_owned())
    }

    pub fn set(&self, content: &str) -> anyhow::Result<()> {
        let mut f = OpenOptions::new()
            .truncate(true)
            .write(true)
            .create(true)
            .open(&self.filename)?;
        if write!(&mut f, "{}", content).is_err() {
            error!("Could not write '{}' to '{}'", content, self.filename);
            bail!("Could not write content to file");
        } else {
            debug!("Wrote '{}' to '{}'", content, self.filename);
            Ok(())
        }
    }
}
