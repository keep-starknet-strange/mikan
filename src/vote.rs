use malachitebft_test::Address;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Vote {
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
