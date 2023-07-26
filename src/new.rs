use std::path::PathBuf;

use clap::{builder::ValueParser, Arg, ArgMatches, Command};

use cargo_casper_lib::subcommands::new::{CasperOverrides, Options};

pub const SUBCOMMAND_NAME: &str = "new";
const ABOUT: &str = "Create a new Casper contract and test suite";

fn long_about() -> String {
    format!(
        "{ABOUT}\n\nThis produces a Rust project containing two crates: the contract and a test \
        suite for it. There is also a generated Makefile to simplify compiling the contract and \
        tests."
    )
}

fn usage() -> String {
    format!(
        "cargo casper {SUBCOMMAND_NAME} <DIRECTORY>\n       \
        cd <DIRECTORY>\n       \
        make prepare\n       \
        make test"
    )
}

pub fn subcommand(display_order: usize) -> Command {
    Command::new(SUBCOMMAND_NAME)
        .about(ABOUT)
        .override_usage(usage())
        .long_about(long_about())
        .display_order(display_order)
        .arg(root_path::arg())
        .arg(workspace_path::arg())
        .arg(git_url::arg())
        .arg(git_branch::arg())
}

pub fn get_options(matches: &ArgMatches) -> Options {
    let root_path = root_path::get(matches);
    let maybe_workspace_path = workspace_path::get(matches);
    let maybe_git_url = git_url::get(matches);
    let maybe_git_branch = git_branch::get(matches);

    let casper_overrides = match (maybe_workspace_path, maybe_git_url, maybe_git_branch) {
        (Some(path), None, None) => Some(CasperOverrides::WorkspacePath(path)),
        (None, Some(url), Some(branch)) => Some(CasperOverrides::GitRepo {
            url: url.to_string(),
            branch: branch.to_string(),
        }),
        (None, None, None) => None,
        _ => unreachable!("Clap rules enforce either both or neither git args are present"),
    };

    Options {
        root_path,
        casper_overrides,
    }
}

mod root_path {
    use super::*;

    const ARG_NAME: &str = "root-path";
    const ARG_VALUE_NAME: &str = "DIRECTORY";

    pub(super) fn arg() -> Arg {
        Arg::new(ARG_NAME)
            .required(true)
            .value_name(ARG_VALUE_NAME)
            .value_parser(ValueParser::path_buf())
            .help("Path to new folder for contract and tests")
    }

    pub(super) fn get(matches: &ArgMatches) -> PathBuf {
        matches.get_one::<PathBuf>(ARG_NAME).unwrap().clone()
    }
}

mod workspace_path {
    use super::*;

    pub(super) const ARG_NAME: &str = "workspace-path";

    pub(super) fn arg() -> Arg {
        Arg::new(ARG_NAME)
            .hide(true)
            .long(ARG_NAME)
            .value_parser(ValueParser::path_buf())
    }

    pub(super) fn get(matches: &ArgMatches) -> Option<PathBuf> {
        matches.get_one::<PathBuf>(ARG_NAME).cloned()
    }
}

mod git_url {
    use super::*;

    pub(super) const ARG_NAME: &str = "git-url";

    pub(super) fn arg() -> Arg {
        Arg::new(ARG_NAME)
            .hide(true)
            .long(ARG_NAME)
            .conflicts_with(workspace_path::ARG_NAME)
            .requires(git_branch::ARG_NAME)
    }

    pub(super) fn get(matches: &ArgMatches) -> Option<String> {
        matches.get_one::<String>(ARG_NAME).cloned()
    }
}

mod git_branch {
    use super::*;

    pub(super) const ARG_NAME: &str = "git-branch";

    pub(super) fn arg() -> Arg {
        Arg::new(ARG_NAME)
            .hide(true)
            .long(ARG_NAME)
            .conflicts_with(workspace_path::ARG_NAME)
            .requires(git_url::ARG_NAME)
    }

    pub(super) fn get(matches: &ArgMatches) -> Option<String> {
        matches.get_one::<String>(ARG_NAME).cloned()
    }
}
