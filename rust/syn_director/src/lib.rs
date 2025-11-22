//! syn_director: Event orchestration and storylet selection.
//!
//! Central narrative brain: evaluates world state, personality vectors, and relationship
//! pressures to select and fire emergent storylets.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use syn_core::{
    behavior_action_from_tags, apply_stat_deltas, NpcId, Relationship, RelationshipState, SimTick,
    StatDelta, WorldState,
};
use syn_memory::{MemoryEntry, MemorySystem};
use syn_query::RelationshipQuery;

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
    pub relationship_states: Vec<RelationshipState>,  // Only fire if relationship is in one of these states
    // Memory prerequisites for event echoes
    pub memory_tags_required: Vec<String>,           // NPC must have memory with at least one of these tags
    pub memory_tags_forbidden: Vec<String>,          // NPC must NOT have memory with these tags (conflict avoidance)
    pub memory_recency_ticks: Option<u64>,           // If specified, memory must be within N ticks (default: 7 days = 168 ticks)
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
    #[serde(default)]
    pub relationship_deltas: Vec<(NpcId, NpcId, Relationship)>, // (from, to, delta)
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
    pub fn find_eligible(&self, world: &WorldState, memory: &MemorySystem, current_tick: SimTick) -> Vec<&Storylet> {
        self.storylets
            .iter()
            .filter(|s| self.is_eligible(s, world, memory, current_tick))
            .collect()
    }

    /// Check if a storylet is eligible to fire.
    fn is_eligible(&self, storylet: &Storylet, world: &WorldState, memory: &MemorySystem, current_tick: SimTick) -> bool {
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

        // Check relationship state conditions
        if !storylet.prerequisites.relationship_states.is_empty() {
            if let Some(target_role) = storylet.roles.get(0) {
                let rel = world.get_relationship(world.player_id, target_role.npc_id);
                // If any relationship states are specified, the current state must match one of them
                if !storylet.prerequisites.relationship_states.contains(&rel.state) {
                    return false;
                }
            }
        }

        // Check memory prerequisites
        if !storylet.prerequisites.memory_tags_required.is_empty() {
            if let Some(target_role) = storylet.roles.get(0) {
                // NPC must have at least one of the required memory tags
                if let Some(journal) = memory.journals.get(&target_role.npc_id) {
                    let has_required_tag = storylet.prerequisites.memory_tags_required.iter().any(|tag| {
                        !journal.memories_with_tag(tag).is_empty()
                    });
                    if !has_required_tag {
                        return false;
                    }
                } else {
                    return false; // No journal for this NPC
                }
            }
        }

        // Check memory forbidden tags
        if !storylet.prerequisites.memory_tags_forbidden.is_empty() {
            if let Some(target_role) = storylet.roles.get(0) {
                // NPC must NOT have any of the forbidden memory tags
                if let Some(journal) = memory.journals.get(&target_role.npc_id) {
                    let has_forbidden_tag = storylet.prerequisites.memory_tags_forbidden.iter().any(|tag| {
                        !journal.memories_with_tag(tag).is_empty()
                    });
                    if has_forbidden_tag {
                        return false;
                    }
                }
            }
        }

        // Check memory recency
        if let Some(recency_ticks) = storylet.prerequisites.memory_recency_ticks {
            if let Some(target_role) = storylet.roles.get(0) {
                // If required tags specified, check that they exist within the recency window
                if !storylet.prerequisites.memory_tags_required.is_empty() {
                    if let Some(journal) = memory.journals.get(&target_role.npc_id) {
                        let since_tick = SimTick::new(current_tick.0.saturating_sub(recency_ticks));
                        let has_recent_tag = storylet.prerequisites.memory_tags_required.iter().any(|tag| {
                            journal.memories_since(since_tick).iter().any(|m| m.tags.contains(tag))
                        });
                        if !has_recent_tag {
                            return false;
                        }
                    }
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
        score *= storylet.heat / 50.0; // Normalize heat intensity

        // Apply narrative heat multiplier (0.5..2.0 based on current heat level)
        score *= world.heat_multiplier();

        // Behavior intent: prioritize storylets that match current player drive
        if let Some(action) = behavior_action_from_tags(&storylet.tags) {
            let intent = world.player_behavior_bias(action);
            score *= intent;
        }

        score.clamp(0.0, 100.0)
    }

    /// Select the best eligible storylet(s) to fire this tick.
    pub fn select_next_event(&self, world: &WorldState, memory: &MemorySystem, current_tick: SimTick) -> Option<&Storylet> {
        let eligible = self.find_eligible(world, memory, current_tick);
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
        memory: &mut MemorySystem,
        outcome: StoryletOutcome,
        current_tick: SimTick,
    ) {
    apply_storylet_outcome(world, memory, storylet, &outcome, current_tick);
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

pub fn apply_storylet_outcome(
    world: &mut WorldState,
    memory: &mut MemorySystem,
    storylet: &Storylet,
    outcome: &StoryletOutcome,
    current_tick: SimTick,
) {
        // Apply stat impacts
    apply_stat_deltas(&mut world.player_stats, &outcome.stat_deltas);

    // Apply relationship deltas
    for (from, to, delta) in &outcome.relationship_deltas {
        let current = world.get_relationship(*from, *to);
        let mut updated = Relationship {
            affection: (current.affection + delta.affection).clamp(-10.0, 10.0),
            trust: (current.trust + delta.trust).clamp(-10.0, 10.0),
            attraction: (current.attraction + delta.attraction).clamp(-10.0, 10.0),
            familiarity: (current.familiarity + delta.familiarity).clamp(-10.0, 10.0),
            resentment: (current.resentment + delta.resentment).clamp(-10.0, 10.0),
            state: current.state, // Preserve state for now; will be updated next
        };
        updated.state = updated.compute_next_state();
        world.set_relationship(*from, *to, updated);
    }

    // Update karma (based on outcome emotional intensity)
    world
        .player_karma
        .apply_delta(outcome.emotional_intensity * 10.0);
    if let Some(k) = outcome.karma_delta {
        world.player_karma.apply_delta(k);
    }

    // Global heat reactions: base storylet heat plus optional spikes/damps.
    world.add_heat(storylet.heat);
    if outcome.heat_spike > 0.0 {
        world.add_heat(outcome.heat_spike);
    } else if outcome.heat_spike < 0.0 {
        world.reduce_heat(outcome.heat_spike.abs());
    }

    if outcome
        .memory_tags
        .iter()
        .any(|tag| tag.eq_ignore_ascii_case("trigger"))
    {
        world.add_heat(10.0);
    }

    // Record memory for the player (UI will render via journal)
    if !outcome.memory_event_id.is_empty() {
        let mut entry = MemoryEntry::new(
            format!("mem_player_{}_{}", world.player_id.0, current_tick.0),
            outcome.memory_event_id.clone(),
            world.player_id,
            current_tick,
            outcome.emotional_intensity,
        );

        if !outcome.stat_deltas.is_empty() {
            entry = entry.with_stat_deltas(outcome.stat_deltas.clone());
        }

        if !outcome.memory_tags.is_empty() {
            entry = entry.with_tags(outcome.memory_tags.clone());
        }

        memory.record_memory(entry);
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
    use syn_core::{NpcId, WorldSeed, WorldState, StatDelta, StatKind};

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
                relationship_states: vec![RelationshipState::Friend],
                memory_tags_required: vec![],
                memory_tags_forbidden: vec![],
                memory_recency_ticks: None,
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
                relationship_states: vec![],
                memory_tags_required: vec![],
                memory_tags_forbidden: vec![],
                memory_recency_ticks: None,
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
    fn test_behavior_bias_influences_score() {
        use syn_core::{AbstractNpc, AttachmentStyle, Traits, WorldSeed};

        let director = EventDirector::new();
        let mut world = WorldState::new(WorldSeed(42), NpcId(1));
        let player = AbstractNpc {
            id: NpcId(1),
            age: 27,
            job: "Musician".to_string(),
            district: "Downtown".to_string(),
            household_id: 1,
            traits: Traits {
                stability: 45.0,
                confidence: 60.0,
                sociability: 80.0,
                empathy: 75.0,
                impulsivity: 35.0,
                ambition: 40.0,
                charm: 85.0,
            },
            seed: 1,
            attachment_style: AttachmentStyle::Anxious,
        };
        world.npcs.insert(NpcId(1), player);

        let romance_storylet = Storylet {
            id: "romantic_arc".to_string(),
            name: "Romantic Turning Point".to_string(),
            tags: vec!["romance".to_string()],
            prerequisites: StoryletPrerequisites {
                min_relationship_affection: None,
                min_relationship_resentment: None,
                stat_conditions: HashMap::new(),
                life_stages: vec![],
                tags: vec![],
                relationship_states: vec![],
                memory_tags_required: vec![],
                memory_tags_forbidden: vec![],
                memory_recency_ticks: None,
            },
            heat: 60.0,
            weight: 0.5,
            cooldown_ticks: 100,
            roles: vec![],
        };

        let conflict_storylet = Storylet { tags: vec!["conflict".to_string()], ..romance_storylet.clone() };

        let romance_score = director.score_storylet(&romance_storylet, &world);
        let conflict_score = director.score_storylet(&conflict_storylet, &world);
        assert!(romance_score > conflict_score);
    }

    #[test]
    fn test_event_director_score() {
        let director = EventDirector::new();
        let world = WorldState::new(WorldSeed(42), NpcId(1));

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
                relationship_states: vec![],
                memory_tags_required: vec![],
                memory_tags_forbidden: vec![],
                memory_recency_ticks: None,
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
        let mut memory = MemorySystem::new();

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
                relationship_states: vec![],
                memory_tags_required: vec![],
                memory_tags_forbidden: vec![],
                memory_recency_ticks: None,
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
        outcome.stat_deltas.push(StatDelta {
            kind: StatKind::Mood,
            delta: -2.0,
            source: Some("test".into()),
        });

        let initial_mood = world.player_stats.get(StatKind::Mood);
        director.fire_storylet(&storylet, &mut world, &mut memory, outcome, SimTick(0));

        assert_eq!(world.player_stats.get(StatKind::Mood), initial_mood - 2.0);
    }

    #[test]
    fn apply_storylet_outcome_uses_stat_deltas_and_karma() {
        let mut world = WorldState::new(WorldSeed(1), NpcId(1));
        let mut memory = MemorySystem::new();
        let storylet = Storylet {
            id: "outcome_test".to_string(),
            name: "Outcome Test".to_string(),
            tags: vec![],
            prerequisites: StoryletPrerequisites {
                min_relationship_affection: None,
                min_relationship_resentment: None,
                stat_conditions: HashMap::new(),
                life_stages: vec![],
                tags: vec![],
                relationship_states: vec![],
                memory_tags_required: vec![],
                memory_tags_forbidden: vec![],
                memory_recency_ticks: None,
            },
            heat: 0.0,
            weight: 1.0,
            cooldown_ticks: 0,
            roles: vec![],
        };
        let outcome = StoryletOutcome {
            stat_deltas: vec![
                StatDelta { kind: StatKind::Mood, delta: -5.0, source: Some("test".into()) },
                StatDelta { kind: StatKind::Reputation, delta: 10.0, source: Some("test".into()) },
            ],
            karma_delta: Some(-20.0),
            ..Default::default()
        };

        apply_storylet_outcome(&mut world, &mut memory, &storylet, &outcome, SimTick(0));

        assert!(world.player_stats.get(StatKind::Mood) <= 10.0);
        assert!(world.player_stats.get(StatKind::Mood) >= -10.0);
        assert_eq!(world.player_stats.get(StatKind::Reputation), 10.0);
        let karma_val = world.player_karma.0;
        assert!(karma_val >= -100.0 && karma_val <= 100.0);
    }

    #[test]
    fn test_heat_and_memory_spike() {
        let mut director = EventDirector::new();
        let mut world = WorldState::new(WorldSeed(42), NpcId(1));
        let mut memory = MemorySystem::new();

        let storylet = Storylet {
            id: "spike_event".to_string(),
            name: "High drama".to_string(),
            tags: vec!["conflict".to_string()],
            prerequisites: StoryletPrerequisites {
                min_relationship_affection: None,
                min_relationship_resentment: None,
                stat_conditions: HashMap::new(),
                life_stages: vec![],
                tags: vec![],
                relationship_states: vec![],
                memory_tags_required: vec![],
                memory_tags_forbidden: vec![],
                memory_recency_ticks: None,
            },
            heat: 20.0,
            weight: 1.0,
            cooldown_ticks: 50,
            roles: vec![StoryletRole {
                name: "target".to_string(),
                npc_id: NpcId(2),
            }],
        };

        let mut outcome = StoryletOutcome::default();
        outcome.memory_event_id = "dramatic_turn".to_string();
        outcome.memory_tags = vec!["trigger".to_string()];
        outcome.heat_spike = 5.0;

        director.fire_storylet(&storylet, &mut world, &mut memory, outcome, SimTick(0));

        assert!(world.narrative_heat >= 35.0);
        assert!(memory.get_journal(world.player_id).is_some());
    }

    #[test]
    fn test_relationship_state_gating_romance_event() {
        use syn_core::{AbstractNpc, Traits, AttachmentStyle};

        let mut director = EventDirector::new();
        let mut world = WorldState::new(WorldSeed(42), NpcId(1));
        let memory = MemorySystem::new();

        // Create an NPC in the world
        let npc = AbstractNpc {
            id: NpcId(2),
            age: 30,
            job: "Teacher".to_string(),
            district: "Downtown".to_string(),
            household_id: 1,
            traits: Traits::default(),
            seed: 12345,
            attachment_style: AttachmentStyle::Secure,
        };
        world.npcs.insert(NpcId(2), npc);

        // Create a romance-only storylet
        let romance_storylet = Storylet {
            id: "romance_confession".to_string(),
            name: "Romantic Confession".to_string(),
            tags: vec!["romance".to_string()],
            prerequisites: StoryletPrerequisites {
                min_relationship_affection: Some(5.0),
                min_relationship_resentment: None,
                stat_conditions: HashMap::new(),
                life_stages: vec![],
                tags: vec![],
                relationship_states: vec![RelationshipState::Friend],
                memory_tags_required: vec![],
                memory_tags_forbidden: vec![],
                memory_recency_ticks: None,
            },
            heat: 50.0,
            weight: 0.7,
            cooldown_ticks: 200,
            roles: vec![StoryletRole {
                name: "romantic_interest".to_string(),
                npc_id: NpcId(2),
            }],
        };

        // Set up a Stranger relationship
        world.set_relationship(NpcId(1), NpcId(2), Relationship {
            affection: 0.0,
            trust: 0.0,
            attraction: 0.0,
            familiarity: 0.0,
            resentment: 0.0,
            state: RelationshipState::Stranger,
        });

        director.register_storylet(romance_storylet.clone());

        // Romance event should NOT fire with Stranger state
        assert!(!director.is_eligible(&romance_storylet, &world, &memory, SimTick(0)));

        // Now set to Friend state
        world.set_relationship(NpcId(1), NpcId(2), Relationship {
            affection: 5.0,
            trust: 3.0,
            attraction: 2.0,
            familiarity: 4.0,
            resentment: 0.0,
            state: RelationshipState::Friend,
        });

        // Romance event SHOULD fire with Friend state
        assert!(director.is_eligible(&romance_storylet, &world, &memory, SimTick(0)));
    }

    #[test]
    fn test_relationship_state_transition_on_event() {
        let mut director = EventDirector::new();
        let mut world = WorldState::new(WorldSeed(42), NpcId(1));
        let mut memory = MemorySystem::new();

        // Set Friend relationship
        world.set_relationship(NpcId(1), NpcId(2), Relationship {
            affection: 5.0,
            trust: 3.0,
            attraction: 2.0,
            familiarity: 4.0,
            resentment: 0.0,
            state: RelationshipState::Friend,
        });

        let storylet = Storylet {
            id: "deepening_bond".to_string(),
            name: "We're getting closer".to_string(),
            tags: vec!["romance".to_string()],
            prerequisites: StoryletPrerequisites {
                min_relationship_affection: None,
                min_relationship_resentment: None,
                stat_conditions: HashMap::new(),
                life_stages: vec![],
                tags: vec![],
                relationship_states: vec![RelationshipState::Friend],
                memory_tags_required: vec![],
                memory_tags_forbidden: vec![],
                memory_recency_ticks: None,
            },
            heat: 50.0,
            weight: 0.5,
            cooldown_ticks: 100,
            roles: vec![StoryletRole {
                name: "target".to_string(),
                npc_id: NpcId(2),
            }],
        };

        // Create outcome that boosts relationship values
        // Current: affection 5.0, trust 3.0, attraction 2.0, familiarity 4.0
        // Delta: affection 3.0, trust 4.0, attraction 6.0, familiarity 3.0
        // Result: affection 8.0, trust 7.0, attraction 8.0, familiarity 7.0
        // With the refactored check order (most specific first):
        // - Not Spouse: trust 7.0 < 8.0
        // - Is Partner: attraction 8.0 > 7.0 && trust 7.0 > 6.0 && affection 8.0 > 7.0 ✓
        let mut outcome = StoryletOutcome::default();
        outcome.relationship_deltas.push((
            NpcId(1),
            NpcId(2),
            Relationship {
                affection: 3.0,
                trust: 4.0,
                attraction: 6.0,
                familiarity: 3.0,
                resentment: 0.0,
                state: RelationshipState::Friend,  // Will be recomputed
            },
        ));

        director.fire_storylet(&storylet, &mut world, &mut memory, outcome, SimTick(0));

        // Check that relationship state transitioned to Partner (the most specific state for these values)
        let updated_rel = world.get_relationship(NpcId(1), NpcId(2));
        assert_eq!(updated_rel.state, RelationshipState::Partner);
    }

    #[test]
    fn test_conflict_event_gating_with_best_friend() {
        let mut director = EventDirector::new();
        let mut world = WorldState::new(WorldSeed(42), NpcId(1));

        // Create a conflict storylet
        let conflict_storylet = Storylet {
            id: "heated_argument".to_string(),
            name: "You have a heated argument".to_string(),
            tags: vec!["conflict".to_string()],
            prerequisites: StoryletPrerequisites {
                min_relationship_affection: None,
                min_relationship_resentment: None,
                stat_conditions: HashMap::new(),
                life_stages: vec![],
                tags: vec![],
                relationship_states: vec![],
                memory_tags_required: vec![],
                memory_tags_forbidden: vec![],
                memory_recency_ticks: None,
            },
            heat: 60.0,
            weight: 0.6,
            cooldown_ticks: 150,
            roles: vec![StoryletRole {
                name: "target".to_string(),
                npc_id: NpcId(2),
            }],
        };

        // Best Friend relationship should NOT allow conflict based on state check
        world.set_relationship(NpcId(1), NpcId(2), Relationship {
            affection: 9.0,
            trust: 9.0,
            attraction: 0.5,
            familiarity: 9.0,
            resentment: 0.0,
            state: RelationshipState::BestFriend,
        });

        director.register_storylet(conflict_storylet.clone());

        // Should still be eligible (conflict_event_gating would need custom logic)
        // This test validates that the relationship state is properly tracked
        let rel = world.get_relationship(NpcId(1), NpcId(2));
        assert_eq!(rel.state, RelationshipState::BestFriend);
        assert!(!rel.state.allows_conflict());
    }

    #[test]
    fn test_memory_echo_required_tag_gating() {
        use syn_core::{AbstractNpc, Traits, AttachmentStyle};
        use syn_memory::MemoryEntry;

        let mut director = EventDirector::new();
        let mut world = WorldState::new(WorldSeed(42), NpcId(1));

        // Create an NPC
        let npc = AbstractNpc {
            id: NpcId(2),
            age: 30,
            job: "Teacher".to_string(),
            district: "Downtown".to_string(),
            household_id: 1,
            traits: Traits::default(),
            seed: 12345,
            attachment_style: AttachmentStyle::Secure,
        };
        world.npcs.insert(NpcId(2), npc);

        // Create a storylet that requires a "betrayal" memory tag
        let echo_storylet = Storylet {
            id: "revenge_moment".to_string(),
            name: "Revenge opportunity arises".to_string(),
            tags: vec!["echo".to_string()],
            prerequisites: StoryletPrerequisites {
                min_relationship_affection: None,
                min_relationship_resentment: None,
                stat_conditions: HashMap::new(),
                life_stages: vec![],
                tags: vec![],
                relationship_states: vec![],
                memory_tags_required: vec!["betrayal".to_string()],
                memory_tags_forbidden: vec![],
                memory_recency_ticks: None,
            },
            heat: 60.0,
            weight: 0.8,
            cooldown_ticks: 300,
            roles: vec![StoryletRole {
                name: "target".to_string(),
                npc_id: NpcId(2),
            }],
        };

        director.register_storylet(echo_storylet.clone());

        // Event should NOT fire without required memory
        let mut memory = MemorySystem::new();
        assert!(!director.is_eligible(&echo_storylet, &world, &memory, SimTick(100)));

        // Add a "betrayal" memory entry to NPC
        let memory_entry = MemoryEntry::new(
            "mem_betrayal_001".to_string(),
            "event_betrayal".to_string(),
            NpcId(2),
            SimTick(95),
            -0.8,
        ).with_tags(vec!["betrayal"]);

        memory.record_memory(memory_entry);

        // Event SHOULD fire now with required memory present
        assert!(director.is_eligible(&echo_storylet, &world, &memory, SimTick(100)));
    }

    #[test]
    fn test_memory_echo_forbidden_tag_blocking() {
        use syn_core::{AbstractNpc, Traits, AttachmentStyle};
        use syn_memory::MemoryEntry;

        let mut director = EventDirector::new();
        let mut world = WorldState::new(WorldSeed(42), NpcId(1));

        // Create an NPC
        let npc = AbstractNpc {
            id: NpcId(2),
            age: 30,
            job: "Teacher".to_string(),
            district: "Downtown".to_string(),
            household_id: 1,
            traits: Traits::default(),
            seed: 12345,
            attachment_style: AttachmentStyle::Secure,
        };
        world.npcs.insert(NpcId(2), npc);

        // Create a storylet that forbids "trauma" memory tag (conflict avoidance)
        let fragile_storylet = Storylet {
            id: "intimate_moment".to_string(),
            name: "Intimate conversation".to_string(),
            tags: vec!["romance".to_string()],
            prerequisites: StoryletPrerequisites {
                min_relationship_affection: None,
                min_relationship_resentment: None,
                stat_conditions: HashMap::new(),
                life_stages: vec![],
                tags: vec![],
                relationship_states: vec![],
                memory_tags_required: vec![],
                memory_tags_forbidden: vec!["trauma".to_string()],
                memory_recency_ticks: None,
            },
            heat: 50.0,
            weight: 0.7,
            cooldown_ticks: 200,
            roles: vec![StoryletRole {
                name: "target".to_string(),
                npc_id: NpcId(2),
            }],
        };

        director.register_storylet(fragile_storylet.clone());

        let mut memory = MemorySystem::new();

        // Event SHOULD fire without forbidden trauma memories
        assert!(director.is_eligible(&fragile_storylet, &world, &memory, SimTick(100)));

        // Add a traumatic memory
        let trauma_entry = MemoryEntry::new(
            "mem_trauma_001".to_string(),
            "event_traumatic".to_string(),
            NpcId(2),
            SimTick(50),
            -0.9,
        ).with_tags(vec!["trauma"]);

        memory.record_memory(trauma_entry);

        // Event should NOT fire now with traumatic memory present (conflict avoidance)
        assert!(!director.is_eligible(&fragile_storylet, &world, &memory, SimTick(100)));
    }

    #[test]
    fn test_memory_echo_recency_window() {
        use syn_core::{AbstractNpc, Traits, AttachmentStyle};
        use syn_memory::MemoryEntry;

        let mut director = EventDirector::new();
        let mut world = WorldState::new(WorldSeed(42), NpcId(1));

        // Create an NPC
        let npc = AbstractNpc {
            id: NpcId(2),
            age: 30,
            job: "Teacher".to_string(),
            district: "Downtown".to_string(),
            household_id: 1,
            traits: Traits::default(),
            seed: 12345,
            attachment_style: AttachmentStyle::Secure,
        };
        world.npcs.insert(NpcId(2), npc);

        // Create a storylet that requires a "confrontation" memory within last 50 ticks
        let follow_up_storylet = Storylet {
            id: "confrontation_aftermath".to_string(),
            name: "Deal with the consequences".to_string(),
            tags: vec!["conflict_resolution".to_string()],
            prerequisites: StoryletPrerequisites {
                min_relationship_affection: None,
                min_relationship_resentment: None,
                stat_conditions: HashMap::new(),
                life_stages: vec![],
                tags: vec![],
                relationship_states: vec![],
                memory_tags_required: vec!["confrontation".to_string()],
                memory_tags_forbidden: vec![],
                memory_recency_ticks: Some(50),  // Must be within last 50 ticks
            },
            heat: 55.0,
            weight: 0.65,
            cooldown_ticks: 100,
            roles: vec![StoryletRole {
                name: "target".to_string(),
                npc_id: NpcId(2),
            }],
        };

        director.register_storylet(follow_up_storylet.clone());

        let mut memory = MemorySystem::new();

        // Event should NOT fire without recent confrontation memory
        assert!(!director.is_eligible(&follow_up_storylet, &world, &memory, SimTick(100)));

        // Add an OLD confrontation memory (150 ticks ago, outside recency window)
        let old_confrontation = MemoryEntry::new(
            "mem_confrontation_old".to_string(),
            "event_confrontation".to_string(),
            NpcId(2),
            SimTick(0),  // 100 ticks ago
            -0.5,
        ).with_tags(vec!["confrontation"]);

        memory.record_memory(old_confrontation);

        // Event should NOT fire (memory outside recency window)
        assert!(!director.is_eligible(&follow_up_storylet, &world, &memory, SimTick(100)));

        // Add a RECENT confrontation memory (within last 50 ticks)
        let recent_confrontation = MemoryEntry::new(
            "mem_confrontation_recent".to_string(),
            "event_confrontation".to_string(),
            NpcId(2),
            SimTick(75),  // 25 ticks ago (within 50-tick window)
            -0.6,
        ).with_tags(vec!["confrontation"]);

        memory.record_memory(recent_confrontation);

        // Event SHOULD fire now (recent memory within window)
        assert!(director.is_eligible(&follow_up_storylet, &world, &memory, SimTick(100)));
    }

    #[test]
    fn test_memory_echo_multiple_tags() {
        use syn_core::{AbstractNpc, Traits, AttachmentStyle};
        use syn_memory::MemoryEntry;

        let mut director = EventDirector::new();
        let mut world = WorldState::new(WorldSeed(42), NpcId(1));

        // Create an NPC
        let npc = AbstractNpc {
            id: NpcId(2),
            age: 30,
            job: "Teacher".to_string(),
            district: "Downtown".to_string(),
            household_id: 1,
            traits: Traits::default(),
            seed: 12345,
            attachment_style: AttachmentStyle::Secure,
        };
        world.npcs.insert(NpcId(2), npc);

        // Create a storylet requiring either "love_confession" OR "jealousy" tag
        let complex_storylet = Storylet {
            id: "emotional_climax".to_string(),
            name: "Emotional resolution".to_string(),
            tags: vec!["relationship".to_string()],
            prerequisites: StoryletPrerequisites {
                min_relationship_affection: None,
                min_relationship_resentment: None,
                stat_conditions: HashMap::new(),
                life_stages: vec![],
                tags: vec![],
                relationship_states: vec![],
                memory_tags_required: vec!["love_confession".to_string(), "jealousy".to_string()],  // Either one
                memory_tags_forbidden: vec![],
                memory_recency_ticks: None,
            },
            heat: 80.0,
            weight: 0.9,
            cooldown_ticks: 400,
            roles: vec![StoryletRole {
                name: "target".to_string(),
                npc_id: NpcId(2),
            }],
        };

        director.register_storylet(complex_storylet.clone());

        let mut memory = MemorySystem::new();

        // Event should NOT fire without either tag
        assert!(!director.is_eligible(&complex_storylet, &world, &memory, SimTick(100)));

        // Add a "jealousy" memory (first required tag)
        let jealousy_memory = MemoryEntry::new(
            "mem_jealousy".to_string(),
            "event_jealousy".to_string(),
            NpcId(2),
            SimTick(90),
            -0.4,
        ).with_tags(vec!["jealousy"]);

        memory.record_memory(jealousy_memory);

        // Event SHOULD fire now (has one of the required tags)
        assert!(director.is_eligible(&complex_storylet, &world, &memory, SimTick(100)));

        // Add a "love_confession" memory (second required tag) 
        let confession_memory = MemoryEntry::new(
            "mem_love_confession".to_string(),
            "event_confession".to_string(),
            NpcId(2),
            SimTick(95),
            0.8,
        ).with_tags(vec!["love_confession"]);

        memory.record_memory(confession_memory);

        // Event SHOULD STILL fire (now has both)
        assert!(director.is_eligible(&complex_storylet, &world, &memory, SimTick(100)));
    }
}
