use super::VCS;

use std::path::{Path, PathBuf};
use failure::Error;
use std::process::Command;

#[derive(Debug)]
pub struct Git {
    root: PathBuf,
}

impl Git {
    /// Create a new instance of the `Git` VCS impl based
    /// on the passed in root. Will succeed if the given
    /// root contains a root that contains a `.git` directory.
    pub fn new(root: &Path) -> Option<Box<VCS>> {
        if root.join(".git").exists() {
            Some(Box::new(Git {root: root.to_path_buf()}))
        } else {
            None
        }
    }
}

impl VCS for Git {
    fn name(&self) -> &str {
        "git"
    }

    fn checkout(&self, rev: &str) -> Result<(), Error> {
        let status = Command::new("git")
            .args(&["checkout", rev])
            .current_dir(&self.root)
            .status()?;

        if status.success() {
            Ok(())
        } else {
            bail!("git failed with exit code {:?}", status.code())
        }
    }
}
