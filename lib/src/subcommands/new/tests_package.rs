//! Consts and functions used to generate the files comprising the "tests" package when running the
//! tool.

use super::{
    common::{self, CL_CONTRACT, CL_ENGINE_TEST_SUPPORT, CL_EXECUTION_ENGINE, CL_TYPES},
    Error, Options,
};

const PACKAGE_NAME: &str = "tests";
const INTEGRATION_TESTS_RS_CONTENTS: &str =
    include_str!("../../../resources/integration_tests.rs.in");

fn test_dependencies(options: &Options) -> String {
    format!(
        "{}{}{}{}",
        CL_CONTRACT.display_with_features(options, false, vec!["test-support"]),
        CL_ENGINE_TEST_SUPPORT.display_with_features(options, true, vec!["test-support"]),
        CL_EXECUTION_ENGINE.display_with_features(options, true, vec![]),
        CL_TYPES.display_with_features(options, true, vec![])
    )
}

fn cargo_toml_contents(options: &Options) -> String {
    format!(
        r#"[package]
name = "tests"
version = "0.1.0"
edition = "2018"

[dev-dependencies]
{}
[[bin]]
name = "integration-tests"
path = "src/integration_tests.rs"
bench = false
doctest = false

{}"#,
        test_dependencies(options),
        common::patch_section(options)
    )
}

pub fn create(options: &Options) -> Result<(), Error> {
    let root = options.root_path.join(PACKAGE_NAME);

    // Create "tests/src" folder and write test files inside.
    let tests_folder = root.join("src");
    common::create_dir_all(&tests_folder)?;

    // Write "tests/src/integration_tests.rs".
    let integration_tests_rs = tests_folder.join("integration_tests.rs");
    common::write_file(integration_tests_rs, INTEGRATION_TESTS_RS_CONTENTS)?;

    // Write "tests/Cargo.toml".
    let cargo_toml = root.join("Cargo.toml");
    common::write_file(cargo_toml, cargo_toml_contents(options))
}
