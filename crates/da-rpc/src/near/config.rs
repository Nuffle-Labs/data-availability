use near_da_primitives::Namespace;
use serde::Deserialize;
use std::{fmt::Display, path::PathBuf};

#[derive(Debug, Clone, Deserialize)]
pub enum KeyType {
    File(PathBuf),
    Seed(String, String),
    SecretKey(String, String),
}

#[cfg(test)]
impl Default for KeyType {
    fn default() -> Self {
        Self::File(PathBuf::from("throwaway-key.json"))
    }
}

#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(test, derive(Default))]
pub struct Config {
    pub key: KeyType,
    pub contract: String,
    pub network: Network,
    pub namespace: Namespace, // TODO: use this
}

// TODO: stole from near-light-client, create primitives to share this
#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
pub enum Network {
    Mainnet,
    #[default]
    Testnet,
    Localnet,
}

impl Network {
    pub fn to_endpoint(&self) -> &str {
        const MAINNET_RPC_ENDPOINT: &str = "https://rpc.mainnet.near.org";
        const TESTNET_RPC_ENDPOINT: &str = "https://rpc.testnet.near.org";
        match self {
            Self::Mainnet => MAINNET_RPC_ENDPOINT,
            Self::Testnet => TESTNET_RPC_ENDPOINT,
            _ => "http://localhost:3030",
        }
    }
    pub fn archive_endpoint(&self) -> &str {
        const MAINNET_RPC_ARCHIVE_ENDPOINT: &str = "https://archival-rpc.mainnet.near.org";
        const TESTNET_RPC_ARCHIVE_ENDPOINT: &str = "https://archival-rpc.testnet.near.org";
        match self {
            Self::Mainnet => MAINNET_RPC_ARCHIVE_ENDPOINT,
            Self::Testnet => TESTNET_RPC_ARCHIVE_ENDPOINT,
            _ => "http://localhost:3030",
        }
    }
}

impl Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Mainnet => "mainnet",
            Self::Testnet => "testnet",
            _ => "localnet",
        };
        write!(f, "{}", s)
    }
}

impl TryFrom<&str> for Network {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "mainnet" => Ok(Self::Mainnet),
            "testnet" => Ok(Self::Testnet),
            "localnet" => Ok(Self::Localnet),
            _ => Err(format!("Invalid network: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_from_str() {
        let network = Network::try_from("mainnet").unwrap();
        assert_eq!(network, Network::Mainnet);
        let network = Network::try_from("MAINNET").unwrap();
        assert_eq!(network, Network::Mainnet);
        let network = Network::try_from("testnet").unwrap();
        assert_eq!(network, Network::Testnet);
        let network = Network::try_from("localnet").unwrap();
        assert_eq!(network, Network::Localnet);
        let network = Network::try_from("invalid").unwrap_err();
        assert_eq!(network, "Invalid network: invalid");
    }

    #[test]
    fn test_network_case_insensitive() {
        let network = Network::try_from("MAINNET").unwrap();
        assert_eq!(network, Network::Mainnet);
    }
}
