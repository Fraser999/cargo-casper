//! Command line tool for creating and testing a Casper Wasm contract.

#![warn(
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_qualifications
)]

mod new;

use std::{env, process};

use anyhow::{anyhow, bail};
use clap::{crate_description, crate_name, crate_version, Command};
use colour::{e_prnt_ln, e_red};

const FAILURE_EXIT_CODE: i32 = 101;

enum DisplayOrder {
    New,
}

fn main() {
    if let Err(error) = run_main() {
        e_red!("error");
        e_prnt_ln!(": {error}");
        process::exit(FAILURE_EXIT_CODE)
    }
}

fn run_main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let arg_matches = command().get_matches();
    let (subcommand_name, matches) = arg_matches.subcommand().ok_or_else(|| {
        let _ = command().print_long_help();
        anyhow!("failed to get subcommand")
    })?;

    match subcommand_name {
        new::SUBCOMMAND_NAME => Ok(new::get_options(matches).run()?),
        _ => bail!("{} is not a valid subcommand", subcommand_name),
    }
}

fn command() -> Command {
    Command::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .subcommand(new::subcommand(DisplayOrder::New as usize))
}
