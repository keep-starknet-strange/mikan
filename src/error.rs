use thiserror::Error;

#[derive(Debug, Error)]
pub enum BlockError {
    #[error("Null Parent Hash")]
    NullParentFinalityHash,
    #[error("Parent Hash Mismatch")]
    ParentFinalityHashValidationFail,
    #[error("Invalid Block Number {0}")]
    InvalidBlockNumber(usize),
    #[error("Expected :{0:?}. Got: {1:?}")]
    FinalityHashMismatch([u8; 32], [u8; 32]),
    #[error("Expected :{0:?}. Got: {1:?}")]
    DataHashMismatch([u8; 32], [u8; 32]),
    #[error("Merkle Tree Root calculation error")]
    MerkleTreeError,
    #[error("{0}")]
    FriedaError(String),
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
    Protobuf(#[from] malachitebft_proto::Error),

    #[error("Failed to join on task: {0}")]
    TaskJoin(#[from] tokio::task::JoinError),

    #[error("{0}")]
    UnknownError(String),
}
