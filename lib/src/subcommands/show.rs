mod error;

use casper_types::Key;

use crate::{CachedConfig, Storage};
pub use error::Error;

/// Options for the `show` subcommand.
#[derive(Debug)]
pub enum Options {
    /// Show the value stored under the given key.
    Value(Key),
    /// Show all stored global state.
    AllState,
    /// Show the cached config options.
    CachedConfig,
}

impl Options {
    /// Executes the `show` subcommand with the provided options.
    pub fn run(self) -> Result<(), Error> {
        match self {
            Options::Value(key) => show_value(key),
            Options::AllState => show_all_state(),
            Options::CachedConfig => show_cached_config(),
        }
    }
}

fn show_value(key: Key) -> Result<(), Error> {
    let cached_config = CachedConfig::try_read()?.ok_or_else(|| Error::MissingCachedConfig)?;

    let storage = Storage::new(
        &cached_config.storage_dir,
        &cached_config.chain_name,
        &cached_config.state_hash,
        false,
    )?;

    match storage.get(&key) {
        Some(value) => println!("{}", serde_json::to_string_pretty(&value).unwrap()),
        None => println!("value not found."),
    }
    Ok(())
}

fn show_all_state() -> Result<(), Error> {
    let cached_config = CachedConfig::try_read()?.ok_or_else(|| Error::MissingCachedConfig)?;

    let storage = Storage::new(
        &cached_config.storage_dir,
        &cached_config.chain_name,
        &cached_config.state_hash,
        false,
    )?;

    println!("{}", storage);
    Ok(())
}

fn show_cached_config() -> Result<(), Error> {
    match CachedConfig::try_read()? {
        Some(cached_config) => {
            println!(
                "config options cached at `{}`:\n\n{}",
                CachedConfig::path().display(),
                toml::to_string_pretty(&cached_config).unwrap()
            );
        }
        None => {
            println!(
                "no cached config options found at `{}`",
                CachedConfig::path().display()
            );
        }
    }
    Ok(())
}
