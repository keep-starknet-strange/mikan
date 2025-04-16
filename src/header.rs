use crate::malachite_types::address::Address;
use bincode::{Decode, Encode};
use frieda::{api::verify, proof::Proof};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};

use crate::{block::mock_make_validator, error::BlockError};

#[allow(clippy::too_many_arguments, dead_code)]
#[derive(Debug, Serialize, Deserialize, Encode, Decode)]
pub struct Header {
    pub block_number: u64,
    pub timestamp: u64,
    /// Hash of current block
    pub block_hash: [u8; 32],
    /// DA commitment for this block.
    pub da_commitment: Option<[[u8; 32]; 4]>,
    /// block of parent block.
    pub parent_hash: [u8; 32],
    /// Merkle root of the data in the current block.
    /// Leaves of this tree will be the raw bytes of each blob
    pub data_hash: [u8; 32],
    /// address of proposer of this block.
    #[bincode(with_serde)]
    pub proposer_address: Address,
}
impl Default for Header {
    fn default() -> Self {
        Self {
            block_number: 0,
            timestamp: 0,
            block_hash: [0; 32],
            da_commitment: None,
            parent_hash: [0; 32],
            data_hash: [0; 32],
            proposer_address: mock_make_validator(),
        }
    }
}

impl Header {
    #[allow(clippy::too_many_arguments)]
    /// Creates a new block header and computes its hash.
    pub fn new(
        block_number: u64,
        timestamp: u64,
        data_hash: [u8; 32],
        proposer_address: Address,
        da_commitment: Option<[[u8; 32]; 4]>,
        parent_hash: [u8; 32],
    ) -> Self {
        let mut header = Header {
            block_number,
            timestamp,
            da_commitment,
            data_hash,
            proposer_address,
            parent_hash,
            block_hash: [0; 32],
        };
        header.block_hash = header.compute_block_hash();
        header
    }

    pub fn basic_validation(&self) -> Result<(), BlockError> {
        if self.block_number == 0 {
            return Err(BlockError::InvalidBlockNumber(self.block_number));
        }

        Ok(())
    }

    ///Compute block hash
    pub fn compute_block_hash(&self) -> [u8; 32] {
        let mut hasher = Sha3_256::new();

        hasher.update(self.block_number.to_le_bytes());
        hasher.update(self.parent_hash);
        hasher.update(self.data_hash);
        hasher.update(self.proposer_address.into_inner());

        hasher.finalize().into()
    }

    /// Verify the commitment against a proof
    pub fn verify_data(&self, proof: Proof) -> bool {
        verify(proof, None)
    }
}

#[derive(Debug, Default)]
pub struct HeaderBuilder {
    pub block_number: Option<u64>,
    pub timestamp: Option<u64>,
    /// Hash of current block
    pub block_hash: Option<[u8; 32]>,
    /// DA commitment for this block.
    pub da_commitment: Option<[[u8; 32]; 4]>,
    /// block of parent block.
    pub parent_hash: Option<[u8; 32]>,
    /// Merkle root of the data in the current block.
    /// Leaves of this tree will be the raw bytes of each blob
    pub data_hash: Option<[u8; 32]>,
    /// address of proposer of this block.
    pub proposer_address: Option<Address>,
}

impl HeaderBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn block_number(mut self, block: u64) -> Self {
        self.block_number = Some(block);
        self
    }

    pub fn timestamp(mut self, timestamp: u64) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    pub fn block_hash(mut self, block_hash: [u8; 32]) -> Self {
        self.block_hash = Some(block_hash);
        self
    }
    pub fn da_commitment(mut self, da_commitment: [[u8; 32]; 4]) -> Self {
        self.da_commitment = Some(da_commitment);
        self
    }
    pub fn parent_hash(mut self, parent_hash: [u8; 32]) -> Self {
        self.parent_hash = Some(parent_hash);
        self
    }

    pub fn data_hash(mut self, data_hash: [u8; 32]) -> Self {
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
            self.data_hash.unwrap_or_default(),
            self.proposer_address.unwrap(),
            self.da_commitment,
            self.parent_hash.unwrap(),
        )
    }
}
