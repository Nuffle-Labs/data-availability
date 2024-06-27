use crate::{Maintainer, Namespace};
use near_sdk::{
    env::{log_str, predecessor_account_id},
    serde::{Deserialize, Serialize},
    serde_json, AccountId,
};
use near_sdk_contract_tools::owner::OwnerEvent;

const CONTRACT_STANDARD_NAME: &str = "nepXXX";
const CONTRACT_STANDARD_VERSION: &str = "1.0.0";

/// Interface to capture data about an event.
///
/// Arguments:
/// * `standard`: name of standard e.g. nep171
/// * `version`: e.g. 1.0.0
/// * `event`: associate event data
#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct EventLog {
    pub standard: String,
    pub version: String,
    #[serde(flatten)]
    pub event: EventLogVariant,
}

/// Enum that represents the data type of the EventLog.
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "event", content = "data")]
#[serde(rename_all = "snake_case")]
#[serde(crate = "near_sdk::serde")]
#[non_exhaustive]
pub enum EventLogVariant {
    BlobSubmission(BlobSubmissionLog),
    AddMaintainer(AddMaintainerLog),
    OwnerTransfer(OwnerEvent),
    NamespaceRegistration(NamespaceRegistrationLog),
}

impl EventLog {
    /// Create a new event with the default standard and version.
    fn new(event: EventLogVariant) -> Self {
        Self {
            standard: CONTRACT_STANDARD_NAME.to_string(),
            version: CONTRACT_STANDARD_VERSION.to_string(),
            event,
        }
    }

    /// Log the submission of a blob.
    pub(crate) fn blob(namespace: Namespace, blob: Vec<u8>) {
        let log = EventLog::new(EventLogVariant::BlobSubmission(BlobSubmissionLog {
            who: predecessor_account_id(),
            namespace,
            blob,
            memo: None,
        }));
        log_str(&log.to_string());
    }

    /// Log the addition of a maintainer.
    pub(crate) fn maintainer(maintainer: Maintainer) {
        let log = EventLog::new(EventLogVariant::AddMaintainer(AddMaintainerLog {
            maintainer,
            memo: None,
        }));
        log_str(&log.to_string());
    }

    /// Log the transfer of ownership.
    pub(crate) fn owner(old: Option<AccountId>, new: Option<AccountId>) {
        let log = EventLog::new(EventLogVariant::OwnerTransfer(OwnerEvent::Transfer {
            old,
            new,
        }));
        log_str(&log.to_string());
    }

    /// Log the registration of a new namespace.
    pub(crate) fn namespace(namespace: Namespace) {
        let log = EventLog::new(EventLogVariant::NamespaceRegistration(
            NamespaceRegistrationLog {
                namespace,
                memo: None,
            },
        ));
        log_str(&log.to_string());
    }
}

impl std::fmt::Display for EventLog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "EVENT_JSON:{}",
            &serde_json::to_string(self).map_err(|_| std::fmt::Error)?
        ))
    }
}

/// An event to log the submission of a blob.
///
/// Arguments
/// * `namespace`: "namespace.near"
/// * `blob_id`: "blob.near"
/// * `memo`: optional message
#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct BlobSubmissionLog {
    pub who: AccountId,
    pub namespace: u32,
    pub blob: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
}

/// An event log to capture a maintainer inclusion.
///
/// Arguments
/// * `owner_id`: "account.near"
/// * `memo`: optional message
#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct AddMaintainerLog {
    pub maintainer: Maintainer,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
}

/// An event log to capture an ownership transfer.
///
/// Arguments
/// * `authorized_id`: approved account to transfer
/// * `old_owner_id`: "owner.near"
/// * `new_owner_id`: "receiver.near"
/// * `memo`: optional message
#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct OwnershipTransferLog {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorized_id: Option<String>,
    pub old_owner_id: String,
    pub new_owner_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
}

/// An event log to capture a new namespace registration.
///
/// Arguments
/// * `namespace`: u32 that has been registered
/// * `memo`: optional message
#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct NamespaceRegistrationLog {
    pub namespace: Namespace,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
}
