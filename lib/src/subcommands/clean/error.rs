use std::{
    error::Error as StdError,
    fmt::{self, Display, Formatter},
    io,
    path::PathBuf,
};

use crate::cached_config::CachedConfig;
use crate::CachedConfigError;

/// Error while executing `show` subcommand.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Missing cached config file.
    MissingCachedConfig,
    /// Error related to the cached config.
    CachedConfig(CachedConfigError),
    /// Failed to remove the directory at the given path.
    RemoveDir {
        /// The underlying IO error.
        error: io::Error,
        /// The directory path.
        path: PathBuf,
    },
}

impl From<CachedConfigError> for Error {
    fn from(error: CachedConfigError) -> Self {
        Self::CachedConfig(error)
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Error::MissingCachedConfig => {
                write!(
                    formatter,
                    "expected a cached config to exist at `{}` from a previous \"exec\" run",
                    CachedConfig::path().display()
                )
            }
            Error::CachedConfig(error) => Display::fmt(error, formatter),
            Error::RemoveDir { error, path } => {
                write!(
                    formatter,
                    "failed to remove stored global state directory at `{}`: {error}",
                    path.display()
                )
            }
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::MissingCachedConfig => None,
            Error::CachedConfig(error) => Some(error),
            Error::RemoveDir { error, .. } => Some(error),
        }
    }
}
