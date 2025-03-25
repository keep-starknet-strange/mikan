use malachitebft_test::Address;

#[derive(Debug, Clone)]
pub struct Vote {
    pub validator: Address,
    pub signature: Vec<u8>,
    pub block: usize,
}

impl Vote {
    pub fn new(validator: Address, sig: Vec<u8>, block: usize) -> Self {
        Self {
            validator,
            signature: sig,
            block,
        }
    }
}
