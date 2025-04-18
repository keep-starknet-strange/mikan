use bytes::Bytes;
use malachitebft_app_channel::app::types::codec::Codec;
use malachitebft_test::codec::proto::ProtobufCodec;
use redb::{TypeName, Value};

use crate::block::Block;

impl Value for Block {
    type SelfType<'a> = Block;
    type AsBytes<'a> = Bytes;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'b,
    {
        ProtobufCodec.encode(value).expect("unable to encode data")
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        ProtobufCodec
            .decode(Bytes::copy_from_slice(data))
            .expect("unable to decode data")
    }

    fn type_name() -> redb::TypeName {
        TypeName::new("Block")
    }
}
