use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Blob {
    pub app_id: Vec<u8>,
    pub data: Vec<u8>,
}

#[allow(dead_code)]
impl Blob {
    pub fn new(data: Vec<u8>, app_id: Vec<u8>) -> Self {
        Self { data, app_id }
    }
}
