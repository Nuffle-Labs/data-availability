#![allow(dead_code)]

use eyre::ensure;
pub use eyre::Result;
use grid::Grid;
use nalgebra::Scalar;
pub use reed_solomon_novelpoly::ReedSolomon;

pub mod erasure;
pub mod grid;
pub mod scheme;

/// Codewords are the partial slices of the fully encoded data, in some cases
/// this is referred to as "shards", in other cases as "codewords".
pub type Codeword = Option<Vec<u8>>;

/// An erasure commitment, existing of the commitment and the erasure encoded
/// codewords.
pub struct ErasureEncoding {
    /// The encoded data
    pub codewords: Vec<Codeword>,
    // TODO: decouple RS, for example non-MDS simple linear codes
    /// The system responsible for encoding the data
    pub rs: ReedSolomon,
}

pub trait ErasureCodec {
    /// Encode the data
    fn encode(&self, data: &[u8]) -> Result<ErasureEncoding>;
    /// Extract the transcript
    fn extract(&self, encoding: ErasureEncoding) -> Result<Vec<u8>>;
}

pub trait CommitmentScheme<Commitment, Element, ExtraArgs = ()> {
    fn commit(&self, data: &[Element], args: &ExtraArgs) -> Commitment;
    /// Verify the commitments over the transcripts
    fn verify(&self, commitment: Commitment, args: &ExtraArgs) -> bool;
}

pub struct ErasureCommitment<Witness> {
    pub witness: Witness,
    pub encoding: ErasureEncoding,
}

pub trait ErasureCommitmentScheme<Witness> {
    fn encode_commit(&self, data: &[u8]) -> Result<ErasureCommitment<Witness>>;
    fn verify_extract(&self, commitment: ErasureCommitment<Witness>) -> Result<Vec<u8>>;
}

// TODO: move to grid
pub trait ColumnCommitmentScheme<Commitment, Element: Scalar + Default, ExtraArgs = ()>:
    CommitmentScheme<Commitment, Element, ExtraArgs>
{
    fn commit_col(&self, scalars: Vec<Element>, args: &ExtraArgs) -> Result<Vec<Commitment>> {
        let grid = Grid::new(scalars, &Element::default());

        let (rows, columns) = grid.inner.shape();
        ensure!(rows == columns, "Not a grid");

        // Commit to each column
        let commitments: Vec<_> = grid
            .inner
            .column_iter()
            .map(|view| self.commit(view.as_slice(), args))
            .collect();
        Ok(commitments)
    }
}
