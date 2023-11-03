mod error;

use std::fs;

use log::info;

use crate::CachedConfig;
pub use error::Error;

/// Executes the `clean` subcommand.
pub fn run() -> Result<(), Error> {
    let cached_config = CachedConfig::try_read()?.ok_or_else(|| Error::MissingCachedConfig)?;
    let dir = &cached_config.storage_dir;

    if !dir.exists() {
        info!("storage dir at {} doesn't exist", dir.display());
        return Ok(());
    }

    fs::remove_dir_all(dir).map_err(|error| Error::RemoveDir {
        error,
        path: dir.to_path_buf(),
    })?;
    info!("removed storage dir at {}", dir.display());

    Ok(())
}
