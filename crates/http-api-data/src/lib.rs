#![no_std]
extern crate alloc;

use alloc::string::String;
use near_da_primitives::Mode;
pub use near_da_primitives::{Blob, BlobRef, Namespace};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct ConfigureClientRequest {
    pub account_id: String,
    pub secret_key: String,
    pub contract_id: String,
    pub network: String,
    pub namespace: Option<Namespace>,
    pub mode: Option<Mode>,
    /// How big the bytes should be for the lru lookup cache
    #[serde(default = "default_bool::<false>")]
    pub should_cache: bool,
}

pub const fn default_bool<const V: bool>() -> bool {
    V
}

// TODO: tech debt
// impl From<Config> for ConfigureClientRequest {}
