use malachitebft_proto::Protobuf;
use malachitebft_test::{proto, Address};
use prost::Name;

use crate::block::blockproto;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Vote {
    pub validator: Address,
    pub signature: Vec<u8>,
    pub block: usize,
}
#[allow(dead_code)]
impl Vote {
    pub fn new(validator: Address, sig: Vec<u8>, block: usize) -> Self {
        Self {
            validator,
            signature: sig,
            block,
        }
    }
}

impl Name for blockproto::Vote {
    const NAME: &'static str = "Vote";
    const PACKAGE: &'static str = "mikan";

    fn full_name() -> String {
        "mikan.Vote".into()
    }

    fn type_url() -> String {
        "/mikan.Vote".into()
    }
}

impl Protobuf for Vote {
    type Proto = blockproto::Vote;

    fn from_proto(proto: Self::Proto) -> Result<Self, malachitebft_proto::Error> {
        let vote = Vote {
            validator: Address::from_proto(proto::Address {
                value: proto.validator.into(),
            })?,
            signature: proto.signature,
            block: proto
                .block
                .try_into()
                .expect("u64 does not fit in usize for Vote.block"),
        };

        Ok(vote)
    }

    fn to_proto(&self) -> Result<Self::Proto, malachitebft_proto::Error> {
        let proto = blockproto::Vote {
            validator: self.validator.into_inner().to_vec(),
            signature: self.signature.clone(),
            block: self
                .block
                .try_into()
                .expect("usize does not fit into u64 for Vote.block"),
        };

        Ok(proto)
    }
}
