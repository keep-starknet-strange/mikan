use crate::{blob::Blob, header::Header};
use frieda::{
    api::{commit, sample, verify},
    Commitment, FriProof, FriedaError, SampleResult,
};

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Block {
    /// Block Header
    pub header: Header,
    /// Blobs
    pub blobs: Vec<Blob>,
    /// DA commitments
    pub da_commitment: Commitment,
}

impl Default for Block {
    fn default() -> Self {
        Block {
            header: Header::default(),
            blobs: vec![],
            da_commitment: commit(&[]).unwrap(),
        }
    }
}

#[allow(dead_code)]
impl Block {
    /// Create a new block
    pub fn new(header: Header, blobs: Vec<Blob>, da_commitment: Commitment) -> Self {
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
        verify(&self.da_commitment, &proof)
    }
}
