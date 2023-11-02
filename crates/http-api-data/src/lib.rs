#![no_std]
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConfigureClientRequest {
    pub account_id: String,
    pub secret_key: String,
    pub contract_id: String,
    pub network: String,
    pub namespace: Namespace,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Namespace {
    pub version: u8,
    pub id: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BlobRequest {
    pub transaction_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SubmitRequest {
    pub blobs: Vec<Blob>,
}

#[serde_with::serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Blob {
    pub namespace: Namespace,
    pub share_version: u32,
    #[serde_as(as = "serde_with::hex::Hex")]
    pub commitment: [u8; 32],
    #[serde_as(as = "serde_with::hex::Hex")]
    pub data: Vec<u8>,
}
