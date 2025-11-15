//! syn_director: Event orchestration and storylet selection.
//!
//! Central narrative brain: evaluates world state, personality vectors, and relationship
//! pressures to select and fire emergent storylets.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use syn_core::{NpcId, Relationship, SimTick, WorldState};
use syn_query::{ClusterQuery, RelationshipQuery, StoryletQuery};

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
}

/// Conditions that must be met for a storylet to be eligible.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryletPrerequisites {
    pub min_relationship_affection: Option<f32>,
    pub min_relationship_resentment: Option<f32>,
    pub stat_conditions: HashMap<String, (f32, f32)>, // {"mood": (-10.0, -5.0)} means mood in range
    pub life_stages: Vec<String>,                     // ["Teen", "Adult", "Elder"]
    pub tags: Vec<String>,                           // must have these tags
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
    pub stat_impacts: HashMap<String, f32>,         // e.g., {"mood": -2.0}
    pub relationship_deltas: Vec<(NpcId, NpcId, Relationship)>, // (from, to, delta)
    pub memory_event_id: String,
    pub emotional_intensity: f32,                   // -1.0 to +1.0
}

impl Default for StoryletOutcome {
    fn default() -> Self {
        StoryletOutcome {
            stat_impacts: HashMap::new(),
            relationship_deltas: Vec::new(),
            memory_event_id: "unknown".to_string(),
            emotional_intensity: 0.0,
        }
    }
}

/// Cooldown tracker to prevent storylet repetition.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CooldownTracker {
    global_cooldowns: HashMap<String, SimTick>,    // storylet_id -> until_tick
    npc_cooldowns: HashMap<(String, NpcId), SimTick>, // (storylet_id, npc_id) -> until_tick
}

impl CooldownTracker {
    fn new() -> Self {
        CooldownTracker {
            global_cooldowns: HashMap::new(),
            npc_cooldowns: HashMap::new(),
        }
    }

    fn is_ready(&self, storylet_id: &str, npc_id: NpcId, current_tick: SimTick) -> bool {
        let global_ready = self
            .global_cooldowns
            .get(storylet_id)
            .map(|until| current_tick.0 >= until.0)
            .unwrap_or(true);

        let npc_ready = self
            .npc_cooldowns
            .get(&(storylet_id.to_string(), npc_id))
            .map(|until| current_tick.0 >= until.0)
            .unwrap_or(true);

        global_ready && npc_ready
    }

    fn mark_cooldown(&mut self, storylet_id: &str, npc_id: NpcId, cooldown_ticks: u32, current_tick: SimTick) {
        let until = SimTick::new(current_tick.0 + cooldown_ticks as u64);
        self.global_cooldowns.insert(storylet_id.to_string(), until);
        self.npc_cooldowns.insert((storylet_id.to_string(), npc_id), until);
    }
}

/// Event Director: orchestrates storylet selection and firing.
pub struct EventDirector {
    storylets: Vec<Storylet>,
    cooldowns: CooldownTracker,
}

impl EventDirector {
    pub fn new() -> Self {
        EventDirector {
            storylets: Vec::new(),
            cooldowns: CooldownTracker::new(),
        }
    }

    /// Register a storylet.
    pub fn register_storylet(&mut self, storylet: Storylet) {
        self.storylets.push(storylet);
    }

    /// Find eligible storylets based on world state.
    pub fn find_eligible(&self, world: &WorldState, current_tick: SimTick) -> Vec<&Storylet> {
        self.storylets
            .iter()
            .filter(|s| self.is_eligible(s, world, current_tick))
            .collect()
    }

    /// Check if a storylet is eligible to fire.
    fn is_eligible(&self, storylet: &Storylet, world: &WorldState, current_tick: SimTick) -> bool {
        // Check cooldown
        if !self.cooldowns.is_ready(&storylet.id, world.player_id, current_tick) {
            return false;
        }

        // Check prerequisites
        for role in &storylet.roles {
            if !world.npcs.contains_key(&role.npc_id) {
                return false; // Role NPC doesn't exist
            }
        }

        // Check relationship conditions
        if let Some(min_affection) = storylet.prerequisites.min_relationship_affection {
            if let Some(target_role) = storylet.roles.get(0) {
                let rel = world.get_relationship(world.player_id, target_role.npc_id);
                if rel.affection < min_affection {
                    return false;
                }
            }
        }

        true
    }

    /// Score a storylet for selection (0.0..100.0).
    pub fn score_storylet(&self, storylet: &Storylet, world: &WorldState) -> f32 {
        let mut score = storylet.weight;

        // Pressure point bonus: if there's relationship tension, bump "conflict" storylets
        if storylet.tags.contains(&"conflict".to_string()) {
            if let Some(target_role) = storylet.roles.get(0) {
                if RelationshipQuery::has_pressure_point(world, world.player_id, target_role.npc_id) {
                    score *= 1.5;
                }
            }
        }

        // Narrative heat: higher heat = higher priority in emergent moments
        score *= storylet.heat / 50.0; // Normalize

        score.clamp(0.0, 100.0)
    }

    /// Select the best eligible storylet(s) to fire this tick.
    pub fn select_next_event(&self, world: &WorldState, current_tick: SimTick) -> Option<&Storylet> {
        let eligible = self.find_eligible(world, current_tick);
        if eligible.is_empty() {
            return None;
        }

        // Simple greedy: pick highest-scored storylet
        let best = eligible
            .iter()
            .max_by(|a, b| {
                let score_a = self.score_storylet(a, world);
                let score_b = self.score_storylet(b, world);
                score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
            });

        best.copied()
    }

    /// Fire a storylet: update world state with outcomes.
    pub fn fire_storylet(
        &mut self,
        storylet: &Storylet,
        world: &mut WorldState,
        outcome: StoryletOutcome,
        current_tick: SimTick,
    ) {
        // Apply stat impacts
        for (stat, delta) in outcome.stat_impacts {
            match stat.as_str() {
                "mood" => world.player_stats.mood = (world.player_stats.mood + delta).clamp(-10.0, 10.0),
                _ => {} // Extend as needed
            }
        }

        // Apply relationship deltas
        for (from, to, delta) in outcome.relationship_deltas {
            let current = world.get_relationship(from, to);
            let updated = Relationship {
                affection: (current.affection + delta.affection).clamp(-10.0, 10.0),
                trust: (current.trust + delta.trust).clamp(-10.0, 10.0),
                attraction: (current.attraction + delta.attraction).clamp(-10.0, 10.0),
                familiarity: (current.familiarity + delta.familiarity).clamp(-10.0, 10.0),
                resentment: (current.resentment + delta.resentment).clamp(-10.0, 10.0),
            };
            world.set_relationship(from, to, updated);
        }

        // Update karma (based on outcome emotional intensity)
        world.player_karma.0 = (world.player_karma.0 + outcome.emotional_intensity * 10.0).clamp(-100.0, 100.0);

        // Mark cooldown
        if let Some(first_role) = storylet.roles.first() {
            self.cooldowns.mark_cooldown(
                &storylet.id,
                first_role.npc_id,
                storylet.cooldown_ticks,
                current_tick,
            );
        }
    }

    /// Get all registered storylets (for inspection/debugging).
    pub fn all_storylets(&self) -> &[Storylet] {
        &self.storylets
    }
}

impl Default for EventDirector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn_core::{WorldSeed, NpcId, WorldState, AbstractNpc, AttachmentStyle, Traits};

    #[test]
    fn test_storylet_creation() {
        let storylet = Storylet {
            id: "event_001".to_string(),
            name: "First Meeting".to_string(),
            tags: vec!["romance".to_string()],
            prerequisites: StoryletPrerequisites {
                min_relationship_affection: None,
                min_relationship_resentment: None,
                stat_conditions: HashMap::new(),
                life_stages: vec!["Adult".to_string()],
                tags: vec![],
            },
            heat: 50.0,
            weight: 0.5,
            cooldown_ticks: 100,
            roles: vec![StoryletRole {
                name: "target".to_string(),
                npc_id: NpcId(2),
            }],
        };

        assert_eq!(storylet.id, "event_001");
        assert_eq!(storylet.heat, 50.0);
    }

    #[test]
    fn test_event_director_register() {
        let mut director = EventDirector::new();
        let storylet = Storylet {
            id: "event_001".to_string(),
            name: "Test Event".to_string(),
            tags: vec![],
            prerequisites: StoryletPrerequisites {
                min_relationship_affection: None,
                min_relationship_resentment: None,
                stat_conditions: HashMap::new(),
                life_stages: vec![],
                tags: vec![],
            },
            heat: 50.0,
            weight: 0.5,
            cooldown_ticks: 100,
            roles: vec![],
        };

        director.register_storylet(storylet);
        assert_eq!(director.all_storylets().len(), 1);
    }

    #[test]
    fn test_event_director_score() {
        let mut director = EventDirector::new();
        let mut world = WorldState::new(WorldSeed(42), NpcId(1));

        let storylet = Storylet {
            id: "event_001".to_string(),
            name: "Test Event".to_string(),
            tags: vec!["romance".to_string()],
            prerequisites: StoryletPrerequisites {
                min_relationship_affection: None,
                min_relationship_resentment: None,
                stat_conditions: HashMap::new(),
                life_stages: vec![],
                tags: vec![],
            },
            heat: 75.0,
            weight: 0.8,
            cooldown_ticks: 100,
            roles: vec![],
        };

        let score = director.score_storylet(&storylet, &world);
        assert!(score > 0.0);
    }

    #[test]
    fn test_outcome_application() {
        let mut director = EventDirector::new();
        let mut world = WorldState::new(WorldSeed(42), NpcId(1));

        let storylet = Storylet {
            id: "event_001".to_string(),
            name: "Test Event".to_string(),
            tags: vec![],
            prerequisites: StoryletPrerequisites {
                min_relationship_affection: None,
                min_relationship_resentment: None,
                stat_conditions: HashMap::new(),
                life_stages: vec![],
                tags: vec![],
            },
            heat: 50.0,
            weight: 0.5,
            cooldown_ticks: 100,
            roles: vec![StoryletRole {
                name: "target".to_string(),
                npc_id: NpcId(2),
            }],
        };

        let mut outcome = StoryletOutcome::default();
        outcome.stat_impacts.insert("mood".to_string(), -2.0);

        let initial_mood = world.player_stats.mood;
        director.fire_storylet(&storylet, &mut world, outcome, SimTick(0));

        assert_eq!(world.player_stats.mood, initial_mood - 2.0);
    }
}
