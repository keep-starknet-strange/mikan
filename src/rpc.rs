use async_trait::async_trait;
use frieda::proof::{FriConfig, PcsConfig, Proof};
use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::server::{ServerBuilder, ServerHandle};
use jsonrpsee::types::error::INTERNAL_ERROR_CODE;
use jsonrpsee::types::ErrorObject;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tracing::info;

use crate::blob::Blob;
use crate::store::Store;
use crate::transactions::{pool::TransactionPool, Transaction};
use frieda::api::generate_proof;
use malachitebft_test::{PublicKey, Signature};

#[derive(Debug)]
pub struct RpcTransaction {
    pub from: PublicKey,
    pub to: PublicKey,
    pub signature: Signature,
    pub value: u64,
    pub nonce: u64,
    pub gas_price: u64,
    pub data: [Blob; 4],
}

impl RpcTransaction {
    pub fn random() -> Self {
        Transaction::random().into()
    }
}

impl From<Transaction> for RpcTransaction {
    fn from(tx: Transaction) -> Self {
        Self {
            from: tx.from_(),
            to: tx.to(),
            signature: tx.signature(),
            value: tx.value(),
            nonce: tx.nonce(),
            gas_price: tx.gas_price(),
            data: tx.data().clone(),
        }
    }
}

impl Serialize for RpcTransaction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("RpcTransaction", 7)?;
        state.serialize_field("from", &self.from)?;
        state.serialize_field("to", &self.to)?;
        state.serialize_field("signature", &hex::encode(self.signature.to_bytes()))?;
        state.serialize_field("value", &self.value)?;
        state.serialize_field("nonce", &self.nonce)?;
        state.serialize_field("gas_price", &self.gas_price)?;

        // Serialize blobs as an array of hex strings
        let blob_hexes: Vec<String> = self
            .data
            .iter()
            .map(|blob| hex::encode(blob.data()))
            .collect();

        state.serialize_field("data", &blob_hexes)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for RpcTransaction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use bytes::Bytes;
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        struct RpcTransactionVisitor;

        impl<'de> Visitor<'de> for RpcTransactionVisitor {
            type Value = RpcTransaction;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct RpcTransaction")
            }

            fn visit_map<V>(self, mut map: V) -> Result<RpcTransaction, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut from = None;
                let mut to = None;
                let mut signature = None;
                let mut value = None;
                let mut nonce = None;
                let mut gas_price = None;
                let mut data = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "from" => {
                            if from.is_some() {
                                return Err(de::Error::duplicate_field("from"));
                            }
                            from = Some(map.next_value()?);
                        }
                        "to" => {
                            if to.is_some() {
                                return Err(de::Error::duplicate_field("to"));
                            }
                            to = Some(map.next_value()?);
                        }
                        "signature" => {
                            if signature.is_some() {
                                return Err(de::Error::duplicate_field("signature"));
                            }
                            // Deserialize signature from hex string
                            let sig_hex: String = map.next_value()?;
                            let sig_bytes = hex::decode(sig_hex).map_err(|e| {
                                de::Error::custom(format!("Invalid signature hex: {}", e))
                            })?;
                            let signature_obj = Signature::from_bytes(
                                sig_bytes.try_into().map_err(|e: Vec<u8>| {
                                    de::Error::custom(format!(
                                        "Invalid signature bytes len: {}",
                                        e.len()
                                    ))
                                })?,
                            );
                            signature = Some(signature_obj);
                        }
                        "value" => {
                            if value.is_some() {
                                return Err(de::Error::duplicate_field("value"));
                            }
                            value = Some(map.next_value()?);
                        }
                        "nonce" => {
                            if nonce.is_some() {
                                return Err(de::Error::duplicate_field("nonce"));
                            }
                            nonce = Some(map.next_value()?);
                        }
                        "gas_price" => {
                            if gas_price.is_some() {
                                return Err(de::Error::duplicate_field("gas_price"));
                            }
                            gas_price = Some(map.next_value()?);
                        }
                        "data" => {
                            if data.is_some() {
                                return Err(de::Error::duplicate_field("data"));
                            }

                            // Deserialize blob data from hex strings
                            let hex_strings: Vec<String> = map.next_value()?;
                            if hex_strings.len() != 4 {
                                return Err(de::Error::custom("Expected 4 blobs in data array"));
                            }

                            let mut blobs = [
                                Blob::default(),
                                Blob::default(),
                                Blob::default(),
                                Blob::default(),
                            ];
                            for (i, hex) in hex_strings.iter().enumerate() {
                                let bytes = hex::decode(hex).map_err(|e| {
                                    de::Error::custom(format!("Invalid hex in blob {}: {}", i, e))
                                })?;
                                blobs[i] = Blob::new(Bytes::from(bytes));
                            }

                            data = Some(blobs);
                        }
                        _ => {
                            // Skip unknown fields
                            let _ = map.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }

                let from = from.ok_or_else(|| de::Error::missing_field("from"))?;
                let to = to.ok_or_else(|| de::Error::missing_field("to"))?;
                let signature = signature.ok_or_else(|| de::Error::missing_field("signature"))?;
                let value = value.ok_or_else(|| de::Error::missing_field("value"))?;
                let nonce = nonce.ok_or_else(|| de::Error::missing_field("nonce"))?;
                let gas_price = gas_price.ok_or_else(|| de::Error::missing_field("gas_price"))?;
                let data = data.ok_or_else(|| de::Error::missing_field("data"))?;

                Ok(RpcTransaction {
                    from,
                    to,
                    signature,
                    value,
                    nonce,
                    gas_price,
                    data,
                })
            }
        }

        deserializer.deserialize_map(RpcTransactionVisitor)
    }
}

#[rpc(server, namespace = "mikan")]
pub trait MikanApi {
    #[method(name = "sendTransaction")]
    async fn send_transaction(&self, tx: RpcTransaction) -> RpcResult<String>;

    #[method(name = "sampleBlob")]
    async fn sample_blob(
        &self,
        block_height: u64,
        blob_index: usize,
        sampling_seed: Option<u64>,
    ) -> RpcResult<Proof>;

    #[method(name = "blockNumber")]
    async fn block_number(&self) -> u64;

    #[method(name = "getBlob")]
    async fn get_blob(&self, block_height: u64, blob_index: usize) -> RpcResult<Blob>;
}

#[derive(Clone)]
pub struct MikanRpcObj {
    transaction_pool: TransactionPool,
    store: Store,
}

impl MikanRpcObj {
    pub fn new(transaction_pool: TransactionPool, store: Store) -> Self {
        Self {
            transaction_pool,
            store,
        }
    }

    pub async fn start(self, port: u16) -> eyre::Result<(ServerHandle, Self)> {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
        let server = ServerBuilder::default().build(addr).await?;

        let handle = server.start(self.clone().into_rpc());
        info!("RPC server started on {}", addr);

        Ok((handle, self))
    }
    pub fn get_top_transaction(&self) -> Option<Transaction> {
        self.transaction_pool.get_top_transaction()
    }
    pub fn get_transactions(&self, count: usize) -> Vec<Transaction> {
        self.transaction_pool.get_transactions(count)
    }
}

#[async_trait]
impl MikanApiServer for MikanRpcObj {
    async fn send_transaction(&self, tx: RpcTransaction) -> RpcResult<String> {
        let tx = Transaction::from(tx);

        self.transaction_pool.add_transaction(tx.clone());
        info!("Transaction sent: {}", hex::encode(tx.hash()));
        Ok(hex::encode(tx.hash()))
    }

    async fn block_number(&self) -> u64 {
        // Get the latest block height from the store
        let height = self
            .store
            .max_decided_value_height()
            .await
            .unwrap_or_default();

        height.as_u64()
    }

    async fn sample_blob(
        &self,
        block_height: u64,
        blob_index: usize,
        sampling_seed: Option<u64>,
    ) -> RpcResult<Proof> {
        let height = crate::malachite_types::height::Height::new(block_height);

        // Get the block data
        let block_data = self.store.get_decided_block(height).await.map_err(|_| {
            ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                "Couldn't find block",
                Option::<String>::None,
            )
        })?;

        let block_data = block_data.ok_or(ErrorObject::owned(
            INTERNAL_ERROR_CODE,
            "Couldn't find block",
            Option::<String>::None,
        ))?;

        // Decode the block
        let (block, _): (crate::block::Block, _) =
            bincode::borrow_decode_from_slice(&block_data, bincode::config::standard()).map_err(
                |_| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        "Couldn't decode block",
                        Option::<String>::None,
                    )
                },
            )?;

        // Get all blobs from the block
        let blobs = block.blobs();

        // Check if the requested blob index is valid
        if blob_index >= blobs.len() {
            return Err(ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                "Blob index out of bounds",
                Option::<String>::None,
            ));
        };

        // Generate a FRIEDA proof for the blob
        let proof = generate_proof(
            blobs[blob_index].data(),
            sampling_seed,
            PcsConfig {
                pow_bits: 20,
                fri_config: FriConfig {
                    log_blowup_factor: 4,
                    log_last_layer_degree_bound: 0,
                    n_queries: 20,
                },
            },
        );

        // Return the proof as a hex string
        Ok(proof)
    }
    async fn get_blob(&self, block_height: u64, blob_index: usize) -> RpcResult<Blob> {
        let height = crate::malachite_types::height::Height::new(block_height);

        // Get the block data
        let block_data = self.store.get_decided_block(height).await.map_err(|_| {
            ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                "Couldn't find block",
                Option::<String>::None,
            )
        })?;

        let block_data = block_data.ok_or(ErrorObject::owned(
            INTERNAL_ERROR_CODE,
            "Couldn't find block",
            Option::<String>::None,
        ))?;

        // Decode the block
        let (block, _): (crate::block::Block, _) =
            bincode::borrow_decode_from_slice(&block_data, bincode::config::standard()).map_err(
                |_| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        "Couldn't decode block",
                        Option::<String>::None,
                    )
                },
            )?;

        // Get all blobs from the block
        let blobs = block.blobs();

        // Check if the requested blob index is valid
        if blob_index >= blobs.len() {
            return Err(ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                "Blob index out of bounds",
                Option::<String>::None,
            ));
        };

        Ok(blobs[blob_index].clone())
    }
}
