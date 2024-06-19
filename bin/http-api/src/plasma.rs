use anyhow::anyhow;
use axum::{
    extract::{BodyStream, Path, Query, State},
    response::Response,
};
use futures_util::stream::StreamExt;
use itertools::Itertools;
use near_da_rpc::{Blob, BlobRef};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{stream_response, AppError, AppState};

// https://github.com/ethereum-optimism/specs/discussions/135
pub const DA_SELECTOR: u8 = 0x6e;
// https://github.com/ethereum-optimism/optimism/blob/457f33f4fdda9373dcf2839619ebf67182ee5057/op-plasma/commitment.go#L37
pub const OP_PLASMA_GENERIC_COMMITMENT: u8 = 1;

pub fn strip_plasma_bytes(mut bytes: Vec<u8>) -> super::Result<Vec<u8>> {
    bytes
        .strip_prefix(&[OP_PLASMA_GENERIC_COMMITMENT])
        .ok_or_else(|| anyhow!("invalid plasma commitment"))
        .and_then(|stripped| {
            stripped
                .strip_prefix(&[DA_SELECTOR])
                .ok_or_else(|| anyhow!("invalid DA selector"))
        })
        .map(Into::into)
}

pub fn append_plasma_bytes(mut bytes: Vec<u8>) -> super::Result<Vec<u8>> {
    bytes.insert(0, OP_PLASMA_GENERIC_COMMITMENT);
    bytes.insert(0, DA_SELECTOR);
    Ok(bytes)
}

pub(crate) async fn get(
    State(state): State<Arc<RwLock<AppState>>>,
    Path(request): Path<String>,
) -> Result<Response, AppError> {
    let tx = hex::decode(request.strip_prefix("0x").unwrap_or(&request))?;
    if tx.len() % 32 != 0 {
        return Err(anyhow::anyhow!("invalid commitment").into());
    }

    let refs = tx
        .chunks(32)
        .map(TryInto::<[u8; 32]>::try_into)
        .map(|tx| BlobRef::from(tx.unwrap()))
        .collect_vec();

    let mut data = vec![];
    for blob_ref in refs {
        data.extend_from_slice(
            &super::get(State(state.clone()), Query(blob_ref))
                .await?
                .data,
        );
    }
    Ok(stream_response(data))
}

pub(crate) async fn submit(
    State(state): State<Arc<RwLock<AppState>>>,
    mut stream: BodyStream,
) -> Result<Response, AppError> {
    let mut chunks = vec![];
    while let Some(chunk) = stream.next().await {
        chunks.extend_from_slice(&chunk?[..])
    }

    let commits = super::submit(State(state), Blob::new(chunks).into())
        .await
        .map(|r| r.transaction_id.to_vec())?;

    Ok(stream_response(commits))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_plasma_bytes() {
        let bytes = vec![OP_PLASMA_GENERIC_COMMITMENT, DA_SELECTOR, 1, 2, 3];
        let expected = vec![1, 2, 3];
        assert_eq!(strip_plasma_bytes(bytes).unwrap(), expected);
    }

    #[test]
    fn test_strip_plasma_bytes_invalid_commitment() {
        let bytes = vec![0, DA_SELECTOR, 1, 2, 3];
        assert!(strip_plasma_bytes(bytes).is_err());
    }

    #[test]
    fn test_strip_plasma_bytes_invalid_selector() {
        let bytes = vec![OP_PLASMA_GENERIC_COMMITMENT, 0, 1, 2, 3];
        assert!(strip_plasma_bytes(bytes).is_err());
    }

    #[test]
    fn test_append_plasma_bytes() {
        let bytes = vec![1, 2, 3];
        let expected = vec![OP_PLASMA_GENERIC_COMMITMENT, DA_SELECTOR, 1, 2, 3];
        assert_eq!(append_plasma_bytes(bytes).unwrap(), expected);
    }
}
