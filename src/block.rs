use bincode::{Decode, Encode};
use eyre::Ok;
use malachitebft_test::{Address, PrivateKey};
use rand::{thread_rng, Rng};
use rs_merkle::{algorithms::Sha256, Hasher, MerkleTree};
use serde::{Deserialize, Serialize};

use crate::{blob::Blob, error::BlockError, finality_params::FinalityParams, header::Header};

#[derive(Debug, Encode, Decode)]
pub struct Block {
    /// Block Header.
    pub header: Header,
    /// list of blobs in this block.
    pub blobs: Vec<Blob>,
    /// Finality params of this block.
    /// Holds the list of validators that voted on this block.
    pub last_block_params: FinalityParams,
}

impl Block {
    /// Create a new block
    pub fn new(header: Header, blobs: Vec<Blob>, last_block_params: FinalityParams) -> Self {
        Block {
            header,
            blobs,
            last_block_params,
        }
    }

    pub fn basic_validation(&mut self) -> eyre::Result<()> {
        // Populate the fields in the Header
        self.populate()?;

        if self.last_block_params.height >= self.header.block_number {
            return Err(BlockError::InvalidBlockNumber(self.header.block_number))?;
        }

        assert_eq!(
            self.last_block_params.tree_root().unwrap(),
            self.header.parent_finality_hash,
            "{:?}",
            BlockError::FinalityHashMismatch(
                self.last_block_params.tree_root()?,
                self.header.parent_finality_hash.clone()
            )
        );
        println!("Block validation success!");

        // TODO: also validate header
        Ok(())
    }

    /// populate the empty fields in `Header`
    pub fn populate(&mut self) -> eyre::Result<()> {
        // Set the `parent_finality_hash` if not present
        if self.header.parent_finality_hash.is_empty() {
            self.header.parent_finality_hash = self.last_block_params.tree_root()?;
        }

        // Set the `data_hash` if not present
        let blob_tree_root = self.blob_tree_root()?;
        if self.header.data_hash.is_empty() {
            self.header.data_hash = blob_tree_root;
        } else if self.header.data_hash != blob_tree_root {
            return Err(BlockError::DataHashMismatch(
                blob_tree_root,
                self.header.data_hash.clone(),
            )
            .into());
        }

        println!("Header population success!");

        Ok(())
    }

    /// Merklize the raw blob data
    pub fn blob_tree_root(&self) -> eyre::Result<Vec<u8>> {
        let leaves: Vec<[u8; 32]> = self
            .blobs
            .iter()
            .map(|blob| Sha256::hash(&blob.data))
            .collect();

        let merkle_tree = MerkleTree::<Sha256>::from_leaves(&leaves);

        Ok(merkle_tree
            .root()
            .map(Vec::from)
            .ok_or(BlockError::MerkleTreeError)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::height::Height;
    use crate::{header::HeaderBuilder, vote::Vote};
    use malachitebft_core_types::{NilOrVal, Round, VoteType};

    #[test]
    fn mock_block_create() -> eyre::Result<()> {
        let proposer = mock_make_validator();
        let vote_1 = Vote::new(
            mock_make_validator(),
            Vec::from("1234"),
            2,
            Height::new(2),
            Round::new(0),
            VoteType::Prevote,
            proposer,
            NilOrVal::Nil,
            None,
        );
        let vote_2 = Vote::new(
            mock_make_validator(),
            Vec::from("1234"),
            2,
            Height::new(2),
            Round::new(0),
            VoteType::Prevote,
            proposer,
            NilOrVal::Nil,
            None,
        );
        let vote_3 = Vote::new(
            mock_make_validator(),
            Vec::from("1234"),
            2,
            Height::new(2),
            Round::new(0),
            VoteType::Prevote,
            proposer,
            NilOrVal::Nil,
            None,
        );

        let parent_finality_hash_block_2 = FinalityParams::new(2, vec![vote_1, vote_2, vote_3]);
        let blobs = vec![mock_make_blobs(), mock_make_blobs()];
        let header = HeaderBuilder::new()
            .parent_finality_hash(parent_finality_hash_block_2.tree_root().unwrap())
            .block_number(3)
            .timestamp(1978746)
            .proposer_address(proposer)
            .da_commitment(None)
            .parent_hash(vec![1, 2, 3, 4])
            .last_block_number(2)
            .build();

        let mut block = Block::new(header, blobs, parent_finality_hash_block_2);
        block.basic_validation()?;

        // println!("{:?}", block);
        Ok(())
    }
}

pub fn mock_make_validator() -> Address {
    let mut rng = thread_rng();
    let sk = PrivateKey::generate(&mut rng);
    println!("{:?}", sk.public_key());
    Address::from_public_key(&sk.public_key())
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
