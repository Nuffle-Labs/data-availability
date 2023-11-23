#![no_std]

use near_sdk::{assert_one_yocto, env, AccountId};

const ERR_CONTRACT_NOT_INITIALIZED: &str = "Contract is not initialized.";
const ERR_CONTRACT_ALREADY_INITIALIZED: &str = "Contract already initialized.";
const ERR_NOT_OWNER: &str = "Predecessor is not owner.";
const ERR_NO_PROPOSED_OWNER: &str = "No proposed owner.";
const ERR_NOT_PROPOSED_OWNER: &str = "Predecessor is not proposed owner.";
const ERR_MISSING_INVALID_INPUT: &str = "Missing or invalid input.";
const JSON_NULL: &[u8] = b"null";
const JSON_DOUBLE_QUOTE: &[u8] = b"\"";
// storage keys
const KEY_INITIALIZED: &[u8; 1] = &[0];
const KEY_OWNER: &[u8; 1] = &[1]; //            serialized with .as_bytes() NOT Borsh
const KEY_PROPOSED_OWNER: &[u8; 1] = &[2]; //   ditto. Not guaranteed to be a valid AccountId.

fn require_initialized() {
    if !env::storage_has_key(KEY_INITIALIZED) {
        env::panic_str(ERR_CONTRACT_NOT_INITIALIZED);
    }
}

fn require_owner(predecessor: &AccountId) {
    if env::storage_read(KEY_OWNER)
        .filter(|v| v == predecessor.as_bytes())
        .is_none()
    {
        env::panic_str(ERR_NOT_OWNER);
    }
}

#[no_mangle]
pub fn new() {
    if env::storage_has_key(KEY_INITIALIZED) {
        env::panic_str(ERR_CONTRACT_ALREADY_INITIALIZED);
    }

    env::storage_write(KEY_INITIALIZED, &[1]);

    let predecessor_account_id = env::predecessor_account_id();

    env::storage_write(KEY_OWNER, predecessor_account_id.as_bytes());
}

#[no_mangle]
pub fn submit() {
    require_initialized();
    require_owner(&env::predecessor_account_id());

    if env::input().is_none() {
        env::panic_str(ERR_MISSING_INVALID_INPUT);
    }
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

    let current_owner = env::storage_read(KEY_OWNER);

    return_json_string(current_owner.as_deref());
}

#[no_mangle]
pub fn own_get_proposed_owner() {
    require_initialized();

    let current_proposed_owner = env::storage_read(KEY_PROPOSED_OWNER);

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
            env::storage_write(KEY_PROPOSED_OWNER, new_proposed_owner);
        }
        None => {
            env::storage_remove(KEY_PROPOSED_OWNER);
        }
    }
}

#[no_mangle]
pub fn own_accept_owner() {
    require_initialized();
    assert_one_yocto();
    let predecessor = env::predecessor_account_id();
    let current_proposed_owner = env::storage_read(KEY_PROPOSED_OWNER)
        .unwrap_or_else(|| env::panic_str(ERR_NO_PROPOSED_OWNER));

    if predecessor.as_bytes() != current_proposed_owner {
        env::panic_str(ERR_NOT_PROPOSED_OWNER);
    }

    env::storage_remove(KEY_PROPOSED_OWNER);
    env::storage_write(KEY_OWNER, &current_proposed_owner);
}

#[no_mangle]
pub fn own_renounce_owner() {
    require_initialized();
    assert_one_yocto();
    let predecessor = env::predecessor_account_id();
    require_owner(&predecessor);

    env::storage_remove(KEY_OWNER);
    env::storage_remove(KEY_PROPOSED_OWNER);
}
