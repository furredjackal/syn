//! syn_api: FFI aggregation layer for Flutter via flutter_rust_bridge.
//!
//! Exposes the entire SYN simulation engine to Flutter through a typed, async-friendly API.
//! This is the "public interface" of the Rust backend.
//!
//! ## Architecture
//!
//! - [`GameEngine`]: Main game state manager combining world, simulator, director, and memory
//! - [`GameRuntime`]: Shared runtime state for the director loop
//! - API functions prefixed with `engine_*` are `#[frb(sync)]` for Flutter Rust Bridge
//!
//! ## DTOs
//!
//! All DTOs (Data Transfer Objects) are serializable structs for Dart interop:
//! - [`ApiStatsSnapshot`]: Player stats
//! - [`ApiRelationshipSnapshot`]: Player relationships with bands and roles
//! - [`ApiDigitalLegacySnapshot`]: PostLife digital imprint data
//! - [`ApiDistrictSnapshot`]: District economic/crime data
//! - [`ApiPlayerSkillsSnapshot`]: Player skill progression

use flutter_rust_bridge::frb;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use syn_content::load_storylets_from_db;
use syn_core::relationship_model::{derive_role_label, RelationshipVector};
use syn_director::{apply_choice_and_advance, select_next_event_view, DirectorEventView};
use syn_sim::SimState;

/// Storylet library loading utilities.
pub mod library_loader;

// Re-export core types for Dart
pub use syn_core::{
    AbstractNpc, AttachmentStyle, Karma, KarmaBand, LifeStage, MoodBand, NpcId, Relationship,
    SimTick, StatKind, Stats, Traits, WorldSeed, WorldState, ALL_STAT_KINDS,
};
pub use syn_core::character_gen::{
    CharacterArchetype, CharacterGenConfig, Difficulty, EarlyLifeEvent, FamilyStructure,
    GeneratedCharacter, SocioeconomicTier, generate_character,
};
pub use syn_core::district::{
    CrimeLevel, District, DistrictId, DistrictRegistry, DistrictType, EconomicTier,
};
pub use syn_core::skills::{
    PlayerSkills, SkillCategory, SkillDefinition, SkillId, SkillProgress, SkillRegistry,
    SkillState, SkillTier,
};
pub use syn_director::{
    tags_to_bitset, EventDirector, Storylet, StoryletChoice, StoryletCooldown, StoryletLibrary,
    StoryletOutcome, StoryletOutcomeSet, StoryletRole,
};
pub use syn_memory::{Journal, MemoryEntry, MemorySystem};
pub use syn_query::{ClusterQuery, NpcQuery, RelationshipQuery, StatQuery};
pub use syn_sim::{LodTier, Simulator};

/// Main game engine combining world state, simulator, storylets, and memory system.
///
/// This struct is the central hub for all game logic. It manages:
/// - The [`WorldState`] containing all simulation data
/// - The [`Simulator`] for LOD-based NPC tick processing
/// - The [`EventDirector`] for storylet selection
/// - The [`MemorySystem`] for NPC memories
///
/// ## Usage
///
/// ```ignore
/// let engine = GameEngine::new(12345);
/// engine.tick(); // Advance one hour
/// let stats = engine.player_stats();
/// ```
pub struct GameEngine {
    /// The current world state (player, NPCs, relationships, time, etc.).
    world: WorldState,
    /// The LOD-based simulator for tick processing.
    simulator: Simulator,
    /// The event director for storylet selection.
    director: EventDirector,
    /// The memory system tracking NPC memories.
    memory: MemorySystem,
}

/// Shared runtime state for the director loop.
///
/// This is used with the global `RUNTIME` static for FRB functions
/// that need to maintain state across calls.
pub struct GameRuntime {
    /// The world state.
    pub world: WorldState,
    /// The simulation state machine.
    pub sim: SimState,
    /// The loaded storylet library.
    pub storylets: StoryletLibrary,
}

/// Default storylet database filename.
const DEFAULT_STORYLET_DB: &str = "storylets.sqlite";

/// Lazily-initialized global runtime for FRB director loop functions.
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

/// Loads storylets from database and registers them with the event director.
fn register_storylets_from_db(director: &mut EventDirector) {
    let db_path =
        std::env::var("SYN_STORYLET_DB").unwrap_or_else(|_| DEFAULT_STORYLET_DB.to_string());
    match load_storylets_from_db(&db_path) {
        Ok(storylets) => {
            for content_storylet in storylets {
                let tag_list = content_storylet.prerequisites.tags.clone();
                // Convert syn_content::Storylet to syn_director::Storylet
                let mut prereqs = syn_director::StoryletPrerequisites::default();
                prereqs.min_relationship_affection = content_storylet
                    .prerequisites
                    .min_relationship_affection;
                prereqs.min_relationship_resentment = content_storylet
                    .prerequisites
                    .min_relationship_resentment;
                prereqs.stat_ranges = content_storylet.prerequisites.stat_conditions;
                prereqs.life_stages = content_storylet.prerequisites.life_stages;
                prereqs.tags = tag_list.clone();
                prereqs.digital_legacy_prereq = content_storylet
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
                    });
                prereqs.relationship_states = content_storylet.prerequisites.relationship_states;
                prereqs.memory_tags_required = content_storylet.prerequisites.memory_tags_required;
                prereqs.memory_tags_forbidden =
                    content_storylet.prerequisites.memory_tags_forbidden;
                prereqs.memory_recency_ticks = content_storylet.prerequisites.memory_recency_ticks;
                prereqs.relationship_prereqs = content_storylet
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
                    .collect();
                prereqs.allowed_life_stages = content_storylet.prerequisites.allowed_life_stages;
                prereqs.time_and_location = None;

                let director_storylet = Storylet {
                    id: content_storylet.id,
                    name: content_storylet.name,
                    prerequisites: prereqs,
                    heat: content_storylet.heat as i32,
                    weight: content_storylet.weight,
                    cooldown: syn_director::StoryletCooldown {
                        ticks: content_storylet.cooldown_ticks,
                    },
                    roles: content_storylet
                        .roles
                        .into_iter()
                        .map(|r| syn_director::StoryletRole {
                            name: r.name,
                            npc_id: r.npc_id,
                        })
                        .collect::<syn_director::StoryletRoles>(),
                    tags: syn_director::tags_to_bitset(&tag_list),
                    outcomes: syn_director::StoryletOutcomeSet {
                        max_uses: None,
                        choices: vec![],
                        heat_category: content_storylet.heat_category,
                        actors: None,
                        interaction_tone: None,
                        ..Default::default()
                    },
                    ..Default::default()
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
    /// Create a new game engine with the given world seed.
    ///
    /// This initializes the world state, simulator, event director, and memory system.
    /// Storylets are loaded from the database path in `SYN_STORYLET_DB` environment
    /// variable, or from `storylets.sqlite` by default.
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

    /// Get player mood band as a string (e.g., "Neutral", "Happy").
    pub fn player_mood_band(&self) -> String {
        format!("{:?}", self.world.player_stats.mood_band())
    }

    /// Get comprehensive life stage information for UI display.
    ///
    /// Returns visibility flags that control which stats are shown
    /// based on the player's current life stage.
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

    /// Get player karma band as a string (e.g., "Neutral", "Good", "Saint").
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

    /// Get mood band label for UI display.
    pub fn get_mood_band(&self) -> String {
        format!("{:?}", self.world.player_stats.mood_band())
    }

    /// Get karma band label for UI display.
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
        let mut prereqs = syn_director::StoryletPrerequisites::default();
        prereqs.tags = vec![];
        let storylet = Storylet {
            id: storylet_id,
            name,
            tags: tags_to_bitset(&[]),
            prerequisites: prereqs,
            heat: heat as i32,
            weight,
            roles: syn_director::StoryletRoles::default(),
            outcomes: syn_director::StoryletOutcomeSet::default(),
            cooldown: syn_director::StoryletCooldown { ticks: 100 },
            ..Default::default()
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
                heat: s.heat as f32,
            })
    }
}

// ==================== Data Transfer Objects (DTOs) for Dart ====================

/// Player stats snapshot for serialization to Dart.
///
/// Contains all stat values and the current mood band label.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiStatsSnapshot {
    /// All player stats with their current values.
    pub stats: Vec<ApiStat>,
    /// Current mood band label (e.g., "Happy", "Neutral", "Depressed").
    pub mood_band: String,
}

/// Life stage information DTO with UI visibility flags.
///
/// Controls which stats are visible based on player's life stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiLifeStageInfo {
    /// Current life stage label (e.g., "Child", "Teen", "Adult").
    pub life_stage: String,
    /// Player's age in years.
    pub player_age_years: i32,
    /// Whether to show wealth stat.
    pub show_wealth: bool,
    /// Whether to show reputation stat.
    pub show_reputation: bool,
    /// Whether to show wisdom stat.
    pub show_wisdom: bool,
    /// Whether to show karma stat.
    pub show_karma: bool,
}

/// Type alias for backwards compatibility.
pub type PlayerStatsDto = ApiStatsSnapshot;

/// Individual stat DTO with kind and value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiStat {
    /// Stat kind name (e.g., "Health", "Happiness", "Wealth").
    pub kind: String,
    /// Current value (typically 0.0-100.0).
    pub value: f32,
}

/// NPC data transfer object for serialization to Dart.
#[derive(Debug, Clone)]
pub struct NpcDto {
    /// Unique NPC identifier.
    pub id: u64,
    /// NPC's age in years.
    pub age: u32,
    /// NPC's occupation.
    pub job: String,
    /// District where NPC resides.
    pub district: String,
}

/// Relationship axes DTO for serialization to Dart.
///
/// Contains raw axis values and derived heat.
#[derive(Debug, Clone)]
pub struct RelationshipDto {
    /// Affection axis (-10.0 to +10.0).
    pub affection: f32,
    /// Trust axis (-10.0 to +10.0).
    pub trust: f32,
    /// Attraction axis (-10.0 to +10.0).
    pub attraction: f32,
    /// Familiarity axis (-10.0 to +10.0).
    pub familiarity: f32,
    /// Resentment axis (-10.0 to +10.0).
    pub resentment: f32,
    /// Derived relationship heat value.
    pub heat: f32,
}

/// Full relationship DTO with bands and role labels for UI.
///
/// Includes both raw axis values and derived band labels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiRelationship {
    /// Actor (source) NPC ID.
    pub actor_id: i64,
    /// Target NPC ID.
    pub target_id: i64,
    /// Affection axis value (-10.0 to +10.0).
    pub affection: f32,
    /// Trust axis value (-10.0 to +10.0).
    pub trust: f32,
    /// Attraction axis value (-10.0 to +10.0).
    pub attraction: f32,
    /// Familiarity axis value (-10.0 to +10.0).
    pub familiarity: f32,
    /// Resentment axis value (-10.0 to +10.0).
    pub resentment: f32,
    /// Affection band label (e.g., "Warm", "Devoted").
    pub affection_band: String,
    /// Trust band label.
    pub trust_band: String,
    /// Attraction band label.
    pub attraction_band: String,
    /// Resentment band label.
    pub resentment_band: String,
    /// High-level summary for UI tags: "Friend", "Rival", "Crush", "Stranger", etc.
    pub role_label: String,
}

/// Snapshot of all player relationships for UI display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiRelationshipSnapshot {
    /// All player relationships with bands and role labels.
    pub relationships: Vec<ApiRelationship>,
}

/// Memory entry DTO for serialization to Dart.
#[derive(Debug, Clone)]
pub struct MemoryDto {
    /// Unique memory identifier.
    pub id: String,
    /// Associated event ID.
    pub event_id: String,
    /// Emotional intensity (-1.0 to +1.0).
    pub emotional_intensity: f32,
    /// Simulation tick when memory was created.
    pub sim_tick: u64,
}

/// Event/storylet DTO for serialization to Dart.
#[derive(Debug, Clone)]
pub struct EventDto {
    /// Storylet ID.
    pub id: String,
    /// Storylet name/title.
    pub name: String,
    /// Heat value for narrative pacing.
    pub heat: f32,
}

/// Director choice view DTO for UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiDirectorChoiceView {
    /// Choice identifier.
    pub id: String,
    /// Display label for the choice.
    pub label: String,
}

/// Director event view DTO for UI display.
///
/// Represents a selected storylet with its available choices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiDirectorEventView {
    /// The storylet's unique identifier.
    pub storylet_id: String,
    /// Display title for the event.
    pub title: String,
    /// Available choices for the player.
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
///
/// The five-axis legacy vector summarizes the player's life choices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiLegacyVector {
    /// Compassion vs cruelty axis (-1.0 to +1.0).
    pub compassion_vs_cruelty: f32,
    /// Ambition vs comfort axis (-1.0 to +1.0).
    pub ambition_vs_comfort: f32,
    /// Connection vs isolation axis (-1.0 to +1.0).
    pub connection_vs_isolation: f32,
    /// Stability vs chaos axis (-1.0 to +1.0).
    pub stability_vs_chaos: f32,
    /// Light vs shadow axis (-1.0 to +1.0).
    pub light_vs_shadow: f32,
}

/// Legacy relationship role DTO for serialization to Dart.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiLegacyRelationshipRole {
    /// Target NPC ID.
    pub target_id: i64,
    /// Role label (e.g., "Friend", "Rival", "Mentor").
    pub role: String,
}

/// Digital imprint DTO for serialization to Dart.
///
/// Represents the player's digital afterlife imprint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiDigitalImprint {
    /// Unique imprint identifier.
    pub id: i64,
    /// Life stage when imprint was created.
    pub created_at_stage: String,
    /// Age in years when imprint was created.
    pub created_at_age_years: i32,
    /// The legacy vector summarizing life choices.
    pub legacy_vector: ApiLegacyVector,
    /// Relationship roles at time of imprint.
    pub relationship_roles: Vec<ApiLegacyRelationshipRole>,
}

/// Digital legacy snapshot DTO for UI.
///
/// Contains the optional digital imprint if one exists.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiDigitalLegacySnapshot {
    /// Whether an imprint exists.
    pub has_imprint: bool,
    /// The digital imprint, if present.
    pub imprint: Option<ApiDigitalImprint>,
}

// ==================== Character Generation DTOs ====================

/// Character generation config DTO for Flutter.
///
/// Specifies player's chosen archetype and difficulty for procedural generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiCharacterGenConfig {
    /// Player's chosen name.
    pub name: String,
    /// Archetype: "STORYTELLER", "ANALYST", "DREAMER", or "CHALLENGER".
    pub archetype: String,
    /// Difficulty: "FORGIVING", "BALANCED", or "HARSH".
    pub difficulty: String,
    /// Whether to enable SFW (safe for work) mode.
    pub sfw_mode: bool,
}

/// Generated character DTO for Flutter.
///
/// Contains all procedurally generated character data including
/// background, stats, personality, and starting conditions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiGeneratedCharacter {
    /// Player's name.
    pub name: String,
    /// Selected archetype.
    pub archetype: String,
    /// Selected difficulty.
    pub difficulty: String,
    /// Whether SFW mode is enabled.
    pub sfw_mode: bool,
    /// Procedurally generated family structure.
    pub family_structure: String,
    /// Procedurally generated socioeconomic tier.
    pub socioeconomic_tier: String,
    /// Procedurally generated early life event.
    pub early_life_event: String,
    /// Procedurally generated attachment style.
    pub attachment_style: String,
    /// Luck seed for deterministic random events.
    pub luck_seed: u64,
    /// Starting district name.
    pub starting_district: String,
    /// Whether character has early trauma.
    pub has_early_trauma: bool,
    /// Initial stats snapshot.
    pub stats: ApiStatsSnapshot,
    /// Personality axes (0-1 range).
    pub personality: ApiPersonalityVector,
    /// Starting karma value.
    pub starting_karma: f32,
}

/// Personality vector DTO for Flutter.
///
/// Five-axis personality model values, each ranging 0.0 to 1.0.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiPersonalityVector {
    /// Warmth axis (cold to warm).
    pub warmth: f32,
    /// Dominance axis (submissive to dominant).
    pub dominance: f32,
    /// Volatility axis (stable to volatile).
    pub volatility: f32,
    /// Conscientiousness axis (careless to conscientious).
    pub conscientiousness: f32,
    /// Openness axis (closed to open).
    pub openness: f32,
}

// ==================== District DTOs ====================

/// District snapshot DTO for Flutter.
///
/// Complete district data including core values, derived bands, and trends.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiDistrictSnapshot {
    /// District ID.
    pub id: u32,
    /// District name.
    pub name: String,
    /// District type (e.g., "Downtown", "Suburban", "Industrial").
    pub district_type: String,
    /// Economy value (0-100).
    pub economy: f32,
    /// Crime value (0-100).
    pub crime: f32,
    /// Economic tier label (e.g., "Wealthy", "Middle", "Poor").
    pub economic_tier: String,
    /// Crime level label (e.g., "Safe", "Moderate", "Dangerous").
    pub crime_level: String,
    /// Derived safety score.
    pub safety: f32,
    /// Derived livability score.
    pub livability: f32,
    /// Derived desirability score.
    pub desirability: f32,
    /// Unemployment rate (0-100).
    pub unemployment: f32,
    /// Rent index relative to city average.
    pub rent_index: f32,
    /// Community cohesion score.
    pub community_cohesion: f32,
    /// Cultural index score.
    pub cultural_index: f32,
    /// Population count.
    pub population: u32,
    /// Economy trend (-1.0 to +1.0).
    pub economy_trend: f32,
    /// Crime trend (-1.0 to +1.0).
    pub crime_trend: f32,
}

impl From<&District> for ApiDistrictSnapshot {
    fn from(d: &District) -> Self {
        Self {
            id: d.id.0,
            name: d.name.clone(),
            district_type: format!("{:?}", d.district_type),
            economy: d.economy,
            crime: d.crime,
            economic_tier: format!("{:?}", d.economic_tier()),
            crime_level: format!("{:?}", d.crime_level()),
            safety: d.safety(),
            livability: d.livability(),
            desirability: d.desirability(),
            unemployment: d.unemployment,
            rent_index: d.rent_index,
            community_cohesion: d.community_cohesion,
            cultural_index: d.cultural_index,
            population: d.population,
            economy_trend: d.economy_trend,
            crime_trend: d.crime_trend,
        }
    }
}

/// Summary of a district for list views.
///
/// Contains essential information for displaying districts in lists.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiDistrictSummary {
    /// District ID.
    pub id: u32,
    /// District name.
    pub name: String,
    /// District type label.
    pub district_type: String,
    /// Economic tier label.
    pub economic_tier: String,
    /// Crime level label.
    pub crime_level: String,
    /// Safety score.
    pub safety: f32,
}

impl From<&District> for ApiDistrictSummary {
    fn from(d: &District) -> Self {
        Self {
            id: d.id.0,
            name: d.name.clone(),
            district_type: format!("{:?}", d.district_type),
            economic_tier: format!("{:?}", d.economic_tier()),
            crime_level: format!("{:?}", d.crime_level()),
            safety: d.safety(),
        }
    }
}

/// City-wide aggregate statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiCityStats {
    /// Total number of districts.
    pub district_count: u32,
    /// City-wide average economy score.
    pub average_economy: f32,
    /// City-wide average crime score.
    pub average_crime: f32,
    /// Name of the safest district.
    pub safest_district: Option<String>,
    /// Name of the most dangerous district.
    pub most_dangerous_district: Option<String>,
    /// Name of the wealthiest district.
    pub wealthiest_district: Option<String>,
}

// ==================== Director Loop API ====================

/// Replace the shared runtime (primarily for tests).
///
/// Replaces the global `RUNTIME` state with the provided components.
pub fn api_reset_runtime(world: WorldState, sim: SimState, storylets: StoryletLibrary) {
    let mut guard = RUNTIME.lock().expect("GameRuntime poisoned");
    *guard = GameRuntime {
        world,
        sim,
        storylets,
    };
}

/// Get the current event from the director.
///
/// Returns the next eligible storylet event for the player, or None if
/// no events are currently eligible.
#[frb(sync)]
pub fn api_get_current_event() -> Option<ApiDirectorEventView> {
    let mut guard = RUNTIME.lock().expect("GameRuntime poisoned");
    let runtime = &mut *guard;

    let view = select_next_event_view(&mut runtime.world, &mut runtime.sim, &runtime.storylets)?;
    Some(ApiDirectorEventView::from(view))
}

/// Process a player's choice and advance time.
///
/// Applies the selected choice's effects, advances the simulation by
/// `ticks_to_advance` ticks, and returns the next available event.
///
/// # Arguments
///
/// * `storylet_id` - ID of the current storylet
/// * `choice_id` - ID of the selected choice
/// * `ticks_to_advance` - Number of ticks to advance after applying the choice
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

/// Get player relationships snapshot via the global engine.
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

// ==================== Character Generation API ====================

/// Generate a character from seed and config.
/// Returns a fully procedurally generated character with stats, personality, and background.
#[frb(sync)]
pub fn engine_generate_character(
    world_seed: u64,
    name: String,
    archetype: String,
    difficulty: String,
    sfw_mode: bool,
) -> Option<ApiGeneratedCharacter> {
    let archetype_enum = CharacterArchetype::from_str(&archetype)?;
    let difficulty_enum = Difficulty::from_str(&difficulty)?;
    
    let config = CharacterGenConfig {
        name,
        archetype: archetype_enum,
        difficulty: difficulty_enum,
        sfw_mode,
    };
    
    let gen = generate_character(world_seed, &config);
    
    Some(ApiGeneratedCharacter {
        name: gen.name,
        archetype: gen.archetype.as_str().to_string(),
        difficulty: gen.difficulty.as_str().to_string(),
        sfw_mode: gen.sfw_mode,
        family_structure: format!("{:?}", gen.family_structure),
        socioeconomic_tier: format!("{:?}", gen.socioeconomic_tier),
        early_life_event: format!("{:?}", gen.early_life_event),
        attachment_style: format!("{:?}", gen.attachment_style),
        luck_seed: gen.luck_seed,
        starting_district: gen.starting_district,
        has_early_trauma: gen.has_early_trauma,
        stats: ApiStatsSnapshot {
            stats: ALL_STAT_KINDS
                .iter()
                .map(|kind| ApiStat {
                    kind: format!("{:?}", kind),
                    value: gen.stats.get(*kind),
                })
                .collect(),
            mood_band: format!("{:?}", gen.stats.mood_band()),
        },
        personality: ApiPersonalityVector {
            warmth: gen.personality.warmth,
            dominance: gen.personality.dominance,
            volatility: gen.personality.volatility,
            conscientiousness: gen.personality.conscientiousness,
            openness: gen.personality.openness,
        },
        starting_karma: gen.karma.0,
    })
}

/// Initialize game engine with a generated character.
/// Combines character generation and engine init into one call.
#[frb(sync)]
pub fn engine_init_with_character(
    world_seed: u64,
    name: String,
    archetype: String,
    difficulty: String,
    sfw_mode: bool,
) -> bool {
    // Parse enums
    let Some(archetype_enum) = CharacterArchetype::from_str(&archetype) else {
        return false;
    };
    let Some(difficulty_enum) = Difficulty::from_str(&difficulty) else {
        return false;
    };
    
    let config = CharacterGenConfig {
        name,
        archetype: archetype_enum,
        difficulty: difficulty_enum,
        sfw_mode,
    };
    
    let gen = generate_character(world_seed, &config);
    
    // Init the engine
    let mut engine = ENGINE.lock().unwrap();
    let mut game_engine = GameEngine::new(world_seed);
    
    // Apply generated stats to player
    game_engine.world.player_stats = gen.stats;
    game_engine.world.player_karma = gen.karma;
    game_engine.world.player_life_stage = LifeStage::PreSim; // Start at beginning (age 0-5)
    game_engine.world.player_age_years = 0;
    game_engine.world.player_age = 0;
    
    // Store attachment style (stored on player NPC)
    if let Some(player_npc) = game_engine.world.npcs.get_mut(&game_engine.world.player_id) {
        player_npc.attachment_style = gen.attachment_style;
    }
    
    *engine = Some(game_engine);
    true
}

/// Get difficulty modifiers for UI display.
#[frb(sync)]
pub fn get_difficulty_modifiers(difficulty: String) -> Option<(f32, f32)> {
    let diff = Difficulty::from_str(&difficulty)?;
    Some((diff.negative_modifier(), diff.positive_modifier()))
}

// ==================== District API ====================

/// Get all district summaries for list display.
#[frb(sync)]
pub fn engine_list_districts() -> Vec<ApiDistrictSummary> {
    let engine = ENGINE.lock().unwrap();
    engine
        .as_ref()
        .map(|e| {
            e.world
                .districts
                .districts
                .values()
                .map(ApiDistrictSummary::from)
                .collect()
        })
        .unwrap_or_default()
}

/// Get detailed snapshot of a specific district by name.
#[frb(sync)]
pub fn engine_get_district(name: String) -> Option<ApiDistrictSnapshot> {
    let engine = ENGINE.lock().unwrap();
    engine
        .as_ref()
        .and_then(|e| e.world.districts.get_by_name(&name))
        .map(ApiDistrictSnapshot::from)
}

/// Get detailed snapshot of a district by ID.
#[frb(sync)]
pub fn engine_get_district_by_id(id: u32) -> Option<ApiDistrictSnapshot> {
    let engine = ENGINE.lock().unwrap();
    engine
        .as_ref()
        .and_then(|e| e.world.districts.get(DistrictId(id)))
        .map(ApiDistrictSnapshot::from)
}

/// Get city-wide statistics.
#[frb(sync)]
pub fn engine_get_city_stats() -> ApiCityStats {
    let engine = ENGINE.lock().unwrap();
    engine
        .as_ref()
        .map(|e| {
            let registry = &e.world.districts;
            ApiCityStats {
                district_count: registry.len() as u32,
                average_economy: registry.average_economy(),
                average_crime: registry.average_crime(),
                safest_district: registry.safest().map(|d| d.name.clone()),
                most_dangerous_district: registry.most_dangerous().map(|d| d.name.clone()),
                wealthiest_district: registry.wealthiest().map(|d| d.name.clone()),
            }
        })
        .unwrap_or(ApiCityStats {
            district_count: 0,
            average_economy: 0.0,
            average_crime: 0.0,
            safest_district: None,
            most_dangerous_district: None,
            wealthiest_district: None,
        })
}

/// Get player's current district (from their NPC record).
#[frb(sync)]
pub fn engine_get_player_district() -> Option<ApiDistrictSnapshot> {
    let engine = ENGINE.lock().unwrap();
    let e = engine.as_ref()?;
    let player_npc = e.world.npcs.get(&e.world.player_id)?;
    e.world.districts.get_by_name(&player_npc.district).map(ApiDistrictSnapshot::from)
}

/// Apply an economic event to a district.
#[frb(sync)]
pub fn engine_apply_district_economic_event(district_name: String, delta: f32) {
    let mut engine = ENGINE.lock().unwrap();
    if let Some(ref mut e) = *engine {
        if let Some(district) = e.world.districts.get_by_name_mut(&district_name) {
            district.apply_economic_event(delta);
        }
    }
}

/// Apply a crime event to a district.
#[frb(sync)]
pub fn engine_apply_district_crime_event(district_name: String, delta: f32) {
    let mut engine = ENGINE.lock().unwrap();
    if let Some(ref mut e) = *engine {
        if let Some(district) = e.world.districts.get_by_name_mut(&district_name) {
            district.apply_crime_event(delta);
        }
    }
}

// ==================== Skills API ====================

/// Skill progress snapshot DTO for Flutter.
///
/// Tracks a player's progression in a specific skill.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSkillProgress {
    /// Skill identifier.
    pub id: String,
    /// Current XP accumulated.
    pub xp: u32,
    /// Current tier level (0=Novice, 1=Beginner, 2=Intermediate, 3=Advanced, 4=Expert, 5=Master).
    pub level: u8,
    /// Tier name label.
    pub tier_name: String,
    /// Progress to next tier (0.0-1.0).
    pub progress_to_next: f32,
    /// Total times practiced.
    pub practice_count: u32,
    /// Total failures encountered.
    pub failure_count: u32,
    /// Whether mastery (tier 5) was ever achieved.
    pub achieved_mastery: bool,
}

/// Skill definition DTO for Flutter.
///
/// Describes a skill's metadata and XP mechanics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSkillDefinition {
    /// Skill identifier.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Skill description.
    pub description: String,
    /// Skill category (e.g., "Physical", "Social", "Technical").
    pub category: String,
    /// Base XP rate multiplier.
    pub xp_rate: f32,
    /// Stats that provide XP bonuses.
    pub stat_affinities: Vec<String>,
    /// Whether skill can decay over time without practice.
    pub can_decay: bool,
}

/// Full player skills snapshot.
///
/// Summary of all player skill progress and achievements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiPlayerSkillsSnapshot {
    /// All skill progress entries.
    pub skills: Vec<ApiSkillProgress>,
    /// Count of skills at tier 1 or higher.
    pub total_skills_learned: u32,
    /// ID of the highest-tier skill, if any.
    pub highest_tier_skill: Option<String>,
    /// Storylet IDs unlocked by skill requirements.
    pub unlocked_storylets: Vec<String>,
}

/// Get all skill definitions from the registry.
#[frb(sync)]
pub fn engine_get_skill_definitions() -> Vec<ApiSkillDefinition> {
    use syn_core::skills::SkillRegistry;
    let registry = SkillRegistry::with_defaults();
    registry
        .skills
        .values()
        .map(|def| ApiSkillDefinition {
            id: def.id.0.clone(),
            name: def.name.clone(),
            description: def.description.clone(),
            category: format!("{:?}", def.category),
            xp_rate: def.xp_rate,
            stat_affinities: def.stat_affinities.iter().map(|s| format!("{:?}", s)).collect(),
            can_decay: def.can_decay,
        })
        .collect()
}

/// Get player's skill progress for a specific skill.
#[frb(sync)]
pub fn engine_get_skill(skill_id: String) -> Option<ApiSkillProgress> {
    use syn_core::skills::SkillId;
    let engine = ENGINE.lock().unwrap();
    engine.as_ref().and_then(|e| {
        let skill_id = SkillId::new(&skill_id);
        e.world.player_skills.get(&skill_id).map(|p| ApiSkillProgress {
            id: skill_id.0.clone(),
            xp: p.xp,
            level: p.level(),
            tier_name: format!("{:?}", p.tier()),
            progress_to_next: p.progress_to_next_tier(),
            practice_count: p.practice_count,
            failure_count: p.failure_count,
            achieved_mastery: p.achieved_mastery,
        })
    })
}

/// Get all player skill progress.
#[frb(sync)]
pub fn engine_get_player_skills() -> ApiPlayerSkillsSnapshot {
    use syn_core::skills::SkillRegistry;
    let engine = ENGINE.lock().unwrap();
    engine
        .as_ref()
        .map(|e| {
            let registry = SkillRegistry::with_defaults();
            let skills: Vec<ApiSkillProgress> = e
                .world
                .player_skills
                .skills
                .iter()
                .map(|(id, p)| ApiSkillProgress {
                    id: id.0.clone(),
                    xp: p.xp,
                    level: p.level(),
                    tier_name: format!("{:?}", p.tier()),
                    progress_to_next: p.progress_to_next_tier(),
                    practice_count: p.practice_count,
                    failure_count: p.failure_count,
                    achieved_mastery: p.achieved_mastery,
                })
                .collect();

            let learned = skills.iter().filter(|s| s.level >= 1).count() as u32;
            let highest = skills
                .iter()
                .max_by_key(|s| s.level)
                .filter(|s| s.level >= 1)
                .map(|s| s.id.clone());
            let unlocked = e.world.player_skills.get_unlocked_storylets(&registry);

            ApiPlayerSkillsSnapshot {
                skills,
                total_skills_learned: learned,
                highest_tier_skill: highest,
                unlocked_storylets: unlocked,
            }
        })
        .unwrap_or(ApiPlayerSkillsSnapshot {
            skills: vec![],
            total_skills_learned: 0,
            highest_tier_skill: None,
            unlocked_storylets: vec![],
        })
}

/// Practice a skill, granting XP. Returns the new progress or None if skill not found.
#[frb(sync)]
pub fn engine_practice_skill(skill_id: String, base_xp: u32, succeeded: bool) -> Option<ApiSkillProgress> {
    use syn_core::skills::{SkillId, SkillRegistry};
    let mut engine = ENGINE.lock().unwrap();
    let e = engine.as_mut()?;
    
    let registry = SkillRegistry::with_defaults();
    let skill_id = SkillId::new(&skill_id);
    let current_tick = e.world.current_tick.0;
    
    // Calculate XP modifier based on skill definition and player stats
    let modifier = registry
        .get(&skill_id)
        .map(|def| def.calculate_xp_modifier(&e.world.player_stats))
        .unwrap_or(1.0);
    let modified_xp = (base_xp as f32 * modifier).round() as u32;
    
    let progress = e.world.player_skills.get_or_create_mut(&skill_id);
    if succeeded {
        progress.add_xp(modified_xp, current_tick);
    } else {
        progress.add_failure_xp(modified_xp, current_tick);
    }
    
    Some(ApiSkillProgress {
        id: skill_id.0.clone(),
        xp: progress.xp,
        level: progress.level(),
        tier_name: format!("{:?}", progress.tier()),
        progress_to_next: progress.progress_to_next_tier(),
        practice_count: progress.practice_count,
        failure_count: progress.failure_count,
        achieved_mastery: progress.achieved_mastery,
    })
}

/// Check if player meets skill requirements for a storylet.
#[frb(sync)]
pub fn engine_check_skill_requirements(skill_id: String, min_tier: Option<u8>, min_xp: Option<u32>) -> bool {
    use syn_core::skills::SkillId;
    let engine = ENGINE.lock().unwrap();
    engine.as_ref().map(|e| {
        let skill_id = SkillId::new(&skill_id);
        let tier = e.world.player_skills.get_tier(&skill_id);
        let xp = e.world.player_skills.get_xp(&skill_id);
        
        let tier_ok = min_tier.map(|min| tier.as_level() >= min).unwrap_or(true);
        let xp_ok = min_xp.map(|min| xp >= min).unwrap_or(true);
        
        tier_ok && xp_ok
    }).unwrap_or(false)
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

    #[test]
    fn test_character_generation_api() {
        let result = engine_generate_character(
            12345,
            "TestPlayer".to_string(),
            "STORYTELLER".to_string(),
            "BALANCED".to_string(),
            true,
        );
        
        let char = result.expect("should generate character");
        assert_eq!(char.name, "TestPlayer");
        assert_eq!(char.archetype, "STORYTELLER");
        assert_eq!(char.difficulty, "BALANCED");
        assert!(char.sfw_mode);
        
        // Verify stats are present
        assert!(!char.stats.stats.is_empty());
        
        // Verify personality is populated
        assert!(char.personality.warmth >= -1.0 && char.personality.warmth <= 1.0);
        assert!(char.personality.openness >= 0.0 && char.personality.openness <= 1.0);
    }

    #[test]
    fn test_character_generation_deterministic() {
        let char1 = engine_generate_character(
            99999,
            "Alice".to_string(),
            "CHALLENGER".to_string(),
            "HARSH".to_string(),
            false,
        ).unwrap();
        
        let char2 = engine_generate_character(
            99999,
            "Alice".to_string(),
            "CHALLENGER".to_string(),
            "HARSH".to_string(),
            false,
        ).unwrap();
        
        // Same seed + config = identical output
        assert_eq!(char1.family_structure, char2.family_structure);
        assert_eq!(char1.socioeconomic_tier, char2.socioeconomic_tier);
        assert_eq!(char1.luck_seed, char2.luck_seed);
        assert_eq!(char1.starting_karma, char2.starting_karma);
    }

    #[test]
    fn test_invalid_archetype_returns_none() {
        let result = engine_generate_character(
            42,
            "Test".to_string(),
            "INVALID".to_string(),
            "BALANCED".to_string(),
            true,
        );
        assert!(result.is_none());
    }

    #[test]
    fn test_init_with_character() {
        // Clear any existing engine state
        let mut engine = ENGINE.lock().unwrap();
        *engine = None;
        drop(engine);
        
        let success = engine_init_with_character(
            54321,
            "GamePlayer".to_string(),
            "ANALYST".to_string(),
            "FORGIVING".to_string(),
            true,
        );
        
        assert!(success);
        
        // Verify engine was initialized
        let engine = ENGINE.lock().unwrap();
        assert!(engine.is_some());
        let e = engine.as_ref().unwrap();
        assert_eq!(e.world_seed(), 54321);
    }

    #[test]
    fn test_district_api_list() {
        // Clear and init engine
        let mut engine = ENGINE.lock().unwrap();
        *engine = None;
        drop(engine);
        
        init_engine(42);
        
        let districts = engine_list_districts();
        assert!(!districts.is_empty());
        assert_eq!(districts.len(), 10); // Default city has 10 districts
        
        // Check we have expected districts
        let names: Vec<&str> = districts.iter().map(|d| d.name.as_str()).collect();
        assert!(names.contains(&"Downtown"));
        assert!(names.contains(&"Highland Heights"));
    }

    #[test]
    fn test_district_api_get_by_name() {
        // Clear and init engine
        let mut engine = ENGINE.lock().unwrap();
        *engine = None;
        drop(engine);
        
        init_engine(42);
        
        let district = engine_get_district("Downtown".to_string());
        assert!(district.is_some());
        let d = district.unwrap();
        assert_eq!(d.name, "Downtown");
        assert_eq!(d.district_type, "Downtown");
        // Downtown type gives +15 economy modifier, so baseline is 65
        // With random variation ±15, range is 50-80
        assert!(d.economy >= 40.0 && d.economy <= 90.0);
    }

    #[test]
    fn test_district_api_city_stats() {
        // Clear and init engine
        let mut engine = ENGINE.lock().unwrap();
        *engine = None;
        drop(engine);
        
        init_engine(42);
        
        let stats = engine_get_city_stats();
        assert_eq!(stats.district_count, 10);
        assert!(stats.average_economy > 0.0);
        assert!(stats.safest_district.is_some());
        assert!(stats.wealthiest_district.is_some());
    }

    #[test]
    fn test_district_api_economic_event() {
        // Clear and init engine
        let mut engine = ENGINE.lock().unwrap();
        *engine = None;
        drop(engine);
        
        init_engine(42);
        
        // Get initial economy
        let before = engine_get_district("Downtown".to_string()).unwrap();
        let initial_economy = before.economy;
        
        // Apply economic crash
        engine_apply_district_economic_event("Downtown".to_string(), -20.0);
        
        // Verify economy dropped
        let after = engine_get_district("Downtown".to_string()).unwrap();
        assert!(after.economy < initial_economy);
    }
}
