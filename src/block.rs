use eyre::Ok;
use malachitebft_test::{Address, PrivateKey};
use rand::{thread_rng, Rng};
use sha3::{Digest, Sha3_256};

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

    pub fn basic_validation(&self) -> eyre::Result<()> {
        if self.last_block_params.height >= self.header.block_number {
            return Err(BlockError::InvalidBlockNumber(self.header.block_number))?;
        }

        if self.last_block_params.hash()? == self.header.parent_finality_hash {
            return Err(BlockError::FinalityHashMismatch(
                self.last_block_params.hash()?,
                self.header.parent_finality_hash.clone(),
            )
            .into());
        }

        // TODO: also validate header
        Ok(())
    }

    /// populate the empty fields in `Header`
    pub fn populate(&mut self) -> eyre::Result<()> {
        if self.header.parent_finality_hash.is_empty() {
            self.header.parent_finality_hash = self.last_block_params.hash()?;
        }

        if self.header.data_hash.is_empty() {
            self.header.data_hash = self.hash_data();
        } else if self.header.data_hash != self.hash_data() {
            return Err(BlockError::DataHashMismatch(
                self.hash_data(),
                self.header.data_hash.clone(),
            )
            .into());
        }

        Ok(())
    }

    /// Merklize the raw blob data
    pub fn hash_data(&self) -> Vec<u8> {
        let mut hasher = Sha3_256::new();
        for blob in &self.blobs {
            hasher.update(&blob.data);
        }
        let result = hasher.finalize().as_slice().to_owned();
        result
    }
}

#[cfg(test)]
mod tests {

    use crate::vote::Vote;

    use super::*;


    #[test]
    fn mock_block_create() {
        let vote_1 = Vote::new(mock_make_validator(), Vec::from("1234"), 2);
        let vote_2 = Vote::new(mock_make_validator(), Vec::from("1234"), 2);
        let vote_3 = Vote::new(mock_make_validator(), Vec::from("1234"), 2);

        let parent_finality_hash_block_2 = FinalityParams::new(2, vec![vote_1, vote_2, vote_3]);
        let blobs = vec![mock_make_blobs(), mock_make_blobs()];

        let block = Block::new(Header::default(), blobs, parent_finality_hash_block_2);

        println!("{:?}", block);
    }
}


pub fn mock_make_validator() -> Address {
    let mut rng = thread_rng(); 
    let sk = PrivateKey::generate(&mut rng);
    println!("{:?}",sk.public_key());
    return Address::from_public_key(&sk.public_key());
}


pub fn mock_make_blobs() -> Blob {
    let mut rng = thread_rng();

    let random_blob_data: Vec<u8> = (0..16).map(|_| rng.gen()).collect();
    let random_app_id: Vec<u8> = (0..16).map(|_| rng.gen()).collect();

    Blob {
        app_id: random_app_id,
        data: random_blob_data,
    }
}