
/// Simulation engine: advances world state by one tick.
pub struct Simulator {
    rng: DeterministicRng,
    active_npcs: HashMap<NpcId, SimulatedNpc>,
}

fn gather_recent_memory_flags(_world: &WorldState) -> (bool, bool, bool) {
    (false, false, false)
}

fn update_narrative_heat(world: &mut WorldState, config: &NarrativeHeatConfig, stat_profile: Option<&LifeStageConfig>) {
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

fn update_life_stage_from_age(world: &mut WorldState) {
    let new_stage = LifeStage::stage_for_age(world.player_age_years);
    if new_stage != world.player_life_stage {
        world.player_life_stage = new_stage;
    }
}

impl Simulator {
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
    pub fn tick(&mut self, world: &mut WorldState) {
        // Advance world clock
        world.tick();

        // Keep stage in sync with age.
        update_life_stage_from_age(world);

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
        } else if let Some(t) = action.target_npc_id { t } else { world.player_id };
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
pub fn maybe_run_npc_action(
    world: &mut WorldState,
    npc: &mut NpcInstance,
    tick: u64,
) {
    maybe_run_npc_action_with_memory(world, npc, tick, None);
}

/// Variant that can write memories if provided.
pub fn maybe_run_npc_action_with_memory(
    world: &mut WorldState,
    npc: &mut NpcInstance,
    tick: u64,
    mut memory_opt: Option<&mut MemorySystem>,
) {
    if npc.busy_until_tick > tick { return; }
    let behavior = match &npc.behavior { Some(b) => b, None => return };
    // Schedule-aware selection
    let activity = npc.current_activity;
    let action = build_action_instance_from_behavior_and_schedule(npc.id, behavior, activity);
    // Apply effects
    apply_npc_action_effect(world, npc, &action, tick, memory_opt.as_deref_mut());
    // Busy
    if action.effect.busy_for_ticks > 0 { npc.busy_until_tick = tick + action.effect.busy_for_ticks; }
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
pub fn tick_npcs_lod(world: &mut WorldState, registry: &mut crate::npc_registry::NpcRegistry, tick: u64) {
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
    use syn_core::{AttachmentStyle, BehaviorAction, Relationship, Traits, WorldSeed};

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

        assert!(!sim_npc.needs.is_empty());
        assert!(sim_npc.needs.contains_key("social"));
    }

    #[test]
    fn test_behavior_action_scoring_deterministic() {
        let abstract_npc = AbstractNpc {
            id: NpcId(1),
            age: 28,
            job: "Analyst".to_string(),
            district: "Midtown".to_string(),
            household_id: 1,
            traits: Traits {
                stability: 40.0,
                confidence: 60.0,
                sociability: 65.0,
                empathy: 55.0,
                impulsivity: 45.0,
                ambition: 70.0,
                charm: 50.0,
            },
            seed: 999,
            attachment_style: AttachmentStyle::Secure,
        };
        let mut sim_npc = SimulatedNpc::new(abstract_npc);
        sim_npc.stats.set(StatKind::Mood, 2.0);
        sim_npc.stats.set(StatKind::Health, 80.0);
        sim_npc.update_needs();

        let mut world = WorldState::new(WorldSeed(42), NpcId(99));
        world.narrative_heat.set(30.0);

        let first = sim_npc.score_action(BehaviorAction::Work, &world);
        let second = sim_npc.score_action(BehaviorAction::Work, &world);
        assert!((first - second).abs() < f32::EPSILON);
        assert!(first > 0.0);
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
    use NpcActivityKind::*;
    use NpcActionKind::*;

    let mut filtered = Vec::new();
    for &kind in candidates {
        let allowed = match activity {
            Work => matches!(kind, WorkShift | SelfImprovement | Idle),
            School => matches!(kind, SelfImprovement | SocializeWithNpc | Idle),
            Nightlife => matches!(kind, SocialVisitPlayer | SocializeWithNpc | WithdrawAlone | Idle),
            Home => matches!(kind, SocialVisitPlayer | WithdrawAlone | SelfImprovement | Idle),
            Errands => matches!(kind, SocializeWithNpc | WithdrawAlone | Idle),
            OnlineOnly => matches!(kind, SocialVisitPlayer | WithdrawAlone | Idle),
            Offscreen => matches!(kind, Idle),
        };
        if allowed { filtered.push(kind); }
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
use syn_core::time::GameTime;

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
    pub key_stats: syn_core::stats::Stats,
}

#[derive(Debug, Default)]
pub struct PopulationStore {
    pub dormant: std::collections::HashMap<NpcId, DormantNpcData>,
}

/// Top-level simulation container for the canonical tick loop.
#[derive(Debug)]
pub struct SimState {
    /// Active NPCs in Tier1 / Tier2.
    pub npc_registry: crate::npc_registry::NpcRegistry,
    /// Dormant Tier3 population.
    pub population: PopulationStore,
}

impl SimState {
    pub fn new() -> Self {
        Self {
            npc_registry: crate::npc_registry::NpcRegistry::default(),
            population: PopulationStore::default(),
        }
    }
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

fn tick_lod_transitions(_world: &mut WorldState, _sim: &mut SimState) {
    // Placeholder policy hook.
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

use syn_core::WorldState;
use syn_core::time::GameTime as _; // ensure module imported above; alias to avoid warn if shadowed
use crate::{NpcInstance as _, NpcLodTier as _}; // bring types into scope to satisfy lints

/// Advance the simulation by `ticks` ticks (hours).
/// Each tick advances GameTime and ticks NPCs in tier-specific cadences.
pub fn tick_world(world: &mut WorldState, sim: &mut SimState, ticks: u32) {
    for _ in 0..ticks {
        // 1) Advance time by one hour.
        world.game_time.advance_ticks(1);
        let time = world.game_time;
        let mid_freq = is_mid_frequency_tick(&time);
        let low_freq = is_low_frequency_tick(&time);

        // 2) Tick Tier1 and Tier2 NPCs.
        for (_id, npc) in sim.npc_registry.iter_mut() {
            match npc.tier {
                NpcLodTier::Tier1Active => {
                    tick_npc_tier1(world, npc, time.tick_index);
                }
                NpcLodTier::Tier2Background => {
                    if mid_freq {
                        tick_npc_tier2(world, npc, time.tick_index);
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
        if low_freq { tick_memory_decay(world); }

        // 5) LOD transitions
        tick_lod_transitions(world, sim);
    }
}
