use bincode::{Decode, Encode};
use bytes::Bytes;
use rand::{thread_rng, RngCore};

pub const BLOB_SIZE: usize = 245760;
#[derive(Debug, Encode, Decode)]
pub struct Blob {
    /// Data of the blob
    #[bincode(with_serde)]
    data: Bytes,
}
impl Default for Blob {
    fn default() -> Self {
        Self {
            data: Bytes::from_static(&[0; BLOB_SIZE]),
        }
    }
}

impl Blob {
    pub fn new(data: [u8; BLOB_SIZE]) -> Self {
        Self {
            data: Bytes::from_iter(data.iter().copied()),
        }
    }
    pub fn data(&self) -> &[u8] {
        self.data.as_ref()
    }
    pub fn random() -> Self {
        let mut rng = thread_rng();

        let mut blob = [0; BLOB_SIZE];
        rng.fill_bytes(&mut blob);

        Self::new(blob)
    }
}
