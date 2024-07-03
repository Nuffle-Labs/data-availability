use crate::{Maintainer, Namespace};
use near_sdk::{
    env::log_str,
    serde::{Deserialize, Serialize},
    serde_json::to_string,
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
pub(crate) struct EventLog {
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
pub(crate) enum EventLogVariant {
    AddMaintainer(AddMaintainerLog),
    NamespaceRegistration(NamespaceRegistrationLog),
}

/// An event log to capture a maintainer inclusion.
///
/// Arguments
/// * `owner_id`: "account.near"
/// * `memo`: optional message
#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub(crate) struct AddMaintainerLog {
    pub maintainer: Maintainer,
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
pub(crate) struct NamespaceRegistrationLog {
    pub namespace: Namespace,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
}

impl EventLog {
    fn new(event: EventLogVariant) -> Self {
        Self {
            standard: CONTRACT_STANDARD_NAME.to_string(),
            version: CONTRACT_STANDARD_VERSION.to_string(),
            event,
        }
    }

    pub(crate) fn maintainer(maintainer: Maintainer) {
        let log = EventLog::new(EventLogVariant::AddMaintainer(AddMaintainerLog {
            maintainer,
            memo: None,
        }));
        log_str(&to_string(&log).unwrap());
    }

    pub(crate) fn namespace(namespace: Namespace) {
        let log = EventLog::new(EventLogVariant::NamespaceRegistration(
            NamespaceRegistrationLog {
                namespace,
                memo: None,
            },
        ));
        log_str(&to_string(&log).unwrap());
    }
}
