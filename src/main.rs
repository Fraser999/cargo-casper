//! Command line tool for creating and testing a Casper Wasm contract.

#![warn(
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_qualifications
)]

mod clean;
mod exec;
mod new;
mod show;

use std::path::PathBuf;
use std::{env, process};

use anyhow::{anyhow, bail};
use clap::{crate_description, crate_name, crate_version, Command};
use colour::{e_prnt_ln, e_red};
use directories::ProjectDirs;

const FAILURE_EXIT_CODE: i32 = 101;

enum DisplayOrder {
    New,
    Exec,
    Show,
    Clean,
}

fn main() {
    let result = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(run_main());
    if let Err(error) = result {
        e_red!("error");
        e_prnt_ln!(": {error}");
        process::exit(FAILURE_EXIT_CODE)
    }
}

async fn run_main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let arg_matches = command().get_matches();
    let (subcommand_name, matches) = arg_matches.subcommand().ok_or_else(|| {
        let _ = command().print_long_help();
        anyhow!("failed to get subcommand")
    })?;

    match subcommand_name {
        new::SUBCOMMAND_NAME => Ok(new::get_options(matches).run()?),
        exec::SUBCOMMAND_NAME => Ok(exec::get_options(matches)?.run()?),
        show::SUBCOMMAND_NAME => Ok(show::get_options(matches).run()?),
        clean::SUBCOMMAND_NAME => Ok(cargo_casper_lib::subcommands::clean::run()?),
        _ => bail!("{} is not a valid subcommand", subcommand_name),
    }
}

fn command() -> Command {
    Command::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .subcommand(new::subcommand(DisplayOrder::New as usize))
        .subcommand(exec::subcommand(DisplayOrder::Exec as usize))
        .subcommand(show::subcommand(DisplayOrder::Show as usize))
        .subcommand(clean::subcommand(DisplayOrder::Clean as usize))
}

fn default_storage_dir() -> PathBuf {
    if let Some(project_dir) = ProjectDirs::from("", "Casper Labs", crate_name!()) {
        return project_dir.data_dir().to_path_buf();
    }
    env::temp_dir().join(crate_name!())
}
