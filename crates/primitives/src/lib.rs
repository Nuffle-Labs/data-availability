#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::string::String;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[cfg(feature = "crypto")]
use core::str::FromStr;
#[cfg(feature = "crypto")]
use near_primitives::hash::CryptoHash;

pub type Data = alloc::vec::Vec<u8>;
pub type ShareVersion = u32;
pub type Commitment = [u8; 32];
pub type BlockHeight = u64;

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

// TODO: docs
// TODO: tests
//

#[serde_as]
#[derive(Deserialize, Serialize, BorshSerialize, BorshDeserialize, Clone, Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct Blob {
    pub namespace: Namespace,
    pub share_version: ShareVersion,
    #[serde_as(as = "serde_with::hex::Hex")]
    pub commitment: [u8; 32],
    #[serde_as(as = "serde_with::hex::Hex")]
    pub data: Data,
}

impl Blob {
    pub fn new_v0(namespace: Namespace, data: Data) -> Self {
        #[cfg(not(feature = "crypto"))]
        let commitment = [0u8; 32];

        #[cfg(feature = "crypto")]
        let commitment = {
            let chunks: Vec<Vec<u8>> = data.chunks(256).map(|x| x.to_vec()).collect();
            near_primitives::merkle::merklize(&chunks).0 .0
        };
        Self {
            namespace,
            data,
            share_version: 0,
            commitment,
        }
    }
}

#[serde_as]
#[derive(Deserialize, Serialize, Clone)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct FrameRef {
    pub transaction_id: String,
    pub commitment: Commitment,
}

impl FrameRef {
    #[cfg(feature = "crypto")]
    pub fn new(transaction_id: String, commitment: Commitment) -> Self {
        Self {
            transaction_id,
            commitment,
        }
    }
    #[cfg(feature = "crypto")]
    pub fn to_celestia_format(&self) -> [u8; 64] {
        let mut result = [0u8; 64];
        let hash = CryptoHash::from_str(&self.transaction_id).unwrap_or_default();
        result[..32].copy_from_slice(&hash.0);
        result[32..64].copy_from_slice(&self.commitment);
        result
    }
}

#[cfg(feature = "crypto")]
impl From<Blob> for FrameRef {
    fn from(blob: Blob) -> Self {
        Self {
            transaction_id: CryptoHash([0u8; 32]).to_string(),
            commitment: blob.commitment,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_celestia_format() {
        let frame_ref = FrameRef::new(CryptoHash([0u8; 32]).to_string(), [2u8; 32]);
        assert_eq!(
            frame_ref.to_celestia_format(),
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
                2, 2, 2, 2, 2, 2, 2, 2
            ],
            "FrameRef::to_celestia_format() should return 40 bytes array"
        );
    }

    #[cfg(not(feature = "crypto"))]
    #[test]
    fn test_naive_commitment() {
        let blob = Blob::new_v0(Namespace::default(), vec![3u8; 256]);
        assert_eq!(
            blob.commitment, [0u8; 32],
            "Blob::commitment should be all zeroes without crypto feature"
        );
    }

    #[test]
    fn test_naive_commitment_crypto() {
        let blob = Blob::new_v0(Namespace::default(), vec![3u8; 256]);
        assert_eq!(
            hex::encode(blob.commitment),
            "b56ff9af363fc1afe2bd32a239cd8c27d854c320e95afbceb678309ba6352794".to_string()
        );
    }
}
