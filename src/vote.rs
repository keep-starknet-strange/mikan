use bincode::{Decode, Encode};
use malachitebft_test::Address;
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct Vote {
    #[bincode(with_serde)]
    pub validator: Address,
    pub signature: Vec<u8>,
    pub block: usize,
}
#[allow(dead_code)]
impl Vote {
    pub fn new(validator: Address, sig: Vec<u8>, block: usize) -> Self {
        Self {
            validator,
            signature: sig,
            block,
        }
    }
}
