//! Consts and functions used to generate the files comprising the "contract" package when running
//! the tool.

use super::{
    common::{self, CL_CONTRACT, CL_TYPES},
    Error, Options,
};

const PACKAGE_NAME: &str = "contract";
const CONFIG_TOML_CONTENTS: &str = r#"[build]
target = "wasm32-unknown-unknown"
"#;
const MAIN_RS_CONTENTS: &str = include_str!("../../../resources/main.rs.in");

fn contract_dependencies(options: &Options) -> String {
    format!(
        "{}{}",
        CL_CONTRACT.display_with_features(options, true, vec![]),
        CL_TYPES.display_with_features(options, true, vec![]),
    )
}

fn cargo_toml_contents(options: &Options) -> String {
    format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2018"

[dependencies]
{}
[[bin]]
name = "{}"
path = "src/main.rs"
bench = false
doctest = false
test = false

[profile.release]
codegen-units = 1
lto = true

{}"#,
        PACKAGE_NAME,
        contract_dependencies(options),
        PACKAGE_NAME.replace('-', "_"),
        common::patch_section(options)
    )
}

pub fn create(options: &Options) -> Result<(), Error> {
    let root = options.root_path.join(PACKAGE_NAME.replace('-', "_"));

    // Create "<PACKAGE_NAME>/src" folder and write "main.rs" inside.
    let src_folder = root.join("src");
    common::create_dir_all(&src_folder)?;
    let main_rs = src_folder.join("main.rs");
    common::write_file(main_rs, MAIN_RS_CONTENTS)?;

    // Create "<PACKAGE_NAME>/.cargo" folder and write "config.toml" inside.
    let config_folder = root.join(".cargo");
    common::create_dir_all(&config_folder)?;
    let config_toml = config_folder.join("config.toml");
    common::write_file(config_toml, CONFIG_TOML_CONTENTS)?;

    // Write "<PACKAGE_NAME>/Cargo.toml".
    let cargo_toml = root.join("Cargo.toml");
    common::write_file(cargo_toml, cargo_toml_contents(options))
}
