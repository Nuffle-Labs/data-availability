use eyre::Result;
pub use near_da_primitives::{Blob, BlobRef, Commitment, Namespace};
pub use near_primitives::hash::CryptoHash;
use near_primitives::types::BlockHeight;
use serde::{Deserialize, Serialize};

pub mod near;

#[repr(C)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitResult(pub BlobRef);

#[repr(C)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Read(pub Blob);

#[repr(C)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadAll(pub Vec<(BlockHeight, Blob)>);

#[repr(C)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexRead(pub Blob);

#[async_trait::async_trait]
pub trait DataAvailability {
    /// Submit blobs to the da layer
    async fn submit(&self, blob: Blob) -> Result<SubmitResult>;
    /// Read blob by namespace and height
    async fn get(&self, transaction_id: CryptoHash) -> Result<Read>;
}
