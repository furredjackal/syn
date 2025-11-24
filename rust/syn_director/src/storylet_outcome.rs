use serde::{Deserialize, Serialize};

use crate::{InteractionTone, StoryletActors, StoryletChoice, StoryletHeatCategory, EventContext};
use syn_core::{Stats, Relationship};
use syn_memory::MemorySystem;
use syn_storage::HybridStorage;
use crate::{StatDelta, RelationshipDelta};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryEntryTemplate {
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorldFlagUpdate {
    #[serde(default)]
    pub flag: String,
    #[serde(default)]
    pub value: bool,
}

/// Outcome set for a storylet (GDD 3.16.4)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryletOutcomeSet {
    #[serde(default)]
    pub stat_deltas: Vec<StatDelta>,
    #[serde(default)]
    pub mood_delta: i32,
    #[serde(default)]
    pub relationship_deltas: Vec<RelationshipDelta>,
    #[serde(default)]
    pub memory: MemoryEntryTemplate,
    #[serde(default)]
    pub flags: Vec<WorldFlagUpdate>,

    // Existing metadata used by director
    #[serde(default)]
    pub choices: Vec<StoryletChoice>,
    #[serde(default)]
    pub max_uses: Option<u32>,
    #[serde(default)]
    pub heat_category: Option<StoryletHeatCategory>,
    #[serde(default)]
    pub actors: Option<StoryletActors>,
    #[serde(default)]
    pub interaction_tone: Option<InteractionTone>,
}

impl Default for StoryletOutcomeSet {
    fn default() -> Self {
        StoryletOutcomeSet {
            stat_deltas: Vec::new(),
            mood_delta: 0,
            relationship_deltas: Vec::new(),
            memory: MemoryEntryTemplate::default(),
            flags: Vec::new(),
            choices: Vec::new(),
            max_uses: None,
            heat_category: None,
            actors: None,
            interaction_tone: None,
        }
    }
}

/// Placeholder simulation context for outcome application.
#[derive(Default)]
pub struct SimulationContext<'a> {
    pub world: Option<&'a mut crate::WorldState>,
    pub sim: Option<&'a mut crate::SimState>,
    pub stats: Option<&'a mut Stats>,
    pub relationships: Option<&'a mut Relationship>,
    pub memory: Option<&'a mut MemorySystem>,
    pub storage: Option<&'a mut HybridStorage>,
    pub event: EventContext,
    pub seed: u64,
    pub tick: u64,
}

impl StoryletOutcomeSet {
    pub fn apply(&self, _ctx: &mut SimulationContext) {
        // Hook up once full simulation context is integrated.
    }
}
