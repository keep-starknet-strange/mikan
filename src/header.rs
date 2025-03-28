use std::vec;

use frieda::{api::verify, commit::Commitment, proof::Proof};
use malachitebft_proto::Protobuf;
use malachitebft_test::{utils::validators::make_validators, Address};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};

use crate::{
    block::{mock_make_validator, Block},
    error::BlockError,
};

#[allow(clippy::too_many_arguments)]
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
    #[allow(clippy::too_many_arguments)]
    /// Creates a new block header and computes its hash.
    pub fn new(
        block_number: usize,
        timestamp: usize,
        last_block_number: usize,
        data_hash: Vec<u8>,
        proposer_address: Address,
        da_commitment: Option<Commitment>,
        parent_finality_hash: Vec<u8>,
        parent_hash: Vec<u8>,
    ) -> Self {
        let mut header = Header {
            block_number,
            timestamp,
            da_commitment: da_commitment,
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

        if self.block_number == 0 {
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
    pub fn sample(&self) -> Result<(), BlockError> {
        !todo!()
    }

    /// Verify the commitment against a proof
    pub fn verify_data(&self, proof: Proof) -> bool {
        verify(proof, None)
    }
}

#[derive(Debug, Default)]
pub struct HeaderBuilder {
    pub block_number: Option<usize>,
    pub timestamp: Option<usize>,
    /// Hash of current block
    pub block_hash: Option<Vec<u8>>,
    /// DA commitment for this block.
    pub da_commitment: Option<Option<Commitment>>,
    /// block of parent block.
    pub parent_hash: Option<Vec<u8>>,
    /// Hash of the FinalityParams of the parent block
    pub parent_finality_hash: Option<Vec<u8>>,
    /// last block number.
    pub last_block_number: Option<usize>,
    /// Merkle root of the data in the current block.
    /// Leaves of this tree will be the raw bytes of each blob
    pub data_hash: Option<Vec<u8>>,
    /// address of proposer of this block.
    pub proposer_address: Option<Address>,
}

impl HeaderBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn block_number(mut self, block: usize) -> Self {
        self.block_number = Some(block);
        self
    }

    pub fn timestamp(mut self, timestamp: usize) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    pub fn block_hash(mut self, block_hash: Vec<u8>) -> Self {
        self.block_hash = Some(block_hash);
        self
    }
    pub fn da_commitment(mut self, da_commitment: Option<Commitment>) -> Self {
        self.da_commitment = Some(da_commitment);
        self
    }
    pub fn parent_hash(mut self, parent_hash: Vec<u8>) -> Self {
        self.parent_hash = Some(parent_hash);
        self
    }
    pub fn parent_finality_hash(mut self, parent_finality_hash: Vec<u8>) -> Self {
        self.parent_finality_hash = Some(parent_finality_hash);
        self
    }
    pub fn last_block_number(mut self, last_block_number: usize) -> Self {
        self.last_block_number = Some(last_block_number);
        self
    }

    pub fn data_hash(mut self, data_hash: Vec<u8>) -> Self {
        self.data_hash = Some(data_hash);
        self
    }
    pub fn proposer_address(mut self, proposer_address: Address) -> Self {
        self.proposer_address = Some(proposer_address);
        self
    }

    pub fn build(&self) -> Header {
        Header::new(
            self.block_number.unwrap(),
            self.timestamp.unwrap(),
            self.last_block_number.unwrap(),
            self.data_hash.clone().unwrap_or_default(),
            self.proposer_address.unwrap(),
            self.da_commitment.unwrap(),
            self.parent_finality_hash.clone().unwrap(),
            self.parent_hash.clone().unwrap(),
        )
    }
}
