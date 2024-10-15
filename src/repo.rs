use std::{
    convert::Infallible,
    path::{Path, PathBuf},
};

use configparser::ini::Ini;

/// Actions that can be done to a repository.
pub(crate) trait Repository: Sized {
    type Error: std::error::Error;

    fn new(path: &Path) -> Result<Self, Self::Error>;

    fn repo_path<P>(repo: &Path, path: P) -> PathBuf
    where
        P: AsRef<Path>,
    {
        repo.join(path)
    }
}

/// Loading/manipulating a config object.
trait Config: Sized {
    type Error: std::error::Error;

    fn load<P>(path: P) -> Result<Self, Self::Error>
    where
        P: AsRef<Path>;

    fn getuint(&self, section: &str, field: &str) -> Result<u64, Self::Error>;
}

#[derive(Debug, thiserror::Error, PartialEq)]
enum Error {
    #[error("Not a Git repository: {0}")]
    NotGitRepository(PathBuf),
    #[error("Configuration file is missing")]
    ConfigFileMissing,
    #[error("Unsupported repositoryformatversion: {0:?}")]
    UnsupportedVersion(Option<u64>),
    #[error("Invalid config: {0}")]
    InvalidConfig(String),
}

/// A repository for which we have validated that `worktree` and `gitdir` exist.
#[derive(Debug, PartialEq)]
struct Repo<T> {
    inner: UnvalidatedRepo,
    config: T,
}

impl Config for Ini {
    type Error = Error;

    fn load<P>(path: P) -> Result<Self, Self::Error>
    where
        P: AsRef<Path>,
    {
        let mut config = Self::new();
        if !path.as_ref().exists() {
            return Err(Error::ConfigFileMissing);
        }
        config.load(path).map_err(Error::InvalidConfig)?;

        Ok(config)
    }

    fn getuint(&self, section: &str, field: &str) -> Result<u64, Self::Error> {
        Ok(self
            .getuint(section, field)
            .map_err(Error::InvalidConfig)?
            .ok_or(Error::UnsupportedVersion(None))?)
    }
}

impl<T> Repository for Repo<T>
where
    T: Config,
    Error: From<<T as Config>::Error>,
{
    type Error = Error;

    fn new(path: &Path) -> Result<Self, Self::Error> {
        if !path.is_dir() {
            return Err(Error::NotGitRepository(path.to_owned()));
        }

        // check that the version is equal to 0
        let config = T::load(path)?;
        let version = config.getuint("core", "repositoryformatversion")?;
        if version != 0 {
            return Err(Error::UnsupportedVersion(Some(version)));
        }

        Ok(Self {
            inner: UnvalidatedRepo::new(path).unwrap(),
            config,
        })
    }
}

/// A repository where `worktree` and `gitdir` may or may not exist.
///
/// This is primarily useful for `init`.
#[derive(Debug, PartialEq)]
struct UnvalidatedRepo {
    worktree: PathBuf,
    gitdir: PathBuf,
}

impl Repository for UnvalidatedRepo {
    type Error = Infallible;

    fn new(path: &Path) -> Result<Self, Self::Error> {
        Ok(Self {
            worktree: path.to_owned(),
            gitdir: path.join(".git"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct FakeConfig {
        version: u64,
    }

    impl Config for FakeConfig {
        type Error = Error;

        fn load<P>(_path: P) -> Result<Self, Self::Error>
        where
            P: AsRef<Path>,
        {
            Ok(Self { version: 1 })
        }

        fn getuint(&self, _section: &str, _field: &str) -> Result<u64, Self::Error> {
            Ok(self.version)
        }
    }

    #[test]
    fn nonzero_version() {
        let repo: Result<Repo<FakeConfig>, _> = Repo::new(&std::env::current_dir().unwrap());
        assert_eq!(repo, Err(Error::UnsupportedVersion(Some(1))));
    }
}
