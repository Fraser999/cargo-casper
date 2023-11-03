use std::{
    env,
    error::Error as StdError,
    fmt::{self, Display, Formatter},
    fs, io,
    path::PathBuf,
};

use directories::ProjectDirs;
use log::debug;
use serde::{Deserialize, Serialize};

use casper_types::Digest;

const BIN_CRATE_NAME: &str = "cargo-casper";

/// Configuration values which are cached (written to disk, TOML-encoded) which are read and written
/// on each run of the relevant subcommands.
#[derive(Eq, PartialEq, Serialize, Deserialize, Debug)]
pub(crate) struct CachedConfig {
    /// The directory used for storing fetched global state.
    pub(crate) storage_dir: PathBuf,
    /// The network chain name.
    pub(crate) chain_name: String,
    /// The address of a node on the network for querying, in the form "host:port".
    pub(crate) node_address: String,
    /// The state root hash of the global state used.
    pub(crate) state_hash: Digest,
}

impl CachedConfig {
    /// Returns the `CachedConfig` if it exists, or `None` if it doesn't.
    pub(crate) fn try_read() -> Result<Option<Self>, CachedConfigError> {
        let config_path = Self::path();
        if !config_path.exists() {
            return Ok(None);
        }
        let encoded =
            fs::read_to_string(&config_path).map_err(|error| CachedConfigError::Read {
                error,
                path: config_path.clone(),
            })?;
        toml::from_str(&encoded).map_err(|error| CachedConfigError::Decode {
            error,
            path: config_path.clone(),
        })
    }

    /// Writes the `CachedConfig` to disk.
    pub(crate) fn write(&self) -> Result<(), CachedConfigError> {
        let config_path = Self::path();
        let config_dir = config_path.parent().unwrap();
        fs::create_dir_all(config_dir).map_err(|error| CachedConfigError::CreateDir {
            error,
            path: config_dir.to_path_buf(),
        })?;
        let encoded =
            toml::to_string_pretty(&self).map_err(|error| CachedConfigError::Encode { error })?;
        fs::write(&config_path, encoded).map_err(|error| CachedConfigError::Write {
            error,
            path: config_path.clone(),
        })?;
        debug!("cached config at {}", config_path.display());
        Ok(())
    }

    /// Returns the path to the `CachedConfig`.
    ///
    /// The filename is `config.toml` and the directory is as per [`ProjectDirs::config_dir`], or if
    /// that fails, `/tmp/cargo-casper/`.
    pub(crate) fn path() -> PathBuf {
        if cfg!(test) {
            return env::temp_dir()
                .join(format!("test-{BIN_CRATE_NAME}"))
                .join("config.toml");
        }
        if let Some(project_dir) = ProjectDirs::from("", "Casper Labs", BIN_CRATE_NAME) {
            return project_dir.config_dir().join("config.toml");
        }
        env::temp_dir().join(BIN_CRATE_NAME).join("config.toml")
    }
}

/// Error while writing or reading the cached config to or from disk.
#[derive(Debug)]
#[non_exhaustive]
pub enum CachedConfigError {
    /// Failed to read the cached config at the given path.
    Read {
        /// The underlying IO error.
        error: io::Error,
        /// The file path.
        path: PathBuf,
    },
    /// Failed to decode the cached config from TOML.
    Decode {
        /// The underlying toml error.
        error: toml::de::Error,
        /// The file path.
        path: PathBuf,
    },
    /// Failed to create a directory at the given path.
    CreateDir {
        /// The underlying IO error.
        error: io::Error,
        /// The directory path.
        path: PathBuf,
    },
    /// Failed to TOML-encode the cached config.
    Encode {
        /// The underlying toml error.
        error: toml::ser::Error,
    },
    /// Failed to write the cached config at the given path.
    Write {
        /// The underlying IO error.
        error: io::Error,
        /// The file path.
        path: PathBuf,
    },
}

impl Display for CachedConfigError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            CachedConfigError::Read { error, path } => {
                write!(
                    formatter,
                    "failed to read cached config at `{}`: {error}",
                    path.display()
                )
            }
            CachedConfigError::Decode { error, path } => {
                write!(
                    formatter,
                    "failed to decode cached config at `{}`: {error}",
                    path.display()
                )
            }
            CachedConfigError::CreateDir { error, path } => {
                write!(
                    formatter,
                    "failed to create dir for cached config at `{}`: {error}",
                    path.display()
                )
            }
            CachedConfigError::Encode { error } => {
                write!(formatter, "failed to encode cached config: {error}")
            }
            CachedConfigError::Write { error, path } => {
                write!(
                    formatter,
                    "failed to write cached config to `{}`: {error}",
                    path.display()
                )
            }
        }
    }
}

impl StdError for CachedConfigError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            CachedConfigError::Read { error, .. }
            | CachedConfigError::CreateDir { error, .. }
            | CachedConfigError::Write { error, .. } => Some(error),
            CachedConfigError::Decode { error, .. } => Some(error),
            CachedConfigError::Encode { error } => Some(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn toml_roundtrip() {
        let config = CachedConfig {
            storage_dir: Path::new("a/b/c.toml").to_path_buf(),
            chain_name: "casper-net-1".to_string(),
            node_address: "http://localhost:11101".to_string(),
            state_hash: Digest::hash([7, 8, 9]),
        };
        let encoded = toml::to_string_pretty(&config).unwrap();
        let decoded = toml::from_str(&encoded).unwrap();
        assert_eq!(config, decoded);
    }

    #[test]
    fn should_read_write() {
        let _ = fs::remove_file(CachedConfig::path());
        assert!(CachedConfig::try_read().unwrap().is_none());

        let config = CachedConfig {
            storage_dir: Path::new("a/b/c.toml").to_path_buf(),
            chain_name: "casper-net-1".to_string(),
            node_address: "http://localhost:11101".to_string(),
            state_hash: Digest::hash([7, 8, 9]),
        };
        config.write().unwrap();
        let read = CachedConfig::try_read().unwrap();
        assert_eq!(Some(config), read);
    }
}
