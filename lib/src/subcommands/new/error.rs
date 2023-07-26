use std::{
    error::Error as StdError,
    fmt::{self, Display, Formatter},
    io,
    path::PathBuf,
};

/// Error while executing `new` subcommand.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Chosen destination of new project already exists.
    DestinationExists {
        /// The existing path.
        path: PathBuf,
    },
    /// Failed to create a directory at the given path.
    FailedToCreateDir {
        /// The underlying IO error.
        error: io::Error,
        /// The directory path.
        path: PathBuf,
    },
    /// Failed to write a file at the given path.
    FailedToWriteFile {
        /// The underlying IO error.
        error: io::Error,
        /// The file path.
        path: PathBuf,
    },
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Error::DestinationExists { path } => {
                write!(formatter, "destination `{}` already exists", path.display())
            }
            Error::FailedToCreateDir { error, path } => {
                write!(
                    formatter,
                    "failed to create dir `{}`: {error}",
                    path.display()
                )
            }
            Error::FailedToWriteFile { error, path } => {
                write!(
                    formatter,
                    "failed to write file `{}`: {error}",
                    path.display()
                )
            }
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::DestinationExists { .. } => None,
            Error::FailedToCreateDir { error, .. } | Error::FailedToWriteFile { error, .. } => {
                Some(error)
            }
        }
    }
}
