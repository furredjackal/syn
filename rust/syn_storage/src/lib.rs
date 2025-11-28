//! Hybrid NPC storage system with hot (active) and cold (dormant) tiers.
//!
//! This crate provides a tiered storage solution for NPC data:
//! - **Hot storage** (redb): Fast key-value store for active NPCs
//! - **Cold storage** (DuckDB): Columnar database for dormant NPCs
//!
//! The [`HybridStorage`] struct provides a unified API for both tiers,
//! with promote/demote operations for LOD transitions.

/// Hot storage module (redb-based).
pub mod hot;
/// Cold storage module (DuckDB-based).
pub mod cold;
/// Data models for stored entities.
pub mod models;
/// Hybrid storage combining hot and cold tiers.
pub mod hybrid_store;
/// Unified error type for storage operations.
pub mod storage_error;

pub use hybrid_store::HybridStorage;
