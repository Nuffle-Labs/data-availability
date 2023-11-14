#![allow(dead_code)]

pub use eyre::Result;
pub use reed_solomon_novelpoly::ReedSolomon;

pub mod erasure;
pub mod grid;
pub mod scheme;

/// Transcripts are the partial slices of the fully encoded data, in some cases
/// this is referred to as "shards", in other cases as "codewords".
pub type Transcript = Vec<u8>;

/// An erasure commitment, existing of the commitment and the erasure encoded 
/// codewords.
pub struct ErasureCommitment<Commitment> {
    /// The commitment to the encoded data
    pub commitment: Commitment,
    /// The encoded data
    pub encoding: Vec<Transcript>,
    // TODO: decouple RS, for example non-MDS simple linear codes
    /// The system responsible for encoding the data
    pub rs: ReedSolomon,
}

/// The encode interface abstracted over a Commitment.
///
/// Commitment can be any commitment scheme, e.g:
/// - merkle
/// - KZG
/// - hash
/// - homomorphic hash
pub trait Encoding<Commitment> {
    /// Encode and commit the data
    fn encode(&self, data: &[u8]) -> Result<ErasureCommitment<Commitment>>;
    /// Extract the transcript
    fn extract(&self, transcripts: Vec<Option<Transcript>>, rs: ReedSolomon) -> Result<Vec<u8>>;
    /// Verify the commitments over the transcripts
    fn verify(&self, commitment: Commitment) -> bool;
}

