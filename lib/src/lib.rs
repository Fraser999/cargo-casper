//! Library providing functionality for creating creating and testing a Casper Wasm contract.

#![warn(
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_qualifications
)]
#![doc(test(attr(deny(warnings))))]

mod cached_config;
mod storage;
/// The various subcommands supported.
pub mod subcommands;

pub(crate) use cached_config::CachedConfig;
pub use cached_config::CachedConfigError;
pub(crate) use storage::Storage;
pub use storage::StorageError;
