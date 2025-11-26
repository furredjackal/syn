//! Simulation systems for SYN.
//!
//! This module contains subsystems that operate on world state during simulation ticks.

pub mod npc_updates;
pub mod tiers;

pub use npc_updates::{
    update_npcs_for_tick, update_relationships_for_npc, update_stats_for_npc, NpcUpdateConfig,
};
pub use tiers::{update_npc_tiers_for_tick, TierUpdateConfig};
