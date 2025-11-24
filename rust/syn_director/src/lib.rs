//! syn_director: Event orchestration and storylet selection.
//!
//! Central narrative brain: evaluates world state, personality vectors, and relationship
//! pressures to select and fire emergent storylets.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use syn_core::npc::NpcActivityKind;
use syn_core::npc::NpcRoleTag;
use syn_core::npc_behavior::{BehaviorKind, BehaviorSnapshot};
use syn_core::time::DayPhase;
use syn_core::{
    apply_stat_deltas, behavior_action_from_tags, deterministic_rng_from_world,
    narrative_heat::NarrativeHeatBand,
    relationship_milestones::RelationshipMilestoneEvent,
    relationship_model::{
        AffectionBand, AttractionBand, RelationshipAxis as ModelRelationshipAxis, RelationshipDelta, RelationshipVector,
        ResentmentBand, TrustBand,
    },
    relationship_pressure::{RelationshipEventKind, RelationshipPressureEvent},
    LifeStage, NpcId, RelationshipAxis as CoreRelationshipAxis, RelationshipState, SimTick, StatDelta, StoryletUsageState, WorldState,
};
use syn_memory::{MemoryEntry, MemorySystem};
use syn_query::RelationshipQuery;
use syn_sim::{tick_world, NpcRegistry, SimState};

pub mod storylet_library;
pub mod storylet_roles;
pub mod storylet_outcome;
pub mod event_director;
pub mod tag_bitset;
pub mod storylet_loader;
pub use storylet_library::{EventContext, StoryletId, StoryletLibrary, tags_to_bitset};
pub use tag_bitset::TagBitset;
pub use storylet_outcome::{MemoryEntryTemplate, StoryletOutcomeSet, WorldFlagUpdate};
pub use storylet_roles::{RoleAssignment, RoleScoring, RoleSlot, StoryletRoles};

pub type StoryletPrereqs = StoryletPrerequisites;

/// Trigger metadata for a storylet (placeholder, GDD 3.16.1).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StoryletTrigger {
    #[serde(default)]
    pub kind: Option<String>,
}

/// Cooldown wrapper for storylets.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct StoryletCooldown {
    #[serde(default)]
    pub ticks: u32,
}

/// Outcome set for a storylet (choices + metadata).
/// A storylet: condition-driven narrative fragment with roles, outcomes, and cooldowns.
#[derive(Debug, Clone, Serialize)]
pub struct Storylet {
    pub id: StoryletId,
    #[serde(default)]
    pub name: String,
    pub tags: TagBitset,
    pub prerequisites: StoryletPrereqs,
    pub roles: StoryletRoles,
    pub heat: i32,
    #[serde(default)]
    pub triggers: StoryletTrigger,
    #[serde(default)]
    pub outcomes: StoryletOutcomeSet,
    #[serde(default)]
    pub cooldown: StoryletCooldown,
    pub weight: f32,
}

impl Storylet {
    pub fn new(
        id: StoryletId,
        tags: TagBitset,
        prerequisites: StoryletPrereqs,
        roles: StoryletRoles,
        heat: i32,
        triggers: StoryletTrigger,
        outcomes: StoryletOutcomeSet,
        cooldown: StoryletCooldown,
        weight: f32,
    ) -> Self {
        Storylet {
            id,
            name: String::new(),
            tags,
            prerequisites,
            roles,
            heat,
            triggers,
            outcomes,
            cooldown,
            weight,
        }
    }

    pub fn matches(&self, ctx: &EventContext) -> bool {
        ctx.required_tags.is_empty() || (self.tags & ctx.required_tags) == ctx.required_tags
    }
}

impl Default for Storylet {
    fn default() -> Self {
        Storylet::new(
            String::new(),
            TagBitset::default(),
            StoryletPrereqs::default(),
            StoryletRoles::default(),
            0,
            StoryletTrigger::default(),
            StoryletOutcomeSet::default(),
            StoryletCooldown::default(),
            1.0,
        )
    }
}

impl<'de> Deserialize<'de> for Storylet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let helper = storylet_loader::StoryletSerde::deserialize(deserializer)?;
        Ok(helper.into())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StoryletHeatCategory {
    SliceOfLife,
    RisingTension,
    HighDrama,
    CriticalArc,
}

/// High-level tone hint for storylet interactions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InteractionTone {
    Support,
    Conflict,
    Attention,
    Withdrawal,
    Stability,
}

/// A choice within a storylet presented to the player.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryletChoice {
    pub id: String,
    pub label: String,
    pub outcome: StoryletOutcome,
}

/// Relationship-based prerequisite (additive, non-breaking).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipPrereq {
    /// Which actor owns the relationship. None defaults to the player.
    #[serde(default)]
    pub actor_id: Option<u64>,
    /// Target NPC the prereq references.
    pub target_id: u64,
    /// Relationship axis to inspect.
    pub axis: ModelRelationshipAxis,
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

/// Digital legacy prerequisite for PostLife storylets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigitalLegacyPrereq {
    /// Only relevant in Digital/PostLife; if true and stage != Digital, prereq fails.
    #[serde(default)]
    pub require_post_life: bool,

    /// Optional bounds on legacy components (-1.0 .. 1.0)
    #[serde(default)]
    pub min_compassion_vs_cruelty: Option<f32>,
    #[serde(default)]
    pub max_compassion_vs_cruelty: Option<f32>,

    #[serde(default)]
    pub min_ambition_vs_comfort: Option<f32>,
    #[serde(default)]
    pub max_ambition_vs_comfort: Option<f32>,

    #[serde(default)]
    pub min_connection_vs_isolation: Option<f32>,
    #[serde(default)]
    pub max_connection_vs_isolation: Option<f32>,

    #[serde(default)]
    pub min_stability_vs_chaos: Option<f32>,
    #[serde(default)]
    pub max_stability_vs_chaos: Option<f32>,

    #[serde(default)]
    pub min_light_vs_shadow: Option<f32>,
    #[serde(default)]
    pub max_light_vs_shadow: Option<f32>,
}

/// Conditions that must be met for a storylet to be eligible.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StoryletPrerequisites {
    #[serde(default)]
    pub stat_conditions: Vec<StatCondition>,
    #[serde(default)]
    pub personality_conditions: Vec<PersonalityCondition>,
    #[serde(default)]
    pub relationship_conditions: Vec<RelationshipThreshold>,
    #[serde(default)]
    pub district_conditions: Vec<DistrictCondition>,
    #[serde(default)]
    pub memory_echo_conditions: Vec<MemoryEchoFlag>,
    #[serde(default)]
    pub global_conditions: Vec<GlobalWorldStateFlag>,
    #[serde(default)]
    pub life_stage: Option<LifeStage>,

    pub min_relationship_affection: Option<f32>,
    pub min_relationship_resentment: Option<f32>,
    #[serde(default)]
    pub stat_ranges: HashMap<String, (f32, f32)>, // legacy range-based checks
    pub life_stages: Vec<String>,                     // ["Teen", "Adult", "Elder"]
    pub tags: Vec<String>,                            // must have these tags
    pub relationship_states: Vec<RelationshipState>, // Only fire if relationship is in one of these states
    // Memory prerequisites for event echoes
    pub memory_tags_required: Vec<String>, // NPC must have memory with at least one of these tags
    pub memory_tags_forbidden: Vec<String>, // NPC must NOT have memory with these tags (conflict avoidance)
    pub memory_recency_ticks: Option<u64>, // If specified, memory must be within N ticks (default: 7 days = 168 ticks)
    /// Optional relationship-based prerequisites (additive).
    #[serde(default)]
    pub relationship_prereqs: Vec<RelationshipPrereq>,
    /// Optional allowed life stages for this storylet.
    #[serde(default)]
    pub allowed_life_stages: Vec<LifeStage>,
    /// Optional digital legacy prerequisite for PostLife storylets.
    #[serde(default)]
    pub digital_legacy_prereq: Option<DigitalLegacyPrereq>,

    /// Optional time/location gating aligned with NPC schedule.
    #[serde(default)]
    pub time_and_location: Option<TimeAndLocationPrereqs>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StatCondition {
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub min: f32,
    #[serde(default)]
    pub max: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PersonalityCondition {
    #[serde(default)]
    pub trait_name: String,
    #[serde(default)]
    pub min: f32,
    #[serde(default)]
    pub max: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RelationshipThreshold {
    #[serde(default)]
    pub axis: String,
    #[serde(default)]
    pub min: f32,
    #[serde(default)]
    pub max: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DistrictCondition {
    #[serde(default)]
    pub district: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryEchoFlag {
    #[serde(default)]
    pub tag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GlobalWorldStateFlag {
    #[serde(default)]
    pub flag: String,
    #[serde(default)]
    pub value: bool,
}

impl StoryletPrerequisites {
    pub fn passes(&self, _ctx: &EventContext) -> bool {
        true
    }
}

/// Optional time/location prerequisites for storylets.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TimeAndLocationPrereqs {
    /// Allowed day phases for this storylet (if empty: any).
    #[serde(default)]
    pub allowed_phases: Vec<DayPhase>,
    /// Required NPC activity kinds for primary actor (if any).
    #[serde(default)]
    pub allowed_npc_activities: Vec<NpcActivityKind>,
}

/// A role in a storylet (e.g., "target", "rival", "manager").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryletRole {
    pub name: String,
    pub npc_id: NpcId,
}

/// Storylet actor reference used by Director to locate/focus NPCs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StoryActorRef {
    /// Direct NPC id; rarely used in content, more in system-authored events.
    NpcId(u64),

    /// By role tag (e.g. “closest Friend with this role”).
    RoleTag(NpcRoleTag),

    /// Player (for clarity).
    Player,
}

/// Optional actors involved in this storylet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryletActors {
    #[serde(default)]
    pub primary: Option<StoryActorRef>,

    #[serde(default)]
    pub secondary: Option<StoryActorRef>,
}

/// For now, keep this simple:
/// - RoleTag => pick some NPC with that role tag known to the player.
/// - NpcId(u64) => wrap into NpcId.
pub fn resolve_actor_ref_to_npc(
    world: &WorldState,
    registry: &NpcRegistry,
    actor_ref: &StoryActorRef,
) -> Option<NpcId> {
    match actor_ref {
        StoryActorRef::Player => None,
        StoryActorRef::NpcId(id) => {
            let npc_id = NpcId(*id);
            if npc_is_available_for_player(world, registry, npc_id) {
                Some(npc_id)
            } else {
                None
            }
        }
        StoryActorRef::RoleTag(tag) => {
            for npc_id in &world.known_npcs {
                if let Some(proto) = world.npc_prototype(*npc_id) {
                    if proto.role_tags.contains(tag) {
                        let id = *npc_id;
                        if npc_is_available_for_player(world, registry, id) {
                            return Some(id);
                        }
                    }
                }
            }
            None
        }
    }
}

/// Is this NPC available to share a scene with the player right now?
/// Simple rule: offscreen / online-only may only work for certain storylets.
fn npc_is_available_for_player(world: &WorldState, registry: &NpcRegistry, npc_id: NpcId) -> bool {
    if let Some(inst) = registry.get(npc_id) {
        match inst.current_activity {
            NpcActivityKind::Offscreen => false,
            NpcActivityKind::Nightlife => {
                matches!(world.game_time.phase, DayPhase::Evening | DayPhase::Night)
            }
            NpcActivityKind::OnlineOnly => true,
            _ => true,
        }
    } else {
        // If NPC not instantiated yet, allow Director to spawn them
        true
    }
}

/// Check time and NPC location/activity prerequisites against current world/registry state.
fn check_time_and_location_prereqs(
    world: &WorldState,
    registry: &NpcRegistry,
    storylet: &Storylet,
) -> bool {
    let Some(pr) = &storylet.prerequisites.time_and_location else {
        return true;
    };

    // Phase gating
    if !pr.allowed_phases.is_empty() && !pr.allowed_phases.contains(&world.game_time.phase) {
        return false;
    }

    // NPC activity gating (if we have an NPC actor)
    if pr.allowed_npc_activities.is_empty() {
        return true;
    }

    let Some(actors) = &storylet.outcomes.actors else {
        return true;
    };
    if let Some(ref primary) = actors.primary {
        if let Some(npc_id) = resolve_actor_ref_to_npc(world, registry, primary) {
            if let Some(inst) = registry.get(npc_id) {
                return pr.allowed_npc_activities.contains(&inst.current_activity);
            }
        }
    }
    true
}

/// Internal: access an NPC's current behavior snapshot from the registry.
fn get_npc_behavior<'a>(registry: &'a NpcRegistry, npc_id: NpcId) -> Option<&'a BehaviorSnapshot> {
    registry.get(npc_id)?.behavior.as_ref()
}

fn tone_matches_behavior(tone: &InteractionTone, beh: &BehaviorKind) -> bool {
    match (tone, beh) {
        (InteractionTone::Support, BehaviorKind::SeekSocial) => true,
        (InteractionTone::Attention, BehaviorKind::SeekRecognition) => true,
        (InteractionTone::Conflict, BehaviorKind::SeekAutonomy) => true,
        (InteractionTone::Withdrawal, BehaviorKind::SeekComfort) => true,
        (InteractionTone::Stability, BehaviorKind::SeekSecurity) => true,
        _ => false,
    }
}

/// Public helper: compute an intent-based score multiplier for a storylet.
/// Additive and side-effect free.
pub fn npc_intent_score_multiplier(
    world: &WorldState,
    registry: &NpcRegistry,
    storylet: &Storylet,
) -> f32 {
    let tone = match &storylet.outcomes.interaction_tone {
        Some(t) => t,
        None => return 1.0,
    };

    let Some(actors) = &storylet.outcomes.actors else {
        return 1.0;
    };

    if let Some(primary_ref) = &actors.primary {
        if let Some(npc_id) = resolve_actor_ref_to_npc(world, registry, primary_ref) {
            if let Some(snapshot) = get_npc_behavior(registry, npc_id) {
                return if tone_matches_behavior(tone, &snapshot.chosen_intent.kind) {
                    1.3
                } else {
                    0.9
                }
            }
        }
    }

    1.0
}

/// Prepare storylet execution by focusing relevant NPCs in the simulation registry.
pub fn prepare_storylet_execution(
    world: &mut WorldState,
    registry: &mut NpcRegistry,
    storylet: &Storylet,
    tick: u64,
) {
    if let Some(actors) = &storylet.outcomes.actors {
        if let Some(ref primary) = actors.primary {
            if let Some(npc_id) = resolve_actor_ref_to_npc(world, registry, primary) {
                registry.focus_npc_for_scene(world, npc_id, tick);
                world.ensure_npc_known(npc_id);
            }
        }
        if let Some(ref secondary) = actors.secondary {
            if let Some(npc_id) = resolve_actor_ref_to_npc(world, registry, secondary) {
                registry.focus_npc_for_scene(world, npc_id, tick);
                world.ensure_npc_known(npc_id);
            }
        }
    }
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

fn affection_band_rank(band: AffectionBand) -> u8 {
    match band {
        AffectionBand::Stranger => 0,
        AffectionBand::Acquaintance => 1,
        AffectionBand::Friendly => 2,
        AffectionBand::Close => 3,
        AffectionBand::Devoted => 4,
    }
}

fn trust_band_rank(band: TrustBand) -> u8 {
    match band {
        TrustBand::Unknown => 0,
        TrustBand::Wary => 1,
        TrustBand::Neutral => 2,
        TrustBand::Trusted => 3,
        TrustBand::DeepTrust => 4,
    }
}

fn attraction_band_rank(band: AttractionBand) -> u8 {
    match band {
        AttractionBand::None => 0,
        AttractionBand::Curious => 1,
        AttractionBand::Interested => 2,
        AttractionBand::Strong => 3,
        AttractionBand::Intense => 4,
    }
}

fn resentment_band_rank(band: ResentmentBand) -> u8 {
    match band {
        ResentmentBand::None => 0,
        ResentmentBand::Irritated => 1,
        ResentmentBand::Resentful => 2,
        ResentmentBand::Hostile => 3,
        ResentmentBand::Vindictive => 4,
    }
}

fn familiarity_band_rank(value: f32) -> u8 {
    if value <= -5.0 {
        0
    } else if value < 1.0 {
        1
    } else if value < 5.0 {
        2
    } else if value < 8.0 {
        3
    } else {
        4
    }
}

fn band_rank_for(axis: ModelRelationshipAxis, rel: &RelationshipVector) -> u8 {
    match axis {
        ModelRelationshipAxis::Affection => affection_band_rank(rel.affection_band()),
        ModelRelationshipAxis::Trust => trust_band_rank(rel.trust_band()),
        ModelRelationshipAxis::Attraction => attraction_band_rank(rel.attraction_band()),
        ModelRelationshipAxis::Familiarity => familiarity_band_rank(rel.familiarity),
        ModelRelationshipAxis::Resentment => resentment_band_rank(rel.resentment_band()),
    }
}

fn band_rank_from_name(axis: ModelRelationshipAxis, name: &str) -> Option<u8> {
    let lowered = name.to_ascii_lowercase();
    match axis {
        ModelRelationshipAxis::Affection | ModelRelationshipAxis::Familiarity => {
            Some(affection_band_rank(match lowered.as_str() {
                "stranger" => AffectionBand::Stranger,
                "acquaintance" => AffectionBand::Acquaintance,
                "friendly" => AffectionBand::Friendly,
                "close" => AffectionBand::Close,
                "devoted" => AffectionBand::Devoted,
                _ => AffectionBand::Stranger,
            }))
        }
        ModelRelationshipAxis::Trust => Some(trust_band_rank(match lowered.as_str() {
            "unknown" => TrustBand::Unknown,
            "wary" => TrustBand::Wary,
            "neutral" => TrustBand::Neutral,
            "trusted" => TrustBand::Trusted,
            "deeptrust" | "deep_trust" | "deep trust" => TrustBand::DeepTrust,
            _ => TrustBand::Unknown,
        })),
        ModelRelationshipAxis::Attraction => Some(attraction_band_rank(match lowered.as_str() {
            "none" => AttractionBand::None,
            "curious" => AttractionBand::Curious,
            "interested" => AttractionBand::Interested,
            "strong" => AttractionBand::Strong,
            "intense" => AttractionBand::Intense,
            _ => AttractionBand::None,
        })),
        ModelRelationshipAxis::Resentment => Some(resentment_band_rank(match lowered.as_str() {
            "none" => ResentmentBand::None,
            "irritated" => ResentmentBand::Irritated,
            "resentful" => ResentmentBand::Resentful,
            "hostile" => ResentmentBand::Hostile,
            "vindictive" => ResentmentBand::Vindictive,
            _ => ResentmentBand::None,
        })),
    }
}

fn check_relationship_prereqs(
    world: &WorldState,
    prereqs: &[RelationshipPrereq],
    default_actor_id: NpcId,
) -> bool {
    for prereq in prereqs {
        let actor = NpcId(prereq.actor_id.unwrap_or(default_actor_id.0));
        let target = NpcId(prereq.target_id);

        let rel = match world.relationships.get(&(actor, target)) {
            Some(r) => r,
            None => return false,
        };

        let rel_vec = RelationshipVector {
            affection: rel.affection,
            trust: rel.trust,
            attraction: rel.attraction,
            familiarity: rel.familiarity,
            resentment: rel.resentment,
        };

        let value = rel_vec.get(prereq.axis);
        if let Some(min_v) = prereq.min_value {
            if value < min_v {
                return false;
            }
        }
        if let Some(max_v) = prereq.max_value {
            if value > max_v {
                return false;
            }
        }

        let band_rank = band_rank_for(prereq.axis, &rel_vec);
        if let Some(ref min_band_name) = prereq.min_band {
            if let Some(min_rank) = band_rank_from_name(prereq.axis, min_band_name) {
                if band_rank < min_rank {
                    return false;
                }
            }
        }
        if let Some(ref max_band_name) = prereq.max_band {
            if let Some(max_rank) = band_rank_from_name(prereq.axis, max_band_name) {
                if band_rank > max_rank {
                    return false;
                }
            }
        }
    }

    true
}

fn check_life_stage_prereqs(world: &WorldState, pre: &StoryletPrerequisites) -> bool {
    if pre.allowed_life_stages.is_empty() {
        return true;
    }
    pre.allowed_life_stages.contains(&world.player_life_stage)
}

fn check_digital_legacy_prereq(world: &WorldState, pre: &Option<DigitalLegacyPrereq>) -> bool {
    let Some(pre) = pre else {
        return true;
    };

    // If require_post_life is true, and we're not in Digital, fail.
    if pre.require_post_life && !matches!(world.player_life_stage, LifeStage::Digital) {
        return false;
    }

    let imprint = match &world.digital_legacy.primary_imprint {
        Some(i) => i,
        None => {
            // No imprint yet; if we require any bounds, fail.
            return false;
        }
    };

    let lv = &imprint.legacy_vector;

    let between = |v: f32, min: &Option<f32>, max: &Option<f32>| {
        if let Some(m) = min {
            if v < *m {
                return false;
            }
        }
        if let Some(m) = max {
            if v > *m {
                return false;
            }
        }
        true
    };

    between(
        lv.compassion_vs_cruelty,
        &pre.min_compassion_vs_cruelty,
        &pre.max_compassion_vs_cruelty,
    ) && between(
        lv.ambition_vs_comfort,
        &pre.min_ambition_vs_comfort,
        &pre.max_ambition_vs_comfort,
    ) && between(
        lv.connection_vs_isolation,
        &pre.min_connection_vs_isolation,
        &pre.max_connection_vs_isolation,
    ) && between(
        lv.stability_vs_chaos,
        &pre.min_stability_vs_chaos,
        &pre.max_stability_vs_chaos,
    ) && between(
        lv.light_vs_shadow,
        &pre.min_light_vs_shadow,
        &pre.max_light_vs_shadow,
    )
}

fn memory_tags_for_pair(memory: &MemorySystem, actor_id: u64, target_id: u64) -> Vec<String> {
    memory
        .journals
        .values()
        .flat_map(|journal| journal.entries.iter())
        .filter(|m| {
            let has_actor = m.participants.contains(&actor_id);
            let has_target = m.participants.contains(&target_id);
            has_actor && has_target
        })
        .flat_map(|m| m.tags.clone())
        .collect()
}

fn storylet_targets_pair(
    pre: &StoryletPrerequisites,
    actor_id: u64,
    target_id: u64,
    default_actor_id: u64,
) -> bool {
    for r in &pre.relationship_prereqs {
        let a = r.actor_id.unwrap_or(default_actor_id);
        if a == actor_id && r.target_id == target_id {
            return true;
        }
    }
    false
}

fn storylet_matches_pressure_kind(
    pre: &StoryletPrerequisites,
    event: &RelationshipPressureEvent,
) -> bool {
    use RelationshipEventKind::*;

    let axis = match event.kind {
        AffectionBandChanged => Some(ModelRelationshipAxis::Affection),
        TrustBandChanged => Some(ModelRelationshipAxis::Trust),
        AttractionBandChanged => Some(ModelRelationshipAxis::Attraction),
        ResentmentBandChanged => Some(ModelRelationshipAxis::Resentment),
    };

    axis.map(|axis| pre.relationship_prereqs.iter().any(|r| r.axis == axis))
        .unwrap_or(false)
}

fn score_storylet_with_pressure(
    director: &EventDirector,
    world: &WorldState,
    storylet: &Storylet,
    hot_event: Option<&RelationshipPressureEvent>,
) -> f32 {
    let mut score = director.score_storylet(storylet, world);

    let pre = &storylet.prerequisites;
    let default_actor_id = world.player_id.0;

    if let Some(event) = hot_event {
        if storylet_targets_pair(pre, event.actor_id, event.target_id, default_actor_id) {
            score += 50.0;

            if storylet_matches_pressure_kind(pre, event) {
                score += 25.0;
            }
        }
    }

    score
}

fn storylet_heat_band_match(heat_band: NarrativeHeatBand, storylet: &Storylet) -> bool {
    let Some(category) = &storylet.outcomes.heat_category else {
        return true;
    };

    match (heat_band, category) {
        (NarrativeHeatBand::Low, StoryletHeatCategory::SliceOfLife) => true,
        (NarrativeHeatBand::Medium, StoryletHeatCategory::RisingTension) => true,
        (NarrativeHeatBand::High, StoryletHeatCategory::HighDrama) => true,
        (NarrativeHeatBand::Critical, StoryletHeatCategory::CriticalArc) => true,
        (NarrativeHeatBand::Medium, StoryletHeatCategory::SliceOfLife) => true,
        (NarrativeHeatBand::High, StoryletHeatCategory::RisingTension) => true,
        (NarrativeHeatBand::Critical, StoryletHeatCategory::HighDrama) => true,
        _ => false,
    }
}

fn heat_score_multiplier(heat_band: NarrativeHeatBand, storylet: &Storylet) -> f32 {
    let Some(category) = &storylet.outcomes.heat_category else {
        return 1.0;
    };

    match (heat_band, category) {
        (NarrativeHeatBand::Low, StoryletHeatCategory::SliceOfLife) => 1.3,
        (NarrativeHeatBand::Medium, StoryletHeatCategory::RisingTension) => 1.4,
        (NarrativeHeatBand::High, StoryletHeatCategory::HighDrama) => 1.5,
        (NarrativeHeatBand::Critical, StoryletHeatCategory::CriticalArc) => 1.7,
        (NarrativeHeatBand::Medium, StoryletHeatCategory::SliceOfLife) => 0.9,
        (NarrativeHeatBand::High, StoryletHeatCategory::RisingTension) => 1.1,
        (NarrativeHeatBand::Critical, StoryletHeatCategory::HighDrama) => 1.2,
        (NarrativeHeatBand::Low, StoryletHeatCategory::HighDrama) => 0.4,
        (NarrativeHeatBand::Low, StoryletHeatCategory::CriticalArc) => 0.2,
        (NarrativeHeatBand::Medium, StoryletHeatCategory::CriticalArc) => 0.5,
        (NarrativeHeatBand::High, StoryletHeatCategory::SliceOfLife) => 0.7,
        _ => 1.0,
    }
}

fn life_stage_score_multiplier(world: &WorldState, pre: &StoryletPrerequisites) -> f32 {
    if pre.allowed_life_stages.is_empty() {
        return 1.0;
    }
    if pre.allowed_life_stages.contains(&world.player_life_stage) {
        1.2
    } else {
        0.8
    }
}

fn digital_legacy_score_multiplier(world: &WorldState, pre: &Option<DigitalLegacyPrereq>) -> f32 {
    let Some(pre) = pre else {
        return 1.0;
    };

    if !matches!(world.player_life_stage, LifeStage::Digital) {
        return 1.0;
    }

    if world.digital_legacy.primary_imprint.is_none() {
        return 1.0;
    }

    // For now: if we have any bound at all, and prereq passes, give a small boost.
    // (We already checked prereqs in find_eligible.)
    let has_any_bounds = pre.min_compassion_vs_cruelty.is_some()
        || pre.max_compassion_vs_cruelty.is_some()
        || pre.min_ambition_vs_comfort.is_some()
        || pre.max_ambition_vs_comfort.is_some()
        || pre.min_connection_vs_isolation.is_some()
        || pre.max_connection_vs_isolation.is_some()
        || pre.min_stability_vs_chaos.is_some()
        || pre.max_stability_vs_chaos.is_some()
        || pre.min_light_vs_shadow.is_some()
        || pre.max_light_vs_shadow.is_some();

    if has_any_bounds {
        1.25
    } else {
        1.0
    }
}

fn score_storylet_full(
    director: &EventDirector,
    world: &WorldState,
    storylet: &Storylet,
    hot_event: Option<&RelationshipPressureEvent>,
) -> f32 {
    let base = score_storylet_with_pressure(director, world, storylet, hot_event);
    let heat_band = world.narrative_heat.band();
    let heat_mult = heat_score_multiplier(heat_band, storylet);
    let stage_mult = life_stage_score_multiplier(world, &storylet.prerequisites);
    let legacy_mult =
        digital_legacy_score_multiplier(world, &storylet.prerequisites.digital_legacy_prereq);
    let mut score = base * heat_mult * stage_mult * legacy_mult;
    if storylet.outcomes.heat_category.is_some() && !storylet_heat_band_match(heat_band, storylet) {
        score *= 0.5;
    }
    score
}

/// Variant scoring that considers NPC intent via the registry.
pub fn score_storylet_full_with_registry(
    director: &EventDirector,
    world: &WorldState,
    registry: &NpcRegistry,
    storylet: &Storylet,
    hot_event: Option<&RelationshipPressureEvent>,
) -> f32 {
    // If time/location prereqs fail, short-circuit to 0 score.
    if !check_time_and_location_prereqs(world, registry, storylet) {
        return 0.0;
    }
    let base = score_storylet_full(director, world, storylet, hot_event);
    let intent_mult = npc_intent_score_multiplier(world, registry, storylet);
    (base * intent_mult).clamp(0.0, 100.0)
}

/// Variant selection API that uses NPC intent when available.
pub fn select_next_event_with_registry<'a>(
    director: &'a EventDirector,
    world: &WorldState,
    registry: &NpcRegistry,
    memory: &MemorySystem,
    current_tick: SimTick,
) -> Option<&'a Storylet> {
    let eligible = director.find_eligible(world, memory, current_tick);
    if eligible.is_empty() {
        return None;
    }
    let hot_event_opt = world.relationship_pressure.peek_next_event();
    let mut best_storylet: Option<&Storylet> = None;
    let mut best_score = f32::MIN;
    for storylet in eligible {
        let score =
            score_storylet_full_with_registry(director, world, registry, storylet, hot_event_opt);
        if score > best_score {
            best_score = score;
            best_storylet = Some(storylet);
        }
    }
    best_storylet
}

/// Cooldown tracker to prevent storylet repetition.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CooldownTracker {
    global_cooldowns: HashMap<String, SimTick>, // storylet_id -> until_tick
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

    fn mark_cooldown(
        &mut self,
        storylet_id: &str,
        npc_id: NpcId,
        cooldown_ticks: u32,
        current_tick: SimTick,
    ) {
        let until = SimTick::new(current_tick.0 + cooldown_ticks as u64);
        self.global_cooldowns.insert(storylet_id.to_string(), until);
        self.npc_cooldowns
            .insert((storylet_id.to_string(), npc_id), until);
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
    pub fn find_eligible(
        &self,
        world: &WorldState,
        memory: &MemorySystem,
        current_tick: SimTick,
    ) -> Vec<&Storylet> {
        self.storylets
            .iter()
            .filter(|s| self.is_eligible(s, world, memory, current_tick))
            .collect()
    }

    /// Check if a storylet is eligible to fire.
    fn is_eligible(
        &self,
        storylet: &Storylet,
        world: &WorldState,
        memory: &MemorySystem,
        current_tick: SimTick,
    ) -> bool {
        // Check cooldown
        if !self
            .cooldowns
            .is_ready(&storylet.id, world.player_id, current_tick)
        {
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
                if !storylet
                    .prerequisites
                    .relationship_states
                    .contains(&rel.state)
                {
                    return false;
                }
            }
        }

        // Check memory prerequisites
        if !storylet.prerequisites.memory_tags_required.is_empty() {
            if let Some(target_role) = storylet.roles.get(0) {
                // NPC must have at least one of the required memory tags
                if let Some(journal) = memory.journals.get(&target_role.npc_id) {
                    let has_required_tag = storylet
                        .prerequisites
                        .memory_tags_required
                        .iter()
                        .any(|tag| !journal.memories_with_tag(tag).is_empty());
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
                    let has_forbidden_tag = storylet
                        .prerequisites
                        .memory_tags_forbidden
                        .iter()
                        .any(|tag| !journal.memories_with_tag(tag).is_empty());
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
                        let has_recent_tag = storylet
                            .prerequisites
                            .memory_tags_required
                            .iter()
                            .any(|tag| {
                                journal
                                    .memories_since(since_tick)
                                    .iter()
                                    .any(|m| m.tags.contains(tag))
                            });
                        if !has_recent_tag {
                            return false;
                        }
                    }
                }
            }
        }

        if !check_life_stage_prereqs(world, &storylet.prerequisites) {
            return false;
        }

        // Relationship prereqs using the new relationship model (additive, non-breaking).
        if !check_relationship_prereqs(
            world,
            &storylet.prerequisites.relationship_prereqs,
            world.player_id,
        ) {
            return false;
        }

        // Digital legacy prereqs for PostLife storylets.
        if !check_digital_legacy_prereq(world, &storylet.prerequisites.digital_legacy_prereq) {
            return false;
        }

        true
    }

    /// Score a storylet for selection (0.0..100.0).
    pub fn score_storylet(&self, storylet: &Storylet, world: &WorldState) -> f32 {
        let mut score = storylet.weight;

        // Pressure point bonus: if there's relationship tension, bump "conflict" storylets
        if storylet
            .prerequisites
            .tags
            .iter()
            .any(|t| t == "conflict")
        {
            if let Some(target_role) = storylet.roles.get(0) {
                if RelationshipQuery::has_pressure_point(world, world.player_id, target_role.npc_id)
                {
                    score *= 1.5;
                }
            }
        }

        // Narrative heat: higher heat = higher priority in emergent moments
        score *= (storylet.heat as f32) / 50.0; // Normalize heat intensity

        // Apply narrative heat multiplier (0.5..2.0 based on current heat level)
        score *= world.heat_multiplier();

        // Behavior intent: prioritize storylets that match current player drive
        if let Some(action) = behavior_action_from_tags(&storylet.prerequisites.tags) {
            let intent = world.player_behavior_bias(action);
            score *= intent;
        }

        score.clamp(0.0, 100.0)
    }

    /// Select the best eligible storylet(s) to fire this tick.
    pub fn select_next_event(
        &self,
        world: &WorldState,
        memory: &MemorySystem,
        current_tick: SimTick,
    ) -> Option<&Storylet> {
        let eligible = self.find_eligible(world, memory, current_tick);
        if eligible.is_empty() {
            return None;
        }

        let hot_event_opt = world.relationship_pressure.peek_next_event();
        let mut best_storylet: Option<&Storylet> = None;
        let mut best_score = f32::MIN;

        for storylet in eligible {
            let score = score_storylet_full(self, world, storylet, hot_event_opt);
            if score > best_score {
                best_score = score;
                best_storylet = Some(storylet);
            }
        }

        best_storylet
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
        // If the selected storylet targets the current hot pair, consume that event.
        if let Some(event) = world.relationship_pressure.peek_next_event() {
            let default_actor_id = world.player_id.0;
            if storylet_targets_pair(
                &storylet.prerequisites,
                event.actor_id,
                event.target_id,
                default_actor_id,
            ) {
                let _ = world.relationship_pressure.pop_next_event();
            }
        }

        apply_storylet_outcome_with_memory(world, memory, storylet, &outcome, current_tick);
        // Mark cooldown
        if let Some(first_role) = storylet.roles.first() {
            self.cooldowns.mark_cooldown(
                &storylet.id,
                first_role.npc_id,
                storylet.cooldown.ticks,
                current_tick,
            );
        }

        if matches!(world.narrative_heat.band(), NarrativeHeatBand::Critical) {
            if let Some(cat) = &storylet.outcomes.heat_category {
                if matches!(cat, StoryletHeatCategory::CriticalArc) {
                    world.narrative_heat.add(-20.0);
                }
            }
        }
    }

    /// Get all registered storylets (for inspection/debugging).
    pub fn all_storylets(&self) -> &[Storylet] {
        &self.storylets
    }
}

fn apply_relationship_outcome(
    rels: &mut HashMap<(u64, u64), RelationshipVector>,
    deltas: &[RelationshipDelta],
) {
    for d in deltas {
        let vec = rels
            .entry((d.actor_id, d.target_id))
            .or_insert_with(RelationshipVector::default);
        vec.apply_delta(d.axis, d.delta);
    }
}

/// Update relationship pressure flags for pairs that had relationship changes.
/// For now, this simply tracks which pairs had deltas; future refinement
/// can track actual band crossings.
fn update_relationship_pressure_flags(world: &mut WorldState, deltas: &[RelationshipDelta]) {
    for delta in deltas {
        let pair = (delta.actor_id, delta.target_id);
        // Simple check: if this pair already exists, skip duplicate
        if !world.relationship_pressure.changed_pairs.contains(&pair) {
            world.relationship_pressure.changed_pairs.push(pair);
        }
    }
}

pub fn apply_storylet_outcome_with_memory(
    world: &mut WorldState,
    memory: &mut MemorySystem,
    storylet: &Storylet,
    outcome: &StoryletOutcome,
    current_tick: SimTick,
) {
    // Apply stat impacts
    apply_stat_deltas(&mut world.player_stats, &outcome.stat_deltas);

    // New additive relationship delta handling using the unified model (non-breaking).
    let mut rel_buffer: HashMap<(u64, u64), RelationshipVector> = HashMap::new();
    for delta in &outcome.relationship_deltas {
        rel_buffer
            .entry((delta.actor_id, delta.target_id))
            .or_insert_with(|| {
                let current = world.get_relationship(NpcId(delta.actor_id), NpcId(delta.target_id));
                RelationshipVector {
                    affection: current.affection,
                    trust: current.trust,
                    attraction: current.attraction,
                    familiarity: current.familiarity,
                    resentment: current.resentment,
                }
            });
    }
    // Seed pressure snapshots before applying deltas so band changes are detectable immediately.
    for ((actor_id, target_id), vec) in &rel_buffer {
        world.relationship_pressure.update_for_pair(
            *actor_id,
            *target_id,
            vec,
            None,
            Some(current_tick.0),
        );
    }

    apply_relationship_outcome(&mut rel_buffer, &outcome.relationship_deltas);
    for ((actor_id, target_id), vec) in rel_buffer {
        let mut current = world.get_relationship(NpcId(actor_id), NpcId(target_id));
        current.affection = vec.affection;
        current.trust = vec.trust;
        current.attraction = vec.attraction;
        current.familiarity = vec.familiarity;
        current.resentment = vec.resentment;
        current.state = current.compute_next_state();
        world.set_relationship(NpcId(actor_id), NpcId(target_id), current);

        world.relationship_pressure.update_for_pair(
            actor_id,
            target_id,
            &vec,
            Some(format!("storylet:{}", storylet.id)),
            Some(current_tick.0),
        );

        let tags = memory_tags_for_pair(memory, actor_id, target_id);
        world
            .relationship_milestones
            .evaluate_and_record_milestones_for_pair(
                actor_id,
                target_id,
                &vec,
                &tags,
                Some(format!("storylet:{}", storylet.id)),
                Some(current_tick.0),
            );
    }

    // Update karma (based on outcome emotional intensity)
    world
        .player_karma
        .apply_delta(outcome.emotional_intensity * 10.0);
    if let Some(k) = outcome.karma_delta {
        world.player_karma.apply_delta(k);
    }

    // Global heat reactions: base storylet heat plus optional spikes/damps.
    world.add_heat(storylet.heat as f32);
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

    // Update relationship pressure flags for any pairs that had relationship changes
    if !outcome.relationship_deltas.is_empty() {
        update_relationship_pressure_flags(world, &outcome.relationship_deltas);
    }
}

pub fn next_hot_relationship(world: &mut WorldState) -> Option<RelationshipPressureEvent> {
    world.relationship_pressure.pop_next_event()
}

pub fn next_relationship_milestone(world: &mut WorldState) -> Option<RelationshipMilestoneEvent> {
    world.relationship_milestones.pop_next()
}

impl Default for EventDirector {
    fn default() -> Self {
        Self::new()
    }
}

fn storylet_check_stat_prereqs(_world: &WorldState, _pre: &StoryletPrerequisites) -> bool {
    true
}

fn storylet_check_heat_prereqs(_world: &WorldState, _pre: &StoryletPrerequisites) -> bool {
    true
}

fn storylet_check_relationship_prereqs(world: &WorldState, pre: &StoryletPrerequisites) -> bool {
    check_relationship_prereqs(world, &pre.relationship_prereqs, world.player_id)
}

fn storylet_check_time_and_location_prereqs(
    world: &WorldState,
    sim: &SimState,
    storylet: &Storylet,
) -> bool {
    check_time_and_location_prereqs(world, &sim.npc_registry, storylet)
}

fn relationship_pressure_score_multiplier(
    _world: &WorldState,
    _sim: &SimState,
    _storylet: &Storylet,
) -> f32 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectorChoiceView {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectorEventView {
    pub storylet_id: String,
    pub title: String,
    pub choices: Vec<DirectorChoiceView>,
}

pub struct DirectorContext<'a> {
    pub library: &'a StoryletLibrary,
    pub world: &'a WorldState,
    pub sim: &'a SimState,
    pub usage: &'a StoryletUsageState,
}

pub fn storylet_is_eligible(
    world: &WorldState,
    sim: &SimState,
    storylet: &Storylet,
    usage: &StoryletUsageState,
) -> bool {
    let pre = &storylet.prerequisites;

    if let Some(max) = storylet.outcomes.max_uses {
        let used = usage.times_fired.get(&storylet.id).copied().unwrap_or(0);
        if used >= max {
            return false;
        }
    }

    if !storylet_check_stat_prereqs(world, pre) {
        return false;
    }
    if !check_life_stage_prereqs(world, pre) {
        return false;
    }
    if !storylet_check_heat_prereqs(world, pre) {
        return false;
    }
    if !storylet_check_relationship_prereqs(world, pre) {
        return false;
    }
    if !storylet_check_time_and_location_prereqs(world, sim, storylet) {
        return false;
    }
    if !check_digital_legacy_prereq(world, &pre.digital_legacy_prereq) {
        return false;
    }

    true
}

pub fn score_storylet_full_simple(
    world: &WorldState,
    sim: &SimState,
    storylet: &Storylet,
) -> f32 {
    let base = if storylet.weight > 0.0 {
        storylet.weight
    } else {
        1.0
    };

    let heat_band = world.narrative_heat.band();
    let heat_mult = heat_score_multiplier(heat_band, storylet);
    let stage_mult = life_stage_score_multiplier(world, &storylet.prerequisites);
    let legacy_mult =
        digital_legacy_score_multiplier(world, &storylet.prerequisites.digital_legacy_prereq);
    let npc_intent_mult = npc_intent_score_multiplier(world, &sim.npc_registry, storylet);
    let pressure_mult = relationship_pressure_score_multiplier(world, sim, storylet);

    base * heat_mult * stage_mult * legacy_mult * npc_intent_mult * pressure_mult
}

pub fn select_storylet_weighted<'a>(
    world: &WorldState,
    sim: &SimState,
    library: &'a StoryletLibrary,
    usage: &StoryletUsageState,
) -> Option<&'a Storylet> {
    let mut scored: Vec<(&Storylet, f32)> = library
        .storylets
        .iter()
        .filter(|s| storylet_is_eligible(world, sim, s, usage))
        .map(|s| {
            let score = score_storylet_full_simple(world, sim, s).max(0.0);
            (s, score)
        })
        .collect();

    if scored.is_empty() {
        return None;
    }

    let total: f32 = scored.iter().map(|(_, w)| *w).sum();
    if total <= 0.0 {
        scored.sort_by(|(a, _), (b, _)| a.id.cmp(&b.id));
        return Some(scored[0].0);
    }

    let mut rng = deterministic_rng_from_world(world);
    let roll = rng.gen_f32() * total;
    let mut acc = 0.0;
    for (s, w) in &scored {
        acc += *w;
        if roll <= acc {
            return Some(s);
        }
    }

    scored.last().map(|(s, _)| *s)
}

pub fn apply_storylet_outcome(
    world: &mut WorldState,
    _sim: &mut SimState,
    outcome: &StoryletOutcome,
) {
    if !outcome.stat_deltas.is_empty() {
        apply_stat_deltas(&mut world.player_stats, &outcome.stat_deltas);
    }

    if !outcome.relationship_deltas.is_empty() {
        for delta in &outcome.relationship_deltas {
            let actor = NpcId(delta.actor_id);
            let target = NpcId(delta.target_id);
            let mut rel = world.get_relationship(actor, target);
            let axis = match delta.axis {
                ModelRelationshipAxis::Affection => CoreRelationshipAxis::Affection,
                ModelRelationshipAxis::Trust => CoreRelationshipAxis::Trust,
                ModelRelationshipAxis::Attraction => CoreRelationshipAxis::Attraction,
                ModelRelationshipAxis::Familiarity => CoreRelationshipAxis::Familiarity,
                ModelRelationshipAxis::Resentment => CoreRelationshipAxis::Resentment,
            };
            rel.apply_delta(axis, delta.delta);
            rel.state = rel.compute_next_state();
            world.set_relationship(actor, target, rel);
        }
    }

    if let Some(delta) = outcome.karma_delta {
        world.player_karma.apply_delta(delta);
    }
}

pub fn apply_storylet_choice_outcome(
    world: &mut WorldState,
    sim: &mut SimState,
    storylet: &Storylet,
    choice: &StoryletChoice,
) {
    apply_storylet_outcome(world, sim, &choice.outcome);

    let usage = &mut world.storylet_usage;
    let counter = usage.times_fired.entry(storylet.id.clone()).or_insert(0);
    *counter += 1;
}

pub fn select_next_event_view(
    world: &mut WorldState,
    sim: &mut SimState,
    library: &StoryletLibrary,
) -> Option<DirectorEventView> {
    let usage = &world.storylet_usage;
    let storylet = select_storylet_weighted(world, sim, library, usage)?;

    let choices = storylet
        .outcomes
        .choices
        .iter()
        .map(|c| DirectorChoiceView {
            id: c.id.clone(),
            label: c.label.clone(),
        })
        .collect();

    Some(DirectorEventView {
        storylet_id: storylet.id.clone(),
        title: storylet.name.clone(),
        choices,
    })
}

pub fn apply_choice_and_advance(
    world: &mut WorldState,
    sim: &mut SimState,
    library: &StoryletLibrary,
    storylet_id: &str,
    choice_id: &str,
    ticks_to_advance: u32,
) -> Option<DirectorEventView> {
    let storylet = library.storylets.iter().find(|s| s.id == storylet_id)?;
    let choice = storylet
        .outcomes
        .choices
        .iter()
        .find(|c| c.id == choice_id)?;

    apply_storylet_choice_outcome(world, sim, storylet, choice);

    if ticks_to_advance > 0 {
        tick_world(world, sim, ticks_to_advance);
    }

    select_next_event_view(world, sim, library)
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn_core::{
        relationship_model::RelationshipAxis as ModelRelationshipAxis, NpcId, Relationship,
        StatDelta, StatKind, WorldSeed, WorldState,
    };

    fn base_storylet(id: &str) -> Storylet {
        Storylet {
            id: id.to_string(),
            name: id.to_string(),
            ..Storylet::default()
        }
    }

    fn tags(list: &[&str]) -> TagBitset {
        TagBitset::from_tags(list.iter().map(|t| t.to_string()).collect())
    }

    #[test]
    fn test_storylet_creation() {
        let mut storylet = base_storylet("event_001");
        storylet.name = "First Meeting".to_string();
        storylet.tags = tags(&["romance"]);
        storylet.prerequisites = StoryletPrerequisites {
            life_stages: vec!["Adult".to_string()],
            relationship_states: vec![RelationshipState::Friend],
            ..Default::default()
        };
        storylet.heat = 50;
        storylet.weight = 0.5;
        storylet.cooldown = StoryletCooldown { ticks: 100 };
        storylet.roles = StoryletRoles::from(vec![StoryletRole {
            name: "target".to_string(),
            npc_id: NpcId(2),
        }]);

        assert_eq!(storylet.id, "event_001");
        assert_eq!(storylet.heat, 50);
    }

    #[test]
    fn test_event_director_register() {
        let mut director = EventDirector::new();
        let mut storylet = base_storylet("event_001");
        storylet.name = "Test Event".to_string();
        storylet.heat = 50;
        storylet.weight = 0.5;
        storylet.cooldown = StoryletCooldown { ticks: 100 };

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

        let mut romance_storylet = base_storylet("romantic_arc");
        romance_storylet.name = "Romantic Turning Point".to_string();
        romance_storylet.tags = tags(&["romance"]);
        romance_storylet.heat = 60;
        romance_storylet.weight = 0.5;
        romance_storylet.cooldown = StoryletCooldown { ticks: 100 };

        let mut conflict_storylet = romance_storylet.clone();
        conflict_storylet.tags = tags(&["conflict"]);

        let romance_score = director.score_storylet(&romance_storylet, &world);
        let conflict_score = director.score_storylet(&conflict_storylet, &world);
        assert!(romance_score > conflict_score);
    }

    #[test]
    fn test_event_director_score() {
        let mut director = EventDirector::new();
        let mut world = WorldState::new(WorldSeed(42), NpcId(1));

        let mut storylet = base_storylet("event_001");
        storylet.name = "Test Event".to_string();
        storylet.tags = tags(&["romance"]);
        storylet.heat = 75;
        storylet.weight = 0.8;
        storylet.cooldown = StoryletCooldown { ticks: 100 };

        let score = director.score_storylet(&storylet, &world);
        assert!(score > 0.0);
    }

    #[test]
    fn test_outcome_application() {
        let mut director = EventDirector::new();
        let mut world = WorldState::new(WorldSeed(42), NpcId(1));
        let mut memory = MemorySystem::new();

        let mut storylet = base_storylet("event_001");
        storylet.name = "Test Event".to_string();
        storylet.heat = 50;
        storylet.weight = 0.5;
        storylet.cooldown = StoryletCooldown { ticks: 100 };
        storylet.roles = StoryletRoles::from(vec![StoryletRole {
            name: "target".to_string(),
            npc_id: NpcId(2),
        }]);

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
        let mut storylet = base_storylet("outcome_test");
        storylet.name = "Outcome Test".to_string();
        let outcome = StoryletOutcome {
            stat_deltas: vec![
                StatDelta {
                    kind: StatKind::Mood,
                    delta: -5.0,
                    source: Some("test".into()),
                },
                StatDelta {
                    kind: StatKind::Reputation,
                    delta: 10.0,
                    source: Some("test".into()),
                },
            ],
            karma_delta: Some(-20.0),
            ..Default::default()
        };

        apply_storylet_outcome_with_memory(
            &mut world,
            &mut memory,
            &storylet,
            &outcome,
            SimTick(0),
        );

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

        let mut storylet = base_storylet("spike_event");
        storylet.name = "High drama".to_string();
        storylet.tags = tags(&["conflict"]);
        storylet.heat = 20;
        storylet.cooldown = StoryletCooldown { ticks: 50 };
        storylet.roles = StoryletRoles::from(vec![StoryletRole {
            name: "target".to_string(),
            npc_id: NpcId(2),
        }]);

        let mut outcome = StoryletOutcome::default();
        outcome.memory_event_id = "dramatic_turn".to_string();
        outcome.memory_tags = vec!["trigger".to_string()];
        outcome.heat_spike = 5.0;

        director.fire_storylet(&storylet, &mut world, &mut memory, outcome, SimTick(0));

        assert!(world.narrative_heat.value() >= 35.0);
        assert!(memory.get_journal(world.player_id).is_some());
    }

    #[test]
    fn test_relationship_state_gating_romance_event() {
        use syn_core::{AbstractNpc, AttachmentStyle, Traits};

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
        let mut romance_storylet = base_storylet("romance_confession");
        romance_storylet.name = "Romantic Confession".to_string();
        romance_storylet.tags = tags(&["romance"]);
        romance_storylet.prerequisites = StoryletPrerequisites {
            min_relationship_affection: Some(5.0),
            relationship_states: vec![RelationshipState::Friend],
            ..Default::default()
        };
        romance_storylet.heat = 50;
        romance_storylet.weight = 0.7;
        romance_storylet.cooldown = StoryletCooldown { ticks: 200 };
        romance_storylet.roles = StoryletRoles::from(vec![StoryletRole {
            name: "romantic_interest".to_string(),
            npc_id: NpcId(2),
        }]);

        // Set up a Stranger relationship
        world.set_relationship(
            NpcId(1),
            NpcId(2),
            Relationship {
                affection: 0.0,
                trust: 0.0,
                attraction: 0.0,
                familiarity: 0.0,
                resentment: 0.0,
                state: RelationshipState::Stranger,
            },
        );

        director.register_storylet(romance_storylet.clone());

        // Romance event should NOT fire with Stranger state
        assert!(!director.is_eligible(&romance_storylet, &world, &memory, SimTick(0)));

        // Now set to Friend state
        world.set_relationship(
            NpcId(1),
            NpcId(2),
            Relationship {
                affection: 5.0,
                trust: 3.0,
                attraction: 2.0,
                familiarity: 4.0,
                resentment: 0.0,
                state: RelationshipState::Friend,
            },
        );

        // Romance event SHOULD fire with Friend state
        assert!(director.is_eligible(&romance_storylet, &world, &memory, SimTick(0)));
    }

    #[test]
    fn test_relationship_state_transition_on_event() {
        let mut director = EventDirector::new();
        let mut world = WorldState::new(WorldSeed(42), NpcId(1));
        let mut memory = MemorySystem::new();

        // Set Friend relationship
        world.set_relationship(
            NpcId(1),
            NpcId(2),
            Relationship {
                affection: 5.0,
                trust: 3.0,
                attraction: 2.0,
                familiarity: 4.0,
                resentment: 0.0,
                state: RelationshipState::Friend,
            },
        );

        let mut storylet = base_storylet("deepening_bond");
        storylet.name = "We're getting closer".to_string();
        storylet.tags = tags(&["romance"]);
        storylet.prerequisites = StoryletPrerequisites {
            relationship_states: vec![RelationshipState::Friend],
            ..Default::default()
        };
        storylet.heat = 50;
        storylet.weight = 0.5;
        storylet.cooldown = StoryletCooldown { ticks: 100 };
        storylet.roles = StoryletRoles::from(vec![StoryletRole {
            name: "target".to_string(),
            npc_id: NpcId(2),
        }]);

        // Create outcome that boosts relationship values
        // Current: affection 5.0, trust 3.0, attraction 2.0, familiarity 4.0
        // Delta: affection 3.0, trust 4.0, attraction 6.0, familiarity 3.0
        // Result: affection 8.0, trust 7.0, attraction 8.0, familiarity 7.0
        // With the refactored check order (most specific first):
        // - Not Spouse: trust 7.0 < 8.0
        // - Is Partner: attraction 8.0 > 7.0 && trust 7.0 > 6.0 && affection 8.0 > 7.0 ✓
        let mut outcome = StoryletOutcome::default();
        outcome.relationship_deltas.push(RelationshipDelta {
            actor_id: 1,
            target_id: 2,
            axis: ModelRelationshipAxis::Affection,
            delta: 3.0,
            source: None,
        });
        outcome.relationship_deltas.push(RelationshipDelta {
            actor_id: 1,
            target_id: 2,
            axis: ModelRelationshipAxis::Trust,
            delta: 4.0,
            source: None,
        });
        outcome.relationship_deltas.push(RelationshipDelta {
            actor_id: 1,
            target_id: 2,
            axis: ModelRelationshipAxis::Attraction,
            delta: 6.0,
            source: None,
        });
        outcome.relationship_deltas.push(RelationshipDelta {
            actor_id: 1,
            target_id: 2,
            axis: ModelRelationshipAxis::Familiarity,
            delta: 3.0,
            source: None,
        });

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
        let mut conflict_storylet = base_storylet("heated_argument");
        conflict_storylet.name = "You have a heated argument".to_string();
        conflict_storylet.tags = tags(&["conflict"]);
        conflict_storylet.heat = 60;
        conflict_storylet.weight = 0.6;
        conflict_storylet.cooldown = StoryletCooldown { ticks: 150 };
        conflict_storylet.roles = StoryletRoles::from(vec![StoryletRole {
            name: "target".to_string(),
            npc_id: NpcId(2),
        }]);

        // Best Friend relationship should NOT allow conflict based on state check
        world.set_relationship(
            NpcId(1),
            NpcId(2),
            Relationship {
                affection: 9.0,
                trust: 9.0,
                attraction: 0.5,
                familiarity: 9.0,
                resentment: 0.0,
                state: RelationshipState::BestFriend,
            },
        );

        director.register_storylet(conflict_storylet.clone());

        // Should still be eligible (conflict_event_gating would need custom logic)
        // This test validates that the relationship state is properly tracked
        let rel = world.get_relationship(NpcId(1), NpcId(2));
        assert_eq!(rel.state, RelationshipState::BestFriend);
        assert!(!rel.state.allows_conflict());
    }

    #[test]
    fn test_memory_echo_required_tag_gating() {
        use syn_core::{AbstractNpc, AttachmentStyle, Traits};
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
        let mut echo_storylet = base_storylet("revenge_moment");
        echo_storylet.name = "Revenge opportunity arises".to_string();
        echo_storylet.tags = tags(&["echo"]);
        echo_storylet.prerequisites = StoryletPrerequisites {
            memory_tags_required: vec!["betrayal".to_string()],
            ..Default::default()
        };
        echo_storylet.heat = 60;
        echo_storylet.weight = 0.8;
        echo_storylet.cooldown = StoryletCooldown { ticks: 300 };
        echo_storylet.roles = StoryletRoles::from(vec![StoryletRole {
            name: "target".to_string(),
            npc_id: NpcId(2),
        }]);

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
        )
        .with_tags(vec!["betrayal"]);

        memory.record_memory(memory_entry);

        // Event SHOULD fire now with required memory present
        assert!(director.is_eligible(&echo_storylet, &world, &memory, SimTick(100)));
    }

    #[test]
    fn test_memory_echo_forbidden_tag_blocking() {
        use syn_core::{AbstractNpc, AttachmentStyle, Traits};
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
        let mut fragile_storylet = base_storylet("intimate_moment");
        fragile_storylet.name = "Intimate conversation".to_string();
        fragile_storylet.tags = tags(&["romance"]);
        fragile_storylet.prerequisites = StoryletPrerequisites {
            memory_tags_forbidden: vec!["trauma".to_string()],
            ..Default::default()
        };
        fragile_storylet.heat = 50;
        fragile_storylet.weight = 0.7;
        fragile_storylet.cooldown = StoryletCooldown { ticks: 200 };
        fragile_storylet.roles = StoryletRoles::from(vec![StoryletRole {
            name: "target".to_string(),
            npc_id: NpcId(2),
        }]);

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
        )
        .with_tags(vec!["trauma"]);

        memory.record_memory(trauma_entry);

        // Event should NOT fire now with traumatic memory present (conflict avoidance)
        assert!(!director.is_eligible(&fragile_storylet, &world, &memory, SimTick(100)));
    }

    #[test]
    fn test_memory_echo_recency_window() {
        use syn_core::{AbstractNpc, AttachmentStyle, Traits};
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
        let mut follow_up_storylet = base_storylet("confrontation_aftermath");
        follow_up_storylet.name = "Deal with the consequences".to_string();
        follow_up_storylet.tags = tags(&["conflict_resolution"]);
        follow_up_storylet.prerequisites = StoryletPrerequisites {
            memory_tags_required: vec!["confrontation".to_string()],
            memory_recency_ticks: Some(50),
            ..Default::default()
        };
        follow_up_storylet.heat = 55;
        follow_up_storylet.weight = 0.65;
        follow_up_storylet.cooldown = StoryletCooldown { ticks: 100 };
        follow_up_storylet.roles = StoryletRoles::from(vec![StoryletRole {
            name: "target".to_string(),
            npc_id: NpcId(2),
        }]);

        director.register_storylet(follow_up_storylet.clone());

        let mut memory = MemorySystem::new();

        // Event should NOT fire without recent confrontation memory
        assert!(!director.is_eligible(&follow_up_storylet, &world, &memory, SimTick(100)));

        // Add an OLD confrontation memory (150 ticks ago, outside recency window)
        let old_confrontation = MemoryEntry::new(
            "mem_confrontation_old".to_string(),
            "event_confrontation".to_string(),
            NpcId(2),
            SimTick(0), // 100 ticks ago
            -0.5,
        )
        .with_tags(vec!["confrontation"]);

        memory.record_memory(old_confrontation);

        // Event should NOT fire (memory outside recency window)
        assert!(!director.is_eligible(&follow_up_storylet, &world, &memory, SimTick(100)));

        // Add a RECENT confrontation memory (within last 50 ticks)
        let recent_confrontation = MemoryEntry::new(
            "mem_confrontation_recent".to_string(),
            "event_confrontation".to_string(),
            NpcId(2),
            SimTick(75), // 25 ticks ago (within 50-tick window)
            -0.6,
        )
        .with_tags(vec!["confrontation"]);

        memory.record_memory(recent_confrontation);

        // Event SHOULD fire now (recent memory within window)
        assert!(director.is_eligible(&follow_up_storylet, &world, &memory, SimTick(100)));
    }

    #[test]
    fn test_memory_echo_multiple_tags() {
        use syn_core::{AbstractNpc, AttachmentStyle, Traits};
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
        let mut complex_storylet = base_storylet("emotional_climax");
        complex_storylet.name = "Emotional resolution".to_string();
        complex_storylet.tags = tags(&["relationship"]);
        complex_storylet.prerequisites = StoryletPrerequisites {
            memory_tags_required: vec!["love_confession".to_string(), "jealousy".to_string()],
            ..Default::default()
        };
        complex_storylet.heat = 80;
        complex_storylet.weight = 0.9;
        complex_storylet.cooldown = StoryletCooldown { ticks: 400 };
        complex_storylet.roles = StoryletRoles::from(vec![StoryletRole {
            name: "target".to_string(),
            npc_id: NpcId(2),
        }]);

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
        )
        .with_tags(vec!["jealousy"]);

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
        )
        .with_tags(vec!["love_confession"]);

        memory.record_memory(confession_memory);

        // Event SHOULD STILL fire (now has both)
        assert!(director.is_eligible(&complex_storylet, &world, &memory, SimTick(100)));
    }
}
