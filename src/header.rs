use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Header {
    pub block_number: usize,
    pub timestamp: usize,
    pub block_hash: Vec<u8>,
    pub parent_hash: Vec<u8>,
}

#[allow(dead_code)]
impl Header {
    /// Creates a new block header and computes its hash.
    pub fn new(block_number: usize, timestamp: usize, parent_hash: Vec<u8>) -> Self {
        let mut header = Header {
            block_number,
            timestamp,
            parent_hash: parent_hash.clone(),
            block_hash: Vec::new(),
        };
        header.block_hash = header.compute_block_hash();
        header
    }

    /// Compute block hash
    pub fn compute_block_hash(&self) -> Vec<u8> {
        todo!()
    }
}
