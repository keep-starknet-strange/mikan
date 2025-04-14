use bytes::Bytes;
use core::fmt;
use serde::{Deserialize, Serialize};

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
