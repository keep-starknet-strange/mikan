use std::mem::size_of;
use std::ops::RangeBounds;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use bytes::Bytes;
use prost::Message;
use redb::ReadableTable;
use thiserror::Error;
use tracing::error;

use crate::malachite_types::codec::proto as codec;
use crate::malachite_types::codec::proto::ProtobufCodec;
use crate::malachite_types::proto;
use crate::malachite_types::{context::TestContext, height::Height, value::Value};
use malachitebft_app_channel::app::types::codec::Codec;
use malachitebft_app_channel::app::types::core::{CommitCertificate, Round};
use malachitebft_app_channel::app::types::ProposedValue;
use malachitebft_proto::{Error as ProtoError, Protobuf};

use crate::metrics::DbMetrics;
use crate::tables::keys::{HeightKey, UndecidedValueKey};

#[derive(Clone, Debug)]
pub struct DecidedValue {
    pub value: Value,
    pub certificate: CommitCertificate<TestContext>,
}

fn decode_certificate(bytes: &[u8]) -> Result<CommitCertificate<TestContext>, ProtoError> {
    let proto = proto::CommitCertificate::decode(bytes)?;
    codec::decode_certificate(proto)
}

fn encode_certificate(certificate: &CommitCertificate<TestContext>) -> Result<Vec<u8>, ProtoError> {
    let proto = codec::encode_certificate(certificate)?;
    Ok(proto.encode_to_vec())
}

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("Database error: {0}")]
    Database(#[from] redb::DatabaseError),

    #[error("Storage error: {0}")]
    Storage(#[from] redb::StorageError),

    #[error("Table error: {0}")]
    Table(#[from] redb::TableError),

    #[error("Commit error: {0}")]
    Commit(#[from] redb::CommitError),

    #[error("Transaction error: {0}")]
    Transaction(#[from] redb::TransactionError),

    #[error("Failed to encode/decode Protobuf: {0}")]
    Protobuf(#[from] ProtoError),

    #[error("Failed to join on task: {0}")]
    TaskJoin(#[from] tokio::task::JoinError),
}

const CERTIFICATES_TABLE: redb::TableDefinition<HeightKey, Vec<u8>> =
    redb::TableDefinition::new("certificates");

const DECIDED_VALUES_TABLE: redb::TableDefinition<HeightKey, Vec<u8>> =
    redb::TableDefinition::new("decided_values");

const UNDECIDED_PROPOSALS_TABLE: redb::TableDefinition<UndecidedValueKey, Vec<u8>> =
    redb::TableDefinition::new("undecided_values");

const DECIDED_BLOCK_DATA_TABLE: redb::TableDefinition<HeightKey, Vec<u8>> =
    redb::TableDefinition::new("decided_block_data");

const UNDECIDED_BLOCK_DATA_TABLE: redb::TableDefinition<UndecidedValueKey, Vec<u8>> =
    redb::TableDefinition::new("undecided_block_data");

struct Db {
    db: redb::Database,
    metrics: DbMetrics,
}

impl Db {
    fn new(path: impl AsRef<Path>, metrics: DbMetrics) -> Result<Self, StoreError> {
        Ok(Self {
            db: redb::Database::create(path).map_err(StoreError::Database)?,
            metrics,
        })
    }

    fn get_decided_value(&self, height: Height) -> Result<Option<DecidedValue>, StoreError> {
        let start = Instant::now();
        let mut read_bytes = 0;

        let tx = self.db.begin_read()?;

        let value = {
            let table = tx.open_table(DECIDED_VALUES_TABLE)?;
            let value = table.get(&height)?;
            value.and_then(|value| {
                let bytes = value.value();
                read_bytes = bytes.len() as u64;
                Value::from_bytes(&bytes).ok()
            })
        };

        let certificate = {
            let table = tx.open_table(CERTIFICATES_TABLE)?;
            let value = table.get(&height)?;
            value.and_then(|value| {
                let bytes = value.value();
                read_bytes += bytes.len() as u64;
                decode_certificate(&bytes).ok()
            })
        };

        self.metrics.observe_read_time(start.elapsed());
        self.metrics.add_read_bytes(read_bytes);
        self.metrics.add_key_read_bytes(size_of::<Height>() as u64);

        let decided_value = value
            .zip(certificate)
            .map(|(value, certificate)| DecidedValue { value, certificate });

        Ok(decided_value)
    }

    fn insert_decided_value(&self, decided_value: DecidedValue) -> Result<(), StoreError> {
        let start = Instant::now();
        let mut write_bytes = 0;

        let height = decided_value.certificate.height;
        let tx = self.db.begin_write()?;

        {
            let mut values = tx.open_table(DECIDED_VALUES_TABLE)?;
            let values_bytes = decided_value.value.to_bytes()?.to_vec();
            write_bytes += values_bytes.len() as u64;
            values.insert(height, values_bytes)?;
        }

        {
            let mut certificates = tx.open_table(CERTIFICATES_TABLE)?;
            let encoded_certificate = encode_certificate(&decided_value.certificate)?;
            write_bytes += encoded_certificate.len() as u64;
            certificates.insert(height, encoded_certificate)?;
        }

        tx.commit()?;

        self.metrics.observe_write_time(start.elapsed());
        self.metrics.add_write_bytes(write_bytes);

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_undecided_proposal(
        &self,
        height: Height,
        round: Round,
    ) -> Result<Option<ProposedValue<TestContext>>, StoreError> {
        let start = Instant::now();
        let mut read_bytes = 0;

        let tx = self.db.begin_read()?;
        let table = tx.open_table(UNDECIDED_PROPOSALS_TABLE)?;

        let value = if let Ok(Some(value)) = table.get(&(height, round)) {
            let bytes = value.value();
            read_bytes += bytes.len() as u64;

            let proposal = ProtobufCodec
                .decode(Bytes::from(bytes))
                .map_err(StoreError::Protobuf)?;

            Some(proposal)
        } else {
            None
        };

        self.metrics.observe_read_time(start.elapsed());
        self.metrics.add_read_bytes(read_bytes);
        self.metrics
            .add_key_read_bytes(size_of::<(Height, Round)>() as u64);

        Ok(value)
    }

    fn insert_undecided_proposal(
        &self,
        proposal: ProposedValue<TestContext>,
    ) -> Result<(), StoreError> {
        let start = Instant::now();

        let key = (proposal.height, proposal.round);
        let value = ProtobufCodec.encode(&proposal)?;

        let tx = self.db.begin_write()?;
        {
            let mut table = tx.open_table(UNDECIDED_PROPOSALS_TABLE)?;
            // Only insert if no value exists at this key
            if table.get(&key)?.is_none() {
                table.insert(key, value.to_vec())?;
            }
        }
        tx.commit()?;

        self.metrics.observe_write_time(start.elapsed());
        self.metrics.add_write_bytes(value.len() as u64);

        Ok(())
    }

    fn height_range<Table>(
        &self,
        table: &Table,
        range: impl RangeBounds<Height>,
    ) -> Result<Vec<Height>, StoreError>
    where
        Table: redb::ReadableTable<HeightKey, Vec<u8>>,
    {
        Ok(table
            .range(range)?
            .flatten()
            .map(|(key, _)| key.value())
            .collect::<Vec<_>>())
    }

    fn undecided_proposals_range<Table>(
        &self,
        table: &Table,
        range: impl RangeBounds<(Height, Round)>,
    ) -> Result<Vec<(Height, Round)>, StoreError>
    where
        Table: redb::ReadableTable<UndecidedValueKey, Vec<u8>>,
    {
        Ok(table
            .range(range)?
            .flatten()
            .map(|(key, _)| key.value())
            .collect::<Vec<_>>())
    }

    fn block_data_range<Table>(
        &self,
        table: &Table,
        range: impl RangeBounds<(Height, Round)>,
    ) -> Result<Vec<(Height, Round)>, StoreError>
    where
        Table: redb::ReadableTable<UndecidedValueKey, Vec<u8>>,
    {
        Ok(table
            .range(range)?
            .flatten()
            .map(|(key, _)| key.value())
            .collect::<Vec<_>>())
    }

    fn prune(&self, retain_height: Height) -> Result<Vec<Height>, StoreError> {
        let start = Instant::now();

        let tx = self.db.begin_write().unwrap();

        let pruned = {
            let mut undecided = tx.open_table(UNDECIDED_PROPOSALS_TABLE)?;
            let keys = self.undecided_proposals_range(&undecided, ..(retain_height, Round::Nil))?;
            for key in keys {
                undecided.remove(key)?;
            }

            let mut undecided_block_data = tx.open_table(UNDECIDED_BLOCK_DATA_TABLE)?;
            let keys =
                self.block_data_range(&undecided_block_data, ..(retain_height, Round::Nil))?;
            for key in &keys {
                undecided_block_data.remove(key)?;
            }

            let mut decided = tx.open_table(DECIDED_VALUES_TABLE)?;
            let mut certificates = tx.open_table(CERTIFICATES_TABLE)?;
            let mut decided_block_data = tx.open_table(DECIDED_BLOCK_DATA_TABLE)?;

            let keys = self.height_range(&decided, ..retain_height)?;
            for key in &keys {
                decided.remove(key)?;
                certificates.remove(key)?;
                decided_block_data.remove(key)?;
            }

            keys
        };

        tx.commit()?;

        self.metrics.observe_delete_time(start.elapsed());

        Ok(pruned)
    }

    fn min_decided_value_height(&self) -> Option<Height> {
        let start = Instant::now();

        let tx = self.db.begin_read().unwrap();
        let table = tx.open_table(DECIDED_VALUES_TABLE).unwrap();
        let (key, value) = table.first().ok()??;

        self.metrics.observe_read_time(start.elapsed());
        self.metrics.add_read_bytes(value.value().len() as u64);
        self.metrics.add_key_read_bytes(size_of::<Height>() as u64);

        Some(key.value())
    }

    // fn max_decided_value_height(&self) -> Option<Height> {
    //     let tx = self.db.begin_read().unwrap();
    //     let table = tx.open_table(DECIDED_VALUES_TABLE).unwrap();
    //     let (key, _) = table.last().ok()??;
    //     Some(key.value())
    // }

    fn create_tables(&self) -> Result<(), StoreError> {
        let tx = self.db.begin_write()?;

        // Implicitly creates the tables if they do not exist yet
        let _ = tx.open_table(DECIDED_VALUES_TABLE)?;
        let _ = tx.open_table(CERTIFICATES_TABLE)?;
        let _ = tx.open_table(UNDECIDED_PROPOSALS_TABLE)?;
        let _ = tx.open_table(DECIDED_BLOCK_DATA_TABLE)?;
        let _ = tx.open_table(UNDECIDED_BLOCK_DATA_TABLE)?;

        tx.commit()?;

        Ok(())
    }

    fn get_block_data(&self, height: Height, round: Round) -> Result<Option<Bytes>, StoreError> {
        let start = Instant::now();

        let tx = self.db.begin_read()?;

        // Try undecided block data first
        let undecided_table = tx.open_table(UNDECIDED_BLOCK_DATA_TABLE)?;
        if let Some(data) = undecided_table.get(&(height, round))? {
            let bytes = data.value();
            let read_bytes = bytes.len() as u64;
            self.metrics.observe_read_time(start.elapsed());
            self.metrics.add_read_bytes(read_bytes);
            self.metrics
                .add_key_read_bytes((size_of::<Height>() + size_of::<Round>()) as u64);
            return Ok(Some(Bytes::copy_from_slice(&bytes)));
        }

        // Then try decided block data
        let decided_table = tx.open_table(DECIDED_BLOCK_DATA_TABLE)?;
        if let Some(data) = decided_table.get(&height)? {
            let bytes = data.value();
            let read_bytes = bytes.len() as u64;
            self.metrics.observe_read_time(start.elapsed());
            self.metrics.add_read_bytes(read_bytes);
            self.metrics.add_key_read_bytes(size_of::<Height>() as u64);
            return Ok(Some(Bytes::copy_from_slice(&bytes)));
        }

        self.metrics.observe_read_time(start.elapsed());
        Ok(None)
    }

    fn insert_undecided_block_data(
        &self,
        height: Height,
        round: Round,
        data: Bytes,
    ) -> Result<(), StoreError> {
        let start = Instant::now();
        let write_bytes = data.len() as u64;

        let tx = self.db.begin_write()?;
        {
            let mut table = tx.open_table(UNDECIDED_BLOCK_DATA_TABLE)?;
            let key = (height, round);
            // Only insert if no value exists at this key
            if table.get(&key)?.is_none() {
                table.insert(key, data.to_vec())?;
            }
        }
        tx.commit()?;

        self.metrics.observe_write_time(start.elapsed());
        self.metrics.add_write_bytes(write_bytes);

        Ok(())
    }

    fn insert_decided_block_data(&self, height: Height, data: Bytes) -> Result<(), StoreError> {
        let start = Instant::now();
        let write_bytes = data.len() as u64;

        let tx = self.db.begin_write()?;
        {
            let mut table = tx.open_table(DECIDED_BLOCK_DATA_TABLE)?;
            // Only insert if no value exists at this key
            if table.get(&height)?.is_none() {
                table.insert(height, data.to_vec())?;
            }
        }
        tx.commit()?;

        self.metrics.observe_write_time(start.elapsed());
        self.metrics.add_write_bytes(write_bytes);

        Ok(())
    }

    pub fn get_decided_block(&self, height: Height) -> Result<Option<Bytes>, StoreError> {
        let start = Instant::now();
        let tx = self.db.begin_read()?;

        let decided_table = tx.open_table(DECIDED_BLOCK_DATA_TABLE)?;
        if let Some(data) = decided_table.get(&height)? {
            let bytes = data.value();
            let read_bytes = bytes.len() as u64;
            self.metrics.observe_read_time(start.elapsed());
            self.metrics.add_read_bytes(read_bytes);
            self.metrics.add_key_read_bytes(size_of::<Height>() as u64);
            return Ok(Some(Bytes::copy_from_slice(&bytes)));
        }
        Ok(None)
    }
}

#[derive(Clone)]
pub struct Store {
    db: Arc<Db>,
    path: PathBuf,
}

impl Store {
    pub fn open(path: impl AsRef<Path>, metrics: DbMetrics) -> Result<Self, StoreError> {
        let path_buf = path.as_ref().to_path_buf();
        let db = Db::new(&path_buf, metrics)?;
        db.create_tables()?;

        Ok(Self {
            db: Arc::new(db),
            path: path_buf,
        })
    }

    pub fn get_path(&self) -> &PathBuf {
        &self.path
    }

    pub async fn min_decided_value_height(&self) -> Option<Height> {
        let db = Arc::clone(&self.db);
        tokio::task::spawn_blocking(move || db.min_decided_value_height())
            .await
            .ok()
            .flatten()
    }

    // pub async fn max_decided_value_height(&self) -> Option<Height> {
    //     let db = Arc::clone(&self.db);
    //     tokio::task::spawn_blocking(move || db.max_decided_value_height())
    //         .await
    //         .ok()
    //         .flatten()
    // }

    pub async fn get_decided_value(
        &self,
        height: Height,
    ) -> Result<Option<DecidedValue>, StoreError> {
        let db = Arc::clone(&self.db);

        tokio::task::spawn_blocking(move || db.get_decided_value(height)).await?
    }

    pub async fn store_decided_value(
        &self,
        certificate: &CommitCertificate<TestContext>,
        value: Value,
    ) -> Result<(), StoreError> {
        let decided_value = DecidedValue {
            value,
            certificate: certificate.clone(),
        };

        let db = Arc::clone(&self.db);
        tokio::task::spawn_blocking(move || db.insert_decided_value(decided_value)).await?
    }

    pub async fn store_undecided_proposal(
        &self,
        value: ProposedValue<TestContext>,
    ) -> Result<(), StoreError> {
        let db = Arc::clone(&self.db);
        tokio::task::spawn_blocking(move || db.insert_undecided_proposal(value)).await?
    }

    pub async fn get_undecided_proposal(
        &self,
        height: Height,
        round: Round,
    ) -> Result<Option<ProposedValue<TestContext>>, StoreError> {
        let db = Arc::clone(&self.db);
        tokio::task::spawn_blocking(move || db.get_undecided_proposal(height, round)).await?
    }

    pub async fn prune(&self, retain_height: Height) -> Result<Vec<Height>, StoreError> {
        let db = Arc::clone(&self.db);
        tokio::task::spawn_blocking(move || db.prune(retain_height)).await?
    }
    pub async fn get_block_data(
        &self,
        height: Height,
        round: Round,
    ) -> Result<Option<Bytes>, StoreError> {
        let db = Arc::clone(&self.db);
        tokio::task::spawn_blocking(move || db.get_block_data(height, round)).await?
    }

    pub async fn get_decided_block(&self, height: Height) -> Result<Option<Bytes>, StoreError> {
        let db = Arc::clone(&self.db);
        tokio::task::spawn_blocking(move || db.get_decided_block(height)).await?
    }

    pub async fn store_undecided_block_data(
        &self,
        height: Height,
        round: Round,
        data: Bytes,
    ) -> Result<(), StoreError> {
        let db = Arc::clone(&self.db);
        tokio::task::spawn_blocking(move || db.insert_undecided_block_data(height, round, data))
            .await?
    }

    pub async fn store_decided_block_data(
        &self,
        height: Height,
        data: Bytes,
    ) -> Result<(), StoreError> {
        let db = Arc::clone(&self.db);
        tokio::task::spawn_blocking(move || db.insert_decided_block_data(height, data)).await?
    }
}
