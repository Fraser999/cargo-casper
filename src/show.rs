use casper_types::Key;
use clap::{Arg, ArgAction, ArgGroup, ArgMatches, Command};

use cargo_casper_lib::subcommands::show::Options;

pub const SUBCOMMAND_NAME: &str = "show";
const GROUP_NAME: &str = "grp";
const ABOUT: &str =
    "Show a value in global state, or all global state, or show the cached config if no other \
    option specified.";

enum DisplayOrder {
    Key,
    All,
}

pub fn subcommand(display_order: usize) -> Command {
    Command::new(SUBCOMMAND_NAME)
        .about(ABOUT)
        .long_about(format!(
            "{ABOUT}\n\nNo requests are sent to the network: only locally-held global state is \
            queried.",
        ))
        .display_order(display_order)
        .arg(key::arg())
        .arg(all::arg())
        .group(ArgGroup::new(GROUP_NAME).required(false))
}

pub(super) fn get_options(matches: &ArgMatches) -> Options {
    if let Some(key) = key::get(matches) {
        Options::Value(key)
    } else if all::get(matches) {
        Options::AllState
    } else {
        Options::CachedConfig
    }
}

mod key {
    use super::*;

    const ARG_NAME: &str = "key";
    const ARG_SHORT: char = 'k';

    pub(super) fn arg() -> Arg {
        Arg::new(ARG_NAME)
            .long(ARG_NAME)
            .short(ARG_SHORT)
            .required(false)
            .display_order(DisplayOrder::Key as usize)
            .value_name("FORMATTED-KEY")
            .value_parser(key_from_formatted_str)
            .help("Show the value under the given key if available locally")
            .group(GROUP_NAME)
    }

    pub(super) fn get(matches: &ArgMatches) -> Option<Key> {
        matches.get_one::<Key>(ARG_NAME).copied()
    }

    fn key_from_formatted_str(input: &str) -> Result<Key, String> {
        Key::from_formatted_str(input)
            .map_err(|error| format!("failed to parse as a Key: {}", error))
    }
}

mod all {
    use super::*;

    const ARG_NAME: &str = "all";
    const ARG_SHORT: char = 'a';

    pub(super) fn arg() -> Arg {
        Arg::new(ARG_NAME)
            .long(ARG_NAME)
            .short(ARG_SHORT)
            .required(false)
            .display_order(DisplayOrder::All as usize)
            .action(ArgAction::SetTrue)
            .help("Show all key-value pairs available locally")
            .group(GROUP_NAME)
    }

    pub(super) fn get(matches: &ArgMatches) -> bool {
        matches.get_flag(ARG_NAME)
    }
}
