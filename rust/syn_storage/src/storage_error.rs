use thiserror::Error;

/// Unified error type for hybrid storage layers.
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Redb error: {0}")]
    Redb(#[from] redb::Error),
    #[error("DuckDB error: {0}")]
    DuckDb(#[from] duckdb::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),
    #[error("Unknown storage error: {0}")]
    Unknown(String),
}
