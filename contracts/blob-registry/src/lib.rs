use events::EventLog;
use near_sdk::{
    assert_one_yocto, env, near,
    serde::de::{self, Visitor},
    serde::{Deserialize, Deserializer, Serialize, Serializer},
    AccountId, NearToken, PanicOnDefault,
};

use near_sdk_contract_tools::{
    owner::{Owner, OwnerExternal},
    Owner,
};
use std::collections::{HashMap, HashSet};

mod events;

// Error messages.
const ERR_NAMESPACE_MISSING: &str = "Namespace does not exist";
const ERR_UNAUTHORIZED_CALLER: &str = "Caller is not authorized to call method";
const ERR_INVALID_INPUT: &str = "Invalid input";
const ERR_CONTRACT_INITIALIZED: &str = "Contract already initialized";
const ERR_NAMESPACE_EXISTS: &str = "Namespace exists and cannot be registered again";
const ERR_NOT_ENOUGHT_FUNDS: &str = "Not enough funds to register a namespace";
const MINIMUM_DEPOSIT: u8 = 100; // 0.1 NEAR == 100 miliNEAR

/// The contract itself.
#[derive(PanicOnDefault, Owner)]
#[near(contract_state, serializers=[borsh, json])]
pub struct Contract {
    info: HashMap<Namespace, Metadata>,
}

/// Repository information, understood as a set of namespaces and their metadata.
#[derive(Default, Clone)]
#[near(serializers=[borsh, json])]
pub struct Metadata {
    priority: Priority,
    maintainers: HashSet<Maintainer>,
    extra: Option<String>,
}

type Namespace = u32;
type Priority = u32;
type Maintainer = Vec<u8>;
type TransactionId = Hash;

#[near]
impl Contract {
    #[init]
    /// Create a new contract with a given owner.
    pub fn new(owner_id: AccountId) -> Self {
        assert!(!env::state_exists(), "{ERR_CONTRACT_INITIALIZED}");
        let mut contract = Self {
            info: Default::default(),
        };
        Self::init(&mut contract, &owner_id);
        contract
    }

    /// Get the priority level.
    pub fn priority(&self, namespace: Namespace) -> Option<Priority> {
        self.info.get(&namespace).map(|metadata| metadata.priority)
    }

    /// Get the maintainers.
    pub fn maintainers(&self, namespace: Namespace) -> Option<HashSet<Maintainer>> {
        self.info
            .get(&namespace)
            .map(|metadata| metadata.maintainers.clone())
    }

    /// Get the extra information in the metadata.
    pub fn extra(&self, namespace: Namespace) -> Option<String> {
        self.info
            .get(&namespace)
            .and_then(|metadata| metadata.extra.clone())
    }

    /// Add a new maintainer.
    pub fn add_maintainer(&mut self, namespace: Namespace, maintainer: Maintainer) {
        match self.check_authorized(namespace) {
            Some(mut metadata) => {
                // add it to the set and log the inclusion
                if metadata.maintainers.insert(maintainer.clone()) {
                    EventLog::maintainer(maintainer);
                };
            }
            None => {
                env::panic_str(ERR_UNAUTHORIZED_CALLER);
            }
        }
    }

    /// Submit the blob and the namespace.
    pub fn submit(&self, namespace: Namespace, _transaction_ids: Vec<TransactionId>) {
        // check the namespace exists and the caller is in the maintainers list
        match self.check_authorized(namespace) {
            Some(_) => {
                env::input()
                    .is_none()
                    .then(|| env::panic_str(ERR_INVALID_INPUT));
            }
            None => {
                env::panic_str(ERR_UNAUTHORIZED_CALLER);
            }
        }
    }

    /// Transfer the ownership of the contract. An event is emited by `Self::update_owner`.
    pub fn transfer_ownership(&mut self, new_owner_id: AccountId) {
        Self::require_owner();
        assert_one_yocto();
        Self::update_owner(self, Some(new_owner_id.clone()));
    }

    /// Register a DA consumer.
    #[payable]
    pub fn register_consumer(&mut self, namespace: Namespace) {
        if self.info.get(&namespace).is_some() {
            // when the namespace does not exist,
            env::panic_str(ERR_NAMESPACE_EXISTS);
        } else {
            // when the deposit is enough
            if env::attached_deposit() >= NearToken::from_millinear(MINIMUM_DEPOSIT.into()) {
                // and the namespace does not exist, then it can be registered
                let metadata = Metadata {
                    maintainers: HashSet::from([env::predecessor_account_id().as_bytes().to_vec()]),
                    ..Default::default()
                };
                self.info.insert(namespace, metadata);
                // and an event can be emitted
                EventLog::namespace(namespace);
            } else {
                env::panic_str(ERR_NOT_ENOUGHT_FUNDS);
            }
        }
    }
}

impl Contract {
    /// Helper function to check that the caller is authorized to call the method.
    fn check_authorized(&self, namespace: Namespace) -> Option<Metadata> {
        let predecessor = env::predecessor_account_id();
        if let Some(metadata) = self.info.get(&namespace) {
            if self.own_get_owner().unwrap() == predecessor
                || metadata.maintainers.contains(predecessor.as_bytes())
            {
                Some(metadata.clone())
            } else {
                None
            }
        } else {
            env::panic_str(ERR_NAMESPACE_MISSING);
        }
    }
}

/// Hash type for represennting the transaction id.
#[derive(Debug)]
pub struct Hash([u8; 32]);

impl Serialize for Hash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Convert the byte array to a hex string for serialization
        let hex_string = hex::encode(self.0);
        serializer.serialize_str(&hex_string)
    }
}

impl<'de> Deserialize<'de> for Hash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MyHashVisitor;

        impl<'de> Visitor<'de> for MyHashVisitor {
            type Value = Hash;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a hex string representing a hash")
            }

            fn visit_str<E>(self, v: &str) -> Result<Hash, E>
            where
                E: de::Error,
            {
                // Convert the hex string back to a byte array
                let bytes = hex::decode(v).map_err(de::Error::custom)?;
                if bytes.len() != 32 {
                    return Err(de::Error::custom("expected a 32-byte hash"));
                }
                let mut hash = [0u8; 32];
                hash.copy_from_slice(&bytes);
                Ok(Hash(hash))
            }
        }

        deserializer.deserialize_str(MyHashVisitor)
    }
}
