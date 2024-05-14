#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use core::ops::Deref;

pub type Data = alloc::vec::Vec<u8>;
pub type ShareVersion = u32;
pub type Commitment = [u8; 32];
pub type BlockHeight = u64;

/// The namespace is a reference to who is submitting blobs, it will be considered
/// important in the blob registry. This allows users not familiar with NEAR to use a shared
/// contract, with shared proving capabilities.
///
/// TODO: optional namespace for users who submit their own blobs to their own contract
#[derive(
    Clone,
    Copy,
    BorshSerialize,
    BorshDeserialize,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Default,
    Serialize,
    Deserialize,
    Debug,
)]
pub struct Namespace {
    pub version: u8,
    pub id: u32,
}

impl Namespace {
    pub fn new(version: u8, id: u32) -> Self {
        Self { version, id }
    }
}

#[serde_as]
#[derive(Deserialize, Serialize, BorshSerialize, BorshDeserialize, Clone, Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct Blob {
    #[serde_as(as = "serde_with::hex::Hex")]
    pub data: Data,
}

impl Blob {
    pub fn new(data: Data) -> Self {
        Self { data }
    }
}

impl From<Data> for Blob {
    fn from(data: Data) -> Self {
        Self { data }
    }
}

impl From<LegacyBlob> for Blob {
    fn from(legacy_blob: LegacyBlob) -> Self {
        Self {
            data: legacy_blob.data,
        }
    }
}

#[serde_as]
#[derive(Deserialize, Serialize, BorshSerialize, BorshDeserialize, Clone, Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct LegacyBlob {
    pub namespace: Namespace,
    pub share_version: u32,
    #[serde_as(as = "serde_with::hex::Hex")]
    pub commitment: [u8; 32],
    #[serde_as(as = "serde_with::hex::Hex")]
    pub data: Data,
}

#[serde_as]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlobRef {
    #[serde_as(as = "serde_with::hex::Hex")]
    pub transaction_id: [u8; 32],
}

impl From<[u8; 32]> for BlobRef {
    fn from(transaction_id: [u8; 32]) -> Self {
        Self { transaction_id }
    }
}

pub const BLOB_REF_SIZE: usize = 32;

impl BlobRef {
    pub fn new(transaction_id: [u8; BLOB_REF_SIZE]) -> Self {
        Self { transaction_id }
    }
}

impl Deref for BlobRef {
    type Target = [u8; BLOB_REF_SIZE];
    fn deref(&self) -> &Self::Target {
        &self.transaction_id
    }
}

#[serde_with::serde_as]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Clone, Debug)]
pub struct SubmitRequest {
    pub namespace: Option<Namespace>,
    #[serde_as(as = "serde_with::hex::Hex")]
    pub data: Vec<u8>,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, Default)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    /// Wait for
    /// - Inclusion in the block, but not finalized
    Optimistic,
    /// Wait for
    /// - Transaction execution, but additional receipts/refunds were not included
    Standard,
    /// Wait for
    /// - Inclusion in the block
    /// - Execution of the blob (even though theres no execution)
    /// - All other shards execute
    #[default]
    Pessimistic,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format() {
        let frame_ref = BlobRef::new([2u8; BLOB_REF_SIZE]);
        assert_eq!(
            *frame_ref,
            [
                2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
                2, 2, 2, 2
            ],
            "FrameRef::to_celestia_format() should return 40 bytes array"
        );
    }
}
