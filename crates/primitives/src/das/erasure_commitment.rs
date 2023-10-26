use reed_solomon_novelpoly::ReedSolomon;
use eyre::Result;

mod grid;
mod scheme;
mod erasure;

type Transcript = Vec<u8>;
type Codeword = Vec<u8>;

struct ErasureCommitment<Commitment> {
    commitment: Commitment,
    encoding: Vec<Transcript>,
    rs: ReedSolomon,
}

trait Encoding<Commitment> {
    fn encode(&self, data: &[u8]) -> Result<ErasureCommitment<Commitment>>;
    fn extract(
        &self,
        commitment: Commitment,
        transcripts: Vec<Option<Transcript>>,
        rs: ReedSolomon,
    ) -> Result<Vec<u8>>;
}

#[cfg(test)]
mod tests {
    use super::*;
}
