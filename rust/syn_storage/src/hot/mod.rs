//! Hot storage module using redb for active NPCs.
//!
//! Provides fast key-value storage for NPCs that are currently
//! active in the simulation (Tier 1).

/// Redb-based hot storage implementation.
pub mod redb_hot_store;

pub use redb_hot_store::RedbHotStore;
