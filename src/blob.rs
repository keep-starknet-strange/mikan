use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Encode, Decode, Default)]
pub struct Blob {
    pub data: Vec<u8>,
}

impl Blob {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }
}
