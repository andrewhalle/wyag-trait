use std::{error::Error, path::PathBuf};

use application::clap;

use crate::{
    repo::{RealRepoCreator, RepoCreator as _},
    Execute,
};

#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    #[clap(default_value = ".")]
    path: PathBuf,
}

impl Execute for Args {
    fn execute(self) -> Result<(), crate::GitError> {
        RealRepoCreator::create(self.path).map_err(|err| Box::new(err) as Box<dyn Error>)?;
        Ok(())
    }
}
