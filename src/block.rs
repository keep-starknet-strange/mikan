use crate::header::Header;
use frieda::{
    api::{sample, verify},
    Commitment, FriProof, FriedaError, SampleResult,
};
use serde::{Deserialize, Serialize};

#[derive(Debug,Serialize,Deserialize)]
struct Block {
    /// Block Header
    pub header: Header,
    /// Blobs
    pub blobs: Vec<u8>,
    /// DA commitments
    pub da_commitment: Commitment,
}

impl Block {
    /// Create a new block
    pub fn new(header: Header, blobs: Vec<u8>, da_commitment: Commitment) -> Self {
        Block {
            header,
            blobs,
            da_commitment,
        }
    }

    /// Sample from the commitment
    pub fn sample(&self) -> Result<SampleResult, FriedaError> {
        sample(&self.da_commitment)
    }

    /// Verify the commitment against a proof
    pub fn verify_data(&self, proof: FriProof) -> Result<bool, FriedaError> {
        Ok(verify(&self.da_commitment, &proof)?)
    }
}
