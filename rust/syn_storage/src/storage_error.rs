//! Unified error type for storage operations.

use thiserror::Error;

/// Unified error type for hybrid storage layers.
#[derive(Error, Debug)]
pub enum StorageError {
    /// Error from the redb hot storage layer.
    #[error("Redb error: {0}")]
    Redb(#[from] redb::Error),
    /// Error from the DuckDB cold storage layer.
    #[error("DuckDB error: {0}")]
    DuckDb(#[from] duckdb::Error),
    /// Error during serialization/deserialization.
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),
    /// Catch-all for other storage errors.
    #[error("Unknown storage error: {0}")]
    Unknown(String),
}
