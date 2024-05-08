use std::str::FromStr;

use super::{Blob, DataAvailability};
use crate::{Read, SubmitResult};
use config::Config;
use eyre::{eyre, Result};
use futures::{Future, StreamExt, TryFutureExt};
use near_crypto::{InMemorySigner, Signer};
use near_da_primitives::{LegacyBlob, Mode, SubmitRequest};
use near_jsonrpc_client::{
    methods::{
        self, broadcast_tx_commit::RpcBroadcastTxCommitRequest, query::RpcQueryRequest,
        send_tx::RpcSendTransactionRequest, tx::RpcTransactionStatusRequest,
    },
    JsonRpcClient,
};
use near_jsonrpc_primitives::types::{query::QueryResponseKind, transactions::TransactionInfo};
use near_primitives::{
    borsh,
    views::{FinalExecutionOutcomeViewEnum, FinalExecutionStatus},
};
use near_primitives::{
    borsh::{BorshDeserialize, BorshSerialize},
    hash::CryptoHash,
    transaction::{Action, FunctionCallAction, Transaction},
    types::{AccountId, BlockReference, Nonce},
    views::{ActionView, TxExecutionStatus},
};
use serde::{Deserialize, Serialize};
use tokio::pin;
use tracing::{debug, error, trace};

pub mod config;

// TODO: optimise this to avoid refunds, test a 4mb blob
pub const MAX_TGAS: u64 = 100_000_000_000_000;

pub struct Client {
    pub config: Config,
    pub client: JsonRpcClient,
    pub archive: JsonRpcClient,
}

impl Client {
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
            client: JsonRpcClient::connect(config.network.to_endpoint()),
            archive: JsonRpcClient::connect(config.network.archive_endpoint()),
        }
    }

    async fn get_current_nonce(
        &self,
        account_id: &AccountId,
        public_key: &near_crypto::PublicKey,
    ) -> Result<Option<(CryptoHash, Nonce)>> {
        let query_response = self
            .client
            .call(RpcQueryRequest {
                block_reference: BlockReference::latest(),
                request: near_primitives::views::QueryRequest::ViewAccessKey {
                    account_id: account_id.clone(),
                    public_key: public_key.clone(),
                },
            })
            .await;

        match query_response {
            Ok(access_key_query_response) => match access_key_query_response.kind {
                QueryResponseKind::AccessKey(access_key) => Ok(Some((
                    access_key_query_response.block_hash,
                    access_key.nonce,
                ))),
                _ => Err(eyre!("failed to extract current nonce")),
            },
            Err(res) => Err(res)?,
        }
    }

    pub async fn get_nonce_signer(&self) -> Result<(InMemorySigner, CryptoHash, Nonce)> {
        let signer = get_signer(&self.config)?;
        if let Some((latest_hash, current_nonce)) = self
            .get_current_nonce(&signer.account_id, &signer.public_key)
            .await?
        {
            Ok((signer, latest_hash, current_nonce))
        } else {
            Err(eyre!("failed to get current nonce"))
        }
    }

    pub async fn no_signer(&self) -> Result<impl Signer> {
        Ok(near_crypto::EmptySigner {})
    }

    pub fn build_view_call(hash: CryptoHash, sender: AccountId) -> RpcTransactionStatusRequest {
        RpcTransactionStatusRequest {
            transaction_info: TransactionInfo::TransactionId {
                tx_hash: hash,
                sender_account_id: sender,
            },
            wait_until: TxExecutionStatus::IncludedFinal,
        }
    }

    pub fn build_function_call_transaction<S: Signer>(
        signer: &S,
        signer_account_id: &AccountId,
        contract: &AccountId,
        latest_hash: &CryptoHash,
        current_nonce: Nonce,
        action: FunctionCallAction,
    ) -> RpcSendTransactionRequest {
        let tx = Transaction {
            signer_id: signer_account_id.clone(),
            public_key: signer.public_key(),
            nonce: current_nonce + 1,
            receiver_id: contract.clone(),
            block_hash: *latest_hash,
            actions: vec![Action::FunctionCall(Box::new(action))],
        };
        RpcSendTransactionRequest {
            signed_transaction: tx.sign(signer),
            wait_until: TxExecutionStatus::IncludedFinal,
        }
    }
}

pub fn get_signer(config: &Config) -> Result<InMemorySigner> {
    Ok(match config.key {
        config::KeyType::File(ref path) => InMemorySigner::from_file(path)?,
        config::KeyType::Seed(ref account_id, ref seed) => {
            InMemorySigner::from_seed(account_id.parse()?, near_crypto::KeyType::ED25519, seed)
        }
        config::KeyType::SecretKey(ref account_id, ref secret_key) => {
            InMemorySigner::from_secret_key(
                account_id.parse()?,
                near_crypto::SecretKey::from_str(secret_key)?,
            )
        }
    })
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug)]
struct LegacyRequest {
    blobs: Vec<LegacyBlob>,
}

// TODO: mock tests for these
#[async_trait::async_trait]
impl DataAvailability for Client {
    async fn submit(&self, blob: Blob) -> Result<SubmitResult> {
        let (signer, latest_hash, current_nonce) = self.get_nonce_signer().await?;

        let submit_req = SubmitRequest {
            namespace: self.config.namespace,
            data: blob.data,
        };
        let req = Client::build_function_call_transaction(
            &signer,
            &signer.account_id,
            &self.config.contract.parse()?,
            &latest_hash,
            current_nonce,
            FunctionCallAction {
                method_name: "submit".to_string(),
                args: borsh::to_vec(&submit_req)?,
                gas: MAX_TGAS / 3,
                deposit: 0,
            },
        );

        match self
            .client
            .call(&req)
            .await?
            .final_execution_outcome
            .map(FinalExecutionOutcomeViewEnum::into_outcome)
        {
            Some(v) => match v.status {
                FinalExecutionStatus::SuccessValue(r) => {
                    debug!("Transaction submitted, result: {:?}", r);
                    Ok(SubmitResult(v.transaction.hash.0.into()))
                }
                FinalExecutionStatus::Failure(e) => {
                    error!("Error submitting transaction: {:?}", e);
                    Err(eyre!("Error submitting transaction: {:?}", e))
                }
                _ => Err(eyre!(
                    "Transaction not ready yet, this should not be reachable"
                )),
            },
            None => Err(eyre!("Transaction not ready yet")),
        }
    }

    async fn get(&self, transaction_id: CryptoHash) -> Result<Read> {
        let (signer, _, _) = self.get_nonce_signer().await?;

        let req = Client::build_view_call(transaction_id, signer.account_id);
        let result = self
            .client
            .call(&req)
            .or_else(|e| {
                debug!("Error hitting main rpc, falling back to archive: {:?}", e);
                self.archive.call(&req)
            })
            .await
            .map_err(|e| eyre!("Error getting blob: {:?}", e))?;
        trace!("blob status: {:?}", result.final_execution_status);

        match result
            .final_execution_outcome
            .map(FinalExecutionOutcomeViewEnum::into_outcome)
        {
            Some(v) => {
                let args: Vec<u8> = v
                    .transaction
                    .actions
                    .iter()
                    .filter(|x| matches!(x, ActionView::FunctionCall { .. }))
                    .collect::<Vec<_>>()
                    .first()
                    .and_then(|x| {
                        if let ActionView::FunctionCall { args, .. } = x {
                            let args: Vec<u8> = args.clone().into();
                            Some(args)
                        } else {
                            None
                        }
                    })
                    .ok_or_else(|| eyre!("Transaction had no actions: {:?}", v.transaction))?;
                debug!("Got args: {:?}", args.len());

                let original_request: SubmitRequest = BorshDeserialize::try_from_slice(&args)
                    .or_else(|e| {
                        debug!("Error deserializing new blob: {:?}", e);
                        let legacy_request = BorshDeserialize::try_from_slice(&args);
                        legacy_request
                            .map(|lr: LegacyRequest| SubmitRequest {
                                namespace: None,
                                // TODO: unbork
                                data: lr
                                    .blobs
                                    .into_iter()
                                    .map(Blob::from)
                                    .collect::<Vec<_>>()
                                    .first()
                                    .cloned()
                                    .unwrap()
                                    .data,
                            })
                            .map_err(|e| eyre!("Error deserializing old blob: {:?}", e))
                    })?;
                debug!("Got blob: {:?}", original_request.data);
                Ok(Read(original_request.data.into()))
            }
            x => Err(eyre!("Transaction not ready yet: {:?}", x)),
        }
        .map_err(|e| {
            error!("error getting blob: {:?}", e);
            e
        })
    }
}

#[cfg(test)]
mod tests {

    use near_da_primitives::Namespace;
    use tracing_subscriber::EnvFilter;

    use self::config::Network;

    use super::*;

    #[test]
    fn test_get_signer() {
        let account_id = "throwawaykey.testnet";
        let signer = get_signer(&Config {
            key: config::KeyType::Seed(account_id.parse().unwrap(), "ed25519:test".to_string()),
            ..Default::default()
        })
        .unwrap();
        assert_eq!(signer.account_id.to_string(), account_id.to_string());
        assert_eq!(
            signer.public_key.to_string(),
            "ed25519:38FBJoAPGsefiNoTFoDr95zyGeMb6fx6MuQw9HaasxHH".to_string()
        );

        let signer = get_signer(&Config {
            key: config::KeyType::SecretKey(
                account_id.parse().unwrap(),
                "ed25519:2T3R1CBAsKQN1Xa9fN9aL1epRwnxgbvk5RAy3sNAdh1n4nfkD9gyGKDLECBMVkwg1zPeewPG9eoX8XVRC6tr6nDt".to_string(),
            ),
            ..Default::default()
        })
        .unwrap();
        assert_eq!(signer.account_id.to_string(), account_id.to_string());
        assert_eq!(
            signer.public_key.to_string(),
            "ed25519:63gNvWb5ESf9ECcHtVy8E853XrPaSfgT39QHXRo6Zomx".to_string()
        );
    }

    #[tokio::test]
    async fn test_live_read() {
        tracing_subscriber::fmt()
            .with_target(false)
            .with_env_filter(EnvFilter::from_default_env())
            .compact()
            .init();

        let account = "devburnerkey3292389.testnet";
        let secret = "ed25519:2FPg5DHbr3oFLMKGiEhUsKUyf7vCy81qYHqdHNEHqTAaRzv2tJi2NWPLvbLoeTXzQP9jX6pNzfc83k3nSNNrpqQx";

        let config = Config {
            key: config::KeyType::SecretKey(account.to_string(), secret.to_string()),
            contract: "blarg233.testnet".to_string(),
            network: Network::Testnet,
            namespace: None,
            mode: Mode::Standard,
        };
        let client = Client::new(&config);

        client
            .get(CryptoHash::from_str("BHqUhKmttLDhyVhRJXN4wczxnQT4P3d4bkJpYtE6Au6").unwrap())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_live_read_old() {
        tracing_subscriber::fmt()
            .with_target(false)
            .with_env_filter(EnvFilter::from_default_env())
            .compact()
            .init();

        let account = "devburnerkey3292389.testnet";
        let secret = "ed25519:2FPg5DHbr3oFLMKGiEhUsKUyf7vCy81qYHqdHNEHqTAaRzv2tJi2NWPLvbLoeTXzQP9jX6pNzfc83k3nSNNrpqQx";

        let config = Config {
            key: config::KeyType::SecretKey(account.to_string(), secret.to_string()),
            contract: "throwawaykey.testnet".to_string(),
            network: Network::Testnet,
            namespace: None,
        };
        let client = Client::new(&config);

        client
            .get(CryptoHash::from_str("D13iq7DWstN4GZ5JEXJe2SzWxxfzy6v6DF6zgPt8ZCct").unwrap())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_live_read_failed() {
        tracing_subscriber::fmt()
            .with_target(false)
            .with_env_filter(EnvFilter::from_default_env())
            .compact()
            .init();

        let account = "devburnerkey3292389.testnet";
        let secret = "ed25519:2FPg5DHbr3oFLMKGiEhUsKUyf7vCy81qYHqdHNEHqTAaRzv2tJi2NWPLvbLoeTXzQP9jX6pNzfc83k3nSNNrpqQx";

        let config = Config {
            key: config::KeyType::SecretKey(account.to_string(), secret.to_string()),
            contract: "throwawaykey.testnet".to_string(),
            network: Network::Testnet,
            namespace: None,
            mode: Mode::Standard,
        };
        let client = Client::new(&config);

        client
            .get(CryptoHash::from_str("5hAuW1utdpnA5o6GPjJYuGLUvFKsjGPKwuQtfpPZ54uR").unwrap())
            .await
            .unwrap();
    }

    #[test]
    fn test_build_fast_get() {}

    #[test]
    fn test_build_get_all() {}

    #[test]
    fn test_build_get() {}

    #[test]
    fn test_build_submit() {}

    #[test]
    fn test_serialise_submit_no_namespace() {
        let req = SubmitRequest {
            namespace: None,
            data: [5u8; 256].to_vec(),
        };
        let req_str = serde_json::to_string(&req).unwrap();
        let new_req: SubmitRequest = serde_json::from_str(&req_str).unwrap();
        assert_eq!(
            serde_json::to_vec(&new_req).unwrap(),
            serde_json::to_vec(&req).unwrap()
        );
        assert_eq!(
            serde_json::to_vec(&req).unwrap(),
            serde_json::to_string(&req).unwrap().as_bytes()
        );
    }

    #[test]
    fn test_serialise_submit_namespace() {
        let req = SubmitRequest {
            namespace: Some(Namespace {
                version: 1,
                id: 1337,
            }),
            data: [6u8; 256].to_vec(),
        };
        let req_str = serde_json::to_string(&req).unwrap();
        let new_req: SubmitRequest = serde_json::from_str(&req_str).unwrap();
        assert_eq!(
            serde_json::to_vec(&new_req).unwrap(),
            serde_json::to_vec(&req).unwrap()
        );
        assert_eq!(
            serde_json::to_vec(&req).unwrap(),
            serde_json::to_string(&req).unwrap().as_bytes()
        );
    }
}
