use near_da_primitives::Namespace;
use serde::{Deserialize, Deserializer};
use std::net::SocketAddr;
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
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Network {
    Mainnet,
    #[default]
    Testnet,
    // [ip]:[port] string format
    Localnet(String),
}

impl Network {
    fn parse_localnet(s: &str) -> Result<Network, String> {
        s.parse::<SocketAddr>()
            .map_err(|err| err.to_string())
            .and_then(|addr| {
                if addr.ip().is_loopback() {
                    Ok(Network::Localnet(s.into()))
                } else {
                    Err("Non-local socket address".into())
                }
            })
    }
}

impl<'de> Deserialize<'de> for Network {
    fn deserialize<D>(deserializer: D) -> Result<Network, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "mainnet" => Ok(Network::Mainnet),
            "testnet" => Ok(Network::Testnet),
            socket_addr => Self::parse_localnet(socket_addr).map_err(serde::de::Error::custom),
        }
    }
}

impl Network {
    pub fn to_endpoint(&self) -> String {
        const MAINNET_RPC_ENDPOINT: &str = "https://rpc.mainnet.near.org";
        const TESTNET_RPC_ENDPOINT: &str = "https://rpc.testnet.near.org";
        match self {
            Self::Mainnet => MAINNET_RPC_ENDPOINT.into(),
            Self::Testnet => TESTNET_RPC_ENDPOINT.into(),
            Self::Localnet(socket_addr) => ["http://", socket_addr.as_str()].concat(),
        }
    }
    pub fn archive_endpoint(&self) -> String {
        const MAINNET_RPC_ARCHIVE_ENDPOINT: &str = "https://archival-rpc.mainnet.near.org";
        const TESTNET_RPC_ARCHIVE_ENDPOINT: &str = "https://archival-rpc.testnet.near.org";
        match self {
            Self::Mainnet => MAINNET_RPC_ARCHIVE_ENDPOINT.into(),
            Self::Testnet => TESTNET_RPC_ARCHIVE_ENDPOINT.into(),
            Self::Localnet(socket_addr) => ["http://", socket_addr.as_str()].concat(),
        }
    }
}

impl Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Mainnet => "mainnet",
            Self::Testnet => "testnet",
            Self::Localnet(socket_addr) => socket_addr.as_str(),
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
            socket_addr => Self::parse_localnet(socket_addr),
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

        let url = "127.0.0.1:3030";
        let network = Network::try_from(url).unwrap();
        assert_eq!(network, Network::Localnet(url.into()));
    }

    #[test]
    fn test_invalid_local_adress() {
        let network = Network::try_from("invalid").unwrap_err();
        assert_eq!(network, "invalid socket address syntax");
    }

    #[test]
    fn test_network_case_insensitive() {
        let network = Network::try_from("MAINNET").unwrap();
        assert_eq!(network, Network::Mainnet);
    }
}
