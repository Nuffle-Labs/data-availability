use std::{collections::HashMap, default};

use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    log,
    serde::{Deserialize, Serialize},
    store::{LookupMap, UnorderedMap},
    BlockHeight,
};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault};

// TODO: optimise for storage, flat store commitment -> blob
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    pub blobs: LookupMap<String, UnorderedMap<BlockHeight, Blob>>,
}

impl default::Default for Contract {
    fn default() -> Self {
        Self {
            blobs: LookupMap::new(b"b".to_vec()),
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Blob {
    namespace: String,
    data: String,
    share_version: u64,
    commitment: String,
}

#[near_bindgen]
impl Contract {
    pub fn submit(&mut self, blobs: Vec<Blob>) -> BlockHeight {
        let height = env::block_height();

        // TODO: optimise later
        for blob in blobs {
            let map = self
                .blobs
                .entry(blob.namespace.clone())
                .or_insert(UnorderedMap::new(
                    [blob.namespace.as_bytes(), &height.to_le_bytes()[..]].concat(),
                ));
            map.insert(height, blob);
        }
        height
    }
    pub fn clear(&mut self, ns: Vec<String>) {
        if env::predecessor_account_id().as_str() != "datayalla.testnet" {
            panic!("Only datayalla can clear blobs");
        }

        for ns in ns {
            let inner = self.blobs.get_mut(&ns);
            if let Some(inner) = inner {
                inner.clear()
            }
            self.blobs.remove(&ns);
        }
    }

    pub fn get(&self, namespace: String, height: BlockHeight) -> Blob {
        self.blobs
            .get(&namespace)
            .map(|x| x.get(&height))
            .flatten()
            .unwrap()
            .clone()
    }

    pub fn get_all(&self, namespaces: Vec<String>, height: BlockHeight) -> Vec<Blob> {
        let mut blobs = Vec::new();
        for namespace in namespaces {
            blobs.push(
                self.blobs
                    .get(&namespace)
                    .map(|x| x.get(&height))
                    .flatten()
                    .unwrap()
                    .clone(),
            );
        }
        blobs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initializes() {
        let contract = Contract::default();
    }
}
