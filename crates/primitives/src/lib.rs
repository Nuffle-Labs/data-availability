#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use core::ops::Deref;

use near_primitives::hash::CryptoHash;

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
    pub fn new_v0(data: Data) -> Self {
        Self { data }
    }
}

#[serde_as]
#[derive(Deserialize, Serialize, Clone)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct BlobRef {
    /// The near transaction id the blob was included in
    /// encoded as a base58 string
    pub transaction_id: CryptoHash,
}

pub const BLOB_REF_SIZE: usize = 32;

impl BlobRef {
    pub fn new(transaction_id: CryptoHash) -> Self {
        Self { transaction_id }
    }
}

impl Deref for BlobRef {
    type Target = [u8; BLOB_REF_SIZE];
    fn deref(&self) -> &Self::Target {
        &self.transaction_id.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format() {
        let frame_ref = BlobRef::new(CryptoHash([2u8; BLOB_REF_SIZE]));
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
