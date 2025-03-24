use malachitebft_test::Address;

#[derive(Debug,Clone)]
pub struct Vote {
    pub validator: Address,
    pub signature:Vec<u8>,
    pub timestamp: usize,
    pub block: usize,
}
