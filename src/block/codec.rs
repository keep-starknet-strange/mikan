use bytes::Bytes;
use malachitebft_app_channel::app::types::codec::Codec;
use malachitebft_proto::{Error as ProtoError, Protobuf};
use malachitebft_test::codec::proto::ProtobufCodec;
use prost::Message;

use super::blockproto;
use crate::{blob::Blob, block::Block, finality_params::FinalityParams, header::Header};

impl Codec<Block> for ProtobufCodec {
    type Error = ProtoError;

    fn decode(&self, bytes: bytes::Bytes) -> Result<Block, Self::Error> {
        let proto = blockproto::Block::decode(bytes.as_ref())?;

        Ok(Block {
            header: Header::from_proto(proto.header.unwrap())?,
            blobs: proto
                .blobs
                .iter()
                .map(|blob| Blob::from_proto(*blob).unwrap())
                .collect(),
            last_block_params: FinalityParams {},
        })
    }

    fn encode(&self, msg: &Block) -> Result<bytes::Bytes, Self::Error> {
        let proto = blockproto::Block {
            header: Some(msg.header.to_proto()?),
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
