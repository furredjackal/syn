use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use syn_core::{
    relationship_model::{RelationshipAxis, RelationshipDelta},
    RelationshipState, StatDelta,
};
use syn_core::NpcId;
use syn_core::LifeStage;

/// A storylet: condition-driven narrative fragment with roles, outcomes, and cooldowns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Storylet {
    pub id: String,
    pub name: String,
    pub tags: Vec<String>,                      // e.g., ["romance", "crisis", "career"]
    pub prerequisites: StoryletPrerequisites,
    pub heat: f32,                              // narrative intensity (0.0..100.0)
    pub weight: f32,                            // base probability of firing
    pub cooldown_ticks: u32,                    // prevent rapid re-firing
    pub roles: Vec<StoryletRole>,              // required NPC roles
    #[serde(default)]
    pub heat_category: Option<StoryletHeatCategory>,
}

pub use syn_director::StoryletHeatCategory;

/// Relationship-based prerequisite (additive, non-breaking).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipPrereq {
    /// Which actor owns the relationship. None defaults to the player.
    #[serde(default)]
    pub actor_id: Option<u64>,
    /// Target NPC the prereq references.
    pub target_id: u64,
    /// Relationship axis to inspect.
    pub axis: RelationshipAxis,
    /// Optional numeric bounds for the axis value.
    #[serde(default)]
    pub min_value: Option<f32>,
    #[serde(default)]
    pub max_value: Option<f32>,
    /// Optional band-based gating (fuzzy thresholds).
    #[serde(default)]
    pub min_band: Option<String>,
    #[serde(default)]
    pub max_band: Option<String>,
}

/// Conditions that must be met for a storylet to be eligible.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryletPrerequisites {
    pub min_relationship_affection: Option<f32>,
    pub min_relationship_resentment: Option<f32>,
    pub stat_conditions: HashMap<String, (f32, f32)>, // {"mood": (-10.0, -5.0)} means mood in range
    pub life_stages: Vec<String>,                     // ["Teen", "Adult", "Elder"]
    pub tags: Vec<String>,                           // must have these tags
    pub relationship_states: Vec<RelationshipState>,  // Only fire if relationship is in one of these states
    // Memory prerequisites for event echoes
    pub memory_tags_required: Vec<String>,           // NPC must have memory with at least one of these tags
    pub memory_tags_forbidden: Vec<String>,          // NPC must NOT have memory with these tags (conflict avoidance)
    pub memory_recency_ticks: Option<u64>,           // If specified, memory must be within N ticks (default: 7 days = 168 ticks)
    /// Optional relationship-based prerequisites (additive).
    #[serde(default)]
    pub relationship_prereqs: Vec<RelationshipPrereq>,
    /// Optional allowed life stages for this storylet (typed).
    #[serde(default)]
    pub allowed_life_stages: Vec<LifeStage>,
}

/// A role in a storylet (e.g., "target", "rival", "manager").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryletRole {
    pub name: String,
    pub npc_id: NpcId,
}

/// Outcome of a storylet firing: stat changes, relationship shifts, memory entries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryletOutcome {
    /// Unified stat changes from this outcome.
    ///
    /// JSON content uses `stat_impacts`; we expose it as `stat_deltas` in code.
    #[serde(rename = "stat_impacts", alias = "stat_deltas", default)]
    pub stat_deltas: Vec<StatDelta>,
    #[serde(
        rename = "relationship_impacts",
        alias = "relationship_deltas",
        default
    )]
    pub relationship_deltas: Vec<RelationshipDelta>,
    #[serde(default)]
    pub karma_delta: Option<f32>,
    #[serde(default)]
    pub memory_event_id: String,
    #[serde(default)]
    pub emotional_intensity: f32, // -1.0 to +1.0
    #[serde(default)]
    pub memory_tags: Vec<String>, // Tags applied to recorded memory
    #[serde(default)]
    pub heat_spike: f32, // Additional world heat delta from choices
    #[serde(default)]
    pub next_storylet: Option<String>,
}

impl Default for StoryletOutcome {
    fn default() -> Self {
        StoryletOutcome {
            stat_deltas: Vec::new(),
            relationship_deltas: Vec::new(),
            karma_delta: None,
            memory_event_id: "unknown".to_string(),
            emotional_intensity: 0.0,
            memory_tags: Vec::new(),
            heat_spike: 0.0,
            next_storylet: None,
        }
    }
}
