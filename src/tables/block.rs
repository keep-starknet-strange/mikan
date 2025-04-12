use std::sync::Arc;

use crate::error::StoreError;
use eyre::Ok;
use redb::{Database, TypeName, Value};

use crate::block::Block;
use bincode::{decode_from_slice, encode_to_vec};

use super::Table;

const DA_BLOCK_TABLE: redb::TableDefinition<u64, Block> =
    redb::TableDefinition::new("da_block_table");

pub struct DaBlockTable {
    pub db: Arc<Database>,
}

impl Table for DaBlockTable {
    type Key = u64;
    type Value = Block;
    fn get(&self, key: u64) -> Result<Option<Self::Value>, StoreError> {
        let tx = self.db.begin_read().unwrap();
        let table = tx.open_table(DA_BLOCK_TABLE).unwrap();
        let res = table.get(key)?;
        // .map_err(|e| StoreError::DatabaseError(e.to_string()))?;

        Ok(res.map(|value| value.value())).map_err(|e| StoreError::UnknownError(e.to_string()))
    }

    fn insert(&self, key: Self::Key, value: Self::Value) -> Result<(), StoreError> {
        let write_txn = self.db.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(DA_BLOCK_TABLE)?;

            table.insert(key, value)?;
        }
        write_txn.commit()?;
        Ok(()).map_err(|e| StoreError::UnknownError(e.to_string()))
    }
}

impl Value for Block {
    type SelfType<'a> = Block;
    type AsBytes<'a> = Vec<u8>;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        decode_from_slice::<Block, _>(data, bincode::config::standard())
            .expect("Unable to decode data")
            .0
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'b,
    {
        encode_to_vec(value, bincode::config::standard()).expect("Unable to encode data")
    }

    fn type_name() -> redb::TypeName {
        TypeName::new("Block")
    }
}
