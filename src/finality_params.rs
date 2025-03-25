use crate::{error::BlockError, vote::Vote};
use eyre::eyre;
use sha3::{Digest, Sha3_256};

#[derive(Debug)]
pub struct FinalityParams {
    pub height: usize,
    /// list of validators that voted on this block.
    pub votes: Vec<Vote>,
}

impl FinalityParams {
    pub fn new(height: usize, votes: Vec<Vote>) -> Self {
        Self {
            height: height,
            votes: votes,
        }
    }

    // TODO: this should be a merkle root calculation of the `votes`
    pub fn hash(&self) -> Vec<u8> {
        let mut hasher = Sha3_256::new();
        for vote in &self.votes {
            hasher.update(&vote.signature);
        }
        let result = hasher.finalize().as_slice().to_owned();
        result
    }

    pub fn basic_validation(&self) -> eyre::Result<()> {
        if self.height == 0 {
            return Err(BlockError::InvalidBlockNumber(self.height).into());
        }
        for vote in &self.votes {
            // TODO:Validate each signature belongs to the respective validator address
            todo!()
        }

        Ok(())
    }
}
