use near_da_primitives::Namespace;
use serde::Deserialize;
use std::{path::PathBuf, fmt::Display};

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
#[derive(Debug, Clone, Deserialize, Default)]
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
            _ => "http://`localhost:3030",
        }
    }
    pub fn archive_endpoint(&self) -> &str {
        const MAINNET_RPC_ARCHIVE_ENDPOINT: &str = "https://archival-rpc.mainnet.near.org";
        const TESTNET_RPC_ARCHIVE_ENDPOINT: &str = "https://archival-rpc.testnet.near.org";
        match self {
            Self::Mainnet => MAINNET_RPC_ARCHIVE_ENDPOINT,
            Self::Testnet => TESTNET_RPC_ARCHIVE_ENDPOINT,
            _ => "http://`localhost:3030",
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
