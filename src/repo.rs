use std::{
    convert::Infallible,
    fs::{self, DirBuilder, OpenOptions},
    io,
    path::{Path, PathBuf},
};

use configparser::ini::Ini;
use indoc::indoc;

/// Actions that can be done to a repository.
pub(crate) trait Repository: Sized {
    type Error: std::error::Error;

    fn new(path: &Path) -> Result<Self, Self::Error>;
}

/// Loading/manipulating a config object.
trait Config {
    type Error: std::error::Error;

    fn load<P>(path: P) -> Result<Self, Self::Error>
    where
        P: AsRef<Path>,
        Self: Sized;

    fn getuint(&self, section: &str, field: &str) -> Result<u64, Self::Error>;
}

trait RepoPathHelper {
    fn ensure_dir_exists<P>(path: P) -> Result<PathBuf, io::Error>
    where
        P: AsRef<Path>;

    fn ensure_file_exists<P>(path: P) -> Result<PathBuf, io::Error>
    where
        P: AsRef<Path>;
}

struct PathHelper;
impl RepoPathHelper for PathHelper {
    fn ensure_dir_exists<P>(path: P) -> Result<PathBuf, io::Error>
    where
        P: AsRef<Path>,
    {
        DirBuilder::new().recursive(true).create(path.as_ref())?;
        Ok(path.as_ref().to_owned())
    }

    fn ensure_file_exists<P>(path: P) -> Result<PathBuf, io::Error>
    where
        P: AsRef<Path>,
    {
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(path.as_ref())?;
        Ok(path.as_ref().to_owned())
    }
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
    #[error("Error occurred during I/O: {0}")]
    Io(String),
    #[error("{0} is not a directory")]
    NotADirectory(PathBuf),
    #[error("{0} is no empty")]
    NotEmpty(PathBuf),
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
        self.getuint(section, field)
            .map_err(Error::InvalidConfig)?
            .ok_or(Error::UnsupportedVersion(None))
    }
}

impl<T> Repository for Repo<T>
where
    T: Config,
    Error: From<<T as Config>::Error>,
{
    type Error = Error;

    fn new(path: &Path) -> Result<Self, Self::Error> {
        let unvalidated = UnvalidatedRepo::new(path).expect("UnvalidatedRepo::new() cannot fail");
        unvalidated.try_into()
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

impl<T> TryFrom<UnvalidatedRepo> for Repo<T>
where
    T: Config,
    Error: From<<T as Config>::Error>,
{
    type Error = Error;

    fn try_from(inner: UnvalidatedRepo) -> Result<Self, Self::Error> {
        if !inner.worktree.is_dir() {
            return Err(Error::NotGitRepository(inner.worktree));
        }

        // check that the version is equal to 0
        let config = T::load(inner.gitdir.join("config"))?;
        let version = config.getuint("core", "repositoryformatversion")?;
        if version != 0 {
            return Err(Error::UnsupportedVersion(Some(version)));
        }

        Ok(Self { inner, config })
    }
}

trait RepoCreator {
    type Repo: Repository;
    type Error: std::error::Error;

    fn create<P>(path: P) -> Result<Self::Repo, Self::Error>
    where
        P: AsRef<Path>;
}

struct RealRepoCreator;
impl RepoCreator for RealRepoCreator {
    type Repo = Repo<Ini>;
    type Error = Error;

    fn create<P>(path: P) -> Result<Self::Repo, Self::Error>
    where
        P: AsRef<Path>,
    {
        let Ok(repo) = UnvalidatedRepo::new(path.as_ref());
        if repo.worktree.exists() {
            if !repo.worktree.is_dir() {
                return Err(Error::NotADirectory(repo.worktree));
            }
            if repo.gitdir.exists()
                && fs::read_dir(&repo.gitdir)
                    .map_err(|err| Error::Io(err.to_string()))?
                    .count()
                    != 0
            {
                return Err(Error::NotEmpty(repo.gitdir));
            }
        } else {
            PathHelper::ensure_dir_exists(&repo.worktree)
                .map_err(|err| Error::Io(err.to_string()))?;
        }

        PathHelper::ensure_dir_exists(repo.gitdir.join("branches"))
            .map_err(|err| Error::Io(err.to_string()))?;
        PathHelper::ensure_dir_exists(repo.gitdir.join("objects"))
            .map_err(|err| Error::Io(err.to_string()))?;
        PathHelper::ensure_dir_exists(repo.gitdir.join("refs/tags"))
            .map_err(|err| Error::Io(err.to_string()))?;
        PathHelper::ensure_dir_exists(repo.gitdir.join("refs/heads"))
            .map_err(|err| Error::Io(err.to_string()))?;

        fs::write(
            repo.gitdir.join("description"),
            "Unnamed repository; edit this file 'description' to name the repository.\n",
        )
        .map_err(|err| Error::Io(err.to_string()))?;
        fs::write(repo.gitdir.join("HEAD"), "ref: refs/heads/master\n")
            .map_err(|err| Error::Io(err.to_string()))?;
        fs::write(repo.gitdir.join("config"), DefaultConfig.to_string())
            .map_err(|err| Error::Io(err.to_string()))?;

        repo.try_into()
    }
}

struct DefaultConfig;
impl std::fmt::Display for DefaultConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let config = indoc! {"
            [core]
            repositoryformatversion = 0
            filemode = false
            bare = false
        "};
        write!(f, "{config}")
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

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
        let repo: Result<Repo<FakeConfig>, _> = Repo::new(&std::env::temp_dir());
        assert_eq!(repo, Err(Error::UnsupportedVersion(Some(1))));
    }

    #[test]
    fn create() {
        let tempdir = TempDir::new().unwrap();
        let _ = RealRepoCreator::create(tempdir.as_ref().join("test")).unwrap();
        assert!(!fs::read(tempdir.as_ref().join("test/.git/config"))
            .unwrap()
            .is_empty());
    }
}
