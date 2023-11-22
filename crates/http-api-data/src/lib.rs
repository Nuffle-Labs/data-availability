#![no_std]
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

pub use near_da_primitives::{Blob, Namespace};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct ConfigureClientRequest {
    pub account_id: String,
    pub secret_key: String,
    pub contract_id: String,
    pub network: String,
    pub namespace: Namespace,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct BlobRequest {
    #[serde(rename = "tx")]
    pub transaction_id: String,
}

#[serde_with::serde_as]
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct SubmitRequest {
    #[serde_as(as = "serde_with::hex::Hex")]
    pub data: Vec<u8>,
}
