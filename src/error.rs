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
    FinalityHashMismatch(Vec<u8>,Vec<u8>),
    #[error("Expected :{0:?}. Got: {1:?}")]
    DataHashMismatch(Vec<u8>,Vec<u8>)
}
