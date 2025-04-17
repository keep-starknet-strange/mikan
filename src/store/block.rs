
use redb::Value;

use crate::block::Block;

impl Value for Block {
    fn fixed_width() -> Option<usize> {
        None
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
        where
            Self: 'b {
        
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
        where
            Self: 'a {
        
    }

    fn type_name() -> redb::TypeName {
        
    }
}