use thiserror::Error as ThisError;
use anyhow::Error as AnyhowError;
use std::result;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, ThisError)]
pub enum StorageError {
    #[error("log compacted")]
    Compacted,
    #[error("log unavailable")]
    Unavailable,
    #[error("snapshot out of date")]
    SnapshotOutOfDate,
    #[error("snapshot temporarily unavailable")]
    SnapshotTemporarilyUnavailable,
    #[error("log temporarily unavailable")]
    LogTemporarilyUnavailable,
}

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("storage error: {0}")]
    Store(#[from] StorageError),
    #[error("anyhow error: {0}")]
    Anyhow(#[from] AnyhowError),
}
