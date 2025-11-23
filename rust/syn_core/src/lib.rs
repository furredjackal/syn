//! syn_core: Foundation types, RNG, and persistence for SYN simulation engine.
//!
//! This crate provides:
//! - Seeded RNG for deterministic simulation
//! - Core types (Stats, Traits, Relationships, NPCs, World)
//! - SQLite persistence layer
//! - Utility types for serialization and querying

pub mod digital_legacy;
pub mod errors;
pub mod life_stage;
pub mod narrative_heat;
pub mod npc;
pub mod npc_actions;
pub mod npc_behavior;
pub mod persistence;
pub mod relationship_milestones;
pub mod relationship_model;
pub mod relationship_pressure;
pub mod relationships;
pub mod rng;
pub mod stats;
pub mod time;
pub mod types;

pub use errors::*;
pub use persistence::*;
pub use relationships::*;
pub use rng::*;
pub use stats::*;
pub use types::*;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
