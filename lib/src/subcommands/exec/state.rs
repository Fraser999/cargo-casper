use std::{
    collections::VecDeque,
    fmt::{self, Display, Formatter},
};

use log::{debug, error};
use tokio::runtime::Handle;

use casper_client::{rpcs::GlobalStateIdentifier, Error as ClientError, JsonRpcId, Verbosity};
use casper_storage::global_state::{
    error::Error as GlobalStateError,
    state::{CommitProvider, StateProvider, StateReader},
    trie::{merkle_proof::TrieMerkleProof, TrieRaw},
    trie_store::operations::DeleteResult,
};
use casper_types::{bytesrepr, execution::ExecutionJournal, Digest, Key, StoredValue};

use crate::Storage;

/// Struct which implements the `casper_storage` traits to allow its use in EE's `EngineState`.
#[derive(Clone)]
pub(super) struct State {
    state_hash: Digest,
    storage: Storage,
    node_address: String,
}

impl State {
    pub(super) fn new(state_hash: Digest, storage: Storage, node_address: String) -> Self {
        Self {
            state_hash,
            storage,
            node_address,
        }
    }
}

impl StateReader<Key, StoredValue> for State {
    type Error = GlobalStateError;

    fn read(&self, key: &Key) -> Result<Option<StoredValue>, Self::Error> {
        if let Some(value) = self.storage.get(key) {
            debug!("got cached: {key}");
            return Ok(Some(value));
        }

        let handle = Handle::current();
        let addr = self.node_address.clone();
        let state_hash = self.state_hash;
        let key = *key;
        let join_handle = std::thread::spawn(move || {
            handle.block_on(async {
                casper_client::query_global_state(
                    JsonRpcId::Number(1),
                    &addr,
                    Verbosity::Low,
                    GlobalStateIdentifier::StateRootHash(state_hash.value().into()),
                    key,
                    vec![],
                )
                .await
            })
        });

        let value = match join_handle.join().unwrap() {
            Ok(response) => {
                debug!("got remote: {key}");
                response.result.stored_value
            }
            Err(ClientError::ResponseIsRpcError { error, .. }) if error.code == -32003 => {
                // Value not found in global state.
                debug!("{:?}", error);
                return Ok(None);
            }
            Err(error) => {
                error!("{:?}", error);
                // This is not ideal, but we don't have much flexibility as to the error type.
                return Err(GlobalStateError::BytesRepr(
                    bytesrepr::Error::NotRepresentable,
                ));
            }
        };

        self.storage.insert(key, value.clone());
        Ok(Some(value))
    }

    fn read_with_proof(
        &self,
        key: &Key,
    ) -> Result<Option<TrieMerkleProof<Key, StoredValue>>, Self::Error> {
        Ok(self
            .read(key)?
            .map(|value| TrieMerkleProof::new(*key, value, VecDeque::new())))
    }

    fn keys_with_prefix(&self, _prefix: &[u8]) -> Result<Vec<Key>, Self::Error> {
        unimplemented!()
    }
}

impl StateProvider for State {
    type Error = GlobalStateError;
    type Reader = Self;

    fn checkout(&self, state_hash: Digest) -> Result<Option<Self::Reader>, Self::Error> {
        assert_eq!(state_hash, self.state_hash);
        Ok(Some(self.clone()))
    }

    fn empty_root(&self) -> Digest {
        Digest::default()
    }

    fn get_trie_full(&self, _trie_key: &Digest) -> Result<Option<TrieRaw>, Self::Error> {
        unimplemented!()
    }

    /// Insert a trie node into the trie
    fn put_trie(&self, _trie: &[u8]) -> Result<Digest, Self::Error> {
        unimplemented!()
    }

    fn missing_children(&self, _trie_raw: &[u8]) -> Result<Vec<Digest>, Self::Error> {
        unimplemented!()
    }

    fn delete_keys(
        &self,
        _root: Digest,
        _keys_to_delete: &[Key],
    ) -> Result<DeleteResult, Self::Error> {
        unimplemented!()
    }
}

impl CommitProvider for State {
    fn commit(&self, state_hash: Digest, effects: ExecutionJournal) -> Result<Digest, Self::Error> {
        assert_eq!(state_hash, self.state_hash);
        self.storage.commit(effects)?;
        Ok(Digest::default())
    }
}

impl Display for State {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.storage, formatter)
    }
}
