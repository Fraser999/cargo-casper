use std::{
    cell::RefCell,
    collections::HashMap,
    error::Error as StdError,
    fmt::{self, Display, Formatter},
    fs, io,
    path::{Path, PathBuf},
    rc::Rc,
};

use casper_storage::global_state::state::CommitError;
use casper_types::{
    bytesrepr::{self, ToBytes},
    execution::{ExecutionJournal, TransformKind},
    Digest, Key, StoredValue,
};
use log::{debug, error, trace};

/// A simple key, value store, with an in-memory map and an on-disk file for persisting the data.
#[derive(Clone)]
pub(crate) struct Storage {
    path: PathBuf,
    data: Rc<RefCell<HashMap<Key, StoredValue>>>,
}

impl Storage {
    /// Returns a new `Storage` backed by a file at "root_path/chain_name-short_hash" where
    /// short_hash is the first 7 chars of hex-encoded state_hash.
    ///
    /// If the file already exists, it is opened and parsed into the in-memory map.  If the file
    /// doesn't exist, it is created if `create` is `true`, otherwise an error is returned.
    ///
    /// The in-memory map is only written to disk when `persist()` is called or when the `Storage`
    /// instance is dropped.
    pub(crate) fn new(
        root_path: &Path,
        chain_name: &str,
        state_hash: &Digest,
        create_if_missing: bool,
    ) -> Result<Self, StorageError> {
        let mut short_hash = format!("{:?}", state_hash);
        short_hash.truncate(7);
        let path = root_path.join(format!("{}-{}", chain_name, short_hash));

        if !path.is_file() {
            if create_if_missing {
                let storage = Storage {
                    path,
                    data: Rc::new(RefCell::new(HashMap::new())),
                };
                storage.persist()?;
                return Ok(storage);
            }
            return Err(StorageError::MissingFile { path });
        }

        let serialized = fs::read(&path).map_err(|error| StorageError::Read {
            error,
            path: path.clone(),
        })?;
        let data =
            bytesrepr::deserialize(serialized).map_err(|error| StorageError::Deserialize {
                error,
                path: path.clone(),
            })?;
        Ok(Storage {
            path,
            data: Rc::new(RefCell::new(data)),
        })
    }

    /// Insert the value to the in-memory map.
    pub(crate) fn insert(&self, key: Key, value: StoredValue) {
        self.data.borrow_mut().insert(key, value);
    }

    /// Get the value from the in-memory map.
    pub(crate) fn get(&self, key: &Key) -> Option<StoredValue> {
        self.data.borrow().get(key).cloned()
    }

    /// Put the effects to the in-memory map.
    pub(crate) fn commit(&self, effects: ExecutionJournal) -> Result<(), CommitError> {
        for effect in effects.transforms() {
            match effect.kind().clone() {
                TransformKind::Identity => {}
                TransformKind::Write(value) => {
                    trace!("put to storage: {}, {:?}", effect.key(), value);
                    self.insert(*effect.key(), value);
                }
                kind => {
                    let current_value = self
                        .data
                        .borrow()
                        .get(effect.key())
                        .ok_or_else(|| {
                            error!("failed to get {} from storage", effect.key());
                            CommitError::KeyNotFound(*effect.key())
                        })?
                        .clone();
                    let error_context =
                        format!("failed to apply {:?} to {:?}", kind, current_value);
                    let new_value = match kind.apply(current_value) {
                        Ok(value) => value,
                        Err(error) => {
                            error!("{error_context}: {}", error);
                            return Err(CommitError::TransformError(error));
                        }
                    };
                    trace!("put to storage: {}, {:?}", effect.key(), new_value);
                    self.insert(*effect.key(), new_value);
                }
            }
        }
        Ok(())
    }

    /// Write the in-memory map to disk.
    pub(crate) fn persist(&self) -> Result<(), StorageError> {
        let serialized = (*self.data.borrow())
            .to_bytes()
            .map_err(|error| StorageError::Serialize { error })?;
        if let Some(dir) = self.path.parent() {
            fs::create_dir_all(dir).map_err(|error| StorageError::CreateDir {
                error,
                path: dir.to_path_buf(),
            })?;
        }
        fs::write(&self.path, serialized).map_err(|error| StorageError::Write {
            error,
            path: self.path.clone(),
        })?;
        debug!("wrote global state to {}", self.path.display());
        Ok(())
    }
}

// impl Drop for Storage {
//     fn drop(&mut self) {
//         if Rc::strong_count(&self.data) == 1 {
//             let _ = self.persist();
//         }
//     }
// }

impl Display for Storage {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "storage at `{}`", self.path.display())?;
        for (key, value) in self.data.borrow().iter() {
            writeln!(
                formatter,
                "  {}: {}",
                key,
                serde_json::to_string(value).unwrap()
            )?;
        }
        Ok(())
    }
}

/// Error while reading or writing the global state.
#[derive(Debug)]
#[non_exhaustive]
pub enum StorageError {
    /// The stored global state file is not at the expected path.
    MissingFile {
        /// The file path.
        path: PathBuf,
    },
    /// Failed to read the stored global state file at the given path.
    Read {
        /// The underlying IO error.
        error: io::Error,
        /// The file path.
        path: PathBuf,
    },
    /// Failed to decode the stored global state file.
    Deserialize {
        /// The underlying bytesrepr error.
        error: bytesrepr::Error,
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
    /// Failed to bytesrepr-serialize the global state.
    Serialize {
        /// The underlying bytesrepr error.
        error: bytesrepr::Error,
    },
    /// Failed to write the stored global state file to the given path.
    Write {
        /// The underlying IO error.
        error: io::Error,
        /// The file path.
        path: PathBuf,
    },
}

impl Display for StorageError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            StorageError::MissingFile { path } => {
                write!(formatter, "no global state found at `{}`", path.display())
            }
            StorageError::Read { error, path } => {
                write!(
                    formatter,
                    "failed to read stored global state file at `{}`: {error}",
                    path.display()
                )
            }
            StorageError::Deserialize { error, path } => {
                write!(
                    formatter,
                    "failed to deserialize stored global state file at `{}`: {error}",
                    path.display()
                )
            }
            StorageError::CreateDir { error, path } => {
                write!(
                    formatter,
                    "failed to create dir for stored global state at `{}`: {error}",
                    path.display()
                )
            }
            StorageError::Serialize { error } => {
                write!(
                    formatter,
                    "failed to serialize stored global state: {error}"
                )
            }
            StorageError::Write { error, path } => {
                write!(
                    formatter,
                    "failed to write stored global state file to `{}`: {error}",
                    path.display()
                )
            }
        }
    }
}

impl StdError for StorageError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            StorageError::MissingFile { .. } => None,
            StorageError::Read { error, .. }
            | StorageError::CreateDir { error, .. }
            | StorageError::Write { error, .. } => Some(error),
            StorageError::Deserialize { error, .. } | StorageError::Serialize { error } => {
                Some(error)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use casper_types::{CLValue, HashAddr};

    #[test]
    fn should_persist_storage() {
        let temp_dir = tempfile::tempdir().unwrap();
        let root_path = temp_dir.path();
        let state_hash = Digest::hash([1]);

        let key = Key::Hash(HashAddr::from(Digest::hash([2])));
        let stored_value = StoredValue::CLValue(CLValue::from_t("stuff").unwrap());

        {
            let storage = Storage::new(root_path, "net", &state_hash, true).unwrap();
            storage.data.borrow_mut().insert(key, stored_value.clone());
            storage.persist().unwrap();
        }

        let storage = Storage::new(root_path, "net", &state_hash, true).unwrap();
        let retrieved_value = storage.data.borrow().get(&key).cloned().unwrap();
        assert_eq!(retrieved_value, stored_value);
    }
}
