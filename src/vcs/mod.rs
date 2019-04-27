mod git;
mod hg;

use failure::{err_msg, Error};
use std::path::Path;

use self::git::Git;
use self::hg::Hg;

pub trait VCS {
    fn name(&self) -> &str;
    fn checkout(&self, rev: &str) -> Result<(), Error>;
}

#[allow(clippy::type_complexity)]
static SUPPORTED_VCS: [fn(&Path) -> Option<Box<dyn VCS>>; 2] = [Git::new, Hg::new];

pub fn detect_vcs(path: &Path) -> Result<Box<dyn VCS>, Error> {
    let mut pathbuf = path.to_path_buf();

    loop {
        if let Some(v) = SUPPORTED_VCS
            .iter()
            .map(|f| f(&pathbuf))
            .find(Option::is_some)
            .and_then(|a| a)
        {
            return Ok(v);
        }

        if !pathbuf.pop() {
            return Err(err_msg("Cannot find VCS in directory"));
        }
    }
}
