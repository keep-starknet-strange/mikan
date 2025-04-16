use bytes::Bytes;
use core::fmt;
use malachitebft_proto::{Error as ProtoError, Protobuf};
use serde::{Deserialize, Serialize};

use super::proto;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Copy, Serialize, Deserialize)]
pub struct ValueId(u64);

impl ValueId {
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

impl From<u64> for ValueId {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for ValueId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

impl Protobuf for ValueId {
    type Proto = proto::ValueId;

    fn from_proto(proto: Self::Proto) -> Result<Self, ProtoError> {
        Ok(ValueId::new(proto.value))
    }

    fn to_proto(&self) -> Result<Self::Proto, ProtoError> {
        Ok(proto::ValueId { value: self.0 })
    }
}

/// The value to decide on
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Value {
    pub value: u64,
    pub extensions: Bytes,
}

impl Value {
    /// Creates a new Value by hashing the provided bytes using SipHash
    pub fn new(data: Bytes) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new(); // Uses SipHash
        data.hash(&mut hasher);

        Self {
            value: hasher.finish(), // Get the u64 hash
            extensions: data,       // Store original bytes as extensions
        }
    }

    pub fn id(&self) -> ValueId {
        ValueId(self.value)
    }

    pub fn size_bytes(&self) -> usize {
        std::mem::size_of_val(&self.value) + self.extensions.len()
    }
}

impl malachitebft_core_types::Value for Value {
    type Id = ValueId;

    fn id(&self) -> ValueId {
        self.id()
    }
}

impl Protobuf for Value {
    type Proto = proto::Value;

    fn from_proto(proto: Self::Proto) -> Result<Self, ProtoError> {
        let value = proto.value;
        let extensions = proto.extensions;

        Ok(Value { value, extensions })
    }

    fn to_proto(&self) -> Result<Self::Proto, ProtoError> {
        Ok(proto::Value {
            value: self.value,
            extensions: self.extensions.clone(),
        })
    }
}
