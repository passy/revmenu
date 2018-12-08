use super::VCS;

use failure::{bail, Error};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug)]
pub struct Hg {
    root: PathBuf,
}

impl Hg {
    /// Create a new instance of the `Hg` VCS impl based
    /// on the passed in root. Will succeed if the given
    /// root contains a root that contains a `.hg` directory.
    pub fn new(root: &Path) -> Option<Box<dyn VCS>> {
        if root.join(".hg").exists() {
            Some(Box::new(Hg {
                root: root.to_path_buf(),
            }))
        } else {
            None
        }
    }
}

impl VCS for Hg {
    fn name(&self) -> &str {
        "hg"
    }

    fn checkout(&self, rev: &str) -> Result<(), Error> {
        let status = Command::new("hg")
            .args(&["update", rev])
            .current_dir(&self.root)
            .status()?;

        if status.success() {
            Ok(())
        } else {
            bail!("hg failed with exit code {:?}", status.code())
        }
    }
}
