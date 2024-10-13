use application::clap;

use crate::Execute;

#[derive(Debug, clap::Args)]
pub(crate) struct Args;

impl Execute for Args {}
