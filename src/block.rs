use bytes::Bytes;
use malachitebft_app_channel::app::types::codec::Codec;
use malachitebft_proto::Error as ProtoError;
use malachitebft_test::{codec::proto::ProtobufCodec, Address, PrivateKey};
use prost::Message;
use rand::{thread_rng, Rng};
use rs_merkle::{algorithms::Sha256, Hasher, MerkleTree};

use crate::{blob::Blob, error::BlockError, finality_params::FinalityParams, header::Header};
pub mod blockproto;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Block {
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

impl Codec<Block> for ProtobufCodec {
    type Error = ProtoError;

    fn decode(&self, bytes: bytes::Bytes) -> Result<Block, Self::Error> {
        // let proto =
    }

    fn encode(&self, msg: &Block) -> Result<bytes::Bytes, Self::Error> {
        let proto = blockproto::Block {
            header: Some(blockproto::Header {
                block_number: msg
                    .header
                    .block_number
                    .try_into()
                    .expect("usize does not fit in u64 for block_number"),
                timestamp: msg
                    .header
                    .timestamp
                    .try_into()
                    .expect("usize does not fit in u64 for timestamp"),
                block_hash: msg.header.block_hash.clone(),
                da_commitment: match msg.header.da_commitment {
                    Some(commitment) => Some(commitment.to_vec()),
                    None => None,
                },
                parent_hash: msg.header.parent_hash.clone(),
                parent_finality_hash: msg.header.parent_finality_hash.clone(),
                last_block_number: msg
                    .header
                    .last_block_number
                    .try_into()
                    .expect("usize does not fit in u64 for last_block_number"),
                data_hash: msg.header.data_hash.clone(),
                proposer_address: msg.header.proposer_address.into_inner().to_vec(),
            }),
            blobs: msg
                .blobs
                .iter()
                .map(|blob| blockproto::Blob {
                    app_id: blob.app_id.clone(),
                    data: blob.data.clone(),
                })
                .collect(),
            last_block_params: Some(blockproto::FinalityParams {
                height: msg
                    .last_block_params
                    .height
                    .try_into()
                    .expect("usize does not fit in u64 for last_block_params.height"),
                votes: msg
                    .last_block_params
                    .votes
                    .iter()
                    .map(|vote| blockproto::Vote {
                        validator: vote.validator.into_inner().to_vec(),
                        signature: vote.signature.to_vec(),
                        block: vote
                            .block
                            .try_into()
                            .expect("usize does not fit in u64 for vote.block"),
                    })
                    .collect(),
            }),
        };

        Ok(Bytes::from(proto.encode_to_vec()))
    }
}

#[cfg(test)]
mod tests {

    use crate::{header::HeaderBuilder, vote::Vote};

    use super::*;

    #[test]
    fn mock_block_create() -> eyre::Result<()> {
        let proposer = mock_make_validator();
        let vote_1 = Vote::new(mock_make_validator(), Vec::from("1234"), 2);
        let vote_2 = Vote::new(mock_make_validator(), Vec::from("1234"), 2);
        let vote_3 = Vote::new(mock_make_validator(), Vec::from("1234"), 2);

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
#[allow(dead_code)]
pub fn mock_make_blobs() -> Blob {
    let mut rng = thread_rng();

    let random_blob_data: Vec<u8> = (0..16).map(|_| rng.gen()).collect();
    let random_app_id: Vec<u8> = (0..16).map(|_| rng.gen()).collect();

    Blob {
        app_id: random_app_id,
        data: random_blob_data,
    }
}
