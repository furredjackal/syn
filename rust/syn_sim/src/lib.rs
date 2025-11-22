//! syn_sim: Simulation tick and world state advancement.
//!
//! Handles the core simulation loop, NPC updates, time progression, and LOD tier management.

use syn_core::{
    AbstractNpc, BehaviorAction, BehaviorNeed, DeterministicRng, NpcId, StatKind, Stats, WorldState,
};
use std::collections::HashMap;

/// Level of Detail tier for NPC simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LodTier {
    /// Active NPCs: full utility scoring, complex mood updates, complete memory checks
    High,
    /// Background NPCs: simplified behavior, limited event evaluation
    Medium,
    /// Dormant NPCs: stat drift only, no minute-to-minute decisions
    Low,
}

/// NPC state during simulation (instantiated in ECS-like system).
#[derive(Debug, Clone)]
pub struct SimulatedNpc {
    pub id: NpcId,
    pub abstract_npc: AbstractNpc,
    pub stats: Stats,
    pub lod_tier: LodTier,
    pub needs: HashMap<String, f32>, // e.g., {"social": 0.7, "stimulation": 0.4}
    pub behavioral_cooldowns: HashMap<String, u32>, // e.g., {"dialogue": 5 ticks}
}

impl SimulatedNpc {
    pub fn new(abstract_npc: AbstractNpc) -> Self {
        SimulatedNpc {
            id: abstract_npc.id,
            abstract_npc,
            stats: Stats::default(),
            lod_tier: LodTier::Low,
            needs: HashMap::new(),
            behavioral_cooldowns: HashMap::new(),
        }
    }

    /// Advance NPC cooldowns by one tick.
    pub fn tick_cooldowns(&mut self) {
        self.behavioral_cooldowns.retain(|_, remaining| {
            *remaining = remaining.saturating_sub(1);
            *remaining > 0
        });
    }

    /// Apply mood decay toward baseline (linear slow drift).
    pub fn apply_mood_decay(&mut self) {
        const MOOD_DECAY_RATE: f32 = 0.05;
        let current = self.stats.get(StatKind::Mood);
        let delta = -current * MOOD_DECAY_RATE;
        self.stats.apply_delta(StatKind::Mood, delta);
    }

    /// Recalculate needs based on stats, mood, and personality.
    pub fn update_needs(&mut self) {
        let mut needs = HashMap::new();
        let mood = self.stats.get(StatKind::Mood);

        // Social need: influenced by mood and sociability
        let social_base = (self.abstract_npc.traits.sociability - 50.0) / 100.0;
        let social_mood = if mood < -5.0 { 1.0 } else if mood > 5.0 { -0.5 } else { 0.0 };
        needs.insert("social".to_string(), (social_base + social_mood).clamp(0.0, 1.0));

        // Stimulation need: curiosity, impulsivity, boredom
        let stim_base = (self.abstract_npc.traits.impulsivity - 50.0) / 100.0;
        needs.insert("stimulation".to_string(), stim_base.clamp(0.0, 1.0));

        // Security need: affected by stability and health
        let security_stability = (100.0 - self.abstract_npc.traits.stability) / 100.0;
        let security_health = if self.stats.get(StatKind::Health) < 30.0 { 0.7 } else { 0.0 };
        needs.insert("security".to_string(), (security_stability + security_health).clamp(0.0, 1.0));

        // Recognition need: ambition + mood boost
        let recognition_base = (self.abstract_npc.traits.ambition - 50.0) / 100.0;
        let recognition_mood = mood.max(0.0) / 10.0;
        needs.insert("recognition".to_string(), (recognition_base + recognition_mood).clamp(0.0, 1.0));

        // Comfort need: mood dips + stability
        let comfort_base = ((0.0 - mood).max(0.0) / 10.0) + ((100.0 - self.abstract_npc.traits.stability) / 200.0);
        needs.insert("comfort".to_string(), comfort_base.clamp(0.0, 1.0));

        self.needs = needs;
    }

    fn need_value(&self, need: BehaviorNeed) -> f32 {
        let value = self
            .needs
            .get(need.as_key())
            .copied()
            .unwrap_or(0.5);
        (0.8 + value).clamp(0.4, 1.8)
    }

    /// Score a behavior action using the full utility equation.
    pub fn score_action(&self, action: BehaviorAction, world: &WorldState) -> f32 {
        let base = action.base_weight();
        let trait_bias = action.trait_bias(&self.abstract_npc.traits);
        let attachment_bias = action.attachment_bias(self.abstract_npc.attachment_style);
        let primary = self.need_value(action.primary_need());
        let need_component = if let Some(secondary) = action.secondary_need() {
            (primary + self.need_value(secondary)) / 2.0
        } else {
            primary
        };
        let mood_mult = action.mood_multiplier(self.stats.get(StatKind::Mood));
        let context = action.context_fit(world);
        (base * trait_bias * attachment_bias * need_component * mood_mult * context).clamp(0.0, 100.0)
    }
}

/// Simulation engine: advances world state by one tick.
pub struct Simulator {
    rng: DeterministicRng,
    active_npcs: HashMap<NpcId, SimulatedNpc>,
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

        // Update LOD assignments
        self.update_lod_tiers(world, world.player_id);

        // Tick all active NPCs
        self.tick_npcs(world);

        // Slowly drift passive stats for non-instantiated NPCs
        self.apply_population_drift(world);

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
        world.narrative_heat = 30.0;

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
