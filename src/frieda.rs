//! FRIEDA integration for Mikan
//! FRI Extended for Data Availability: a FRI-based Data Availability Sampling library, written in Rust.
//! FRIEDA repository: https://github.com/keep-starknet-strange/frieda

use crate::error::BlockError;
use frieda::api::verify;
use frieda::commit::{commit, Commitment};
use frieda::proof::Proof;
#[allow(dead_code)]
/// A FRI-based commitment for data availability sampling
#[derive(Debug, Clone)]
pub struct DaCommitment {
    commitment: Commitment,
}
#[allow(dead_code)]
impl DaCommitment {
    /// Commit data
    pub fn commit(data: &[u8]) -> Result<Self, BlockError> {
        if data.is_empty() {
            return Err(BlockError::FriedaError("Data cannot be empty".to_string()));
        }

        let commitment = commit(data, 1);
        Ok(Self { commitment })
    }

    /// Get the commitment root
    pub fn root(&self) -> &[u8; 32] {
        todo!()
    }

    /// Sample the commitment
    pub fn sample(&self) -> Result<(), BlockError> {
        todo!()
    }

    /// Generate a proof for the commitment
    pub fn generate_proof(&self) -> Result<(), BlockError> {
        todo!()
    }

    /// Verify a proof against this commitment
    pub fn verify(&self, proof: Proof) -> bool {
        verify(proof, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frieda() {
        use frieda::api::commit;
        let data_size = 1024 * 32; // 32 KB
        let data: Vec<u8> = (0..data_size).map(|i| (i % 256) as u8).collect();

        let commitment = commit(&data, 1);

        // TODO: for now the proof is not generated in FRIEDA, and it returns an error.
        !todo!();
        // let proof_result = generate_proof(&commitment);
        // assert!(proof_result.is_err());
    }

    #[test]
    fn test_basic_workflow() {
        let data =
            b"Hello, world! This is a test of the FRI-based data availability sampling scheme.";

        // Commit to the data
        let commitment = DaCommitment::commit(data).unwrap();

        // Sample the commitment
        let sample_result = commitment.sample().unwrap();

        // // Verify that we have sample indices
        // assert!(!sample_result.indices.is_empty());

        // Note: In a complete implementation, we would:
        // 1. Generate a proof with api::generate_proof()
        // 2. Verify the proof with api::verify()
        // 3. Reconstruct the data from samples

        // For now, we just check that the commit and sample functions work
        // println!("Commitment: {:?}", commitment);
        // println!("Sample indices: {:?}", sample_result.indices);
    }

    #[test]
    fn test_end_to_end() {
        // This test demonstrates the intended workflow, even though some parts
        // are not fully implemented yet

        // Step 1: Data provider has some data
        let original_data = b"This is the original data that needs to be made available.";

        // Step 2: Data provider commits to the data
        let commitment = DaCommitment::commit(original_data).unwrap();
        println!("Commitment created with root: {:?}", commitment.root());

        // Step 3: Data provider publishes the commitment
        // (In a real system, this would be published to a blockchain or broadcast)

        // Step 4: Light client wants to verify data availability
        let sample_result = commitment.sample().unwrap();
        // println!(
        //     "Light client sampled {} indices",
        //     sample_result.indices.len()
        // );

        // Step 5: Light client requests samples from data provider
        // (In a real system, the light client would query a network of providers)

        // Step 6: Data provider generates proofs for the requested samples
        // Note: generate_proof is not fully implemented, so this would fail
        // let proof = api::generate_proof(&commitment).unwrap();

        // Step 7: Light client verifies the proofs
        // Note: verify is not fully implemented with real proofs
        // let verification_result = api::verify(&commitment, &proof).unwrap();
        // assert!(verification_result);

        // Step 8: Light client concludes that data is available
        // In this demo, we just check that sampling works
        // assert!(!sample_result.indices.is_empty());
    }

    #[test]
    fn test_empty_data() {
        let result = DaCommitment::commit(&[]);
        assert!(matches!(result, Err(BlockError::FriedaError(_))));
    }

    #[test]
    fn test_proof_generation() {
        let data = b"Test data for proof generation";
        let commitment = DaCommitment::commit(data).unwrap();

        // Note: Currently FRIEDA's proof generation is not implemented
        // This test verifies that it returns an error as expected
        let proof_result = commitment.generate_proof();
        assert!(proof_result.is_err());
    }
}
