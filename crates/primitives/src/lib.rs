#![cfg_attr(not(feature = "std"), no_std)]

use serde::{Deserialize, Serialize};
use serde_with::serde_as;

pub type Namespace = [u8; 32];
pub type Data = Vec<u8>;
pub type ShareVersion = u32;
pub type Commitment = [u8; 32];
pub type BlockHeight = u64;

// TODO: docs
// TODO: tests
//
#[serde_as]
#[derive(Deserialize, Serialize, Clone)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Blob {
    #[serde_as(as = "serde_with::hex::Hex")]
    pub namespace: Namespace,
    pub share_version: ShareVersion,
    #[serde_as(as = "serde_with::hex::Hex")]
    pub commitment: [u8; 32],
    #[serde_as(as = "serde_with::hex::Hex")]
    pub data: Data,
}

impl Blob {
    pub fn new_v0(namespace: Namespace, data: Data) -> Self {
        let commitment = [0u8; 32];

        #[cfg(feature = "crypto")]
        let commitment = {
            let chunks: Vec<Vec<u8>> = data.chunks(256).map(|x| x.to_vec()).collect();
            near_primitives::merkle::merklize(&chunks).0.0
        };
        // TODO: validation
        Self {
            namespace,
            data,
            share_version: 0,
            commitment,
        }
    }
    // TODO: commitment building with crypto feature
}

#[serde_as]
#[derive(Deserialize, Serialize, Clone)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct FrameRef {
    pub height: BlockHeight,
    pub commitment: Commitment,
}

impl FrameRef {
    pub fn new(height: BlockHeight, commitment: Commitment) -> Self {
        Self { height, commitment }
    }
    pub fn with_height(mut self, height: BlockHeight) -> Self {
        self.height = height;
        self
    }
    // TODO: decide on a slimmer format
    pub fn to_celestia_format(&self) -> [u8; 40] {
        let mut result = [0u8; 40];
        result[..8].copy_from_slice(&self.height.to_be_bytes());
        result[8..40].copy_from_slice(&self.commitment);
        result
    }
}

impl From<Blob> for FrameRef {
    fn from(blob: Blob) -> Self {
        Self {
            height: 0,
            commitment: blob.commitment,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_celestia_format() {
        let frame_ref = FrameRef::new(1, [2u8; 32]);
        assert_eq!(
            frame_ref.to_celestia_format(),
            [0, 0, 0, 0, 0, 0, 0, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2],
            "FrameRef::to_celestia_format() should return 40 bytes array"
        );
        
    }

    #[cfg(not(feature = "crypto"))]
    #[test]
    fn test_naive_commitment() {
        let blob = Blob::new_v0([1u8; 32], vec![3u8; 256]);
        assert_eq!(
            blob.commitment,
            [0u8; 32],
            "Blob::commitment should be all zeroes without crypto feature"
        );
    }

    #[test]
    fn test_naive_commitment_crypto() {
       let blob = Blob::new_v0([1u8; 32], vec![3u8; 256]); 
        assert_eq!(hex::encode(blob.commitment), "b56ff9af363fc1afe2bd32a239cd8c27d854c320e95afbceb678309ba6352794".to_string());

    }
}
