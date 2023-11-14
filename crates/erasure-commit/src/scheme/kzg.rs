use crate::{erasure::ReedSolomon, grid::Grid, Encoding, ErasureCommitment, Transcript};
use eyre::{ensure, Result};
use lambdaworks_crypto::commitments::{
    kzg::{KateZaveruchaGoldberg, StructuredReferenceString},
    traits::IsCommitmentScheme,
};
use lambdaworks_math::elliptic_curve::{
    short_weierstrass::curves::bls12_381::twist::BLS12381TwistCurve, traits::IsEllipticCurve,
};
use lambdaworks_math::{
    cyclic_group::IsGroup,
    elliptic_curve::{
        short_weierstrass::{
            curves::bls12_381::{
                curve::BLS12381Curve,
                default_types::{FrElement, FrField},
                pairing::BLS12381AtePairing,
            },
            point::ShortWeierstrassProjectivePoint,
        },
        traits::IsPairing,
    },
    polynomial::Polynomial,
    traits::ByteConversion,
    unsigned_integer::element::U256,
};
use rand::{Rng, RngCore};
use reed_solomon_novelpoly::WrappedShard;
use std::path::PathBuf;

pub type KzgCommitment = ShortWeierstrassProjectivePoint<BLS12381Curve>;
pub type PolynomialCommitment = KzgCommitment;
pub type KzgProof = KzgCommitment;

/// The size of a compressed KZG commitment, in bytes
pub const COMMITMENT_LEN_BYTES: usize = 48;
/// The expected size of each BLS Field Element, which is a U256
pub const BLS_FE_SIZE_BYTES: usize = 32;

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

    pub fn poly_commit(
        &self,
        fields: &[FrElement],
        x: &FrElement,
        u: &FrElement,
    ) -> (KzgCommitment, KzgCommitment) {
        log::debug!("Committing points: {:?}", fields);
        let poly = Polynomial::new(fields);
        let commitment = self.kzg.commit(&poly);
        let proof = self.kzg.open_batch(x, fields, &[poly], u);
        log::debug!("Commitment: {:?}, Proof: {:?}", commitment, proof);
        (commitment, proof)
    }

    pub fn field_commit(
        &self,
        fields: &[FrElement],
        x: &FrElement,
    ) -> (ColumnCommitment, Polynomial<FrElement>) {
        let poly = Polynomial::new(fields);
        let y = poly.evaluate(x);
        // Commit to the fields
        let poly_c = self.kzg.commit(&poly);

        let p = self.kzg.open(x, &y, &poly);
        assert!(self.kzg.verify(x, &y, &poly_c, &p));

        (ColumnCommitment::new(poly_c, y, p), poly)
    }
}

/// The commitment to a column, containing the polynomial commitment and each fields commitment
pub struct ColumnCommitment {
    // Polynomial commitment to the column
    pub poly_c: PolynomialCommitment,
    // The committed element
    pub y: FrElement,
    // The proof of the element
    pub proof: KzgProof,
}

impl ColumnCommitment {
    pub fn new(poly_c: PolynomialCommitment, y: FrElement, proof: KzgProof) -> Self {
        Self { poly_c, y, proof }
    }
}

pub struct KzgWitness {
    pub x: FrElement,
    pub commitments: Vec<ColumnCommitment>,
    pub root: KzgCommitment,
}

impl KzgWitness {
    pub fn new(x: FrElement, commitments: Vec<ColumnCommitment>, root: KzgCommitment) -> Self {
        Self {
            x,
            commitments,
            root,
        }
    }
    pub fn verify(&self, cs: &KzgCommitmentScheme) -> bool {
        self.commitments
            .iter()
            .all(|c| cs.kzg.verify(&self.x, &c.y, &c.poly_c, &c.proof))
    }
}

impl Encoding<KzgWitness> for KzgCommitmentScheme {
    fn encode(&self, data: &[u8]) -> Result<ErasureCommitment<KzgWitness>> {
        let data = data.to_vec();

        let (rs, encoded_data) = ReedSolomon::encode_fit(data, BLS_FE_SIZE_BYTES)?;
        let encoded_data = ReedSolomon::shards_to_bytes(encoded_data);

        // Scalar the data
        let scalars = Self::scalars(&encoded_data.iter().flatten().cloned().collect::<Vec<_>>())?;

        let nullifier_scalar = FrElement::one();
        let grid = Grid::new(scalars, &nullifier_scalar);
        let (rows, columns) = grid.inner.shape();
        ensure!(rows == columns, "Not a grid");

        // Pick random field coordinates
        let mut rng = rand::thread_rng();
        let x = Self::rng_field(&mut rng);

        let mut polynomials = vec![];
        // Commit to each column
        let commitments: Vec<_> = grid
            .inner
            .column_iter()
            .map(|view| {
                let (commitment, poly) = self.field_commit(view.as_slice(), &x);
                polynomials.push(poly);
                commitment
            })
            .collect();

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
            commitment: KzgWitness::new(x, commitments, root),
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

    fn verify(&self, witness: KzgWitness) -> bool {
        witness
            .commitments
            .iter()
            .all(|c| self.kzg.verify(&witness.x, &c.y, &c.poly_c, &c.proof))
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
    fn test_poly_commit() {
        let fields = test_fields(4);
        let mut rand = rand::thread_rng();
        let fields = KzgCommitmentScheme::scalars(&fields).unwrap();
        let kzgcs = KzgCommitmentScheme::insecure_generate(15);

        let x = FrElement::from(rand.gen::<u64>());
        let commitments = kzgcs.poly_commit(&fields, &x, &x);
        assert!(kzgcs
            .kzg
            .verify_batch(&x, &fields, &[commitments.0.clone()], &commitments.1, &x));
    }

    #[test]
    fn test_field_commit() {
        let fields = test_fields(4);
        let fields = KzgCommitmentScheme::scalars(&fields).unwrap();
        let kzgcs = KzgCommitmentScheme::insecure_generate(5);
        let x = FrElement::one();
        let (c, _) = kzgcs.field_commit(&fields, &x);
        assert!(kzgcs.kzg.verify(&x, &c.y, &c.poly_c, &c.proof));
    }

    #[test]
    fn test_kzg_commit_encode() {
        let bytes = test_fields(4);
        let kzgcs = KzgCommitmentScheme::insecure_generate(5);
        let commitments = kzgcs.encode(&bytes).unwrap();
        assert!(commitments.commitment.verify(&kzgcs));
    }

    #[test]
    fn test_recoverability() {
        let data = test_fields(5);
        let kzgcs = KzgCommitmentScheme::insecure_generate(6);
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
            .reconstruct(ReedSolomon::shards_to_nullifiers(encoded))
            .unwrap();
        println!("recovered: {:?}", recovered);
        let scalars = KzgCommitmentScheme::scalars(&recovered);
        println!("scalars: {:?}", scalars);
    }

    #[test]
    fn test_verify() {
        let data = test_fields(4);
        let kzgcs = KzgCommitmentScheme::insecure_generate(6);
        let ErasureCommitment { commitment, .. } = kzgcs.encode(&data).unwrap();
        let recovered = kzgcs.verify(commitment);
        assert!(recovered);
    }
}
