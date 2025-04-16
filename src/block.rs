use bincode::{Decode, Encode};
use eyre::Ok;
use rand::{thread_rng, Rng};
use rs_merkle::{algorithms::Sha256, Hasher, MerkleTree};

use crate::malachite_types::{address::Address, signing::PrivateKey};
use crate::{blob::Blob, error::BlockError, header::Header};

#[derive(Debug, Encode, Decode, Default)]
pub struct Block {
    /// Block Header.
    pub header: Header,
    /// list of blobs in this block.
    pub blobs: Vec<Blob>,
}

impl Block {
    /// Create a new block
    pub fn new(header: Header, blobs: Vec<Blob>) -> Self {
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
        // Set the `parent_finality_hash` if not present
        if self.header.parent_finality_hash.is_empty() {
            self.header.parent_finality_hash = vec![self.header.block_number as u8; 32];
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
