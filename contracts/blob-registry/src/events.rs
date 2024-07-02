use crate::{Maintainer, Namespace};
use near_sdk::{
    env::log_str,
    serde::{Deserialize, Serialize},
    serde_json,
};

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
    AddMaintainer(AddMaintainerLog),
    OwnerTransfer(OwnerEvent),
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

/// Log the submission of a blob by `log_str`ing an [`EventLogVariant::BlobSubmissionLog`].
pub(crate) fn log_blob_submission(namespace: u32, blob: Vec<u8>) {
    let log = EventLog {
        standard: CONTRACT_STANDARD_NAME.to_string(),
        version: CONTRACT_STANDARD_VERSION.to_string(),
        event: EventLogVariant::BlobSubmission(BlobSubmissionLog {
            who: predecessor_account_id(),
            namespace,
            blob,
            memo: None,
        }),
    };
    log_str(&log.to_string());
}

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
