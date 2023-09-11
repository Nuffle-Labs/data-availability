use near_da_primitives::{Blob as ExternalBlob, ShareVersion};
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    log,
    store::{LookupMap, UnorderedMap},
    BlockHeight,
};
use near_sdk::{env, near_bindgen};
use near_sdk_contract_tools::owner::OwnerExternal;
use near_sdk_contract_tools::{owner::Owner, Owner};
use std::vec::Vec;
use std::{collections::HashMap, default};

type Namespace = [u8; 32];
type Commitment = [u8; 32];

#[near_bindgen]
#[derive(Owner, BorshDeserialize, BorshSerialize)]
pub struct Contract {
    pub blobs: UnorderedMap<Commitment, Blob>,
    pub indices: LookupMap<Namespace, HashMap<BlockHeight, Commitment>>,
}

impl default::Default for Contract {
    fn default() -> Self {
        let mut contract = Self {
            indices: LookupMap::new(b"i".to_vec()),
            blobs: UnorderedMap::new(b"b"),
        };
        Owner::init(&mut contract, &near_sdk::env::predecessor_account_id());

        contract
    }
}

#[derive(BorshDeserialize, BorshSerialize, Clone)]
pub struct Blob {
    data: Vec<u8>,
    // Keep track of these to make sure there are no breaking changes which
    // might bork the store
    share_version: ShareVersion,
}

impl TryFrom<ExternalBlob> for Blob {
    type Error = ();
    fn try_from(value: ExternalBlob) -> Result<Self, Self::Error> {
        Ok(Blob {
            data: value.data.try_to_vec().map_err(|_| ())?,
            share_version: value.share_version,
        })
    }
}

#[near_bindgen]
impl Contract {
    pub fn submit(&mut self, blobs: Vec<ExternalBlob>) -> BlockHeight {
        Self::require_owner();

        let height = env::block_height();

        for blob in blobs {
            log!(
                "Submitting blob for namespace {:?} at height {}",
                blob.namespace,
                height
            );
            let map = self
                .indices
                .entry(blob.namespace.clone())
                .or_insert(HashMap::default());
            map.insert(height, blob.commitment);
            self.blobs.insert(
                blob.commitment,
                blob.try_into().expect("Failed to write blob to store"),
            );
        }
        height
    }
    pub fn clear(&mut self, ns: Vec<Namespace>) {
        Self::require_owner();

        for ns in ns {
            let inner = self.indices.get_mut(&ns);
            if let Some(inner) = inner {
                inner.iter().for_each(|(_, commitment)| {
                    self.blobs.remove(commitment);
                });
            }
            self.indices.remove(&ns);
        }
    }

    pub fn get(&self, namespace: Namespace, height: BlockHeight) -> Option<ExternalBlob> {
        self.indices
            .get(&namespace)
            .and_then(|x| x.get(&height))
            .and_then(|commitment| self.blobs.get(commitment).map(|x| (commitment, x)))
            .and_then(|(commitment, inner)| {
                Some(ExternalBlob {
                    namespace,
                    data: BorshDeserialize::try_from_slice(&inner.data).ok()?,
                    share_version: inner.share_version,
                    commitment: commitment.clone(),
                })
            })
            .clone()
    }

    pub fn get_all(&self, namespaces: Vec<Namespace>, height: BlockHeight) -> Vec<ExternalBlob> {
        let mut blobs = Vec::new();
        for namespace in namespaces {
            self.indices
                .get(&namespace)
                .and_then(|x| x.get(&height))
                .and_then(|commitment| self.blobs.get(commitment).map(|x| (commitment, x)))
                .and_then(|(commitment, inner)| {
                    Some(ExternalBlob {
                        namespace,
                        data: BorshDeserialize::try_from_slice(&inner.data).ok()?,
                        share_version: inner.share_version,
                        commitment: commitment.clone(),
                    })
                })
                .map(|blob| blobs.push(blob.clone()));
        }
        blobs
    }

    // Shortcut read if you already know the namespace of the commitment
    pub fn fast_get(&self, commitment: Commitment) -> Option<ExternalBlob> {
        self.blobs.get(&commitment).and_then(|inner| {
            Some(ExternalBlob {
                namespace: [0_u8; 32],
                data: BorshDeserialize::try_from_slice(&inner.data).ok()?,
                share_version: inner.share_version,
                commitment: commitment.clone(),
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initializes() {
        let _ = Contract::default();
    }

    #[test]
    fn test_submit_indices() {
        let mut contract = Contract::default();
        let blobs = vec![ExternalBlob {
            namespace: [1_u8; 32],
            data: "fake".try_to_vec().unwrap(),
            share_version: 1,
            commitment: [2_u8; 32],
        }];
        let height = contract.submit(blobs.clone());
        assert_eq!(height, 0);
        assert_eq!(contract.blobs.len(), 1);
        let height_commitment = contract.indices.get(&blobs[0].namespace).unwrap();
        assert_eq!(height_commitment.len(), 1);
        assert_eq!(height_commitment.get(&0).unwrap(), &[2_u8; 32]);
    }

    #[test]
    fn test_remove() {
        let mut contract = Contract::default();
        let blobs = vec![ExternalBlob {
            namespace: [1_u8; 32],
            data: "fake".try_to_vec().unwrap(),
            share_version: 1,
            commitment: [2_u8; 32],
        }];
        contract.submit(blobs.clone());
        contract.clear(vec![blobs[0].namespace]);
        assert_eq!(contract.blobs.len(), 0);
        assert!(contract.indices.get(&blobs[0].namespace).is_none());
    }
}
