use near_da_primitives::Namespace;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub enum KeyType {
    File(PathBuf),
    Seed(String, String),
    SecretKey(String, String),
}

impl Default for KeyType {
    fn default() -> Self {
        Self::File(PathBuf::from("throwaway-key.json"))
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
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
    pub fn to_string(&self) -> String {
        match self {
            Self::Mainnet => "mainnet".to_string(),
            Self::Testnet => "testnet".to_string(),
            _ => "localnet".to_string(),
        }
    }
}

