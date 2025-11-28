//! NPC storage model.

use serde::{Deserialize, Serialize};

/// Minimal canonical representation for Tier 3 (dormant) NPC storage.
///
/// This is a lightweight struct optimized for cold storage, containing
/// only the essential fields needed to reconstruct an NPC when promoted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbstractNpc {
    /// Unique NPC identifier.
    pub id: u64,
    /// Age in years.
    pub age: u16,
    /// District identifier.
    pub district: u16,
    /// Wealth level.
    pub wealth: i32,
    /// Health value (0.0-100.0).
    pub health: f32,
    /// Deterministic generation seed.
    pub seed: u64,
}
