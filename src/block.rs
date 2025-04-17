use bincode::config::standard;
use bincode::{Decode, Encode};
use bytes::Bytes;
use chrono::Utc;
use eyre::Ok;
use frieda::api::commit;
use rand::{thread_rng, RngCore};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rs_merkle::{algorithms::Sha256, Hasher, MerkleTree};
use tracing::{error, info};

use crate::blob::BLOB_SIZE;
use crate::malachite_types::{address::Address, signing::PrivateKey};
use crate::{blob::Blob, error::BlockError, header::Header};

#[derive(Debug, Encode, Decode, Default)]
pub struct Block {
    /// Block Header.
    header: Header,
    /// list of blobs in this block.
    blobs: [Blob; 4],
}

impl Block {
    /// Create a new block
    pub fn new(
        block_number: u64,
        timestamp: u64,
        parent_hash: [u8; 32],
        proposer_address: Address,
        blobs: [Blob; 4],
    ) -> Self {
        let leaves: Vec<[u8; 32]> = blobs.iter().map(|blob| Sha256::hash(blob.data())).collect();

        let merkle_tree = MerkleTree::<Sha256>::from_leaves(&leaves);

        let data_hash = merkle_tree.root().unwrap();
        let da_commitment = blobs
            .par_iter()
            .map(|blob| commit(blob.data(), 4))
            .collect::<Vec<[u8; 32]>>()
            .try_into()
            .unwrap();
        let header = Header::new(
            block_number,
            timestamp,
            data_hash,
            proposer_address,
            da_commitment,
            parent_hash,
        );
        Self { header, blobs }
    }
    pub fn parent_hash(&self) -> [u8; 32] {
        self.header.parent_hash()
    }

    pub fn hash(&self) -> [u8; 32] {
        self.header.block_hash()
    }

    pub fn genesis() -> Self {
        Self::new(
            0,
            69420,
            [0; 32],
            Address::default(),
            [
                Blob::new([0; BLOB_SIZE]),
                Blob::new([0; BLOB_SIZE]),
                Blob::new([0; BLOB_SIZE]),
                Blob::new([0; BLOB_SIZE]),
            ],
        )
    }
    pub fn to_bytes(&self) -> eyre::Result<Bytes> {
        let bytes = bincode::encode_to_vec(self, standard())?;
        Ok(Bytes::from(bytes))
    }

    pub fn is_valid(&self, height: u64, prev_block: &Block) -> eyre::Result<bool> {
        info!("Validating block at height {}", height);
        let expected = prev_block.hash();
        let actual = self.parent_hash();
        if expected != actual {
            error!("Parent hash: expected {:?}, got {:?}", expected, actual);
            return Ok(false);
        }
        let expected = height;
        let actual = self.header.block_number;
        if expected != actual {
            error!("Block number: expected {}, got {}", expected, actual);
            return Ok(false);
        }
        if self.header.timestamp < prev_block.header.timestamp {
            error!(
                "Timestamp in the past: prev timestamp {}, current timestamp {}",
                prev_block.header.timestamp, self.header.timestamp
            );
            return Ok(false);
        }

        if self.header.timestamp < Utc::now().timestamp() as u64 - 600
            || self.header.timestamp > Utc::now().timestamp() as u64 + 600
        {
            error!(
                "Timestamp out of range: lower bound {}, upper bound {}, current timestamp {}",
                Utc::now().timestamp() as u64 - 600,
                Utc::now().timestamp() as u64 + 600,
                self.header.timestamp
            );
            return Ok(false);
        }
        let expected = self.blob_tree_root()?;
        let actual = self.header.data_hash;
        if expected != actual {
            error!(
                "Data hash mismatch: expected {:?}, got {:?}",
                expected, actual
            );
            return Ok(false);
        }
        let expected_commitments = self
            .blobs
            .par_iter()
            .map(|blob| commit(blob.data(), 4))
            .collect::<Vec<[u8; 32]>>();
        let actual_commitments = self.header.da_commitment;
        if expected_commitments != actual_commitments {
            error!(
                "DA commitment mismatch: expected {:?}, got {:?}",
                expected_commitments, actual_commitments
            );
            return Ok(false);
        }

        let expected = self.header.compute_block_hash();
        let actual = self.header.block_hash;
        if expected != actual {
            error!(
                "Block hash mismatch: expected {:?}, got {:?}",
                expected, actual
            );
            return Ok(false);
        }

        Ok(true)
    }

    /// populate the empty fields in `Header`
    pub fn populate(&mut self) -> eyre::Result<()> {
        // Set the `data_hash` if not present
        let blob_tree_root = self.blob_tree_root()?;
        if self.header.data_hash.is_empty() {
            self.header.data_hash = blob_tree_root;
        } else if self.header.data_hash != blob_tree_root {
            return Err(BlockError::DataHashMismatch(blob_tree_root, self.header.data_hash).into());
        }

        println!("Header population success!");

        Ok(())
    }

    /// Merklize the raw blob data
    pub fn blob_tree_root(&self) -> eyre::Result<[u8; 32]> {
        let leaves: Vec<[u8; 32]> = self
            .blobs
            .iter()
            .map(|blob| Sha256::hash(blob.data()))
            .collect();

        let merkle_tree = MerkleTree::<Sha256>::from_leaves(&leaves);

        merkle_tree.root().ok_or(BlockError::MerkleTreeError.into())
    }
}

pub fn mock_make_validator() -> Address {
    let mut rng = thread_rng();
    let mut bytes = [0u8; 32];
    rng.fill_bytes(&mut bytes);
    let sk = PrivateKey::from(bytes);
    Address::from_public_key(&sk.public_key())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_is_valid() {
        let prev_block = Block::default();
        let block = Block::new(
            1,
            Utc::now().timestamp() as u64,
            prev_block.hash(),
            mock_make_validator(),
            [
                Blob::random(),
                Blob::random(),
                Blob::random(),
                Blob::random(),
            ],
        );
        assert!(block.is_valid(1, &prev_block).unwrap());
    }
}
