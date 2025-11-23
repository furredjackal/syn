//! syn_api: FFI aggregation layer for Flutter via flutter_rust_bridge.
//!
//! Exposes the entire SYN simulation engine to Flutter through a typed, async-friendly API.
//! This is the "public interface" of the Rust backend.

use flutter_rust_bridge::frb;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use syn_content::load_storylets_from_db;
use syn_core::relationship_model::{derive_role_label, RelationshipVector};
use syn_director::{
    apply_choice_and_advance, select_next_event_view, DirectorEventView, StoryletLibrary,
};
use syn_sim::SimState;

// Re-export core types for Dart
pub use syn_core::{
    AbstractNpc, AttachmentStyle, Karma, KarmaBand, LifeStage, MoodBand, NpcId, Relationship,
    SimTick, StatKind, Stats, Traits, WorldSeed, WorldState, ALL_STAT_KINDS,
};
pub use syn_director::{
    EventDirector, Storylet, StoryletChoice, StoryletLibrary, StoryletOutcome, StoryletRole,
};
pub use syn_memory::{Journal, MemoryEntry, MemorySystem};
pub use syn_query::{ClusterQuery, NpcQuery, RelationshipQuery, StatQuery};
pub use syn_sim::{LodTier, Simulator};

/// Global game engine state (wrapped in Mutex for thread safety).
pub struct GameEngine {
    world: WorldState,
    simulator: Simulator,
    director: EventDirector,
    memory: MemorySystem,
}

pub struct GameRuntime {
    pub world: WorldState,
    pub sim: SimState,
    pub storylets: StoryletLibrary,
}

const DEFAULT_STORYLET_DB: &str = "storylets.sqlite";

static RUNTIME: Lazy<Mutex<GameRuntime>> = Lazy::new(|| {
    let world = WorldState::new(WorldSeed::new(0), NpcId(1));
    let sim = SimState::new();
    let storylets = StoryletLibrary::load_default().unwrap_or_default();

    Mutex::new(GameRuntime {
        world,
        sim,
        storylets,
    })
});

fn register_storylets_from_db(director: &mut EventDirector) {
    let db_path =
        std::env::var("SYN_STORYLET_DB").unwrap_or_else(|_| DEFAULT_STORYLET_DB.to_string());
    match load_storylets_from_db(&db_path) {
        Ok(storylets) => {
            for content_storylet in storylets {
                // Convert syn_content::Storylet to syn_director::Storylet
                let director_storylet = Storylet {
                    id: content_storylet.id,
                    name: content_storylet.name,
                    tags: content_storylet.tags,
                    prerequisites: syn_director::StoryletPrerequisites {
                        min_relationship_affection: content_storylet
                            .prerequisites
                            .min_relationship_affection,
                        min_relationship_resentment: content_storylet
                            .prerequisites
                            .min_relationship_resentment,
                        stat_conditions: content_storylet.prerequisites.stat_conditions,
                        life_stages: content_storylet.prerequisites.life_stages,
                        tags: content_storylet.prerequisites.tags,
                        digital_legacy_prereq: content_storylet
                            .prerequisites
                            .digital_legacy_prereq
                            .as_ref()
                            .map(|p| syn_director::DigitalLegacyPrereq {
                                require_post_life: p.require_post_life,
                                min_compassion_vs_cruelty: p.min_compassion_vs_cruelty,
                                max_compassion_vs_cruelty: p.max_compassion_vs_cruelty,
                                min_ambition_vs_comfort: p.min_ambition_vs_comfort,
                                max_ambition_vs_comfort: p.max_ambition_vs_comfort,
                                min_connection_vs_isolation: p.min_connection_vs_isolation,
                                max_connection_vs_isolation: p.max_connection_vs_isolation,
                                min_stability_vs_chaos: p.min_stability_vs_chaos,
                                max_stability_vs_chaos: p.max_stability_vs_chaos,
                                min_light_vs_shadow: p.min_light_vs_shadow,
                                max_light_vs_shadow: p.max_light_vs_shadow,
                            }),
                        relationship_states: content_storylet.prerequisites.relationship_states,
                        memory_tags_required: content_storylet.prerequisites.memory_tags_required,
                        memory_tags_forbidden: content_storylet.prerequisites.memory_tags_forbidden,
                        memory_recency_ticks: content_storylet.prerequisites.memory_recency_ticks,
                        relationship_prereqs: content_storylet
                            .prerequisites
                            .relationship_prereqs
                            .into_iter()
                            .map(|r| syn_director::RelationshipPrereq {
                                actor_id: r.actor_id,
                                target_id: r.target_id,
                                axis: r.axis,
                                min_value: r.min_value,
                                max_value: r.max_value,
                                min_band: r.min_band,
                                max_band: r.max_band,
                            })
                            .collect(),
                        allowed_life_stages: content_storylet.prerequisites.allowed_life_stages,
                        time_and_location: None,
                    },
                    heat: content_storylet.heat,
                    weight: content_storylet.weight,
                    cooldown_ticks: content_storylet.cooldown_ticks,
                    roles: content_storylet
                        .roles
                        .into_iter()
                        .map(|r| syn_director::StoryletRole {
                            name: r.name,
                            npc_id: r.npc_id,
                        })
                        .collect(),
                    max_uses: None,
                    choices: vec![],
                    heat_category: content_storylet.heat_category,
                    actors: None,
                    interaction_tone: None,
                };
                director.register_storylet(director_storylet);
            }
        }
        Err(err) => {
            eprintln!(
                "Warning: failed to load storylets from {}: {}",
                db_path, err
            );
        }
    }
}

impl GameEngine {
    pub fn new(seed: u64) -> Self {
        let world_seed = WorldSeed::new(seed);
        let player_id = NpcId(1);
        let world = WorldState::new(world_seed, player_id);

        let mut director = EventDirector::new();
        register_storylets_from_db(&mut director);

        GameEngine {
            world,
            simulator: Simulator::new(seed),
            director,
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

    pub fn player_mood_band(&self) -> String {
        format!("{:?}", self.world.player_stats.mood_band())
    }

    pub fn life_stage_info(&self) -> ApiLifeStageInfo {
        let cfg = self.world.player_life_stage.config();
        ApiLifeStageInfo {
            life_stage: format!("{:?}", self.world.player_life_stage),
            player_age_years: self.world.player_age_years as i32,
            show_wealth: cfg.visibility.show_wealth,
            show_reputation: cfg.visibility.show_reputation,
            show_wisdom: cfg.visibility.show_wisdom,
            show_karma: cfg.visibility.show_karma,
        }
    }

    /// Get player karma.
    pub fn player_karma(&self) -> f32 {
        self.world.player_karma.0
    }

    pub fn player_karma_band(&self) -> String {
        format!("{:?}", self.world.player_karma.band())
    }

    /// TEST/utility: override player stage + age (deterministic).
    pub fn set_player_life_stage(&mut self, stage: LifeStage, age_years: u32) {
        self.world.player_life_stage = stage;
        self.world.player_age_years = age_years;
        self.world.player_age = age_years;
    }

    /// Get current narrative heat.
    pub fn narrative_heat(&self) -> f32 {
        self.world.narrative_heat.value()
    }

    /// Get textual heat level (Low/Medium/High/Critical).
    pub fn narrative_heat_level(&self) -> String {
        self.world.heat_level().to_string()
    }

    /// Get normalized trend for UI (–1.0 cooling .. +1.0 spiking).
    pub fn narrative_heat_trend(&self) -> f32 {
        self.world.heat_trend()
    }

    /// Get player stats (serialized for Dart).
    pub fn player_stats(&self) -> ApiStatsSnapshot {
        ApiStatsSnapshot {
            stats: ALL_STAT_KINDS
                .iter()
                .map(|kind| ApiStat {
                    kind: format!("{:?}", kind),
                    value: self.world.player_stats.get(*kind),
                })
                .collect(),
            mood_band: format!("{:?}", self.world.player_stats.mood_band()),
        }
    }

    /// Alias for FRB: get unified stats snapshot.
    pub fn get_player_stats(&self) -> ApiStatsSnapshot {
        self.player_stats()
    }

    pub fn get_mood_band(&self) -> String {
        format!("{:?}", self.world.player_stats.mood_band())
    }

    pub fn get_karma_band(&self) -> String {
        format!("{:?}", self.world.player_karma.band())
    }

    /// Get player relationships snapshot (with bands and role labels).
    pub fn player_relationships(&self) -> ApiRelationshipSnapshot {
        let player_id = self.world.player_id;
        let mut relationships = Vec::new();

        for (&(actor_id, target_id), rel) in self.world.relationships.iter() {
            // Only expose relationships where the actor is the player
            if actor_id != player_id {
                continue;
            }

            // Convert Relationship to RelationshipVector for band methods
            let rel_vec = RelationshipVector {
                affection: rel.affection,
                trust: rel.trust,
                attraction: rel.attraction,
                familiarity: rel.familiarity,
                resentment: rel.resentment,
            };

            let api_rel = ApiRelationship {
                actor_id: actor_id.0 as i64,
                target_id: target_id.0 as i64,
                affection: rel.affection,
                trust: rel.trust,
                attraction: rel.attraction,
                familiarity: rel.familiarity,
                resentment: rel.resentment,
                affection_band: rel_vec.affection_band().to_string(),
                trust_band: rel_vec.trust_band().to_string(),
                attraction_band: rel_vec.attraction_band().to_string(),
                resentment_band: rel_vec.resentment_band().to_string(),
                role_label: derive_role_label(&rel_vec),
            };

            relationships.push(api_rel);
        }

        ApiRelationshipSnapshot { relationships }
    }

    // ==================== Simulation ====================

    /// Advance the simulation by one tick.
    pub fn tick(&mut self) {
        let previous_stage = self.world.player_life_stage;
        self.simulator.tick(&mut self.world);

        // Auto-create digital imprint if we just entered Digital stage
        if previous_stage != self.world.player_life_stage
            && matches!(self.world.player_life_stage, LifeStage::Digital)
        {
            self.ensure_digital_imprint();
        }

        // Tick PostLife drift if in Digital stage
        syn_sim::post_life::tick_postlife_drift(&mut self.world);
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
        let mut rel = Relationship {
            affection: affection.clamp(-10.0, 10.0),
            trust: trust.clamp(-10.0, 10.0),
            attraction: attraction.clamp(-10.0, 10.0),
            familiarity: familiarity.clamp(-10.0, 10.0),
            resentment: resentment.clamp(-10.0, 10.0),
            state: syn_core::RelationshipState::Stranger,
        };
        // Compute the correct state based on axes
        rel.state = rel.compute_next_state();
        self.world
            .set_relationship(NpcId(from_npc_id), NpcId(to_npc_id), rel);
    }

    /// Get a relationship between two NPCs.
    pub fn get_relationship(&self, from_npc_id: u64, to_npc_id: u64) -> RelationshipDto {
        let rel = self
            .world
            .get_relationship(NpcId(from_npc_id), NpcId(to_npc_id));
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

    // ==================== Digital Legacy ====================

    /// Ensure digital imprint is created when entering PostLife/Digital stage.
    pub fn ensure_digital_imprint(&mut self) {
        // Collect all memory entries for the builder
        let all_memories: Vec<MemoryEntry> = self
            .memory
            .journals
            .values()
            .flat_map(|journal| journal.entries.clone())
            .collect();

        syn_sim::post_life::ensure_digital_imprint_for_postlife(&mut self.world, &all_memories);
    }

    /// Get digital legacy snapshot for UI.
    pub fn get_digital_legacy_snapshot(&self) -> ApiDigitalLegacySnapshot {
        if let Some(imprint) = &self.world.digital_legacy.primary_imprint {
            let lv = &imprint.legacy_vector;
            let api_lv = ApiLegacyVector {
                compassion_vs_cruelty: lv.compassion_vs_cruelty,
                ambition_vs_comfort: lv.ambition_vs_comfort,
                connection_vs_isolation: lv.connection_vs_isolation,
                stability_vs_chaos: lv.stability_vs_chaos,
                light_vs_shadow: lv.light_vs_shadow,
            };

            let roles = imprint
                .relationship_roles
                .iter()
                .map(|(target_id, role)| ApiLegacyRelationshipRole {
                    target_id: target_id.0 as i64,
                    role: role.to_string(),
                })
                .collect::<Vec<_>>();

            ApiDigitalLegacySnapshot {
                has_imprint: true,
                imprint: Some(ApiDigitalImprint {
                    id: imprint.id as i64,
                    created_at_stage: format!("{:?}", imprint.created_at_stage),
                    created_at_age_years: imprint.created_at_age_years as i32,
                    legacy_vector: api_lv,
                    relationship_roles: roles,
                }),
            }
        } else {
            ApiDigitalLegacySnapshot {
                has_imprint: false,
                imprint: None,
            }
        }
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
                digital_legacy_prereq: None,
                relationship_states: vec![],
                memory_tags_required: vec![],
                memory_tags_forbidden: vec![],
                memory_recency_ticks: None,
                relationship_prereqs: vec![],
                allowed_life_stages: vec![],
                time_and_location: None,
                time_and_location: None,
            },
            heat,
            weight,
            cooldown_ticks: 100,
            roles: vec![],
            max_uses: None,
            choices: vec![],
            heat_category: None,
            actors: None,
            interaction_tone: None,
        };
        self.director.register_storylet(storylet);
    }

    /// Select and return the next eligible event.
    pub fn select_next_event(&self) -> Option<EventDto> {
        self.director
            .select_next_event(&self.world, &self.memory, self.world.current_tick)
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
pub struct ApiStatsSnapshot {
    pub stats: Vec<ApiStat>,
    pub mood_band: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiLifeStageInfo {
    pub life_stage: String,
    pub player_age_years: i32,
    pub show_wealth: bool,
    pub show_reputation: bool,
    pub show_wisdom: bool,
    pub show_karma: bool,
}

pub type PlayerStatsDto = ApiStatsSnapshot;

#[derive(Debug, Clone)]
pub struct ApiStat {
    pub kind: String,
    pub value: f32,
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

/// Relationship DTO for serialization to Dart (with bands and role label).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiRelationship {
    pub actor_id: i64,
    pub target_id: i64,
    pub affection: f32,
    pub trust: f32,
    pub attraction: f32,
    pub familiarity: f32,
    pub resentment: f32,
    pub affection_band: String,
    pub trust_band: String,
    pub attraction_band: String,
    pub resentment_band: String,
    /// High-level summary for UI tags: "Friend", "Rival", "Crush", "Stranger", etc.
    pub role_label: String,
}

/// Snapshot of all player relationships for UI display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiRelationshipSnapshot {
    pub relationships: Vec<ApiRelationship>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiDirectorChoiceView {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiDirectorEventView {
    pub storylet_id: String,
    pub title: String,
    pub choices: Vec<ApiDirectorChoiceView>,
}

impl From<DirectorEventView> for ApiDirectorEventView {
    fn from(view: DirectorEventView) -> Self {
        ApiDirectorEventView {
            storylet_id: view.storylet_id,
            title: view.title,
            choices: view
                .choices
                .into_iter()
                .map(|c| ApiDirectorChoiceView {
                    id: c.id,
                    label: c.label,
                })
                .collect(),
        }
    }
}

/// Digital legacy vector DTO for serialization to Dart.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiLegacyVector {
    pub compassion_vs_cruelty: f32,
    pub ambition_vs_comfort: f32,
    pub connection_vs_isolation: f32,
    pub stability_vs_chaos: f32,
    pub light_vs_shadow: f32,
}

/// Legacy relationship role DTO for serialization to Dart.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiLegacyRelationshipRole {
    pub target_id: i64,
    pub role: String,
}

/// Digital imprint DTO for serialization to Dart.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiDigitalImprint {
    pub id: i64,
    pub created_at_stage: String,
    pub created_at_age_years: i32,
    pub legacy_vector: ApiLegacyVector,
    pub relationship_roles: Vec<ApiLegacyRelationshipRole>,
}

/// Digital legacy snapshot DTO for serialization to Dart.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiDigitalLegacySnapshot {
    pub has_imprint: bool,
    pub imprint: Option<ApiDigitalImprint>,
}

// ==================== Director Loop API ====================

/// Replace the shared runtime (primarily for tests).
pub fn api_reset_runtime(world: WorldState, sim: SimState, storylets: StoryletLibrary) {
    let mut guard = RUNTIME.lock().expect("GameRuntime poisoned");
    *guard = GameRuntime {
        world,
        sim,
        storylets,
    };
}

#[frb(sync)]
pub fn api_get_current_event() -> Option<ApiDirectorEventView> {
    let mut guard = RUNTIME.lock().expect("GameRuntime poisoned");
    let runtime = &mut *guard;

    let view = select_next_event_view(&mut runtime.world, &mut runtime.sim, &runtime.storylets)?;
    Some(ApiDirectorEventView::from(view))
}

#[frb(sync)]
pub fn api_choose_option(
    storylet_id: String,
    choice_id: String,
    ticks_to_advance: u32,
) -> Option<ApiDirectorEventView> {
    let mut guard = RUNTIME.lock().expect("GameRuntime poisoned");
    let runtime = &mut *guard;

    let view = apply_choice_and_advance(
        &mut runtime.world,
        &mut runtime.sim,
        &runtime.storylets,
        &storylet_id,
        &choice_id,
        ticks_to_advance,
    )?;

    Some(ApiDirectorEventView::from(view))
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
        .map(|e| e.world.player_stats.get(StatKind::Mood))
        .unwrap_or(0.0)
}

#[frb(sync)]
pub fn engine_player_relationships() -> ApiRelationshipSnapshot {
    let engine = ENGINE.lock().unwrap();
    engine
        .as_ref()
        .map(|e| e.player_relationships())
        .unwrap_or(ApiRelationshipSnapshot {
            relationships: vec![],
        })
}

/// Get current narrative heat value.
#[frb(sync)]
pub fn engine_narrative_heat() -> f32 {
    let engine = ENGINE.lock().unwrap();
    engine.as_ref().map(|e| e.narrative_heat()).unwrap_or(0.0)
}

/// Get current narrative heat level label.
#[frb(sync)]
pub fn engine_narrative_heat_level() -> String {
    let engine = ENGINE.lock().unwrap();
    engine
        .as_ref()
        .map(|e| e.narrative_heat_level())
        .unwrap_or_else(|| "Low".to_string())
}

/// Get normalized heat trend (-1.0..1.0).
#[frb(sync)]
pub fn engine_narrative_heat_trend() -> f32 {
    let engine = ENGINE.lock().unwrap();
    engine
        .as_ref()
        .map(|e| e.narrative_heat_trend())
        .unwrap_or(0.0)
}

/// Get life stage info (stage label, age, visibility flags).
#[frb(sync)]
pub fn engine_life_stage_info() -> ApiLifeStageInfo {
    let engine = ENGINE.lock().unwrap();
    engine
        .as_ref()
        .map(|e| e.life_stage_info())
        .unwrap_or(ApiLifeStageInfo {
            life_stage: "Unknown".to_string(),
            player_age_years: 0,
            show_wealth: false,
            show_reputation: false,
            show_wisdom: false,
            show_karma: false,
        })
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

/// Ensure digital imprint is created for PostLife stage.
#[frb(sync)]
pub fn engine_ensure_digital_imprint() {
    let mut engine = ENGINE.lock().unwrap();
    if let Some(ref mut e) = *engine {
        e.ensure_digital_imprint();
    }
}

/// Get digital legacy snapshot (imprint).
#[frb(sync)]
pub fn engine_get_digital_legacy() -> ApiDigitalLegacySnapshot {
    let engine = ENGINE.lock().unwrap();
    engine
        .as_ref()
        .map(|e| e.get_digital_legacy_snapshot())
        .unwrap_or(ApiDigitalLegacySnapshot {
            has_imprint: false,
            imprint: None,
        })
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

    #[test]
    fn test_narrative_heat_accessors() {
        let engine = GameEngine::new(42);
        assert_eq!(engine.narrative_heat(), 10.0);
        assert_eq!(engine.narrative_heat_level(), "Low");
        assert_eq!(engine.narrative_heat_trend(), 0.0);
    }

    #[test]
    fn test_digital_legacy_snapshot_exposes_imprint() {
        use std::collections::HashMap;
        use syn_core::digital_legacy::{DigitalImprint, LegacyVector};
        use syn_core::relationship_model::RelationshipRole;

        let mut engine = GameEngine::new(99);
        let mut relationship_roles = HashMap::new();
        relationship_roles.insert(NpcId(2), RelationshipRole::Friend);

        engine.world.digital_legacy.primary_imprint = Some(DigitalImprint {
            id: 7,
            created_at_stage: LifeStage::Digital,
            created_at_age_years: 93,
            final_stats: Stats::default(),
            final_karma: Karma(12.0),
            legacy_vector: LegacyVector {
                compassion_vs_cruelty: 0.25,
                ..Default::default()
            },
            relationship_roles,
            relationship_milestones: Vec::new(),
            memory_tag_counts: HashMap::new(),
        });

        let snapshot = engine.get_digital_legacy_snapshot();
        assert!(snapshot.has_imprint);

        let imprint = snapshot.imprint.expect("imprint should be present");
        assert_eq!(imprint.id, 7);
        assert_eq!(imprint.created_at_stage, "Digital");
        assert_eq!(imprint.created_at_age_years, 93);
        assert!((imprint.legacy_vector.compassion_vs_cruelty - 0.25).abs() < f32::EPSILON);
        assert_eq!(imprint.relationship_roles.len(), 1);
        assert_eq!(imprint.relationship_roles[0].target_id, 2);
        assert_eq!(imprint.relationship_roles[0].role, "Friend");
    }
}
