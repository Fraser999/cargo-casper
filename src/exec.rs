use std::path::PathBuf;

use anyhow::bail;
use clap::{value_parser, Arg, ArgAction, ArgGroup, ArgMatches, Command};

use casper_types::{BlockHash, Digest};

use super::default_storage_dir;
use cargo_casper_lib::subcommands::exec::{Options, SnapshotId, UserProvidedOrDefault};

pub const SUBCOMMAND_NAME: &str = "exec";
const SNAPSHOT_ID_GROUP: &str = "SnapshotId";
const ABOUT: &str =
    "Execute a Casper Transaction locally, by fetching any required global state from the \
    specified network and storing that state along with any changes made to it.";

enum DisplayOrder {
    StorageDir,
    ChainName,
    NodeAddress,
    Latest,
    StateHash,
    BlockHeight,
    BlockHash,
    TransactionPath,
}

pub fn subcommand(display_order: usize) -> Command {
    Command::new(SUBCOMMAND_NAME)
        .about(ABOUT)
        .long_about(format!(
            "{ABOUT}\n\nAll options other than the Transaction path are cached and will be reused \
            on subsequent runs unless changed via the command line again. Default values for the \
            options are only applied if no corresponding cached option is available. If --{ltst} \
            is passed, the option is cached as the value of the latest state hash once that is \
            fetched from the node. If you wish to always use the latest state from the network on \
            every run, --{ltst} must be passed on the command line every time.",
            ltst = latest::ARG_NAME,
        ))
        .display_order(display_order)
        .arg(storage_dir::arg())
        .arg(chain_name::arg())
        .arg(node_address::arg())
        .arg(latest::arg())
        .arg(state_hash::arg())
        .arg(block_height::arg())
        .arg(block_hash::arg())
        .arg(transaction_path::arg())
        .group(ArgGroup::new(SNAPSHOT_ID_GROUP).required(false))
}

pub fn get_options(matches: &ArgMatches) -> anyhow::Result<Options> {
    let snapshot_id = match (
        latest::get(matches),
        state_hash::get(matches),
        block_height::get(matches),
        block_hash::get(matches),
    ) {
        (true, None, None, None) => UserProvidedOrDefault::User(SnapshotId::Latest),
        (false, Some(value), None, None) => {
            UserProvidedOrDefault::User(SnapshotId::StateHash(value))
        }
        (false, None, Some(value), None) => {
            UserProvidedOrDefault::User(SnapshotId::BlockHeight(value))
        }
        (false, None, None, Some(value)) => {
            UserProvidedOrDefault::User(SnapshotId::BlockHash(value))
        }
        (false, None, None, None) => UserProvidedOrDefault::Default(SnapshotId::Latest),
        _ => bail!(
            "should not provide more than one of --{}, --{}, --{}, or --{}",
            latest::ARG_NAME,
            state_hash::ARG_NAME,
            block_height::ARG_NAME,
            block_hash::ARG_NAME,
        ),
    };
    Ok(Options {
        storage_dir: storage_dir::get(matches),
        chain_name: chain_name::get(matches),
        node_address: node_address::get(matches),
        snapshot_id,
        transaction_path: transaction_path::get(matches),
    })
}

mod storage_dir {
    use super::*;

    const ARG_NAME: &str = "storage-dir";
    const ARG_SHORT: char = 'g';

    pub(super) fn arg() -> Arg {
        Arg::new(ARG_NAME)
            .long(ARG_NAME)
            .short(ARG_SHORT)
            .required(false)
            .display_order(DisplayOrder::StorageDir as usize)
            .value_name("DIRECTORY")
            .value_parser(value_parser!(PathBuf))
            .help(format!(
                "Path of the directory where global state will be written [default: {}]",
                default_storage_dir().display()
            ))
    }

    pub(super) fn get(matches: &ArgMatches) -> UserProvidedOrDefault<PathBuf> {
        match matches.get_one::<PathBuf>(ARG_NAME) {
            Some(value) => UserProvidedOrDefault::User(value.clone()),
            None => UserProvidedOrDefault::Default(default_storage_dir()),
        }
    }
}

mod chain_name {
    use super::*;

    const ARG_NAME: &str = "chain-name";
    const ARG_SHORT: char = 'c';
    const ARG_DEFAULT: &str = "casper-net-1";

    pub(super) fn arg() -> Arg {
        Arg::new(ARG_NAME)
            .long(ARG_NAME)
            .short(ARG_SHORT)
            .required(false)
            .display_order(DisplayOrder::ChainName as usize)
            .value_name("NAME")
            .help(format!(
                "Name of the network to query for global state [default: {}]",
                ARG_DEFAULT
            ))
    }

    pub(super) fn get(matches: &ArgMatches) -> UserProvidedOrDefault<String> {
        match matches.get_one::<String>(ARG_NAME) {
            Some(value) => UserProvidedOrDefault::User(value.clone()),
            None => UserProvidedOrDefault::Default(ARG_DEFAULT.to_string()),
        }
    }
}

mod node_address {
    use super::*;

    const ARG_NAME: &str = "node-address";
    const ARG_SHORT: char = 'n';
    const ARG_VALUE_NAME: &str = "HOST:PORT";
    const ARG_DEFAULT: &str = "http://localhost:11101";

    pub(super) fn arg() -> Arg {
        Arg::new(ARG_NAME)
            .long(ARG_NAME)
            .short(ARG_SHORT)
            .required(false)
            .display_order(DisplayOrder::NodeAddress as usize)
            .value_name(ARG_VALUE_NAME)
            .help(format!(
                "Address of the node to query for global state [default: {}]",
                ARG_DEFAULT
            ))
    }

    pub(super) fn get(matches: &ArgMatches) -> UserProvidedOrDefault<String> {
        match matches.get_one::<String>(ARG_NAME) {
            Some(value) => UserProvidedOrDefault::User(value.clone()),
            None => UserProvidedOrDefault::Default(ARG_DEFAULT.to_string()),
        }
    }
}

mod latest {
    use super::*;

    pub(super) const ARG_NAME: &str = "latest";
    const ARG_SHORT: char = 'l';
    const ARG_HELP: &str = "Use the latest global state snapshot available on the given node";

    pub(super) fn arg() -> Arg {
        Arg::new(ARG_NAME)
            .long(ARG_NAME)
            .short(ARG_SHORT)
            .required(false)
            .display_order(DisplayOrder::Latest as usize)
            .action(ArgAction::SetTrue)
            .help(ARG_HELP)
            .long_help(format!(
                "{ARG_HELP}. If none of --{ARG_NAME}, --{}, --{}, or --{} are provided and none of \
                these are cached options, then --{ARG_NAME} is used",
                state_hash::ARG_NAME,
                block_height::ARG_NAME,
                block_hash::ARG_NAME,
            ))
            .group(SNAPSHOT_ID_GROUP)
    }

    pub(super) fn get(matches: &ArgMatches) -> bool {
        matches.get_flag(ARG_NAME)
    }
}

mod state_hash {
    use super::*;

    pub(super) const ARG_NAME: &str = "state-hash";
    const ARG_SHORT: char = 's';
    const ARG_VALUE_NAME: &str = "HEX-ENCODED-DIGEST";

    pub(super) fn arg() -> Arg {
        Arg::new(ARG_NAME)
            .long(ARG_NAME)
            .short(ARG_SHORT)
            .required(false)
            .display_order(DisplayOrder::StateHash as usize)
            .value_name(ARG_VALUE_NAME)
            .value_parser(digest_from_hex)
            .help("Use the global state snapshot at the given state hash")
            .group(SNAPSHOT_ID_GROUP)
    }

    pub(super) fn get(matches: &ArgMatches) -> Option<Digest> {
        matches.get_one::<Digest>(ARG_NAME).copied()
    }
}

mod block_height {
    use super::*;

    pub(super) const ARG_NAME: &str = "block-height";
    const ARG_SHORT: char = 'b';
    const ARG_VALUE_NAME: &str = "INTEGER";

    pub(super) fn arg() -> Arg {
        Arg::new(ARG_NAME)
            .long(ARG_NAME)
            .short(ARG_SHORT)
            .required(false)
            .display_order(DisplayOrder::BlockHeight as usize)
            .value_name(ARG_VALUE_NAME)
            .value_parser(value_parser!(u64))
            .help("Use the global state snapshot at the given block height")
            .group(SNAPSHOT_ID_GROUP)
    }

    pub(super) fn get(matches: &ArgMatches) -> Option<u64> {
        matches.get_one::<u64>(ARG_NAME).cloned()
    }
}

mod block_hash {
    use super::*;

    pub(super) const ARG_NAME: &str = "block-hash";
    const ARG_VALUE_NAME: &str = "HEX-ENCODED-DIGEST";

    pub(super) fn arg() -> Arg {
        Arg::new(ARG_NAME)
            .long(ARG_NAME)
            .required(false)
            .display_order(DisplayOrder::BlockHash as usize)
            .value_name(ARG_VALUE_NAME)
            .value_parser(block_hash_from_hex)
            .help("Use the global state snapshot at the given block hash")
            .group(SNAPSHOT_ID_GROUP)
    }

    pub(super) fn get(matches: &ArgMatches) -> Option<BlockHash> {
        matches.get_one::<BlockHash>(ARG_NAME).copied()
    }
}

mod transaction_path {
    use super::*;

    const ARG_NAME: &str = "transaction-path";
    const ARG_VALUE_NAME: &str = "FILE";

    pub(super) fn arg() -> Arg {
        Arg::new(ARG_NAME)
            .required(true)
            .display_order(DisplayOrder::TransactionPath as usize)
            .value_name(ARG_VALUE_NAME)
            .value_parser(value_parser!(PathBuf))
            .help("Path of the JSON-encoded Transaction/Deploy file to execute")
    }

    pub(super) fn get(matches: &ArgMatches) -> PathBuf {
        matches.get_one::<PathBuf>(ARG_NAME).unwrap().clone()
    }
}

fn digest_from_hex(input: &str) -> Result<Digest, String> {
    Digest::from_hex(input)
        .map_err(|error| format!("Failed to parse as a hex-encoded hash Digest: {}", error))
}

fn block_hash_from_hex(input: &str) -> Result<BlockHash, String> {
    Ok(BlockHash::from(digest_from_hex(input)?))
}
