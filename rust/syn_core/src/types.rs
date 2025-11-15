//! Core types: Stats, Traits, Relationships, NPCs, World state.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for a world seed (ensures determinism).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorldSeed(pub u64);

impl WorldSeed {
    pub fn new(seed: u64) -> Self {
        WorldSeed(seed)
    }
}

/// Visible player stats by life stage.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Stats {
    pub health: f32,
    pub intelligence: f32,
    pub charisma: f32,
    pub wealth: f32,
    pub mood: f32,
    pub appearance: f32,
    pub reputation: f32,
    pub wisdom: f32,
    /// Child-exclusive
    pub curiosity: Option<f32>,
    /// Child-exclusive
    pub energy: Option<f32>,
    /// Teen+ NSFW mode
    pub libido: Option<f32>,
}

impl Default for Stats {
    fn default() -> Self {
        Stats {
            health: 50.0,
            intelligence: 50.0,
            charisma: 50.0,
            wealth: 50.0,
            mood: 0.0,
            appearance: 50.0,
            reputation: 0.0,
            wisdom: 20.0,
            curiosity: Some(50.0),
            energy: Some(50.0),
            libido: None,
        }
    }
}

impl Stats {
    /// Clamp all stats to valid range [0..100] except mood [-10..10].
    pub fn clamp(&mut self) {
        self.health = self.health.clamp(0.0, 100.0);
        self.intelligence = self.intelligence.clamp(0.0, 100.0);
        self.charisma = self.charisma.clamp(0.0, 100.0);
        self.wealth = self.wealth.clamp(0.0, 100.0);
        self.mood = self.mood.clamp(-10.0, 10.0);
        self.appearance = self.appearance.clamp(0.0, 100.0);
        self.reputation = self.reputation.clamp(-100.0, 100.0);
        self.wisdom = self.wisdom.clamp(0.0, 100.0);

        if let Some(ref mut c) = self.curiosity {
            *c = c.clamp(0.0, 100.0);
        }
        if let Some(ref mut e) = self.energy {
            *e = e.clamp(0.0, 100.0);
        }
        if let Some(ref mut l) = self.libido {
            *l = l.clamp(0.0, 100.0);
        }
    }
}

/// Permanent personality trait dimensions (set at NPC generation, rarely change).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Traits {
    pub stability: f32,         // calm ↔ volatile
    pub confidence: f32,        // insecure ↔ self-assured
    pub sociability: f32,       // introverted ↔ extroverted
    pub empathy: f32,           // detached ↔ sensitive
    pub impulsivity: f32,       // cautious ↔ reckless
    pub ambition: f32,          // apathetic ↔ driven
    pub charm: f32,             // awkward ↔ charismatic
}

impl Default for Traits {
    fn default() -> Self {
        Traits {
            stability: 50.0,
            confidence: 50.0,
            sociability: 50.0,
            empathy: 50.0,
            impulsivity: 50.0,
            ambition: 50.0,
            charm: 50.0,
        }
    }
}

impl Traits {
    /// Clamp all traits to [0..100].
    pub fn clamp(&mut self) {
        self.stability = self.stability.clamp(0.0, 100.0);
        self.confidence = self.confidence.clamp(0.0, 100.0);
        self.sociability = self.sociability.clamp(0.0, 100.0);
        self.empathy = self.empathy.clamp(0.0, 100.0);
        self.impulsivity = self.impulsivity.clamp(0.0, 100.0);
        self.ambition = self.ambition.clamp(0.0, 100.0);
        self.charm = self.charm.clamp(0.0, 100.0);
    }
}

/// 5-axis relationship vector between two NPCs.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Relationship {
    pub affection: f32,     // -10..+10 (warmth, emotional closeness)
    pub trust: f32,         // -10..+10 (reliability, safety, openness)
    pub attraction: f32,    // -10..+10 (romantic/sexual pull)
    pub familiarity: f32,   // -10..+10 (shared time, history, routine)
    pub resentment: f32,    // -10..+10 (hostility, grudges)
}

impl Default for Relationship {
    fn default() -> Self {
        Relationship {
            affection: 0.0,
            trust: 0.0,
            attraction: 0.0,
            familiarity: 0.0,
            resentment: 0.0,
        }
    }
}

impl Relationship {
    /// Clamp all axes to [-10..+10].
    pub fn clamp(&mut self) {
        self.affection = self.affection.clamp(-10.0, 10.0);
        self.trust = self.trust.clamp(-10.0, 10.0);
        self.attraction = self.attraction.clamp(-10.0, 10.0);
        self.familiarity = self.familiarity.clamp(-10.0, 10.0);
        self.resentment = self.resentment.clamp(-10.0, 10.0);
    }

    /// Calculate relationship "heat" (0..1 scale) based on axes.
    /// High heat = high intensity (emotional or conflictual).
    pub fn heat(&self) -> f32 {
        (self.affection.abs() + self.trust.abs() + self.resentment.abs()) / 30.0
    }
}

/// Life stage of a character (affects visible stats and event eligibility).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LifeStage {
    PreSim,      // 0-5 (not playable; used for generation)
    Child,       // 6-12
    Teen,        // 13-18
    YoungAdult,  // 19-29
    Adult,       // 30-59
    Elder,       // 60-89
    Digital,     // 90+
}

impl LifeStage {
    pub fn from_age(age: u32) -> Self {
        match age {
            0..=5 => LifeStage::PreSim,
            6..=12 => LifeStage::Child,
            13..=18 => LifeStage::Teen,
            19..=29 => LifeStage::YoungAdult,
            30..=59 => LifeStage::Adult,
            60..=89 => LifeStage::Elder,
            _ => LifeStage::Digital,
        }
    }

    pub fn age_range(&self) -> (u32, u32) {
        match self {
            LifeStage::PreSim => (0, 5),
            LifeStage::Child => (6, 12),
            LifeStage::Teen => (13, 18),
            LifeStage::YoungAdult => (19, 29),
            LifeStage::Adult => (30, 59),
            LifeStage::Elder => (60, 89),
            LifeStage::Digital => (90, 999),
        }
    }
}

/// Attachment style (affects social trait modifiers).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AttachmentStyle {
    Anxious,
    Avoidant,
    Secure,
}

impl Default for AttachmentStyle {
    fn default() -> Self {
        AttachmentStyle::Secure
    }
}

/// NPC identifier (unique within a world).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NpcId(pub u64);

impl NpcId {
    pub fn new(id: u64) -> Self {
        NpcId(id)
    }
}

/// Lightweight NPC (stored in PopulationStore, not yet instantiated in ECS).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbstractNpc {
    pub id: NpcId,
    pub age: u32,
    pub job: String,
    pub district: String,
    pub household_id: u64,
    pub traits: Traits,
    pub seed: u64,
    pub attachment_style: AttachmentStyle,
}

/// Timestamp (in simulation ticks, deterministic).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SimTick(pub u64);

impl SimTick {
    pub fn new(tick: u64) -> Self {
        SimTick(tick)
    }

    pub fn days_elapsed(&self) -> u32 {
        (self.0 / 24) as u32 // Assume 24 ticks per day
    }
}

/// Karma: accumulated ethical field (-100..+100).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Karma(pub f32);

impl Karma {
    pub fn new() -> Self {
        Karma(0.0)
    }

    pub fn clamp(&mut self) {
        self.0 = self.0.clamp(-100.0, 100.0);
    }
}

impl Default for Karma {
    fn default() -> Self {
        Karma(0.0)
    }
}

/// Global world state snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldState {
    pub seed: WorldSeed,
    pub current_tick: SimTick,
    pub player_id: NpcId,
    pub player_stats: Stats,
    pub player_age: u32,
    pub player_life_stage: LifeStage,
    pub player_karma: Karma,
    /// Relationship storage: (npc_id, other_id) → Relationship
    pub relationships: HashMap<(NpcId, NpcId), Relationship>,
    /// NPC population cache
    pub npcs: HashMap<NpcId, AbstractNpc>,
}

impl WorldState {
    pub fn new(seed: WorldSeed, player_id: NpcId) -> Self {
        WorldState {
            seed,
            current_tick: SimTick(0),
            player_id,
            player_stats: Stats::default(),
            player_age: 6, // Start at age 6
            player_life_stage: LifeStage::Child,
            player_karma: Karma::default(),
            relationships: HashMap::new(),
            npcs: HashMap::new(),
        }
    }

    /// Get or initialize relationship between two NPCs.
    pub fn get_relationship(&self, from: NpcId, to: NpcId) -> Relationship {
        self.relationships
            .get(&(from, to))
            .copied()
            .unwrap_or_default()
    }

    /// Update relationship between two NPCs.
    pub fn set_relationship(&mut self, from: NpcId, to: NpcId, rel: Relationship) {
        self.relationships.insert((from, to), rel);
    }

    /// Advance world by one tick.
    pub fn tick(&mut self) {
        self.current_tick.0 += 1;
        // Age player every 24 ticks (1 simulated day)
        if self.current_tick.0 % 24 == 0 {
            self.player_age += 1;
            self.player_life_stage = LifeStage::from_age(self.player_age);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_clamp() {
        let mut stats = Stats::default();
        stats.health = 150.0;
        stats.mood = 15.0;
        stats.clamp();
        assert_eq!(stats.health, 100.0);
        assert_eq!(stats.mood, 10.0);
    }

    #[test]
    fn test_relationship_heat() {
        let rel = Relationship {
            affection: 5.0,
            trust: 5.0,
            resentment: 5.0,
            ..Default::default()
        };
        let heat = rel.heat();
        assert!(heat > 0.0 && heat < 1.0);
    }

    #[test]
    fn test_life_stage_from_age() {
        assert_eq!(LifeStage::from_age(10), LifeStage::Child);
        assert_eq!(LifeStage::from_age(15), LifeStage::Teen);
        assert_eq!(LifeStage::from_age(25), LifeStage::YoungAdult);
    }

    #[test]
    fn test_world_state_tick() {
        let mut world = WorldState::new(WorldSeed(42), NpcId(1));
        assert_eq!(world.player_age, 6);
        for _ in 0..24 {
            world.tick();
        }
        assert_eq!(world.player_age, 7);
    }
}
