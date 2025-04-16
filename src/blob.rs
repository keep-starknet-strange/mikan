use bincode::{Decode, Encode};
use rand::{thread_rng, RngCore};

#[derive(Debug, Encode, Decode)]
pub struct Blob {
    /// Data of the blob
    pub data: [u8; 131072 / 4],
}
impl Default for Blob {
    fn default() -> Self {
        Self {
            data: [0; 131072 / 4],
        }
    }
}

impl Blob {
    pub fn new(data: [u8; 131072 / 4]) -> Self {
        Self { data }
    }
    pub fn random() -> Self {
        let mut rng = thread_rng();

        let mut blob = Self::default();
        rng.fill_bytes(&mut blob.data);

        blob
    }
}
