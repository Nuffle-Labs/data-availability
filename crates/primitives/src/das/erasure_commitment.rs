use reed_solomon_novelpoly::ReedSolomon;
use eyre::Result;

mod grid;
mod scheme;
mod erasure;

type Transcript = Vec<u8>;

struct ErasureCommitment<Commitment> {
    commitment: Commitment,
    encoding: Vec<Transcript>,
    rs: ReedSolomon,
}

trait Encoding<Commitment> {
    fn encode(&self, data: &[u8]) -> Result<ErasureCommitment<Commitment>>;
    fn extract(
        &self,
        transcripts: Vec<Option<Transcript>>,
        rs: ReedSolomon,
    ) -> Result<Vec<u8>>;
    fn verify(&self, commitment: Commitment, transcripts: Vec<Option<Transcript>>) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;
}
