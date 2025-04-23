use crate::{blob::Blob, rpc::RpcTransaction};
use bincode::{Decode, Encode};
use malachitebft_test::{PrivateKey, PublicKey, Signature};
use rand::{thread_rng, Rng};
use sha3::Digest;
use std::cmp::Ordering;

pub mod pool;
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct Transaction {
    #[bincode(with_serde)]
    signature: Signature,
    #[bincode(with_serde)]
    from: PublicKey,
    #[bincode(with_serde)]
    to: PublicKey,
    value: u64,
    data: [Blob; 4],
    nonce: u64,
    gas_price: u64,
    hash: [u8; 32],
}
impl From<RpcTransaction> for Transaction {
    fn from(rpc_tx: RpcTransaction) -> Self {
        Self::new(
            rpc_tx.from,
            rpc_tx.to,
            rpc_tx.signature,
            rpc_tx.value,
            rpc_tx.data,
            rpc_tx.nonce,
            rpc_tx.gas_price,
        )
    }
}
impl Transaction {
    pub fn new(
        from: PublicKey,
        to: PublicKey,
        signature: Signature,
        value: u64,
        data: [Blob; 4],
        nonce: u64,
        gas_price: u64,
    ) -> Self {
        let mut tx = Self {
            signature,
            from,
            to,
            value,
            data,
            nonce,
            gas_price,
            hash: Default::default(),
        };
        let tx_bytes = tx.to_bytes();
        let hash: [u8; 32] = sha3::Keccak256::digest(&tx_bytes).into();
        tx.hash = hash;
        tx
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend_from_slice(self.from.as_bytes());
        bytes.extend_from_slice(self.to.as_bytes());
        bytes.extend_from_slice(&self.value.to_be_bytes());
        for blob in &self.data {
            bytes.extend_from_slice(blob.data());
        }
        bytes.extend_from_slice(&self.nonce.to_be_bytes());
        bytes.extend_from_slice(&self.gas_price.to_be_bytes());
        bytes
    }
    pub fn validate(&self) -> bool {
        if self.data.len() > 4 {
            return false;
        }
        let tx_bytes = self.to_bytes();
        let hash: [u8; 32] = sha3::Keccak256::digest(&tx_bytes).into();
        self.hash == hash && self.from.verify(&hash, &self.signature).is_ok()
    }
    pub fn data(&self) -> &[Blob; 4] {
        &self.data
    }
    pub fn hash(&self) -> [u8; 32] {
        self.hash
    }
    pub fn from_(&self) -> PublicKey {
        self.from
    }
    pub fn to(&self) -> PublicKey {
        self.to
    }
    pub fn value(&self) -> u64 {
        self.value
    }
    pub fn nonce(&self) -> u64 {
        self.nonce
    }
    pub fn gas_price(&self) -> u64 {
        self.gas_price
    }
    pub fn signature(&self) -> Signature {
        self.signature
    }

    pub fn random() -> Self {
        let mut rng = thread_rng();
        let private_key = PrivateKey::generate(&mut rng);
        let public_key = private_key.public_key();
        let data = [
            Blob::random(),
            Blob::random(),
            Blob::random(),
            Blob::random(),
        ];
        let signature = private_key.sign(&[]);
        let value = rng.gen_range(0..1000000000000000000);
        let mut tx = Self {
            signature,
            from: public_key,
            to: public_key,
            value,
            data,
            nonce: rng.gen_range(0..1000000000000000000),
            gas_price: rng.gen_range(0..1000000000000000000),
            hash: Default::default(),
        };
        let tx_bytes = tx.to_bytes();
        let hash: [u8; 32] = sha3::Keccak256::digest(&tx_bytes).into();
        tx.hash = hash;
        let signature = private_key.sign(&hash);
        tx.signature = signature;
        tx
    }
}
impl Ord for Transaction {
    fn cmp(&self, other: &Self) -> Ordering {
        self.gas_price.cmp(&other.gas_price)
    }
}
impl PartialOrd for Transaction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_random() {
        let tx = Transaction::random();
        println!("tx: {:?}", tx);
        assert!(tx.validate());
    }
}
