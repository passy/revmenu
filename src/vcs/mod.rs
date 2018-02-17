mod git;
mod hg;

use std::path::Path;
use failure::{err_msg, Error};

use self::git::Git;
use self::hg::Hg;

pub trait VCS {
    fn name(&self) -> &str;
    fn checkout(&self, rev: &str) -> Result<(), Error>;
}

static SUPPORTED_VCS: [fn(&Path) -> Option<Box<VCS>>; 2] = [Git::new, Hg::new];

pub fn detect_vcs(path: &Path) -> Result<Box<VCS>, Error> {
    let mut pathbuf = path.to_path_buf();

    loop {
        if let Some(v) = SUPPORTED_VCS
            .iter()
            .map(|f| f(&pathbuf))
            .find(|v| v.is_some())
            .and_then(|a| a)
        {
            return Ok(v);
        }

        if !pathbuf.pop() {
            return Err(err_msg("Cannot find VCS in directory"));
        }
    }
}
