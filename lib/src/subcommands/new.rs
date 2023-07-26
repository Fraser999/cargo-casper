mod common;
mod contract_package;
mod dependency;
mod error;
mod makefile;
mod rust_toolchain;
mod tests_package;
mod travis_yml;

use std::path::PathBuf;

use dependency::Dependency;
pub use error::Error;

/// Can be used (via hidden command line args) to specify a patch section for the casper crates in
/// the generated Cargo.toml files.
#[derive(Debug)]
pub enum CasperOverrides {
    /// The path to local copy of the casper-node repository.
    WorkspacePath(PathBuf),
    /// The details of an online copy of the casper-node repository.
    GitRepo {
        /// The URL of the repository.
        url: String,
        /// The branch of the repository.
        branch: String,
    },
}

/// Options for the `new` subcommand.
#[derive(Debug)]
pub struct Options {
    /// The path at which the new project should be created.
    pub root_path: PathBuf,
    /// Optional overrides to be applied to the generated Cargo.toml files.
    pub casper_overrides: Option<CasperOverrides>,
}

impl Options {
    /// Executes the `new` subcommand with the provided options.
    pub fn run(self) -> Result<(), Error> {
        if self.root_path.exists() {
            return Err(Error::DestinationExists {
                path: self.root_path,
            });
        }

        common::create_dir_all(&self.root_path)?;

        contract_package::create(&self)?;
        tests_package::create(&self)?;
        rust_toolchain::create(&self)?;
        makefile::create(&self)?;
        travis_yml::create(&self)
    }
}
