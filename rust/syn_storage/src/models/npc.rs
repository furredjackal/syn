use serde::{Deserialize, Serialize};

/// Minimal canonical representation for Tier 3 (dormant) NPC storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbstractNpc {
    pub id: u64,       // unique NPC id
    pub age: u8,       // age in years
    pub district: u16,
    pub wealth: i32,
    pub health: f32,
    pub seed: u64,     // deterministic generation seed
}
