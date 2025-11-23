//! syn_core: Foundation types, RNG, and persistence for SYN simulation engine.
//!
//! This crate provides:
//! - Seeded RNG for deterministic simulation
//! - Core types (Stats, Traits, Relationships, NPCs, World)
//! - SQLite persistence layer
//! - Utility types for serialization and querying

pub mod types;
pub mod rng;
pub mod persistence;
pub mod errors;
pub mod stats;
pub mod relationships;
pub mod relationship_model;
pub mod relationship_pressure;
pub mod relationship_milestones;
pub mod narrative_heat;
pub mod life_stage;
pub mod digital_legacy;

pub use types::*;
pub use rng::*;
pub use persistence::*;
pub use errors::*;
pub use stats::*;
pub use relationships::*;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
