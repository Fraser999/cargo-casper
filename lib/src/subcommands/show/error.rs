use std::{
    error::Error as StdError,
    fmt::{self, Display, Formatter},
};

use crate::cached_config::CachedConfig;
use crate::{CachedConfigError, StorageError};

/// Error while executing `show` subcommand.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Missing cached config file.
    MissingCachedConfig,
    /// Error related to the cached config.
    CachedConfig(CachedConfigError),
    /// Error related to storage of global state.
    Storage(StorageError),
}

impl From<CachedConfigError> for Error {
    fn from(error: CachedConfigError) -> Self {
        Self::CachedConfig(error)
    }
}

impl From<StorageError> for Error {
    fn from(error: StorageError) -> Self {
        Self::Storage(error)
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
            Error::Storage(error) => Display::fmt(error, formatter),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::MissingCachedConfig => None,
            Error::CachedConfig(error) => Some(error),
            Error::Storage(error) => Some(error),
        }
    }
}
