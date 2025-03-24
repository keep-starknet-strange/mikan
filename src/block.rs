use crate::{blob::Blob, error::BlockError, finality_params::FinalityParams, header::Header};

#[allow(dead_code)]
#[derive(Debug)]
struct Block {
    /// Block Header.
    pub header: Header,
    /// list of blobs in this block.
    pub blobs: Vec<Blob>,
    /// Finality params of this block.
    /// Holds the list of validators that voted on this block.
    pub last_block_params: FinalityParams,
}

#[allow(dead_code)]
impl Block {
    /// Create a new block
    pub fn new(header: Header, blobs: Vec<Blob>, last_block_params: FinalityParams) -> Self {
        Block {
            header,
            blobs,
            last_block_params,
        }
    }

    pub fn basic_validation(&self) -> Result<(), BlockError> {
        if self.header.parent_hash.is_empty() {
            return Err(BlockError::NullParentHash);
        }

        if self.last_block_params.height >= self.header.block_number {
            return Err(BlockError::InvalidBlockNumber(self.header.block_number));
        }

        if self.last_block_params.hash() == self.header.parent_finality_hash {
            return Err(BlockError::FinalityHashMismatch(
                self.last_block_params.hash(),
                self.header.parent_finality_hash.clone(),
            ));
        }

        // TODO: also validate header
        Ok(())
    }

    pub fn populate(&mut self) {
        if self.header.parent_finality_hash.is_empty() {
            self.header.parent_finality_hash = self.last_block_params.hash();
        }

        if self.header.data_hash.is_empty() {
            //TODO: add hash of data
        }
    }
}
