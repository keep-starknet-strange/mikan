use std::vec;

use frieda::{
    api::{sample, verify},
    Commitment, FriProof, FriedaError, SampleResult,
};
use malachitebft_proto::Protobuf;
use malachitebft_test::{utils::validators::make_validators, Address};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};

use crate::{block::mock_make_validator, error::BlockError};

#[derive(Debug)]
pub struct Header {
    pub block_number: usize,
    pub timestamp: usize,
    /// Hash of current block
    pub block_hash: Vec<u8>,
    /// DA commitment for this block.
    pub da_commitment: Option<Commitment>,
    /// block of parent block.
    pub parent_hash: Vec<u8>,
    /// Hash of the FinalityParams of the parent block
    pub parent_finality_hash: Vec<u8>,
    /// last block number.
    pub last_block_number: usize,
    /// Merkle root of the data in the current block.
    /// Leaves of this tree will be the raw bytes of each blob
    pub data_hash: Vec<u8>,
    /// address of proposer of this block.
    pub proposer_address: Address,
}

#[allow(dead_code)]
impl Header {
    /// Creates a new block header and computes its hash.
    pub fn new(
        block_number: usize,
        timestamp: usize,
        last_block_number: usize,
        data_hash: Vec<u8>,
        proposer_address: Address,
        da_commitment: Commitment,
        parent_finality_hash: Vec<u8>,
        parent_hash: Vec<u8>,
    ) -> Self {
        let mut header = Header {
            block_number,
            timestamp,
            da_commitment: Some(da_commitment),
            last_block_number,
            parent_finality_hash,
            data_hash,
            proposer_address,
            parent_hash,
            block_hash: Vec::new(),
        };
        header.block_hash = header.compute_block_hash();
        header
    }

    pub fn default() -> Self {
        Self {
            block_number: 0,
            timestamp: 0,
            block_hash: Vec::new(),
            da_commitment: None,
            parent_hash: Vec::new(),
            parent_finality_hash: Vec::new(),
            last_block_number: 0,
            data_hash: Vec::new(),
            proposer_address: mock_make_validator(),
        }
    }

    pub fn basic_validation(&self) -> Result<(), BlockError> {
        if self.parent_finality_hash.is_empty() {
            return Err(BlockError::NullParentFinalityHash);
        }

        if self.block_number <= 0 {
            return Err(BlockError::InvalidBlockNumber(self.block_number));
        }

        Ok(())
    }

    ///Compute block hash
    pub fn compute_block_hash(&self) -> Vec<u8> {
        let mut hasher = Sha3_256::new();

        hasher.update(self.block_number.to_le_bytes());
        hasher.update(self.parent_hash.clone());
        hasher.update(self.data_hash.clone());
        hasher.update(self.proposer_address.to_string().as_bytes());

        let result = hasher.finalize().as_slice().to_owned();
        result
    }

    /// Sample from the commitment
    pub fn sample(&self) -> Result<SampleResult, FriedaError> {
        sample(&self.da_commitment.clone().unwrap())
    }

    /// Verify the commitment against a proof
    pub fn verify_data(&self, proof: FriProof) -> Result<bool, FriedaError> {
        verify(&self.da_commitment.clone().unwrap(), &proof)
    }
}
