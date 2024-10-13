use application::{clap, Application};

struct Git;
#[derive(Debug, thiserror::Error)]
#[error("git error")]
struct GitError;

#[application::main]
static APP: Git = Git;

#[derive(clap::Parser, Debug)]
enum Command {
    Add,
    Init,
}

impl Application for Git {
    type Error = GitError;
    type Args = Command;

    fn main(&self, args: Self::Args) -> Result<(), Self::Error> {
        dbg!(args);
        Ok(())
    }
}
