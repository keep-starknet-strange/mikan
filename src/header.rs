use std::vec;

use frieda::{
    api::{sample, verify},
    Commitment, FriProof, FriedaError, SampleResult,
};
use malachitebft_test::Address;
use serde::{Deserialize, Serialize};

use crate::error::BlockError;

#[derive(Debug)]
pub struct Header {
    pub block_number: usize,
    pub timestamp: usize,
    pub block_hash: Vec<u8>,
    /// DA commitment for this block.
    pub da_commitment: Commitment,
    /// block of parent block.
    pub parent_hash: Vec<u8>,
    /// Hash of the FinalityParams of the parent block
    pub parent_finality_hash: Vec<u8>,
    /// last block number.
    pub last_block_number: usize,
    /// hash of the data in the current block.
    pub data_hash: Vec<u8>,
    /// hash of the validators.
    pub validator_hash: Vec<u8>,
    /// address of proposer of this block.
    pub proposer_address: Address,
}

#[allow(dead_code)]
impl Header {
    /// Creates a new block header and computes its hash.
    pub fn new(
        block_number: usize,
        timestamp: usize,
        parent_hash: Vec<u8>,
        last_block_number: usize,
        data_hash: Vec<u8>,
        validator_hash: Vec<u8>,
        proposer_address: Address,
        da_commitment: Commitment,
        parent_finality_hash:Vec<u8>
    ) -> Self {
        let mut header = Header {
            block_number,
            timestamp,
            parent_hash: parent_hash.clone(),
            block_hash: Vec::default(),
            da_commitment,
            last_block_number,
            parent_finality_hash,
            data_hash,
            validator_hash,
            proposer_address,
        };
        header.block_hash = header.compute_block_hash();
        header
    }

    pub fn basic_validation(&self) -> Result<(), BlockError> {
        if self.parent_hash.is_empty() {
            return Err(BlockError::ParentHashValidationFail);
        }

        Ok(())
    }

    ///Compute block hash
    pub fn compute_block_hash(&self) -> Vec<u8> {
        todo!()
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
