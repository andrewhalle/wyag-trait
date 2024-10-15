// TODO: remove
#![allow(dead_code)]

use application::{clap, Application};

struct Git;
#[derive(Debug, thiserror::Error)]
#[error("git error")]
struct GitError;

#[application::main]
static APP: Git = Git;

mod add;
mod cat_file;
mod check_ignore;
mod checkout;
mod commit;
mod hash_object;
mod init;
mod log;
mod ls_files;
mod ls_tree;
mod repo;
mod rev_parse;
mod rm;
mod show_ref;
mod status;
mod tag;

#[derive(clap::Parser, Debug)]
#[command(name = "wyag", about = "the stupidest content tracker")]
enum Command {
    Add(add::Args),
    CatFile(cat_file::Args),
    CheckIgnore(check_ignore::Args),
    Checkout(checkout::Args),
    Commit(commit::Args),
    HashObject(hash_object::Args),
    Init(init::Args),
    Log(log::Args),
    LsFiles(ls_files::Args),
    LsTree(ls_tree::Args),
    RevParse(rev_parse::Args),
    Rm(rm::Args),
    ShowRef(show_ref::Args),
    Status(status::Args),
    Tag(tag::Args),
}

trait Execute: Sized {
    fn execute(self) -> Result<(), GitError> {
        Ok(())
    }
}

impl Execute for Command {
    fn execute(self) -> Result<(), GitError> {
        match self {
            Command::Add(args) => args.execute(),
            Command::CatFile(args) => args.execute(),
            Command::CheckIgnore(args) => args.execute(),
            Command::Checkout(args) => args.execute(),
            Command::Commit(args) => args.execute(),
            Command::HashObject(args) => args.execute(),
            Command::Init(args) => args.execute(),
            Command::Log(args) => args.execute(),
            Command::LsFiles(args) => args.execute(),
            Command::LsTree(args) => args.execute(),
            Command::RevParse(args) => args.execute(),
            Command::Rm(args) => args.execute(),
            Command::ShowRef(args) => args.execute(),
            Command::Status(args) => args.execute(),
            Command::Tag(args) => args.execute(),
        }
    }
}

impl Application for Git {
    type Error = GitError;
    type Args = Command;

    fn main(&self, args: Self::Args) -> Result<(), Self::Error> {
        args.execute()
    }
}
