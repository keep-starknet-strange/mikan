use thiserror::Error;

#[derive(Debug, Error)]
pub enum BlockError {
    #[error("Null Parent Hash")]
    NullParentHash,
    #[error("Parent Hash Mismatch")]
    ParentHashValidationFail,
    #[error("Invalid Block Number {0}")]
    InvalidBlockNumber(usize),
    #[error("Expected :{0:?}. Got: {1:?}")]
    FinalityHashMismatch(Vec<u8>,Vec<u8>)
}
