use near_da_primitives::Blob;
use near_sdk::{borsh, borsh::BorshDeserialize, borsh::BorshSerialize, BlockHeight};
use near_sdk::{env, near_bindgen};
use near_sdk_contract_tools::{owner::Owner, Owner};
use std::default;
use std::vec::Vec;

#[allow(unused_imports)] // Justification: Proc macro needs this
use near_sdk_contract_tools::owner::OwnerExternal;

#[near_bindgen]
#[derive(Owner, BorshDeserialize, BorshSerialize)]
pub struct Contract {}

impl default::Default for Contract {
    fn default() -> Self {
        let mut contract = Self {};
        Owner::init(&mut contract, &near_sdk::env::predecessor_account_id());

        contract
    }
}

#[near_bindgen]
impl Contract {
    pub fn submit(&mut self, blobs: Vec<Blob>) -> BlockHeight {
        Self::require_owner();
        near_sdk::env::log_str(&format!("submitting {} blobs", blobs.len()));
        env::block_height()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initializes() {
        let _ = Contract::default();
    }
}
