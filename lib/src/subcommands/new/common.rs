use std::{fs, path::Path};

use super::{CasperOverrides, Dependency, Error, Options};

pub const CL_CONTRACT: Dependency = Dependency::new("casper-contract", "3.0.0");
pub const CL_TYPES: Dependency = Dependency::new("casper-types", "3.0.0");
pub const CL_ENGINE_TEST_SUPPORT: Dependency =
    Dependency::new("casper-engine-test-support", "5.0.0");
pub const CL_EXECUTION_ENGINE: Dependency = Dependency::new("casper-execution-engine", "5.0.0");

pub(super) fn patch_section(options: &Options) -> String {
    match &options.casper_overrides {
        Some(CasperOverrides::WorkspacePath(path)) => {
            format!(
                r#"[patch.crates-io]
casper-contract = {{ path = "{0}/smart_contracts/contract" }}
casper-engine-test-support = {{ path = "{0}/execution_engine_testing/test_support" }}
casper-execution-engine = {{ path = "{0}/execution_engine" }}
casper-types = {{ path = "{0}/types" }}
"#,
                path.display()
            )
        }
        Some(CasperOverrides::GitRepo { url, branch }) => {
            format!(
                r#"[patch.crates-io]
casper-contract = {{ git = "{0}", branch = "{1}" }}
casper-engine-test-support = {{ git = "{0}", branch = "{1}" }}
casper-execution-engine = {{ git = "{0}", branch = "{1}" }}
casper-types = {{ git = "{0}", branch = "{1}" }}
"#,
                url, branch
            )
        }
        None => String::new(),
    }
}

pub fn create_dir_all<P: AsRef<Path>>(path: P) -> Result<(), Error> {
    fs::create_dir_all(path.as_ref()).map_err(|error| Error::CreateDir {
        error,
        path: path.as_ref().to_path_buf(),
    })
}

pub fn write_file<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> Result<(), Error> {
    fs::write(path.as_ref(), contents).map_err(|error| Error::WriteFile {
        error,
        path: path.as_ref().to_path_buf(),
    })
}

#[cfg(test)]
pub mod tests {
    use reqwest::blocking;
    use serde_json::Value;

    use super::*;

    const CRATES_IO_RAW_INDEX_URL_FOR_CASPER_CRATES: &str =
        "https://raw.githubusercontent.com/rust-lang/crates.io-index/master/ca/sp/";
    const CRATES_IO_INDEX_URL_FOR_CASPER_CRATES: &str =
        "https://github.com/rust-lang/crates.io-index/blob/master/ca/sp/";
    const VERSION_FIELD_NAME: &str = "vers";

    /// Checks the version of the package specified by the Cargo.toml at `toml_path` is equal to
    /// the hard-coded one specified in `dep.version()`.

    /// https://crates.io/data-access
    fn check_latest_published_casper_package_version(dep: &Dependency) {
        let url = format!(
            "{}{}",
            CRATES_IO_RAW_INDEX_URL_FOR_CASPER_CRATES,
            dep.name()
        );
        let crate_io_index_contents = blocking::get(url)
            .unwrap_or_else(|error| {
                panic!(
                    "should get index file for {} from GitHub: {}",
                    dep.name(),
                    error
                )
            })
            .text()
            .unwrap_or_else(|error| {
                panic!("should parse index file for {}: {}", dep.name(), error)
            });

        let latest_entry: Value = serde_json::from_str(
            crate_io_index_contents
                .lines()
                .last()
                .expect("index file should contain at least one entry"),
        )
        .expect("latest entry from index file should parse as JSON");
        let latest_publish_version = latest_entry[VERSION_FIELD_NAME].as_str().unwrap();

        // If this fails, ensure `dep.version()` is updated to match the value in the Cargo.toml at
        // `toml_url`.
        assert_eq!(
            latest_publish_version,
            dep.version(),
            "\n\nEnsure local version of {:?} in common.rs is updated to {} as defined in last \
            version of {}{}\n\n",
            dep,
            latest_publish_version,
            CRATES_IO_INDEX_URL_FOR_CASPER_CRATES,
            dep.name()
        );
    }

    #[test]
    fn check_cl_contract_version() {
        check_latest_published_casper_package_version(&CL_CONTRACT);
    }

    #[test]
    fn check_cl_types_version() {
        check_latest_published_casper_package_version(&CL_TYPES);
    }

    #[test]
    fn check_cl_engine_test_support_version() {
        check_latest_published_casper_package_version(&CL_ENGINE_TEST_SUPPORT);
    }

    #[test]
    fn check_cl_execution_engine_version() {
        check_latest_published_casper_package_version(&CL_EXECUTION_ENGINE);
    }
}
