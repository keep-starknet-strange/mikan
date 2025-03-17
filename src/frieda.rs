//! FRIEDA integration for DAZK
//! FRI Extended for Data Availability: a FRI-based Data Availability Sampling library, written in Rust.
//! FRIEDA repository: https://github.com/keep-starknet-strange/frieda
//! TODO: Integrate FRIEDA library into DAZK for the DA primitives.

mod tests {

    #[test]
    fn test_frieda() {
        use frieda::api::{commit, generate_proof, sample};
        let data_size = 1024 * 32; // 32 KB
        let data: Vec<u8> = (0..data_size).map(|i| (i % 256) as u8).collect();

        let commitment = commit(&data).unwrap();

        let _sample_result = sample(&commitment).unwrap();

        let proof_result = generate_proof(&commitment);
        // TODO: for now the proof is not generated in FRIEDA, and it returns an error.
        assert!(proof_result.is_err());
    }
}
