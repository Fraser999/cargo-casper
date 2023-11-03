mod error;
mod state;

use std::{path::PathBuf, str};

use log::{debug, info, trace};
use serde::Deserialize;
use tokio::runtime::Handle;

use casper_client::{rpcs::common::BlockIdentifier, JsonRpcId, Verbosity};
use casper_execution_engine::engine_state::{
    DeployItem, EngineConfig, EngineState, ExecuteRequest, ExecutionResult,
};
use casper_types::{
    account::AccountHash, execution::ExecutionResultV2, BlockHash, CoreConfig, DeployConfig,
    Digest, Key, NetworkConfig, ProtocolConfig, ProtocolVersion, PublicKey, Timestamp,
};

use crate::{CachedConfig, Storage};
pub use error::Error;
use state::State;

/// An option which was either provided by the user on the command line, or a default value to use
/// if no corresponding cached option is available.
#[derive(Debug)]
pub enum UserProvidedOrDefault<T> {
    /// The user provided the given value.
    User(T),
    /// The user did not provide a value, and this default should be considered.
    Default(T),
}

impl<T> UserProvidedOrDefault<T> {
    /// The wrapped value.
    pub fn value(self) -> T {
        match self {
            UserProvidedOrDefault::User(value) | UserProvidedOrDefault::Default(value) => value,
        }
    }
}

/// Identifier for a snapshot of global state.
#[derive(Debug)]
pub enum SnapshotId {
    /// The latest known by the node.
    Latest,
    /// The given hash.
    StateHash(Digest),
    /// The state hash specified in the block at the given height.
    BlockHeight(u64),
    /// The state hash specified in the block with the given BlockHash.
    BlockHash(BlockHash),
}

/// Options for the `exec` subcommand.
#[derive(Debug)]
pub struct Options {
    /// The directory used for storing fetched global state.
    pub storage_dir: UserProvidedOrDefault<PathBuf>,
    /// The network chain name.
    pub chain_name: UserProvidedOrDefault<String>,
    /// The address of a node on the network for querying, in the form "host:port".
    pub node_address: UserProvidedOrDefault<String>,
    /// The identifier of the global state to be used.
    pub snapshot_id: UserProvidedOrDefault<SnapshotId>,
    /// The transaction/deploy to execute.
    pub transaction_path: PathBuf,
}

impl Options {
    /// Executes the `exec` subcommand with the provided options.
    pub fn run(self) -> Result<(), Error> {
        let Options {
            storage_dir,
            chain_name,
            node_address,
            snapshot_id,
            transaction_path,
        } = self;
        let runtime = Handle::current();
        let cached_config = match CachedConfig::try_read()? {
            Some(mut config) => {
                // Overwrite cached values with any provided by the user on this run.
                if let UserProvidedOrDefault::User(storage_dir) = storage_dir {
                    config.storage_dir = storage_dir;
                }
                if let UserProvidedOrDefault::User(chain_name) = chain_name {
                    config.chain_name = chain_name;
                }
                if let UserProvidedOrDefault::User(node_address) = node_address {
                    config.node_address = node_address;
                }
                if let UserProvidedOrDefault::User(snapshot_id) = snapshot_id {
                    let addr = config.node_address.clone();
                    config.state_hash = get_state_root_hash(addr, snapshot_id, runtime)?;
                }

                config
            }
            None => {
                // If there's no cached values, just use whatever we got from CLI; either user-input
                // or defaults.
                let node_address = node_address.value();
                let state_hash =
                    get_state_root_hash(node_address.clone(), snapshot_id.value(), runtime)?;
                CachedConfig {
                    storage_dir: storage_dir.value(),
                    chain_name: chain_name.value(),
                    node_address,
                    state_hash,
                }
            }
        };

        // Save the updated options.
        cached_config.write()?;

        let chainspec = get_chainspec(&cached_config.node_address, Handle::current())?;
        println!("{chainspec:?}");

        let state_hash = cached_config.state_hash;

        // Try to read in the Transaction.
        let transaction = casper_client::read_deploy_file(&transaction_path).map_err(|error| {
            Error::ReadTransaction {
                error,
                path: transaction_path.clone(),
            }
        })?;

        // Check it's config compliant - this is checked by the deploy/transaction acceptor on the node.
        transaction.is_config_compliant(
            &cached_config.chain_name,
            &DeployConfig::default(), // TODO - get from chainspec
            100,                      // TODO - get from chainspec
            transaction.timestamp(),
        )?;

        let account_key = Key::from(AccountHash::from(transaction.header().account()));

        // Construct the EE.
        let storage = Storage::new(
            &cached_config.storage_dir,
            &cached_config.chain_name,
            &state_hash,
            true,
        )?;
        let state = State::new(
            state_hash,
            storage.clone(),
            cached_config.node_address.clone(),
        );
        let engine_config = EngineConfig::default(); // TODO - get from chainspec
        let engine_state = EngineState::new(state.clone(), engine_config);

        // Execute the Transaction.
        let deploy_item = DeployItem::from(transaction);
        let execute_request = ExecuteRequest::new(
            state_hash,
            Timestamp::now().millis(),
            vec![deploy_item],
            ProtocolVersion::V1_0_0, // TODO - get from chainspec
            PublicKey::System,       // TODO - does this have issues?
        );
        let results = engine_state
            .run_execute(execute_request)
            .map_err(Error::Execution)?;
        assert_eq!(results.len(), 1, "should only be one execution result");
        let result = results.front().unwrap();
        trace!(
            "execution result: {}",
            serde_json::to_string_pretty(&ExecutionResultV2::from(result.clone())).unwrap()
        );
        match result {
            ExecutionResult::Failure { cost, error, .. } => {
                info!("execution failed with cost: {}, error: {}", cost, error)
            }
            ExecutionResult::Success { cost, .. } => {
                info!("execution succeeded with cost: {}", cost)
            }
        }

        // Save the changes to global state.
        let _ = engine_state
            .apply_effects(state_hash, result.effects().clone())
            .map_err(Error::Commit)?;
        storage.persist()?;

        if let Some(account) = storage
            .get(&account_key)
            .and_then(|stored_value| stored_value.as_account().cloned())
        {
            info!(
                "account after execution:\n{}",
                serde_json::to_string_pretty(&account).unwrap()
            );
        }

        Ok(())
    }
}

fn rpc_id() -> JsonRpcId {
    JsonRpcId::Number(0)
}

/// If the user provided a snapshot ID of a block or "latest", get the state hash from the node.  If
/// they provided a state hash, just return that.
fn get_state_root_hash(
    node_address: String,
    snapshot_id: SnapshotId,
    runtime: Handle,
) -> Result<Digest, Error> {
    let maybe_block_id = match snapshot_id {
        SnapshotId::Latest => None,
        SnapshotId::StateHash(state_hash) => return Ok(state_hash),
        SnapshotId::BlockHeight(height) => Some(BlockIdentifier::Height(height)),
        SnapshotId::BlockHash(block_hash) => Some(BlockIdentifier::Hash(block_hash)),
    };

    let join_handle = std::thread::spawn(move || {
        runtime.block_on(async {
            casper_client::get_state_root_hash(
                rpc_id(),
                &node_address,
                Verbosity::Low,
                maybe_block_id,
            )
            .await
        })
    });

    let response = join_handle
        .join()
        .expect("thread should join")
        .map_err(Error::FailedToGetStateHash)?;
    debug!("got state root hash response: {:?}", response);
    response
        .result
        .state_root_hash
        .ok_or(Error::UnknownStateHash)
}

#[derive(PartialEq, Eq, Deserialize, Debug)]
struct Chainspec {
    #[serde(rename = "protocol")]
    protocol_config: ProtocolConfig,
    #[serde(rename = "network")]
    network_config: NetworkConfig,
    #[serde(rename = "core")]
    core_config: CoreConfig,
    #[serde(rename = "deploys")]
    deploy_config: DeployConfig,
}

fn get_chainspec(node_address: &str, runtime: Handle) -> Result<Chainspec, Error> {
    let join_handle = std::thread::spawn(move || {
        runtime.block_on(async {
            casper_client::get_chainspec(rpc_id(), &node_address, Verbosity::Low).await
        })
    });

    let response = join_handle
        .join()
        .expect("thread should join")
        .map_err(Error::FailedToGetStateHash)?;
    debug!("got chainspec response: {:?}", response);
    let chainspec_str = str::from_utf8(response.result.chainspec_bytes.chainspec_bytes())
        .map_err(Error::ChainspecBytesToStr)?;
    let chainspec: Chainspec =
        toml::from_str(chainspec_str).map_err(Error::ChainspecDeserialization)?;
    Ok(chainspec)
}
