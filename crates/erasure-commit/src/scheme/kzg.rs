use crate::{
    erasure::ReedSolomon, ColumnCommitmentScheme, CommitmentScheme, ErasureCodec,
    ErasureCommitment, ErasureCommitmentScheme, ErasureEncoding,
};
use eyre::{ensure, Result};
use lambdaworks_crypto::commitments::{
    kzg::{KateZaveruchaGoldberg, StructuredReferenceString},
    traits::IsCommitmentScheme,
};
pub use lambdaworks_math::elliptic_curve::short_weierstrass::curves::bls12_381::default_types::FrElement;
use lambdaworks_math::{
    cyclic_group::IsGroup,
    elliptic_curve::{
        short_weierstrass::{
            curves::bls12_381::{
                curve::BLS12381Curve, default_types::FrField, pairing::BLS12381AtePairing,
            },
            point::ShortWeierstrassProjectivePoint,
        },
        traits::IsPairing,
    },
    polynomial::Polynomial,
    traits::ByteConversion,
    unsigned_integer::element::U256,
};
use lambdaworks_math::{
    elliptic_curve::{
        short_weierstrass::curves::bls12_381::{
            field_extension::BLS12381PrimeField, twist::BLS12381TwistCurve,
        },
        traits::{FromAffine, IsEllipticCurve},
    },
    field::element::FieldElement,
};
use rand::{Rng, RngCore};
use reed_solomon_novelpoly::WrappedShard;
use std::path::PathBuf;

// Standard projective points
pub type KzgCommitment = ShortWeierstrassProjectivePoint<BLS12381Curve>;
pub type PolynomialCommitment = KzgCommitment;
pub type KzgProof = KzgCommitment;

// Points compressed, currently only affine coordinates
pub type CompressedPolynomialCommitment = BLS12381Affine;
pub type CompressedKzgProof = BLS12381Affine;
pub type FpElement = FieldElement<BLS12381PrimeField>;

/// The size of a compressed KZG commitment, in bytes
pub const COMMITMENT_LEN_BYTES: usize = 48;
/// The expected size of each BLS Field Element, which is a U256
pub const BLS_FE_SIZE_BYTES: usize = 32;

/// A KZG commitment in affine representation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BLS12381Affine {
    x: FpElement,
    y: FpElement,
}

impl From<KzgCommitment> for BLS12381Affine {
    fn from(x: KzgCommitment) -> Self {
        let point = x.to_affine();
        Self {
            x: point.x().clone(),
            y: point.y().clone(),
        }
    }
}

impl TryFrom<BLS12381Affine> for KzgCommitment {
    type Error = eyre::Error;
    fn try_from(value: BLS12381Affine) -> Result<Self, Self::Error> {
        ShortWeierstrassProjectivePoint::from_affine(value.x, value.y)
            .map_err(|e| eyre::eyre!("{:?}", e))
    }
}

/// The KZG erasure commitment scheme
pub struct KzgCommitmentScheme {
    kzg: KateZaveruchaGoldberg<FrField, BLS12381AtePairing>,
}

impl TryFrom<PathBuf> for KzgCommitmentScheme {
    type Error = eyre::Error;
    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        let srs = <StructuredReferenceString<
            <BLS12381AtePairing as IsPairing>::G1Point,
            <BLS12381AtePairing as IsPairing>::G2Point,
        >>::from_file(&path.display().to_string())
        .map_err(|e| eyre::eyre!("{:?}", e))?;
        Ok(Self::new(KateZaveruchaGoldberg::new(srs)))
    }
}

impl KzgCommitmentScheme {
    pub fn new(kzg: KateZaveruchaGoldberg<FrField, BLS12381AtePairing>) -> Self {
        Self { kzg }
    }

    /// Used to generate an SRS with a limited size of ptau
    ///
    /// Insecurity: SRS should be generated using more than one field of toxic waste!
    pub fn insecure_generate(powers_of_tau_len: usize) -> Self {
        type G1 = KzgCommitment;

        let mut rng = rand::thread_rng();

        let toxic_waste = Self::rng_field(&mut rng);
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

        let srs = StructuredReferenceString::new(&powers_main_group, &powers_secondary_group);
        let kzg = KateZaveruchaGoldberg::<FrField, BLS12381AtePairing>::new(srs);
        Self { kzg }
    }

    /// Convert an arbitrary byte array to a vector of KZG scalars
    pub fn scalars(data: &[u8]) -> Result<Vec<FrElement>> {
        let (oks, errs): (Vec<_>, Vec<_>) = data
            .chunks(BLS_FE_SIZE_BYTES)
            .map(Self::bytes_to_element)
            .partition(Result::is_ok);
        if !errs.is_empty() {
            Err(eyre::eyre!("Failed to parse scalars: {:?}", errs))
        } else {
            // Safety: already filtered these by above
            Ok(oks.into_iter().map(Result::unwrap).collect())
        }
    }

    /// Helper function to convert bytes to element, used to avoid any assumptions
    /// about endianness
    pub fn bytes_to_element(bytes: &[u8]) -> Result<FrElement> {
        FrElement::from_bytes_le(bytes).map_err(|e| eyre::eyre!("{:?}", e))
    }

    /// Helper function to convert element to bytes, used to avoid any assumptions
    /// about endianness
    pub fn element_to_bytes(element: FrElement) -> Vec<u8> {
        element.to_bytes_le()
    }

    pub fn rng_field<R: RngCore>(rng: &mut R) -> FrElement {
        FrElement::new(U256 {
            limbs: [
                rng.gen::<u64>(),
                rng.gen::<u64>(),
                rng.gen::<u64>(),
                rng.gen::<u64>(),
            ],
        })
    }
}

/// The commitment to a column, containing the polynomial commitment and each fields commitment
#[derive(Debug, Clone)]
pub struct ColumnCommitment {
    pub poly_c: PolynomialCommitment,
    pub poly: Polynomial<FrElement>,
    pub proof: KzgProof,
}

impl ColumnCommitment {
    pub fn new(poly_c: PolynomialCommitment, poly: Polynomial<FrElement>, proof: KzgProof) -> Self {
        Self {
            poly_c,
            poly,
            proof,
        }
    }

    pub fn y(&self, x: &FrElement) -> FrElement {
        self.poly.evaluate(x)
    }
}

/// Column commitments in compressed form
#[derive(Debug, Clone)]
pub struct CompressedColumnCommitment {
    pub poly_c: CompressedPolynomialCommitment,
    pub y: FrElement,
    pub proof: CompressedKzgProof,
}

impl CompressedColumnCommitment {
    pub fn try_from(col_c: ColumnCommitment, x: &FrElement) -> Result<Self> {
        let y = col_c.y(x);
        Ok(Self {
            poly_c: col_c.poly_c.try_into()?,
            y,
            proof: col_c.proof.try_into()?,
        })
    }
}

pub struct KzgWitness {
    pub x: FrElement,
    pub commitments: Vec<CompressedColumnCommitment>,
    pub root: BLS12381Affine,
}

impl KzgWitness {
    pub fn new(
        x: FrElement,
        commitments: Vec<CompressedColumnCommitment>,
        root: BLS12381Affine,
    ) -> Self {
        Self {
            x,
            commitments,
            root,
        }
    }
}

impl CommitmentScheme<ColumnCommitment, FrElement, FrElement> for KzgCommitmentScheme {
    fn commit(&self, data: &[FrElement], args: &FrElement) -> ColumnCommitment {
        let x = args;
        let poly = Polynomial::new(data);
        let y = poly.evaluate(x);
        // Commit to the fields
        let poly_c = self.kzg.commit(&poly);

        let p = self.kzg.open(x, &y, &poly);

        ColumnCommitment::new(poly_c, poly, p)
    }

    fn verify(&self, commitment: ColumnCommitment, args: &FrElement) -> bool {
        self.kzg.verify(
            &args,
            &commitment.poly.evaluate(&args),
            &commitment.poly_c,
            &commitment.proof,
        )
    }
}

impl ColumnCommitmentScheme<ColumnCommitment, FrElement, FrElement> for KzgCommitmentScheme {}

impl ErasureCodec for KzgCommitmentScheme {
    fn encode(&self, data: &[u8]) -> Result<ErasureEncoding> {
        let data = data.to_vec();

        let (rs, encoded_data) = ReedSolomon::encode_fit(data, BLS_FE_SIZE_BYTES)?;
        let encoding = ReedSolomon::shards_to_bytes(encoded_data);

        Ok(ErasureEncoding {
            rs,
            codewords: encoding,
        })
    }

    fn extract(&self, encoding: ErasureEncoding) -> Result<Vec<u8>> {
        let transcripts: Vec<_> = encoding
            .codewords
            .into_iter()
            .map(|x| x.map(WrappedShard::from))
            .collect();
        let rs = encoding.rs.reconstruct(transcripts.clone())?;
        Ok(rs)
    }
}

impl ErasureCommitmentScheme<KzgWitness> for KzgCommitmentScheme {
    fn encode_commit(&self, data: &[u8]) -> Result<ErasureCommitment<KzgWitness>> {
        let encoding = self.encode(data)?;

        // Scalar the data
        let scalars = Self::scalars(
            &encoding
                .codewords
                .iter()
                .flatten()
                .cloned()
                .flatten()
                .collect::<Vec<_>>(),
        )?;

        // Pick random field coordinates
        let mut rng = rand::thread_rng();
        let x = Self::rng_field(&mut rng);

        // Commit to each column
        let (commitments, polynomials): (Vec<ColumnCommitment>, Vec<Polynomial<FrElement>>) = self
            .commit_col(scalars, &x)?
            .into_iter()
            .map(|c| (c.clone(), c.poly))
            .unzip();

        // TODO: maybe remove this batching
        let u = FrElement::one();
        let root = self.kzg.commit(
            &polynomials
                .iter()
                .rev()
                .fold(Polynomial::zero(), |acc, polynomial| {
                    acc * u.to_owned() + polynomial
                }),
        );

        Ok(ErasureCommitment {
            witness: KzgWitness::new(
                x.clone(),
                commitments
                    .iter()
                    .filter_map(|c| CompressedColumnCommitment::try_from(c.clone(), &x).ok())
                    .collect(),
                root.into(),
            ),
            encoding,
        })
    }

    fn verify_extract(&self, ec: ErasureCommitment<KzgWitness>) -> Result<Vec<u8>> {
        let verified = ec.witness.commitments.into_iter().all(|c| {
            if let (Ok(poly_c), Ok(proof)) = (c.poly_c.try_into(), c.proof.try_into()) {
                self.kzg.verify(&ec.witness.x, &c.y, &poly_c, &proof)
            } else {
                false
            }
        });
        ensure!(verified, "Commitment verification failed");

        let transcripts: Vec<_> = ec
            .encoding
            .codewords
            .into_iter()
            .map(|x| x.map(WrappedShard::from))
            .collect();
        let rs = ec.encoding.rs.reconstruct(transcripts.clone())?;
        // TODO: verify witness parts are actually part of the witness
        Ok(rs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::usize;
    use rand::distributions::Standard;

    fn test_fields(amt: usize) -> Vec<u8> {
        let _ = pretty_env_logger::try_init();
        let rng = rand::thread_rng();

        rng.sample_iter(&Standard)
            .take(amt)
            .map(|x: u64| FrElement::from(x))
            .map(|x| {
                let bytes = KzgCommitmentScheme::element_to_bytes(x);
                assert_eq!(bytes.len(), BLS_FE_SIZE_BYTES);
                bytes
            })
            .flatten()
            .collect::<Vec<u8>>()
    }

    #[test]
    fn test_kzg_from_srs() {
        let base_dir = env!("CARGO_MANIFEST_DIR");
        let srs_file = base_dir.to_owned() + "/test_srs/srs_3_g1_elements.bin";
        let path = PathBuf::from(srs_file);
        assert!(path.exists());
        KzgCommitmentScheme::try_from(path).unwrap();
    }

    #[test]
    fn test_field_commit() {
        let fields = test_fields(4);
        let fields = KzgCommitmentScheme::scalars(&fields).unwrap();
        let kzgcs = KzgCommitmentScheme::insecure_generate(5);
        let x = FrElement::one();
        let c = kzgcs.commit(&fields, &x);
        assert!(kzgcs.kzg.verify(
            &x,
            &c.y(&x),
            &c.poly_c.try_into().unwrap(),
            &c.proof.try_into().unwrap()
        ));
    }

    #[test]
    fn test_kzg_commit_encode() {
        let bytes = test_fields(4);
        let kzgcs = KzgCommitmentScheme::insecure_generate(5);
        let commitments = kzgcs.encode_commit(&bytes).unwrap();
        assert!(kzgcs.verify_extract(commitments).is_ok());
    }

    #[test]
    fn test_recoverability() {
        let data = test_fields(5);
        let kzgcs = KzgCommitmentScheme::insecure_generate(6);
        let encoding = kzgcs.encode(&data).unwrap();
        let recovered = kzgcs.extract(encoding).unwrap();
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
        let scalar = KzgCommitmentScheme::element_to_bytes(fe.clone());
        assert_eq!(scalar.len(), BLS_FE_SIZE_BYTES);

        let new_scalar = KzgCommitmentScheme::scalars(&scalar).unwrap();
        assert_eq!(fe, new_scalar[0]);

        let scalars = vec![FrElement::from(64_u64), FrElement::from(128_u64)];
        let bytes: Vec<u8> = scalars
            .clone()
            .into_iter()
            .map(KzgCommitmentScheme::element_to_bytes)
            .flatten()
            .collect();
        let new_scalars = KzgCommitmentScheme::scalars(&bytes).unwrap();

        assert_eq!(new_scalars[0], scalars[0]);
        assert_eq!(new_scalars[1], scalars[1]);
    }

    #[test]
    fn test_scalar_recreation() {
        let fe = FrElement::from(1);
        let fe2 = FrElement::from(2);
        let fe3 = FrElement::from(3);
        let fe4 = FrElement::from(4);
        let scalar_bytes = vec![fe.clone(), fe2.clone(), fe3.clone(), fe4.clone()]
            .into_iter()
            .map(KzgCommitmentScheme::element_to_bytes)
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
            .reconstruct(ReedSolomon::shards_to_option(encoded))
            .unwrap();
        println!("recovered: {:?}", recovered);
        let scalars = KzgCommitmentScheme::scalars(&recovered).unwrap();
        println!("scalars: {:?}", scalars);
        assert_eq!(fe, scalars[0]);
        assert_eq!(fe2, scalars[1]);
        assert_eq!(fe3, scalars[2]);
        assert_eq!(fe4, scalars[3]);
    }

    #[test]
    fn test_verify() {
        let data = test_fields(4);
        let kzgcs = KzgCommitmentScheme::insecure_generate(6);
        let c = kzgcs.encode_commit(&data).unwrap();
        let recovered = kzgcs.verify_extract(c);
        assert!(recovered.is_ok());
    }
}
