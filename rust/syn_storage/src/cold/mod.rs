//! Cold storage module using DuckDB for dormant NPCs.
//!
//! Provides columnar storage for NPCs that are not currently
//! active in the simulation (Tier 3).

/// DuckDB-based cold storage implementation.
pub mod duckdb_cold_store;

pub use duckdb_cold_store::DuckDbColdStore;
