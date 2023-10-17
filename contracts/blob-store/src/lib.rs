#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use borsh::BorshDeserialize;
use core::primitive::*;
use near_da_primitives::Blob;
use near_sdk::{assert_one_yocto, env, AccountId};

#[no_mangle]
pub fn submit() {
    require_initialized();
    let predecessor = env::predecessor_account_id();
    require_owner(&predecessor);

    env::input()
        .and_then(|i| <Vec<Blob> as BorshDeserialize>::try_from_slice(&i).ok())
        .unwrap_or_else(|| env::panic_str(ERR_MISSING_INVALID_INPUT));

    env::log_str("blobs submitted");

    env::value_return(&env::block_height().to_be_bytes())
}

const ERR_CONTRACT_NOT_INITIALIZED: &str = "Contract is not initialized.";
const ERR_CONTRACT_ALREADY_INITIALIZED: &str = "Contract already initialized.";
const ERR_NOT_OWNER: &str = "Predecessor is not owner.";
const ERR_NO_PROPOSED_OWNER: &str = "No proposed owner.";
const ERR_NOT_PROPOSED_OWNER: &str = "Predecessor is not proposed owner.";
const ERR_MISSING_INVALID_INPUT: &str = "Missing or invalid input.";
const JSON_NULL: &[u8] = b"null";
const JSON_DOUBLE_QUOTE: &[u8] = b"\"";

#[repr(u8)]
enum StorageKey {
    Initialized,
    Owner,         // serialized with .as_bytes() NOT Borsh
    ProposedOwner, // ditto. Not guaranteed to be a valid AccountId.
}

macro_rules! key {
    ($i: ident) => {
        alloc::slice::from_ref(&(StorageKey::$i as u8))
    };
}

fn require_initialized() {
    if !env::storage_has_key(key!(Initialized)) {
        env::panic_str(ERR_CONTRACT_NOT_INITIALIZED);
    }
}

fn require_owner(predecessor: &AccountId) {
    if env::storage_read(key!(Owner))
        .filter(|v| v == predecessor.as_bytes())
        .is_none()
    {
        env::panic_str(ERR_NOT_OWNER);
    }
}

#[no_mangle]
pub fn new() {
    if env::storage_has_key(key!(Initialized)) {
        env::panic_str(ERR_CONTRACT_ALREADY_INITIALIZED);
    }

    env::storage_write(key!(Initialized), &[1]);

    let predecessor_account_id = env::predecessor_account_id();

    env::storage_write(key!(Owner), predecessor_account_id.as_bytes());
}

fn return_json_string(v: Option<&[u8]>) {
    let r = v.map_or_else(
        || JSON_NULL.to_vec(),
        |v| [JSON_DOUBLE_QUOTE, v, JSON_DOUBLE_QUOTE].concat(),
    );
    env::value_return(&r);
}

#[no_mangle]
pub fn own_get_owner() {
    require_initialized();

    let current_owner = env::storage_read(key!(Owner));

    return_json_string(current_owner.as_deref());
}

#[no_mangle]
pub fn own_get_proposed_owner() {
    require_initialized();

    let current_proposed_owner = env::storage_read(key!(ProposedOwner));

    return_json_string(current_proposed_owner.as_deref());
}

#[no_mangle]
pub fn own_propose_owner() {
    require_initialized();
    assert_one_yocto();
    let predecessor = env::predecessor_account_id();
    require_owner(&predecessor);

    let payload = env::input().unwrap_or_else(|| env::panic_str(ERR_MISSING_INVALID_INPUT));

    let new_proposed_owner = if payload == b"{}" {
        None
    } else if let Some(account_id) = payload
        .strip_prefix(br#"{"account_id":""#) // jank JSON "parsing"
        .and_then(|s| s.strip_suffix(br#""}"#))
    {
        Some(account_id)
    } else {
        env::panic_str(ERR_MISSING_INVALID_INPUT);
    };

    match new_proposed_owner {
        Some(new_proposed_owner) => {
            env::storage_write(key!(ProposedOwner), new_proposed_owner);
        }
        None => {
            env::storage_remove(key!(ProposedOwner));
        }
    }
}

#[no_mangle]
pub fn own_accept_owner() {
    require_initialized();
    assert_one_yocto();
    let predecessor = env::predecessor_account_id();
    let current_proposed_owner = env::storage_read(key!(ProposedOwner))
        .unwrap_or_else(|| env::panic_str(ERR_NO_PROPOSED_OWNER));

    if predecessor.as_bytes() != current_proposed_owner {
        env::panic_str(ERR_NOT_PROPOSED_OWNER);
    }

    env::storage_remove(key!(ProposedOwner));
    env::storage_write(key!(Owner), &current_proposed_owner);
}

#[no_mangle]
pub fn own_renounce_owner() {
    require_initialized();
    assert_one_yocto();
    let predecessor = env::predecessor_account_id();
    require_owner(&predecessor);

    env::storage_remove(key!(Owner));
    env::storage_remove(key!(ProposedOwner));
}
