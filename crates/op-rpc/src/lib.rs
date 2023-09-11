use eyre::Result;
use near_da_primitives::{Blob, Commitment, Namespace};
use near_primitives::types::BlockHeight;
use serde::{Deserialize, Serialize};

pub mod ffi;
pub mod near;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameRef {
    pub height: BlockHeight,
    pub commitment: Commitment,
}

#[repr(C)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitResult(pub BlockHeight);

#[repr(C)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Read(pub Blob);

#[repr(C)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadAll(pub Vec<Blob>);

#[repr(C)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexRead(pub Blob);

#[async_trait::async_trait]
pub trait DataAvailability {
    /// Submit blobs to the da layer
    async fn submit(&self, blobs: &[Blob]) -> Result<SubmitResult>;
    /// Read blob by namespace and height
    async fn get(&self, namespace: &Namespace, height: BlockHeight) -> Result<Read>;
    /// Get all blobs for a namespace
    async fn get_all(&self, namespace: &Namespace) -> Result<ReadAll>;
    /// Shortcut to get the latest blob if you already know the commitment
    async fn fast_get(&self, commitment: &Commitment) -> Result<IndexRead>;
}
