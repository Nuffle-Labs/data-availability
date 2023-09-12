use near_da_primitives::Namespace;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub key_path: PathBuf,
    pub contract: String,
    pub network: Network,
    pub namespace: Namespace, // TODO: use this
}

impl Config {
    pub fn account_from_key_path(&self) -> String {
        self.key_path
            .to_str()
            .unwrap_or_default()
            .split_terminator("/")
            .last()
            .unwrap_or_default()
            .to_string()
    }
}

// TODO: stole from near-light-client, create primitives to share this
#[derive(Debug, Clone, Deserialize)]
pub enum Network {
    Mainnet,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acc_from_keypath() {
        let cfg = Config {
            key_path: "/home/hello/rubbish.near".into(),
            contract: Default::default(),
            network: Network::Localnet,
            namespace: [1_u8; 32],
        };
        assert_eq!(cfg.account_from_key_path(), "rubbish.near".to_string())
    }
}
