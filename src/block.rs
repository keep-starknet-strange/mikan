use bincode::{Decode, Encode};
use eyre::Ok;
use rand::{thread_rng, RngCore};
use rs_merkle::{algorithms::Sha256, Hasher, MerkleTree};

use crate::malachite_types::{address::Address, signing::PrivateKey};
use crate::{blob::Blob, error::BlockError, header::Header};

#[derive(Debug, Encode, Decode, Default)]
pub struct Block {
    /// Block Header.
    pub header: Header,
    /// list of blobs in this block.
    pub blobs: [Blob; 4],
}

impl Block {
    /// Create a new block
    pub fn new(header: Header, blobs: [Blob; 4]) -> Self {
        Block { header, blobs }
    }

    pub fn basic_validation(&mut self) -> eyre::Result<()> {
        // Populate the fields in the Header
        self.populate()?;

        println!("Block validation success!");

        // TODO: also validate header
        Ok(())
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
            .map(|blob| Sha256::hash(&blob.data))
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
    println!("{:?}", sk.public_key());
    Address::from_public_key(&sk.public_key())
}
