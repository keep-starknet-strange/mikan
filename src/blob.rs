use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Encode, Decode, Default)]
pub struct Blob {
    pub app_id: Vec<u8>,
    pub data: Vec<u8>,
}

impl Blob {
    pub fn new(data: Vec<u8>, app_id: Vec<u8>) -> Self {
        Self { data, app_id }
    }
}
