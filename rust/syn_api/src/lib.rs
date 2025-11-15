//! syn_api: FFI aggregation layer for Flutter via flutter_rust_bridge.
//!
//! Exposes the entire SYN simulation engine to Flutter through a typed, async-friendly API.
//! This is the "public interface" of the Rust backend.

use flutter_rust_bridge::frb;
use std::sync::Mutex;

// Re-export core types for Dart
pub use syn_core::{
    AbstractNpc, AttachmentStyle, Karma, LifeStage, NpcId, Relationship, SimTick, Stats,
    Traits, WorldSeed, WorldState,
};
pub use syn_director::{EventDirector, Storylet, StoryletOutcome, StoryletRole};
pub use syn_memory::{Journal, MemoryEntry, MemorySystem};
pub use syn_sim::{LodTier, Simulator};
pub use syn_query::{ClusterQuery, NpcQuery, RelationshipQuery, StatQuery};

/// Global game engine state (wrapped in Mutex for thread safety).
pub struct GameEngine {
    world: WorldState,
    simulator: Simulator,
    director: EventDirector,
    memory: MemorySystem,
}

impl GameEngine {
    pub fn new(seed: u64) -> Self {
        let world_seed = WorldSeed::new(seed);
        let player_id = NpcId(1);
        let world = WorldState::new(world_seed, player_id);

        GameEngine {
            world,
            simulator: Simulator::new(seed),
            director: EventDirector::new(),
            memory: MemorySystem::new(),
        }
    }

    // ==================== World Management ====================

    /// Get current world seed.
    pub fn world_seed(&self) -> u64 {
        self.world.seed.0
    }

    /// Get current simulation tick.
    pub fn current_tick(&self) -> u64 {
        self.world.current_tick.0
    }

    /// Get player age.
    pub fn player_age(&self) -> u32 {
        self.world.player_age
    }

    /// Get player life stage.
    pub fn player_life_stage(&self) -> String {
        format!("{:?}", self.world.player_life_stage)
    }

    /// Get player karma.
    pub fn player_karma(&self) -> f32 {
        self.world.player_karma.0
    }

    /// Get player stats (serialized for Dart).
    pub fn player_stats(&self) -> PlayerStatsDto {
        PlayerStatsDto {
            health: self.world.player_stats.health,
            intelligence: self.world.player_stats.intelligence,
            charisma: self.world.player_stats.charisma,
            wealth: self.world.player_stats.wealth,
            mood: self.world.player_stats.mood,
            appearance: self.world.player_stats.appearance,
            reputation: self.world.player_stats.reputation,
            wisdom: self.world.player_stats.wisdom,
        }
    }

    // ==================== Simulation ====================

    /// Advance the simulation by one tick.
    pub fn tick(&mut self) {
        self.simulator.tick(&mut self.world);
    }

    /// Advance the simulation by N ticks.
    pub fn tick_many(&mut self, count: u32) {
        for _ in 0..count {
            self.simulator.tick(&mut self.world);
        }
    }

    /// Get LOD tier counts (high, medium, low).
    pub fn lod_counts(&self) -> (u32, u32, u32) {
        let (h, m, l) = self.simulator.count_by_lod();
        (h as u32, m as u32, l as u32)
    }

    // ==================== NPC Management ====================

    /// Register an NPC in the world.
    pub fn register_npc(&mut self, npc_id: u64, age: u32, job: String, district: String) {
        let npc = AbstractNpc {
            id: NpcId(npc_id),
            age,
            job,
            district,
            household_id: 1,
            traits: Traits::default(),
            seed: npc_id,
            attachment_style: AttachmentStyle::Secure,
        };
        self.world.npcs.insert(NpcId(npc_id), npc.clone());
        self.simulator.instantiate_npc(npc);
    }

    /// Get NPC by ID.
    pub fn get_npc(&self, npc_id: u64) -> Option<NpcDto> {
        self.world.npcs.get(&NpcId(npc_id)).map(|npc| NpcDto {
            id: npc.id.0,
            age: npc.age,
            job: npc.job.clone(),
            district: npc.district.clone(),
        })
    }

    /// List all NPCs in the world.
    pub fn list_npcs(&self) -> Vec<u64> {
        self.world.npcs.keys().map(|id| id.0).collect()
    }

    // ==================== Relationships ====================

    /// Set a relationship between two NPCs.
    pub fn set_relationship(
        &mut self,
        from_npc_id: u64,
        to_npc_id: u64,
        affection: f32,
        trust: f32,
        attraction: f32,
        familiarity: f32,
        resentment: f32,
    ) {
        let rel = Relationship {
            affection: affection.clamp(-10.0, 10.0),
            trust: trust.clamp(-10.0, 10.0),
            attraction: attraction.clamp(-10.0, 10.0),
            familiarity: familiarity.clamp(-10.0, 10.0),
            resentment: resentment.clamp(-10.0, 10.0),
        };
        self.world
            .set_relationship(NpcId(from_npc_id), NpcId(to_npc_id), rel);
    }

    /// Get a relationship between two NPCs.
    pub fn get_relationship(&self, from_npc_id: u64, to_npc_id: u64) -> RelationshipDto {
        let rel = self.world.get_relationship(NpcId(from_npc_id), NpcId(to_npc_id));
        RelationshipDto {
            affection: rel.affection,
            trust: rel.trust,
            attraction: rel.attraction,
            familiarity: rel.familiarity,
            resentment: rel.resentment,
            heat: rel.heat(),
        }
    }

    // ==================== Memory ====================

    /// Record a memory for an NPC.
    pub fn record_memory(
        &mut self,
        npc_id: u64,
        event_id: String,
        emotional_intensity: f32,
    ) -> String {
        let memory_id = format!("mem_{}_{}", npc_id, self.world.current_tick.0);
        let entry = MemoryEntry::new(
            memory_id.clone(),
            event_id,
            NpcId(npc_id),
            self.world.current_tick,
            emotional_intensity.clamp(-1.0, 1.0),
        );
        self.memory.record_memory(entry);
        memory_id
    }

    /// Get memories for an NPC.
    pub fn get_npc_memories(&self, npc_id: u64) -> Vec<MemoryDto> {
        self.memory
            .get_journal(NpcId(npc_id))
            .map(|journal| {
                journal
                    .timeline()
                    .iter()
                    .map(|entry| MemoryDto {
                        id: entry.id.clone(),
                        event_id: entry.event_id.clone(),
                        emotional_intensity: entry.emotional_intensity,
                        sim_tick: entry.sim_tick.0,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    // ==================== Events ====================

    /// Register a storylet with the Event Director.
    pub fn register_storylet(&mut self, storylet_id: String, name: String, heat: f32, weight: f32) {
        let storylet = Storylet {
            id: storylet_id,
            name,
            tags: vec![],
            prerequisites: syn_director::StoryletPrerequisites {
                min_relationship_affection: None,
                min_relationship_resentment: None,
                stat_conditions: std::collections::HashMap::new(),
                life_stages: vec![],
                tags: vec![],
            },
            heat,
            weight,
            cooldown_ticks: 100,
            roles: vec![],
        };
        self.director.register_storylet(storylet);
    }

    /// Select and return the next eligible event.
    pub fn select_next_event(&self) -> Option<EventDto> {
        self.director
            .select_next_event(&self.world, self.world.current_tick)
            .map(|s| EventDto {
                id: s.id.clone(),
                name: s.name.clone(),
                heat: s.heat,
            })
    }
}

// ==================== Data Transfer Objects (DTOs) for Dart ====================

/// Player stats DTO for serialization to Dart.
#[derive(Debug, Clone)]
pub struct PlayerStatsDto {
    pub health: f32,
    pub intelligence: f32,
    pub charisma: f32,
    pub wealth: f32,
    pub mood: f32,
    pub appearance: f32,
    pub reputation: f32,
    pub wisdom: f32,
}

/// NPC DTO for serialization to Dart.
#[derive(Debug, Clone)]
pub struct NpcDto {
    pub id: u64,
    pub age: u32,
    pub job: String,
    pub district: String,
}

/// Relationship DTO for serialization to Dart.
#[derive(Debug, Clone)]
pub struct RelationshipDto {
    pub affection: f32,
    pub trust: f32,
    pub attraction: f32,
    pub familiarity: f32,
    pub resentment: f32,
    pub heat: f32,
}

/// Memory DTO for serialization to Dart.
#[derive(Debug, Clone)]
pub struct MemoryDto {
    pub id: String,
    pub event_id: String,
    pub emotional_intensity: f32,
    pub sim_tick: u64,
}

/// Event DTO for serialization to Dart.
#[derive(Debug, Clone)]
pub struct EventDto {
    pub id: String,
    pub name: String,
    pub heat: f32,
}

// ==================== Frb Wrapper (Async Support) ====================

/// Global engine instance (protected by Mutex for thread safety).
static ENGINE: Mutex<Option<GameEngine>> = Mutex::new(None);

/// Initialize the game engine.
#[frb(sync)]
pub fn init_engine(seed: u64) {
    let mut engine = ENGINE.lock().unwrap();
    *engine = Some(GameEngine::new(seed));
}

/// Tick the engine (thread-safe).
#[frb(sync)]
pub fn engine_tick() {
    let mut engine = ENGINE.lock().unwrap();
    if let Some(ref mut e) = *engine {
        e.tick();
    }
}

/// Get player age.
#[frb(sync)]
pub fn engine_player_age() -> u32 {
    let engine = ENGINE.lock().unwrap();
    engine.as_ref().map(|e| e.player_age()).unwrap_or(0)
}

/// Get player mood.
#[frb(sync)]
pub fn engine_player_mood() -> f32 {
    let engine = ENGINE.lock().unwrap();
    engine
        .as_ref()
        .map(|e| e.player_stats().mood)
        .unwrap_or(0.0)
}

/// Get all NPC IDs.
#[frb(sync)]
pub fn engine_list_npcs() -> Vec<u64> {
    let engine = ENGINE.lock().unwrap();
    engine.as_ref().map(|e| e.list_npcs()).unwrap_or_default()
}

/// Register an NPC.
#[frb(sync)]
pub fn engine_register_npc(npc_id: u64, age: u32, job: String, district: String) {
    let mut engine = ENGINE.lock().unwrap();
    if let Some(ref mut e) = *engine {
        e.register_npc(npc_id, age, job, district);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = GameEngine::new(42);
        assert_eq!(engine.world_seed(), 42);
    }

    #[test]
    fn test_engine_tick() {
        let mut engine = GameEngine::new(42);
        let initial_tick = engine.current_tick();
        engine.tick();
        assert_eq!(engine.current_tick(), initial_tick + 1);
    }

    #[test]
    fn test_register_npc() {
        let mut engine = GameEngine::new(42);
        engine.register_npc(2, 25, "Engineer".to_string(), "Downtown".to_string());
        assert!(engine.get_npc(2).is_some());
    }

    #[test]
    fn test_relationship() {
        let mut engine = GameEngine::new(42);
        engine.set_relationship(1, 2, 5.0, 3.0, 2.0, 1.0, 0.0);
        let rel = engine.get_relationship(1, 2);
        assert_eq!(rel.affection, 5.0);
    }

    #[test]
    fn test_memory() {
        let mut engine = GameEngine::new(42);
        let mem_id = engine.record_memory(1, "event_test".to_string(), 0.8);
        assert!(!mem_id.is_empty());
    }
}
