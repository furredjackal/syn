//! # SYN Simulation Engine
//!
//! **Canonical simulation entrypoint:** `tick_simulation` + `WorldSimState` + `SimState`.
//!
//! **Legacy (deprecated):** `Simulator` + `tick()` / `tick_world()`.
//!
//! The new tier-based simulation system uses:
//! - [`WorldSimState`]: Tracks NPC tiers and update timestamps
//! - [`SimState`]: Core simulation state machine
//! - [`SimulationTickConfig`]: Configuration for tick behavior
//! - [`tick_simulation`]: Main simulation tick function
//! - [`tick_simulation_n`]: Advance multiple ticks
//! - [`advance_simulation_ticks`]: Helper for N-tick advances
//!
//! The legacy `Simulator` struct and related types are deprecated and will be removed.

mod npc_registry;
pub mod relationship_drift;
pub mod post_life;
pub mod systems;
pub use npc_registry::NpcRegistry;
pub use systems::{
    update_npc_tiers_for_tick, update_npcs_for_tick, update_relationships_for_npc,
    update_stats_for_npc, NpcUpdateConfig, TierUpdateConfig,
};

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use syn_core::life_stage::LifeStageConfig;
use syn_core::narrative_heat::{compute_heat_delta, NarrativeHeatConfig, NarrativeHeatInputs};
use syn_core::npc::NpcPrototype;
use syn_core::npc::{NpcActivityKind};
use syn_core::npc_actions::{
    base_effect_for_action, behavior_to_candidate_actions, NpcActionInstance, NpcActionKind,
};
use syn_core::npc_behavior::{
    choose_best_intent, compute_behavior_intents, compute_needs_from_state, BehaviorSnapshot,
};
use syn_core::relationship_model::RelationshipVector as CoreRelationshipVector;
use syn_core::{
    AbstractNpc, DeterministicRng, NpcId, RelationshipDelta, StatKind, Stats, WorldState,
};
use syn_core::apply_stat_deltas;
use syn_core::time::{GameTime, TickContext};
use syn_memory::MemorySystem;
use syn_storage::models::AbstractNpc as StorageNpc;
use syn_storage::{HybridStorage};
use syn_storage::storage_error::StorageError;

/// Simulation engine: advances world state by one tick.
///
/// **DEPRECATED:** Use `tick_simulation` + `WorldSimState` instead.
#[deprecated(
    since = "0.1.0",
    note = "Use `tick_simulation` + `WorldSimState` instead; this legacy struct will be removed."
)]
pub struct Simulator {
    rng: DeterministicRng,
    active_npcs: HashMap<NpcId, SimulatedNpc>,
}

/// Legacy LOD tiers used by Simulator.
///
/// **DEPRECATED:** Use `NpcTier` instead.
#[deprecated(
    since = "0.1.0",
    note = "Use `NpcTier` (Tier0/Tier1/Tier2) instead; this legacy enum will be removed."
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LodTier {
    High,
    Medium,
    Low,
}

/// Legacy NpcLod used alongside new tiers for compatibility.
///
/// **DEPRECATED:** Use `NpcTier` instead.
#[deprecated(
    since = "0.1.0",
    note = "Use `NpcTier` instead; this legacy enum will be removed."
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpcLod {
    Tier2Active,
    Tier1Neighborhood,
    Tier0Dormant,
}

/// NPC fidelity tiers for simulation updates.
/// - Tier0: Always simulated, high fidelity (e.g., player's immediate circle).
/// - Tier1: Active but batched updates (e.g., neighborhood NPCs).
/// - Tier2: Coarse / background simulation (e.g., distant population).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum NpcTier {
    /// Always simulated, high fidelity.
    Tier0,
    /// Active but batched updates.
    Tier1,
    /// Coarse / background simulation.
    #[default]
    Tier2,
}

/// Central simulation world state for tracking NPC fidelity tiers and update timestamps.
/// This struct tracks which tier each NPC belongs to and when they were last updated,
/// enabling LOD-based simulation throttling.
#[derive(Debug, Default)]
pub struct WorldSimState {
    /// Maps NPC IDs to their current fidelity tier.
    npc_tiers: HashMap<NpcId, NpcTier>,
    /// Tracks the last tick each NPC was updated.
    last_update_tick: HashMap<NpcId, syn_core::SimTick>,
}

impl WorldSimState {
    /// Creates a new empty WorldSimState.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the fidelity tier for an NPC.
    pub fn set_npc_tier(&mut self, id: NpcId, tier: NpcTier) {
        self.npc_tiers.insert(id, tier);
    }

    /// Returns the fidelity tier for an NPC, defaulting to Tier2 if not set.
    pub fn npc_tier(&self, id: NpcId) -> NpcTier {
        self.npc_tiers.get(&id).copied().unwrap_or(NpcTier::Tier2)
    }

    /// Records that an NPC was updated at the given tick.
    pub fn mark_npc_updated(&mut self, id: NpcId, tick: syn_core::SimTick) {
        self.last_update_tick.insert(id, tick);
    }

    /// Returns the last tick an NPC was updated, or None if never updated.
    pub fn last_npc_update(&self, id: NpcId) -> Option<syn_core::SimTick> {
        self.last_update_tick.get(&id).copied()
    }

    /// Registers a new NPC with a default tier (Tier2) and initial update tick.
    pub fn register_npc(&mut self, id: NpcId, initial_tick: syn_core::SimTick) {
        self.npc_tiers.entry(id).or_insert(NpcTier::Tier2);
        self.last_update_tick.entry(id).or_insert(initial_tick);
    }

    /// Removes an NPC from tracking.
    pub fn remove_npc(&mut self, id: NpcId) {
        self.npc_tiers.remove(&id);
        self.last_update_tick.remove(&id);
    }

    /// Returns an iterator over all tracked NPC IDs and their tiers.
    pub fn iter_tiers(&self) -> impl Iterator<Item = (&NpcId, &NpcTier)> {
        self.npc_tiers.iter()
    }

    /// Returns the number of tracked NPCs.
    pub fn npc_count(&self) -> usize {
        self.npc_tiers.len()
    }
}

/// Minimal simulated NPC container used by the simulator.
#[derive(Debug, Clone)]
pub struct SimulatedNpc {
    pub id: NpcId,
    pub abstract_npc: AbstractNpc,
    pub stats: Stats,
    pub lod_tier: LodTier,
    pub busy_until_tick: u64,
    pub last_action: Option<NpcActionInstance>,
}

impl SimulatedNpc {
    pub fn new(abstract_npc: AbstractNpc) -> Self {
        let mut stats = Stats::default();
        stats.clamp();
        SimulatedNpc {
            id: abstract_npc.id,
            abstract_npc,
            stats,
            lod_tier: LodTier::Low,
            busy_until_tick: 0,
            last_action: None,
        }
    }

    pub fn current_stats(&self) -> &Stats {
        &self.stats
    }

    pub fn stats_mut(&mut self) -> &mut Stats {
        &mut self.stats
    }

    pub fn apply_mood_decay(&mut self) {
        let mood = self.stats.get(StatKind::Mood) * 0.99;
        self.stats.set(StatKind::Mood, mood);
    }

    pub fn update_needs(&mut self) {
        // Placeholder: needs model lives in npc_behavior; keep as no-op here.
    }

    pub fn tick_cooldowns(&mut self) {
        // Placeholder for future action cooldowns.
    }
}

fn gather_recent_memory_flags(_world: &WorldState) -> (bool, bool, bool) {
    (false, false, false)
}

fn apply_rel_deltas(world: &mut WorldState, deltas: &[RelationshipDelta]) {
    world.apply_relationship_deltas(deltas);
}

fn update_narrative_heat(
    world: &mut WorldState,
    config: &NarrativeHeatConfig,
    stat_profile: Option<&LifeStageConfig>,
) {
    let rel_pairs: Vec<((u64, u64), CoreRelationshipVector)> = world
        .relationships
        .iter()
        .map(|((actor_id, target_id), rel)| {
            (
                (actor_id.0, target_id.0),
                CoreRelationshipVector {
                    affection: rel.affection,
                    trust: rel.trust,
                    attraction: rel.attraction,
                    familiarity: rel.familiarity,
                    resentment: rel.resentment,
                },
            )
        })
        .collect();

    let rel_refs: Vec<(&(u64, u64), &CoreRelationshipVector)> =
        rel_pairs.iter().map(|(k, v)| (k, v)).collect();

    let (has_recent_trauma, has_recent_betrayal, has_recent_win) =
        gather_recent_memory_flags(world);

    let inputs = NarrativeHeatInputs {
        player_stats: &world.player_stats,
        relationships: &rel_refs,
        has_recent_trauma,
        has_recent_betrayal,
        has_recent_major_win: has_recent_win,
        stat_profile: stat_profile.map(|cfg| &cfg.stat_profile),
    };

    let delta = compute_heat_delta(&inputs, config);
    world.narrative_heat.add(delta);
    world
        .narrative_heat
        .decay_toward(config.base_decay_toward, config.decay_per_tick);
}

#[allow(deprecated)]
impl Simulator {
    #[deprecated(
        since = "0.1.0",
        note = "Use `WorldSimState::new()` + `SimState::new()` + `tick_simulation` instead."
    )]
    pub fn new(seed: u64) -> Self {
        Simulator {
            rng: DeterministicRng::new(seed),
            active_npcs: HashMap::new(),
        }
    }

    /// Instantiate an NPC into the active simulation (moves from AbstractNpc to SimulatedNpc).
    pub fn instantiate_npc(&mut self, abstract_npc: AbstractNpc) {
        let mut simulated = SimulatedNpc::new(abstract_npc);

        // Initialize stats based on traits and age
        simulated.stats.set(
            StatKind::Health,
            75.0 + (simulated.abstract_npc.traits.stability - 50.0) / 2.0,
        );
        simulated.stats.set(StatKind::Intelligence, 50.0);
        simulated.stats.set(
            StatKind::Charisma,
            (simulated.abstract_npc.traits.charm + simulated.abstract_npc.traits.confidence) / 2.0,
        );
        simulated.stats.set(StatKind::Mood, 0.0);
        simulated.stats.clamp();

        self.active_npcs.insert(simulated.id, simulated);
    }

    /// Despawn an NPC (serializes key changes back to AbstractNpc, removes from active).
    pub fn despawn_npc(&mut self, npc_id: NpcId) -> Option<SimulatedNpc> {
        self.active_npcs.remove(&npc_id)
    }

    /// Assign LOD tiers based on proximity/relevance to player.
    pub fn update_lod_tiers(&mut self, world: &WorldState, player_id: NpcId) {
        for (npc_id, simulated_npc) in &mut self.active_npcs {
            // Simple LOD: player's close relations are High, others Medium/Low
            if let Some(rel) = world.relationships.get(&(player_id, *npc_id)) {
                if rel.affection > 3.0 || rel.trust > 3.0 {
                    simulated_npc.lod_tier = LodTier::High;
                } else if rel.familiarity > 2.0 {
                    simulated_npc.lod_tier = LodTier::Medium;
                } else {
                    simulated_npc.lod_tier = LodTier::Low;
                }
            } else {
                simulated_npc.lod_tier = LodTier::Low;
            }
        }
    }

    /// Advance all active NPCs by one tick.
    fn tick_npcs(&mut self, _world: &WorldState) {
        for (_npc_id, simulated_npc) in &mut self.active_npcs {
            // High fidelity: full mood and need updates
            if simulated_npc.lod_tier == LodTier::High {
                simulated_npc.apply_mood_decay();
                simulated_npc.update_needs();
            }
            // Medium fidelity: simplified updates
            else if simulated_npc.lod_tier == LodTier::Medium {
                let mood = simulated_npc.stats.get(StatKind::Mood) * 0.98; // Slower decay
                simulated_npc.stats.set(StatKind::Mood, mood);
            }
            // Low fidelity: minimal updates (already handled in abstract layer)

            // All tiers: tick cooldowns
            simulated_npc.tick_cooldowns();
        }
    }

    /// Main simulation loop: advance world state by one tick.
    ///
    /// **DEPRECATED:** Use `tick_simulation(world, sim_state, config)` instead.
    #[deprecated(
        since = "0.1.0",
        note = "Use `tick_simulation(world, sim_state, config)` instead; this method will be removed."
    )]
    pub fn tick(&mut self, world: &mut WorldState) {
        // Advance world clock
        let mut tick_ctx = TickContext::default();
        world.tick(&mut tick_ctx);

        // Update LOD assignments
        self.update_lod_tiers(world, world.player_id);

        // Tick all active NPCs
        self.tick_npcs(world);

        // Slowly drift passive stats for non-instantiated NPCs
        self.apply_population_drift(world);

        // Relationship drift (additive, deterministic)
        let drift = relationship_drift::RelationshipDriftSystem::new(
            relationship_drift::RelationshipDriftConfig {
                affection_decay_per_tick: 0.05,
                trust_decay_per_tick: 0.03,
                resentment_decay_per_tick: 0.02,
                familiarity_growth_per_tick: 0.01,
            },
        );
        drift.tick(world);

        let stage_cfg = world.player_life_stage.config();
        update_narrative_heat(world, &stage_cfg.heat_config, Some(&stage_cfg));

        // Derive RNG for next tick's stochastic events
        let next_seed = self.rng.derive_seed();
        self.rng.reseed(next_seed);
    }

    /// Apply background drift to abstract NPCs (population-level effects).
    fn apply_population_drift(&mut self, _world: &mut WorldState) {
        // Placeholder: in full implementation, update age, economic status, etc.
        // for NPCs not currently instantiated.
    }

    /// Get active NPC for inspection/mutation during event firing.
    pub fn get_active_npc_mut(&mut self, npc_id: NpcId) -> Option<&mut SimulatedNpc> {
        self.active_npcs.get_mut(&npc_id)
    }

    /// Get active NPC for reading.
    pub fn get_active_npc(&self, npc_id: NpcId) -> Option<&SimulatedNpc> {
        self.active_npcs.get(&npc_id)
    }

    /// Count active NPCs at each LOD tier.
    pub fn count_by_lod(&self) -> (usize, usize, usize) {
        let mut high = 0;
        let mut medium = 0;
        let mut low = 0;

        for sim_npc in self.active_npcs.values() {
            match sim_npc.lod_tier {
                LodTier::High => high += 1,
                LodTier::Medium => medium += 1,
                LodTier::Low => low += 1,
            }
        }

        (high, medium, low)
    }
}

/// Wrap SimulatedNpc with ID + LOD + last_tick for registry use.
#[derive(Debug)]
pub struct NpcInstance {
    pub id: NpcId,
    pub lod: NpcLod,
    /// Canonical LOD tier for the new world tick loop. Defaults to Tier2Background.
    /// This exists alongside the legacy `lod` for backward compatibility.
    #[allow(dead_code)]
    pub tier: NpcLodTier,
    pub sim: SimulatedNpc,
    /// Last sim tick index this NPC was updated (for LOD throttling).
    pub last_tick: u64,
    /// Current evaluated behavior snapshot (Tier1/Tier2 only).
    pub behavior: Option<BehaviorSnapshot>,
    /// If > current tick, NPC is considered busy and won't take new major actions.
    pub busy_until_tick: u64,
    /// Last executed action snapshot for debugging / UI.
    pub last_action: Option<NpcActionInstance>,

    /// Current coarse-grained location/activity derived from schedule + busy state.
    pub current_activity: NpcActivityKind,
}

/// Create a SimulatedNpc from a prototype & world state deterministically.
pub fn instantiate_simulated_npc_from_prototype(
    proto: &NpcPrototype,
    world: &WorldState,
    _tick: u64,
) -> SimulatedNpc {
    // Deterministic construction of AbstractNpc from prototype.
    let seed = world.seed.0 ^ proto.id.0 ^ 0x9E37_79B9_7F4A_7C15u64;

    // Map minimal fields; keep defaults for others.
    let abstract_npc = AbstractNpc {
        id: proto.id,
        age: 18,
        job: String::new(),
        district: String::from(""),
        household_id: 0,
        traits: Default::default(),
        seed,
        attachment_style: Default::default(),
    };

    let mut sim = SimulatedNpc::new(abstract_npc);
    // Initialize stats from prototype baseline deterministically.
    sim.stats = proto.base_stats;
    sim.stats.clamp();
    sim
}

/// Evaluate current behavior for this NPC using prototype + stats + relationship to player.
pub fn evaluate_npc_behavior(world: &WorldState, npc: &mut NpcInstance) {
    // 1) Prototype & personality
    let proto = match world.npc_prototype(npc.id) {
        Some(p) => p,
        None => {
            npc.behavior = None;
            return;
        }
    };

    // 2) Stats snapshot
    let stats = npc.sim.current_stats();

    // 3) Relationship to player if exists (use relationship_model vectors if available elsewhere)
    let rel_to_player = world
        .relationships
        .get(&(npc.id, world.player_id))
        .or_else(|| world.relationships.get(&(world.player_id, npc.id)))
        .map(|r| {
            // Convert crate::types::Relationship to relationship_model::RelationshipVector for band methods
            syn_core::relationship_model::RelationshipVector {
                affection: r.affection,
                trust: r.trust,
                attraction: r.attraction,
                familiarity: r.familiarity,
                resentment: r.resentment,
            }
        });

    // We need an Option<&RelationshipVector>, so create a temp and take ref
    let rel_tmp;
    let rel_ref_opt = if let Some(rv) = rel_to_player {
        rel_tmp = rv;
        Some(&rel_tmp)
    } else {
        None
    };

    let needs = compute_needs_from_state(stats, &proto.personality, rel_ref_opt);
    let intents = compute_behavior_intents(&needs, &proto.personality);
    let best = choose_best_intent(&intents);

    // Target heuristics
    let mut target_player = false;
    let target_npc_id = None;

    if matches!(
        best.kind,
        syn_core::npc_behavior::BehaviorKind::SeekSocial
            | syn_core::npc_behavior::BehaviorKind::SeekRecognition
            | syn_core::npc_behavior::BehaviorKind::SeekAutonomy
    ) {
        if let Some(rel_vec) = rel_ref_opt {
            use syn_core::relationship_model::AffectionBand::*;
            let aff = rel_vec.affection_band();
            if matches!(aff, Friendly | Close | Devoted) {
                target_player = true;
            }
        }
    }

    npc.behavior = Some(BehaviorSnapshot {
        needs,
        chosen_intent: best,
        target_player,
        target_npc_id,
    });
}

/// Apply the concrete effects of an NPC action to world and memory.
fn apply_npc_action_effect(
    world: &mut WorldState,
    npc: &mut NpcInstance,
    action: &NpcActionInstance,
    tick: u64,
    memory_opt: Option<&mut MemorySystem>,
) {
    let eff = &action.effect;
    // NPC stat deltas
    {
        let stats = npc.sim.stats_mut();
        apply_stat_deltas(stats, &eff.npc_stat_deltas);
    }

    // Player stat deltas
    {
        let player_stats = &mut world.player_stats;
        apply_stat_deltas(player_stats, &eff.player_stat_deltas);
    }

    // Relationship deltas: resolve target id
    if !eff.relationship_deltas.is_empty() {
        let mut resolved = Vec::with_capacity(eff.relationship_deltas.len());
        let target_id = if action.targets_player {
            world.player_id
        } else if let Some(t) = action.target_npc_id {
            t
        } else {
            world.player_id
        };
        for mut d in eff.relationship_deltas.clone() {
            d.target_id = target_id;
            resolved.push(d);
        }
        apply_rel_deltas(world, &resolved);
    }

    // Optional memory echo
    if let Some(memory) = memory_opt {
        if action.targets_player && !eff.memory_tags_for_player.is_empty() {
            // Build a memory entry with tags and participants [npc, player]
            syn_memory::add_npc_behavior_memory_with_tags(
                memory,
                npc.id.0,
                world.player_id.0,
                eff.memory_tags_for_player.clone(),
                syn_core::SimTick::new(tick),
            );
        }
    }
}

/// Decide and execute an action for a single NPC, if not busy.
pub fn maybe_run_npc_action(world: &mut WorldState, npc: &mut NpcInstance, tick: u64) {
    maybe_run_npc_action_with_memory(world, npc, tick, None);
}

/// Variant that can write memories if provided.
pub fn maybe_run_npc_action_with_memory(
    world: &mut WorldState,
    npc: &mut NpcInstance,
    tick: u64,
    mut memory_opt: Option<&mut MemorySystem>,
) {
    if npc.busy_until_tick > tick {
        return;
    }
    let behavior = match &npc.behavior {
        Some(b) => b,
        None => return,
    };
    // Schedule-aware selection
    let activity = npc.current_activity;
    let action = build_action_instance_from_behavior_and_schedule(npc.id, behavior, activity);
    // Apply effects
    apply_npc_action_effect(world, npc, &action, tick, memory_opt.as_deref_mut());
    // Busy
    if action.effect.busy_for_ticks > 0 {
        npc.busy_until_tick = tick + action.effect.busy_for_ticks;
    }
    npc.last_action = Some(action);
}

/// High-fidelity tick for Tier2Active NPCs.
pub fn tick_simulated_npc_full(npc: &mut SimulatedNpc, _world: &mut WorldState, _tick: u64) {
    npc.apply_mood_decay();
    npc.update_needs();
    npc.tick_cooldowns();
}

/// Medium-fidelity tick for Tier1Neighborhood NPCs.
pub fn tick_simulated_npc_medium(npc: &mut SimulatedNpc, _world: &mut WorldState, _tick: u64) {
    let mood = npc.stats.get(StatKind::Mood) * 0.98;
    npc.stats.set(StatKind::Mood, mood);
    npc.tick_cooldowns();
}

/// Coarse tick for Tier0Dormant NPCs.
pub fn tick_simulated_npc_coarse(npc: &mut SimulatedNpc, _world: &mut WorldState, _tick: u64) {
    // Minimal or no-op; still tick cooldowns to avoid stalling.
    npc.tick_cooldowns();
}

/// LOD-aware ticking over a registry of NPCs.
///
/// **DEPRECATED:** Use `tick_simulation` with `WorldSimState` instead.
#[deprecated(
    since = "0.1.0",
    note = "Use `tick_simulation` + `WorldSimState` instead; this function will be removed."
)]
pub fn tick_npcs_lod(
    world: &mut WorldState,
    registry: &mut crate::npc_registry::NpcRegistry,
    tick: u64,
) {
    for (_id, npc) in registry.iter_mut() {
        // Update scheduled/current activity first
        update_npc_activity_state(world, npc, tick);
        match npc.lod {
            NpcLod::Tier2Active => {
                tick_simulated_npc_full(&mut npc.sim, world, tick);
                evaluate_npc_behavior(world, npc);
                maybe_run_npc_action(world, npc, tick);
                npc.last_tick = tick;
            }
            NpcLod::Tier1Neighborhood => {
                let freq = 5u64;
                if tick % freq == 0 {
                    tick_simulated_npc_medium(&mut npc.sim, world, tick);
                    evaluate_npc_behavior(world, npc);
                    maybe_run_npc_action(world, npc, tick);
                    npc.last_tick = tick;
                }
            }
            NpcLod::Tier0Dormant => {
                let freq = 50u64;
                if tick % freq == 0 {
                    tick_simulated_npc_coarse(&mut npc.sim, world, tick);
                    // For Tier0, skip behavior evaluation, or do it rarely.
                    npc.behavior = None;
                    npc.last_tick = tick;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn_core::{AttachmentStyle, Relationship, Traits, WorldSeed};

    #[test]
    fn test_simulated_npc_mood_decay() {
        let abstract_npc = AbstractNpc {
            id: NpcId(1),
            age: 25,
            job: "Engineer".to_string(),
            district: "Downtown".to_string(),
            household_id: 1,
            traits: Traits::default(),
            seed: 123,
            attachment_style: AttachmentStyle::Secure,
        };

        let mut sim_npc = SimulatedNpc::new(abstract_npc);
        sim_npc.stats.set(StatKind::Mood, 5.0);
        sim_npc.apply_mood_decay();

        assert!(sim_npc.stats.get(StatKind::Mood) < 5.0);
    }

    #[test]
    fn test_simulated_npc_update_needs() {
        let abstract_npc = AbstractNpc {
            id: NpcId(1),
            age: 25,
            job: "Engineer".to_string(),
            district: "Downtown".to_string(),
            household_id: 1,
            traits: Traits::default(),
            seed: 123,
            attachment_style: AttachmentStyle::Secure,
        };

        let mut sim_npc = SimulatedNpc::new(abstract_npc);
        sim_npc.update_needs();
        // No specific needs tracked in this minimal implementation; ensure call succeeds.
    }

    #[test]
    fn test_simulator_instantiate_despawn() {
        let mut simulator = Simulator::new(42);
        let abstract_npc = AbstractNpc {
            id: NpcId(1),
            age: 25,
            job: "Engineer".to_string(),
            district: "Downtown".to_string(),
            household_id: 1,
            traits: Traits::default(),
            seed: 123,
            attachment_style: AttachmentStyle::Secure,
        };

        simulator.instantiate_npc(abstract_npc);
        assert!(simulator.get_active_npc(NpcId(1)).is_some());

        simulator.despawn_npc(NpcId(1));
        assert!(simulator.get_active_npc(NpcId(1)).is_none());
    }

    #[test]
    fn test_simulator_tick() {
        let mut simulator = Simulator::new(42);
        let mut world = WorldState::new(WorldSeed(42), NpcId(1));

        let abstract_npc = AbstractNpc {
            id: NpcId(2),
            age: 25,
            job: "Engineer".to_string(),
            district: "Downtown".to_string(),
            household_id: 1,
            traits: Traits::default(),
            seed: 123,
            attachment_style: AttachmentStyle::Secure,
        };

        world.npcs.insert(NpcId(2), abstract_npc.clone());
        simulator.instantiate_npc(abstract_npc);

        let initial_tick = world.current_tick.0;
        simulator.tick(&mut world);

        assert_eq!(world.current_tick.0, initial_tick + 1);
    }

    #[test]
    fn test_lod_tier_assignment() {
        let mut simulator = Simulator::new(42);
        let mut world = WorldState::new(WorldSeed(42), NpcId(1));

        let abstract_npc = AbstractNpc {
            id: NpcId(2),
            age: 25,
            job: "Engineer".to_string(),
            district: "Downtown".to_string(),
            household_id: 1,
            traits: Traits::default(),
            seed: 123,
            attachment_style: AttachmentStyle::Secure,
        };

        world.npcs.insert(NpcId(2), abstract_npc.clone());
        simulator.instantiate_npc(abstract_npc);

        // High affection relationship
        let high_rel = Relationship {
            affection: 5.0,
            ..Default::default()
        };
        world.set_relationship(NpcId(1), NpcId(2), high_rel);

        simulator.update_lod_tiers(&world, NpcId(1));

        let sim_npc = simulator.get_active_npc(NpcId(2)).unwrap();
        assert_eq!(sim_npc.lod_tier, LodTier::High);
    }

    #[test]
    fn test_tick_updates_mood_via_stats() {
        let mut simulator = Simulator::new(42);
        let mut world = WorldState::new(WorldSeed(42), NpcId(1));

        let abstract_npc = AbstractNpc {
            id: NpcId(2),
            age: 25,
            job: "Engineer".to_string(),
            district: "Downtown".to_string(),
            household_id: 1,
            traits: Traits::default(),
            seed: 123,
            attachment_style: AttachmentStyle::Secure,
        };

        world.npcs.insert(NpcId(2), abstract_npc.clone());
        simulator.instantiate_npc(abstract_npc);

        // Force LOD High via strong relationship so mood decay runs.
        let mut rel = Relationship::default();
        rel.affection = 5.0;
        world.set_relationship(NpcId(1), NpcId(2), rel);

        let sim_npc = simulator.get_active_npc_mut(NpcId(2)).unwrap();
        sim_npc.stats.set(StatKind::Mood, 5.0);

        simulator.tick(&mut world);

        let updated = simulator.get_active_npc(NpcId(2)).unwrap();
        assert!(updated.stats.get(StatKind::Mood) < 5.0);
        assert!(updated.stats.get(StatKind::Mood) <= 10.0);
    }
}

/// Determine scheduled activity for NPC given world time.
fn scheduled_activity_for_npc(world: &WorldState, npc_id: NpcId) -> NpcActivityKind {
    let phase = world.game_time.phase;
    if let Some(proto) = world.npc_prototype(npc_id) {
        proto.schedule.activity_for_phase(phase)
    } else {
        NpcActivityKind::Home
    }
}

/// Update NPC's current_activity each tick, considering busy state and last action.
fn update_npc_activity_state(world: &WorldState, npc: &mut NpcInstance, tick: u64) {
    let scheduled = scheduled_activity_for_npc(world, npc.id);
    if npc.busy_until_tick > tick {
        if let Some(last_action) = &npc.last_action {
            use syn_core::npc_actions::NpcActionKind::*;
            npc.current_activity = match last_action.kind {
                WorkShift => NpcActivityKind::Work,
                SocializeWithNpc => NpcActivityKind::Errands,
                SocialVisitPlayer => NpcActivityKind::Home,
                WithdrawAlone => NpcActivityKind::Home,
                ProvokePlayer => NpcActivityKind::Home,
                SelfImprovement => NpcActivityKind::Home,
                Idle => scheduled,
            };
        } else {
            npc.current_activity = scheduled;
        }
    } else {
        npc.current_activity = scheduled;
    }
}

/// Given a BehaviorKind's action candidates and current activity, filter deterministically.
fn filter_actions_by_schedule(
    candidates: &[NpcActionKind],
    activity: NpcActivityKind,
) -> Vec<NpcActionKind> {
    use NpcActionKind::*;
    use NpcActivityKind::*;

    let mut filtered = Vec::new();
    for &kind in candidates {
        let allowed = match activity {
            Work => matches!(kind, WorkShift | SelfImprovement | Idle),
            School => matches!(kind, SelfImprovement | SocializeWithNpc | Idle),
            Nightlife => matches!(
                kind,
                SocialVisitPlayer | SocializeWithNpc | WithdrawAlone | Idle
            ),
            Home => matches!(
                kind,
                SocialVisitPlayer | WithdrawAlone | SelfImprovement | Idle
            ),
            Errands => matches!(kind, SocializeWithNpc | WithdrawAlone | Idle),
            OnlineOnly => matches!(kind, SocialVisitPlayer | WithdrawAlone | Idle),
            Offscreen => matches!(kind, Idle),
        };
        if allowed {
            filtered.push(kind);
        }
    }
    if filtered.is_empty() {
        filtered.push(NpcActionKind::Idle);
    }
    filtered
}

/// Build action instance considering schedule constraints.
pub fn build_action_instance_from_behavior_and_schedule(
    npc_id: NpcId,
    behavior: &BehaviorSnapshot,
    activity: NpcActivityKind,
) -> NpcActionInstance {
    let base_candidates = behavior_to_candidate_actions(behavior.chosen_intent.kind);
    let candidates = filter_actions_by_schedule(&base_candidates, activity);
    let kind = candidates.first().copied().unwrap_or(NpcActionKind::Idle);
    let effect = base_effect_for_action(kind);
    NpcActionInstance {
        npc_id,
        kind,
        targets_player: behavior.target_player,
        target_npc_id: behavior.target_npc_id,
        effect,
    }
}

/// New canonical LOD tiers for world ticking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpcLodTier {
    /// Full AI, behavior, relationships, storylets.
    Tier1Active,
    /// Reduced-frequency sim (drift, coarse needs).
    Tier2Background,
    /// Dormant population; only macro updates.
    Tier3Dormant,
}

/// Minimal dormant record for macro simulation.
#[derive(Debug)]
pub struct DormantNpcData {
    pub id: NpcId,
    pub age_years: u16,
    pub life_stage: syn_core::LifeStage,
    pub key_stats: Stats,
}

#[derive(Debug, Default)]
pub struct PopulationStore {
    pub dormant: HashMap<NpcId, DormantNpcData>,
}

/// Top-level simulation container for the canonical tick loop.
#[derive(Debug)]
pub struct SimState {
    /// Active NPCs in Tier1 / Tier2.
    pub npc_registry: crate::npc_registry::NpcRegistry,
    /// Dormant Tier3 population.
    pub population: PopulationStore,
    /// Unified hot/cold storage backend.
    pub storage: HybridStorage,
}

impl SimState {
    pub fn new() -> Self {
        let storage = init_default_storage().expect("failed to initialize hybrid storage");
        Self {
            npc_registry: crate::npc_registry::NpcRegistry::default(),
            population: PopulationStore::default(),
            storage,
        }
    }

    /// Create a SimState with temporary storage for testing.
    /// Uses unique paths based on thread ID and timestamp to avoid conflicts.
    #[cfg(any(test, feature = "test-utils"))]
    pub fn new_for_test() -> Self {
        let storage = init_temp_storage().expect("failed to initialize temp storage");
        Self {
            npc_registry: crate::npc_registry::NpcRegistry::default(),
            population: PopulationStore::default(),
            storage,
        }
    }

    pub fn save_active_npc(&self, npc: &StorageNpc) -> Result<(), StorageError> {
        self.storage.save_active(npc)
    }

    pub fn save_dormant_npc(&self, npc: &StorageNpc) -> Result<(), StorageError> {
        self.storage.save_dormant(npc)
    }

    pub fn promote_npc(&mut self, world: &mut WorldState, id: NpcId) -> Result<(), StorageError> {
        self.storage.promote(id.0)?;
        if let Some(stored) = self.storage.load_active(id.0)? {
            let core_npc = storage_to_core_npc(&stored);
            world.npcs.insert(id, core_npc.clone());
            self.population.dormant.remove(&id);
            let mut sim = SimulatedNpc::new(core_npc);
            sim.stats.set(StatKind::Health, stored.health);
            sim.stats.set(StatKind::Wealth, stored.wealth as f32);
            self.npc_registry.instances.insert(
                id,
                NpcInstance {
                    id,
                    lod: NpcLod::Tier2Active,
                    tier: NpcLodTier::Tier1Active,
                    sim,
                    last_tick: world.current_tick.0,
                    behavior: None,
                    busy_until_tick: 0,
                    last_action: None,
                    current_activity: syn_core::npc::NpcActivityKind::Home,
                },
            );
        }
        Ok(())
    }

    pub fn demote_npc(&mut self, _world: &mut WorldState, id: NpcId) -> Result<(), StorageError> {
        if let Some(instance) = self.npc_registry.instances.remove(&id) {
            let storage_npc = core_to_storage_npc(&instance.sim.abstract_npc, Some(instance.sim.current_stats()));
            self.storage.save_active(&storage_npc)?;
            self.storage.demote(id.0)?;
            self.population.dormant.insert(
                id,
                DormantNpcData {
                    id,
                    age_years: instance.sim.abstract_npc.age as u16,
                    life_stage: life_stage_from_age(instance.sim.abstract_npc.age as u16),
                    key_stats: instance.sim.stats.clone(),
                },
            );
        }
        Ok(())
    }
}

fn init_default_storage() -> Result<HybridStorage, StorageError> {
    let data_dir = Path::new("data");
    let _ = fs::create_dir_all(data_dir);
    let hot_path = data_dir.join("hot.redb");
    let cold_path = data_dir.join("world.duckdb");
    HybridStorage::new(
        hot_path.to_string_lossy().as_ref(),
        cold_path.to_string_lossy().as_ref(),
    )
}

/// Initialize storage with unique temporary paths for testing.
#[cfg(any(test, feature = "test-utils"))]
fn init_temp_storage() -> Result<HybridStorage, StorageError> {
    use std::time::SystemTime;
    use std::thread;

    let thread_id = format!("{:?}", thread::current().id());
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    
    let unique_id = format!("{}_{}", thread_id.replace(|c: char| !c.is_alphanumeric(), "_"), timestamp);
    let temp_dir = std::env::temp_dir().join(format!("syn_test_{}", unique_id));
    let _ = fs::create_dir_all(&temp_dir);
    
    let hot_path = temp_dir.join("hot.redb");
    let cold_path = temp_dir.join("world.duckdb");
    HybridStorage::new(
        hot_path.to_string_lossy().as_ref(),
        cold_path.to_string_lossy().as_ref(),
    )
}

fn district_to_code(district: &str) -> u16 {
    district
        .bytes()
        .fold(0u16, |acc, b| acc.wrapping_add(b as u16))
}

fn core_to_storage_npc(core: &AbstractNpc, stats: Option<&Stats>) -> StorageNpc {
    let wealth = stats
        .map(|s| s.get(StatKind::Wealth) as i32)
        .unwrap_or_default();
    let health = stats
        .map(|s| s.get(StatKind::Health))
        .unwrap_or_default();
    StorageNpc {
        id: core.id.0,
        age: core.age.min(u16::MAX as u32) as u16,
        district: district_to_code(&core.district),
        wealth,
        health,
        seed: core.seed,
    }
}

fn storage_to_core_npc(stored: &StorageNpc) -> AbstractNpc {
    AbstractNpc {
        id: NpcId(stored.id),
        age: stored.age as u32,
        // Keep life stage derivable from age; do not copy player state.
        job: String::new(),
        district: format!("District{}", stored.district),
        household_id: 0,
        traits: Default::default(),
        seed: stored.seed,
        attachment_style: Default::default(),
    }
}

fn life_stage_from_age(age: u16) -> syn_core::LifeStage {
    syn_core::LifeStage::from_age(age as u32)
}

// -------- Tick cadence helpers --------
fn is_mid_frequency_tick(time: &GameTime) -> bool {
    time.tick_index % 6 == 0
}

fn is_low_frequency_tick(time: &GameTime) -> bool {
    time.tick_index % 24 == 0
}

// -------- System wrappers (thin stubs for now) --------
/// Relationship drift per-NPC placeholder.
pub fn tick_relationship_drift_for_npc(_world: &mut WorldState, _npc: &mut NpcInstance) {
    // You can wire in RelationshipDriftSystem per-actor later; keep minimal for now.
}

/// Mood/needs decay placeholder.
pub fn tick_mood_for_npc(_world: &mut WorldState, npc: &mut NpcInstance) {
    // Apply a basic decay and tick cooldowns to keep motion.
    npc.sim.apply_mood_decay();
    npc.sim.tick_cooldowns();
}

/// Global memory decay / pruning low-frequency pass.
pub fn tick_memory_decay(_world: &mut WorldState) {
    // Integrate with syn_memory when ready.
}

/// Macro tick for dormant population entries.
pub fn tick_dormant_npc_macro(_world: &mut WorldState, _npc: &mut DormantNpcData) {
    // Placeholder: age, coarse stat drift, etc.
}

fn tick_lod_transitions(world: &mut WorldState, sim: &mut SimState) {
    let to_demote: Vec<NpcId> = sim
        .npc_registry
        .iter()
        .filter_map(|(id, npc)| (npc.lod == NpcLod::Tier0Dormant).then_some(*id))
        .collect();

    for id in to_demote {
        let result = sim.demote_npc(world, id);
        if let Err(err) = result {
            eprintln!("Failed to demote NPC {id:?}: {err}");
        }
    }
}

fn tick_npc_tier1(world: &mut WorldState, npc: &mut NpcInstance, tick: u64) {
    // Keep current activity up to date for schedule-aware actions.
    update_npc_activity_state(world, npc, tick);
    // 1) Behavior
    evaluate_npc_behavior(world, npc);
    // 2) Action
    maybe_run_npc_action(world, npc, tick);
    // 3) Relationship drift
    tick_relationship_drift_for_npc(world, npc);
    // 4) Mood decay
    tick_mood_for_npc(world, npc);
    npc.last_tick = tick;
}

fn tick_npc_tier2(world: &mut WorldState, npc: &mut NpcInstance, tick: u64) {
    update_npc_activity_state(world, npc, tick);
    evaluate_npc_behavior(world, npc);
    tick_relationship_drift_for_npc(world, npc);
    tick_mood_for_npc(world, npc);
    npc.last_tick = tick;
}

/// Advance the simulation by `ticks` ticks (hours).
/// Each tick advances GameTime and ticks NPCs in tier-specific cadences.
#[deprecated(
    since = "0.1.0",
    note = "Use `tick_simulation` + WorldSimState instead; this legacy entrypoint uses the older, less efficient LOD system and will be removed."
)]
pub fn tick_world(world: &mut WorldState, sim: &mut SimState, ticks: u32) {
    let mut tick_ctx = TickContext::default();
    for _ in 0..ticks {
        // 1) Advance time by one hour through the canonical world tick.
        world.tick(&mut tick_ctx);
        let time = world.game_time;
        let tick_index = tick_ctx.tick_index;
        let mid_freq = is_mid_frequency_tick(&time);
        let low_freq = is_low_frequency_tick(&time);

        // 2) Tick Tier1 and Tier2 NPCs.
        for (_id, npc) in sim.npc_registry.iter_mut() {
            match npc.tier {
                NpcLodTier::Tier1Active => {
                    tick_npc_tier1(world, npc, tick_index);
                }
                NpcLodTier::Tier2Background => {
                    if mid_freq {
                        tick_npc_tier2(world, npc, tick_index);
                    }
                }
                NpcLodTier::Tier3Dormant => {
                    // Active registry shouldn't keep Tier3; ignore.
                }
            }
        }

        // 3) Tick dormant population daily.
        if low_freq {
            for (_id, dormant) in sim.population.dormant.iter_mut() {
                tick_dormant_npc_macro(world, dormant);
            }
        }

        // 4) Global low-frequency systems
        if low_freq {
            tick_memory_decay(world);
        }

        // 5) LOD transitions
        tick_lod_transitions(world, sim);
    }
}

/// Configuration for the unified simulation tick.
#[derive(Debug, Clone)]
pub struct SimulationTickConfig {
    /// Configuration for tier promotion/demotion.
    pub tier_config: TierUpdateConfig,
    /// Configuration for per-tier NPC update frequencies.
    pub npc_update_config: NpcUpdateConfig,
}

impl Default for SimulationTickConfig {
    fn default() -> Self {
        Self {
            tier_config: TierUpdateConfig::default(),
            npc_update_config: NpcUpdateConfig::default(),
        }
    }
}

/// Result of a simulation tick, containing director output if any.
#[derive(Debug)]
pub struct SimulationTickResult {
    /// The tick that was just processed.
    pub tick: syn_core::SimTick,
    /// Whether the director fired a storylet this tick.
    pub storylet_fired: bool,
    /// Key of the fired storylet, if any.
    pub fired_storylet_key: Option<u32>,
}

/// Advance the simulation by one tick with the new tier-based system.
///
/// This function performs simulation steps in the correct order:
/// 1. Advance world time
/// 2. Tier reassignment (promotion/demotion of NPCs)
/// 3. Per-tier NPC updates (stats, relationships)
/// 4. [Director step would go here - caller can invoke separately]
///
/// The director step is intentionally left out of this function to maintain
/// separation of concerns. Callers should invoke the director after this
/// function returns, passing the updated world state.
///
/// # Determinism
/// All operations use domain-separated RNG streams derived from the world seed
/// and current tick, ensuring reproducible results.
pub fn tick_simulation(
    world: &mut WorldState,
    sim_state: &mut WorldSimState,
    config: &SimulationTickConfig,
) -> SimulationTickResult {
    // Advance world time first
    let mut tick_ctx = syn_core::time::TickContext::default();
    world.tick(&mut tick_ctx);
    
    let current_tick = world.current_tick;
    let world_seed = world.seed.0;
    
    // 1. Tier reassignment with domain-separated RNG
    let mut rng_tiers = DeterministicRng::with_domain(world_seed, current_tick.0, "tiers");
    systems::update_npc_tiers_for_tick(world, sim_state, &config.tier_config, &mut rng_tiers);
    
    // 2. Per-tier NPC updates with separate RNG stream
    let mut rng_updates = DeterministicRng::with_domain(world_seed, current_tick.0, "npc_updates");
    systems::update_npcs_for_tick(world, sim_state, &config.npc_update_config, &mut rng_updates);
    
    // Return result - caller should invoke director with updated state
    SimulationTickResult {
        tick: current_tick,
        storylet_fired: false,
        fired_storylet_key: None,
    }
}

/// Advance the simulation by multiple ticks.
///
/// Convenience wrapper around `tick_simulation` for advancing multiple ticks.
pub fn tick_simulation_n(
    world: &mut WorldState,
    sim_state: &mut WorldSimState,
    config: &SimulationTickConfig,
    n: u32,
) -> Vec<SimulationTickResult> {
    let mut results = Vec::with_capacity(n as usize);
    for _ in 0..n {
        results.push(tick_simulation(world, sim_state, config));
    }
    results
}

/// Canonical helper to advance the simulation by N ticks using the modern pipeline.
/// Replaces `tick_world` for all new tests and tools.
pub fn advance_simulation_ticks(
    world: &mut WorldState,
    sim_state: &mut WorldSimState,
    config: &SimulationTickConfig,
    ticks: u32,
) {
    for _ in 0..ticks {
        tick_simulation(world, sim_state, config);
    }
}

