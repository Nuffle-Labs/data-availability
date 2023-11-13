use crate::{erasure::ReedSolomon, grid::Grid, Encoding, ErasureCommitment, Transcript};
use core::ops::Neg;
use eyre::Result;
use lambdaworks_crypto::commitments::{
    kzg::{KateZaveruchaGoldberg, StructuredReferenceString},
    traits::IsCommitmentScheme,
};
use lambdaworks_math::{
    cyclic_group::IsGroup,
    elliptic_curve::{
        short_weierstrass::{
            curves::bls12_381::{
                curve::BLS12381Curve,
                default_types::{FrElement, FrField},
                pairing::BLS12381AtePairing,
                twist::BLS12381TwistCurve,
            },
            point::ShortWeierstrassProjectivePoint,
        },
        traits::{IsEllipticCurve, IsPairing},
    },
    field::element::FieldElement,
    polynomial::Polynomial,
    traits::ByteConversion,
    unsigned_integer::element::U256,
};
use rand::Rng;
use reed_solomon_novelpoly::WrappedShard;

const KZG_COMMITMENT_SIZE: usize = 48;
const BLS_FE_SIZE_BYTES: usize = 32;
pub type KzgCommitmentPoint = ShortWeierstrassProjectivePoint<BLS12381Curve>;

pub struct KzgCommitmentScheme {
    kzg: KateZaveruchaGoldberg<FrField, BLS12381AtePairing>,
}

impl KzgCommitmentScheme {
    fn new(powers_of_tau_len: usize) -> Self {
        let kzg = KateZaveruchaGoldberg::<FrField, BLS12381AtePairing>::new(Self::create_srs(
            powers_of_tau_len,
        ));
        Self { kzg }
    }

    // TODO: random test srs, use better one a different time
    fn create_srs(
        powers_of_tau_len: usize,
    ) -> StructuredReferenceString<
        <BLS12381AtePairing as IsPairing>::G1Point,
        <BLS12381AtePairing as IsPairing>::G2Point,
    > {
        type G1 = KzgCommitmentPoint;

        let mut rng = rand::thread_rng();
        let toxic_waste = FrElement::new(U256 {
            limbs: [
                rng.gen::<u64>(),
                rng.gen::<u64>(),
                rng.gen::<u64>(),
                rng.gen::<u64>(),
            ],
        });
        let g1 = BLS12381Curve::generator();
        let g2 = BLS12381TwistCurve::generator();
        let powers_main_group: Vec<G1> = (0..powers_of_tau_len)
            .map(|exponent| {
                g1.operate_with_self(toxic_waste.pow(exponent as u128).representative())
            })
            .collect();
        let powers_secondary_group = [
            g2.clone(),
            g2.operate_with_self(toxic_waste.representative()),
        ];
        StructuredReferenceString::new(&powers_main_group, &powers_secondary_group)
    }

    fn compress_point(point: KzgCommitmentPoint) -> Result<[u8; KZG_COMMITMENT_SIZE]> {
        let is_compressed = true;
        let is_infinity = point.is_neutral_element();
        let is_lexographically_largest =
            point.y().representative() > point.y().neg().representative();
        let mut p = point.x().clone();
        if is_infinity {
            p = FieldElement::zero();
        }

        let x_bytes = p.to_bytes_be();

        // let rep = p.representative().limbs;
        // x_bytes[0..8].copy_from_slice(&rep[5].to_be_bytes());
        // x_bytes[8..16].copy_from_slice(&rep[4].to_be_bytes());
        // x_bytes[16..24].copy_from_slice(&rep[3].to_be_bytes());
        // x_bytes[24..32].copy_from_slice(&rep[2].to_be_bytes());
        // x_bytes[32..40].copy_from_slice(&rep[1].to_be_bytes());
        // x_bytes[40..48].copy_from_slice(&rep[0].to_be_bytes());
        //

        let mut bytes: [u8; 48] = x_bytes[..48].try_into()?;

        if is_compressed {
            bytes[0] |= 1 << 7;
        }

        if is_infinity {
            bytes[0] |= 1 << 6;
        }

        if is_compressed && !is_infinity && is_lexographically_largest {
            bytes[0] |= 1 << 5;
        }

        Ok(bytes)
    }

    fn decompress(bytes: &[u8]) {
        todo!("decompress")
    }

    fn scalars(data: &[u8]) -> Result<Vec<FrElement>> {
        let (oks, errs): (Vec<_>, Vec<_>) = data
            .chunks(BLS_FE_SIZE_BYTES)
            .map(FrElement::from_bytes_le)
            .partition(Result::is_ok);
        if errs.len() > 0 {
            Err(eyre::eyre!("Failed to parse scalars: {:?}", errs))
        } else {
            Ok(oks.into_iter().map(Result::unwrap).collect())
        }
    }

    fn build_root(points: &[KzgCommitmentPoint]) -> KzgCommitmentPoint {
        println!("Building root for points: {:?}", points);
        // KZG is homomorphic, this should work well
        points
            .into_iter()
            .fold(KzgCommitmentPoint::neutral_element(), |acc, next| {
                acc.operate_with(&next)
            })
    }
}

pub struct KzgWitness {
    pub witness: Vec<(KzgCommitmentPoint, KzgCommitmentPoint)>,
    pub x: FrElement,
    pub u: FrElement,
}
impl KzgWitness {
    fn new(
        commitments: Vec<(KzgCommitmentPoint, KzgCommitmentPoint)>,
        x: FrElement,
        u: FrElement,
    ) -> Self {
        Self {
            witness: commitments,
            x,
            u,
        }
    }
}

impl Encoding<KzgWitness> for KzgCommitmentScheme {
    fn encode(&self, data: &[u8]) -> Result<ErasureCommitment<KzgWitness>> {
        let data = data.to_vec();

        let (rs, encoded_data) = ReedSolomon::encode_fit(data, BLS_FE_SIZE_BYTES)?;
        let encoded_data = ReedSolomon::shards_to_bytes(encoded_data);

        // Scalar the data
        let flattened_data = encoded_data.iter().cloned().flatten().collect::<Vec<_>>();
        let scalars = Self::scalars(&flattened_data)?;

        let nullifier_scalar = FrElement::one();
        let grid = Grid::new(scalars, &nullifier_scalar);
        let (rows, columns) = grid.inner.shape();
        assert_eq!(rows, columns, "Not a grid");

        // Commit to each column
        // TODO: pick x at random
        let x = FrElement::one();
        // TODO: pick u at random
        let upsilon = FrElement::one();

        let commitments: Vec<_> = grid
            .inner
            .column_iter()
            .map(|view| {
                let fields = view.iter().cloned().collect::<Vec<_>>();
                let poly = Polynomial::new(&fields);
                let commitment = self.kzg.commit(&poly);
                let proof = self.kzg.open_batch(&x, &fields, &[poly], &upsilon);
                (commitment, proof)
            })
            .collect();

        println!("commitments: {:?}", commitments);

        Ok(ErasureCommitment {
            commitment: KzgWitness::new(commitments, x, upsilon),
            encoding: encoded_data,
            rs,
        })
    }

    fn extract(
        &self,
        transcripts: Vec<Option<Transcript>>,
        rs: reed_solomon_novelpoly::ReedSolomon,
    ) -> Result<Vec<u8>> {
        let transcripts: Vec<_> = transcripts
            .into_iter()
            .map(|x| x.map(WrappedShard::from))
            .collect();
        let rs = rs.reconstruct(transcripts.clone())?;
        Ok(rs)
    }

    // TODO: test me
    fn verify(&self, commitment: KzgWitness, transcripts: Vec<Option<Transcript>>) -> bool {
        let fields = Grid::new(
            transcripts
                .iter()
                .map(|x| {
                    if x.is_none() {
                        FrElement::zero()
                    } else {
                        FrElement::from_bytes_le(&x.clone().unwrap()).unwrap()
                    }
                })
                .collect(),
            &FrElement::zero(),
        );
        commitment
            .witness
            .iter()
            .zip(fields.inner.column_iter())
            .map(|((c, p), col)| {
                let col = col.iter().cloned().collect::<Vec<_>>();
                self.kzg
                    .verify_batch(&commitment.x, &col, &[c.clone()], &p, &commitment.u)
            })
            .reduce(|a, b| a && b)
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::usize;
    use rand::distributions::Standard;

    fn test_fields(amt: usize) -> Vec<u8> {
        let rng = rand::thread_rng();

        rng.sample_iter(&Standard)
            .take(amt)
            .map(|x: u64| FrElement::from(x))
            .map(|x| {
                let bytes = x.to_bytes_be();
                assert_eq!(bytes.len(), BLS_FE_SIZE_BYTES);
                bytes
            })
            .flatten()
            .collect::<Vec<u8>>()
    }

    #[test]
    fn test_kzg_encode() {
        let data = test_fields(512);
        let kzgcs = KzgCommitmentScheme::new(513);
        let ErasureCommitment { commitment, .. } = kzgcs.encode(&data).unwrap();
        println!("commitment: {:?}", commitment.witness);
    }

    #[test]
    fn test_recoverability() {
        let data = test_fields(5);
        let kzgcs = KzgCommitmentScheme::new(6);
        let ErasureCommitment { encoding, rs, .. } = kzgcs.encode(&data).unwrap();
        let recovered = kzgcs
            .extract(encoding.iter().cloned().map(Some).collect(), rs)
            .unwrap();
        let recovered_fields = KzgCommitmentScheme::scalars(&recovered).unwrap();
        let data_fields = KzgCommitmentScheme::scalars(&data).unwrap();
        assert_eq!(recovered_fields[0], data_fields[0]);
        assert_eq!(recovered_fields[1], data_fields[1]);
        assert_eq!(recovered_fields[2], data_fields[2]);
        assert_eq!(recovered_fields[3], data_fields[3]);
    }

    #[test]
    fn test_scalar_creation() {
        let fe = FrElement::one() * FrElement::from(64_u64);
        let scalar = fe.to_bytes_le();
        assert_eq!(scalar.len(), BLS_FE_SIZE_BYTES);
        let new_scalar = KzgCommitmentScheme::scalars(&scalar).unwrap();
        assert_eq!(fe, new_scalar[0]);

        let scalars = vec![FrElement::from(64_u64), FrElement::from(128_u64)];
        let bytes: Vec<u8> = scalars
            .iter()
            .map(FrElement::to_bytes_le)
            .flatten()
            .collect();
        let new_scalars = KzgCommitmentScheme::scalars(&bytes).unwrap();
        assert_eq!(new_scalars[0], scalars[0]);
        assert_eq!(new_scalars[1], scalars[1]);
    }

    #[test]
    fn test_build_root() {}

    #[test]
    fn test_scalar_recreation() {
        let fe = FrElement::from(1);
        let fe2 = FrElement::from(2);
        let fe3 = FrElement::from(3);
        let fe4 = FrElement::from(4);
        let scalar_bytes = vec![fe.clone(), fe2.clone(), fe3.clone(), fe4.clone()]
            .iter()
            .map(FrElement::to_bytes_le)
            .flatten()
            .collect::<Vec<u8>>();
        let scalars = KzgCommitmentScheme::scalars(&scalar_bytes).unwrap();
        println!("scalars: {:?}", scalars);
        assert_eq!(fe, scalars[0]);
        assert_eq!(fe2, scalars[1]);
        assert_eq!(fe3, scalars[2]);
        assert_eq!(fe4, scalars[3]);
        println!("scalar_bytes: {:?}", scalar_bytes);

        let (rs, encoded) =
            ReedSolomon::encode_fit(scalar_bytes.clone(), BLS_FE_SIZE_BYTES).unwrap();
        println!("encoded: {:?}", encoded);
        let recovered = rs
            .reconstruct(ReedSolomon::shards_to_nullifiers(encoded))
            .unwrap();
        println!("recovered: {:?}", recovered);
        let scalars = KzgCommitmentScheme::scalars(&recovered);
        println!("scalars: {:?}", scalars);
    }

    #[test]
    fn test_verify() {
        let data = test_fields(4);
        let kzgcs = KzgCommitmentScheme::new(6);
        let ErasureCommitment {
            encoding,
            commitment,
            ..
        } = kzgcs.encode(&data).unwrap();
        let recovered = kzgcs.verify(commitment, encoding.iter().cloned().map(Some).collect());
        assert!(recovered);
    }
}
