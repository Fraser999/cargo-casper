use std::{
    error::Error as StdError,
    fmt::{self, Display, Formatter},
    path::PathBuf,
    str,
};

use casper_execution_engine::engine_state;
use casper_types::DeployConfigurationFailure;

use crate::{CachedConfigError, StorageError};

/// Error while executing `new` subcommand.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Error related to the cached config.
    CachedConfig(CachedConfigError),
    /// Error related to storage of global state.
    Storage(StorageError),
    /// Failed to get the state root hash from the chosen node.
    FailedToGetStateHash(casper_client::Error),
    /// Failed to get the chainspec from the chosen node.
    FailedToGetChainspec(casper_client::Error),
    /// Failed to convert the chainspec raw bytes to a string slice.
    ChainspecBytesToStr(str::Utf8Error),
    /// Failed to parse the chainspec string as TOML.
    ChainspecDeserialization(toml::de::Error),
    /// State root hash not known on the chosen node.
    UnknownStateHash,
    /// Failed to read the transaction at the given path.
    ReadTransaction {
        /// The underlying client error.
        error: casper_client::Error,
        /// The file path.
        path: PathBuf,
    },
    /// Transaction is invalid.
    InvalidTransaction(DeployConfigurationFailure),
    /// Execution Engine error while executing the transaction.
    Execution(engine_state::Error),
    /// Execution Engine error while committing the changes to global state.
    Commit(engine_state::Error),
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

impl From<DeployConfigurationFailure> for Error {
    fn from(error: DeployConfigurationFailure) -> Self {
        Self::InvalidTransaction(error)
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Error::CachedConfig(error) => Display::fmt(error, formatter),
            Error::Storage(error) => Display::fmt(error, formatter),
            Error::FailedToGetStateHash(error) => {
                write!(formatter, "failed to get state hash from node: {}", error)
            }
            Error::UnknownStateHash => write!(formatter, "requested state hash not found on node"),
            Error::ReadTransaction { error, path } => {
                write!(
                    formatter,
                    "failed to read transaction file at `{}`: {error}",
                    path.display()
                )
            }
            Error::InvalidTransaction(error) => write!(formatter, "invalid transaction: {}", error),
            Error::Execution(error) => write!(formatter, "failed to execute: {}", error),
            Error::Commit(error) => write!(
                formatter,
                "failed to save the changes to global state: {}",
                error
            ),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::CachedConfig(error) => Some(error),
            Error::Storage(error) => Some(error),
            Error::FailedToGetStateHash(error) => Some(error),
            Error::UnknownStateHash => None,
            Error::ReadTransaction { error, .. } => Some(error),
            Error::InvalidTransaction(error) => Some(error),
            Error::Execution(error) | Error::Commit(error) => Some(error),
        }
    }
}
