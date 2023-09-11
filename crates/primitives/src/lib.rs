#![cfg_attr(not(feature = "std"), no_std)]

use serde::{Deserialize, Serialize};
use serde_with::serde_as;

pub type Namespace = [u8; 32];
pub type Data = Vec<u8>;
pub type ShareVersion = u32;
pub type Commitment = [u8; 32];

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
        // TODO: validation
        Self {
            namespace,
            data,
            share_version: 0,
            commitment: Commitment::default(),
        }
    }
    // TODO: commitment building with crypto feature
}
