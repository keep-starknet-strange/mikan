use bincode::{Decode, Encode};
use rand::{thread_rng, RngCore};

const BLOB_SIZE: usize = 245760;
#[derive(Debug, Encode, Decode)]
pub struct Blob {
    /// Data of the blob
    pub data: [u8; BLOB_SIZE],
}
impl Default for Blob {
    fn default() -> Self {
        Self {
            data: [0; BLOB_SIZE],
        }
    }
}

impl Blob {
    pub fn new(data: [u8; BLOB_SIZE]) -> Self {
        Self { data }
    }
    pub fn random() -> Self {
        let mut rng = thread_rng();

        let mut blob = Self::default();
        rng.fill_bytes(&mut blob.data);

        blob
    }
}
