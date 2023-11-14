use eyre::Result;
use reed_solomon_novelpoly::{recoverablity_subset_size, CodeParams, WrappedShard};
use crate::Transcript;

// The rate to expand the matrix
const INV_RATE: usize = 4;

/// A reed-solomon encoder
pub struct ReedSolomon;

impl ReedSolomon {
    pub fn encode(
        data: &[u8],
        shard_size: usize,
    ) -> Result<(reed_solomon_novelpoly::ReedSolomon, Vec<WrappedShard>)> {
        let params = Self::data_to_code_params(data, shard_size)?;
        Self::make_encoder_and_encode(params, data)
    }

    pub fn encode_fit(
        mut data: Vec<u8>,
        shard_size: usize,
    ) -> Result<(reed_solomon_novelpoly::ReedSolomon, Vec<WrappedShard>)> {
        let params = Self::data_to_code_params(&data, shard_size)?;
        if data.len() < params.k() {
            data.resize(params.k(), 0);
        }
        Self::make_encoder_and_encode(params, &data)
    }

    fn make_encoder_and_encode(
        params: CodeParams,
        data: &[u8],
    ) -> Result<(reed_solomon_novelpoly::ReedSolomon, Vec<WrappedShard>)> {
        let rs = params.make_encoder();
        Ok(rs.encode(data).map(|x| (rs, x))?)
    }

    fn data_to_code_params(data: &[u8], shard_size: usize) -> Result<CodeParams> {
        let word_amt = data.len() / shard_size;
        let encode_word_amt = word_amt * INV_RATE;
        log::debug!(
            "data({}), words({}), encoded_words({})",
            data.len(),
            word_amt,
            encode_word_amt
        );
        // Require that we need at least as many fields as we had to recover
        // CodeParams::derive_parameters(resulting_shards, field_amt)
        let validator_count = word_amt * INV_RATE;

        Ok(CodeParams::derive_parameters(
            validator_count,
            recoverablity_subset_size(validator_count),
        )?)
    }

    pub fn shards_to_bytes(shards: Vec<WrappedShard>) -> Vec<Transcript> {
        shards
            .into_iter()
            .map(|x| x.into_inner())
            .collect::<Vec<Transcript>>()
    }

    pub fn shards_to_nullifiers(shards: Vec<WrappedShard>) -> Vec<Option<WrappedShard>> {
        shards.into_iter().map(Option::Some).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{distributions::Standard, Rng};

    const CODEWORD_SIZE: usize = 4;

    fn test_data(n: usize) -> Vec<u8> {
        let rng = rand::thread_rng();
        rng.sample_iter(&Standard)
            .take(n)
            .map(|x: u32| {
                let bytes = x.to_be_bytes();
                assert_eq!(bytes.len(), CODEWORD_SIZE);
                bytes
            })
            .flatten()
            .collect::<Vec<u8>>()
    }

    #[test]
    fn test_parameters_multiple_of_scalar() {
        let data = test_data(32);
        let rs_params = ReedSolomon::data_to_code_params(&data, CODEWORD_SIZE).unwrap();
        // Assert that the number of shards is a perfect square * invrate
        assert_eq!(rs_params.n(), 128);
        // Assert the recovery only requires shards / invrate
        assert_eq!(rs_params.k(), 32);
    }

    #[test]
    fn test_rs_encode_word_length() {
        let data = test_data(100);
        let rs_params = ReedSolomon::data_to_code_params(&data, CODEWORD_SIZE);
        println!("rs_params: {:?}", rs_params);
        let (_, encoded) = ReedSolomon::encode(&data, CODEWORD_SIZE).unwrap();
        encoded
            .into_iter()
            .for_each(|x| assert_eq!(x.into_inner().len(), CODEWORD_SIZE))
    }

    #[test]
    fn test_rs_recover_within_word_size_imperfect_square() {
        let data = test_data(10);
        let rs_params = ReedSolomon::data_to_code_params(&data, CODEWORD_SIZE);
        println!("rs_params: {:?}", rs_params);
        let (rs, encoded) = ReedSolomon::encode(&data, CODEWORD_SIZE).unwrap();
        let decoded = rs
            .reconstruct(ReedSolomon::shards_to_nullifiers(encoded))
            .unwrap();
        assert_eq!(data, decoded[0..data.len()]);
    }

    #[test]
    fn test_rs_recover_within_word_size_perfect_square() {
        let data = test_data(16);
        let rs_params = ReedSolomon::data_to_code_params(&data, CODEWORD_SIZE);
        println!("rs_params: {:?}", rs_params);
        let (rs, encoded) = ReedSolomon::encode(&data, CODEWORD_SIZE).unwrap();
        let decoded = rs
            .reconstruct(ReedSolomon::shards_to_nullifiers(encoded))
            .unwrap();
        assert_eq!(data, decoded);
    }

    #[test]
    fn test_encode_fit_doesnt_care_about_square() {
        let data = test_data(10);
        let (rs, encode) = ReedSolomon::encode_fit(data.clone(), CODEWORD_SIZE).unwrap();
        let decoded = rs
            .reconstruct(ReedSolomon::shards_to_nullifiers(encode))
            .unwrap();
        assert_eq!(data, decoded[0..data.len()]);
    }
}
