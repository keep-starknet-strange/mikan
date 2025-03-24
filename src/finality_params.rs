use crate::vote::Vote;
use sha3::{Digest, Sha3_256};

#[derive(Debug)]
pub struct FinalityParams {
    pub height: usize,
    /// list of validators that voted on this block.
    pub votes: Vec<Vote>,
}

impl FinalityParams {
    pub fn hash(&self) -> Vec<u8> {
        let mut hasher = Sha3_256::new();
        for vote in &self.votes {
            hasher.update(&vote.signature);
        }
        let result = hasher.finalize().as_slice().to_owned();
        result
    }
}
