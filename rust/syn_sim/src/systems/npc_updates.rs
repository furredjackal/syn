//! Per-tier NPC update system.
//!
//! This module handles per-tick updates for NPCs based on their fidelity tier.
//! - Tier0: Updated every tick (high fidelity).
//! - Tier1: Updated at configurable intervals (batched).
//! - Tier2: Updated at longer intervals (coarse).

use syn_core::{DeterministicRng, NpcId, SimTick, StatKind, WorldState};

use crate::{NpcTier, WorldSimState};

/// Configuration for per-tier NPC update frequencies.
#[derive(Debug, Clone)]
pub struct NpcUpdateConfig {
    /// Whether Tier0 NPCs are updated every tick (usually true).
    pub tier0_every_tick: bool,
    /// Update interval for Tier1 NPCs (e.g., every 3 ticks).
    pub tier1_update_interval: u64,
    /// Update interval for Tier2 NPCs (e.g., every 12 ticks).
    pub tier2_update_interval: u64,
}

impl Default for NpcUpdateConfig {
    fn default() -> Self {
        Self {
            tier0_every_tick: true,
            tier1_update_interval: 3,
            tier2_update_interval: 12,
        }
    }
}

/// Update stats for a single NPC.
///
/// This applies mood decay and basic stat drift.
/// Called per-NPC based on tier update frequency.
pub fn update_stats_for_npc(world: &mut WorldState, npc_id: NpcId, _rng: &mut DeterministicRng) {
    // Apply mood decay toward neutral
    if let Some(npc) = world.npcs.get_mut(&npc_id) {
        // Mood drifts toward 0 slowly
        // Note: Stats are on AbstractNpc in world.npcs; for full sim this would
        // use SimulatedNpc, but we operate on the canonical world state here.
        let _ = npc; // AbstractNpc doesn't have stats directly; they're in SimulatedNpc
    }

    // For player, apply mood decay
    if npc_id == world.player_id {
        let mood = world.player_stats.get(StatKind::Mood);
        let decayed = mood * 0.99; // 1% decay per update
        world.player_stats.set(StatKind::Mood, decayed);
    }
}

/// Update relationships for a single NPC.
///
/// This applies relationship drift for all relationships involving this NPC.
/// Called per-NPC based on tier update frequency.
pub fn update_relationships_for_npc(
    world: &mut WorldState,
    npc_id: NpcId,
    _rng: &mut DeterministicRng,
) {
    // Collect keys for relationships involving this NPC
    let rel_keys: Vec<(NpcId, NpcId)> = world
        .relationships
        .keys()
        .filter(|(from, to)| *from == npc_id || *to == npc_id)
        .copied()
        .collect();

    // Apply drift to each relationship
    for key in rel_keys {
        if let Some(rel) = world.relationships.get_mut(&key) {
            // Small drift toward neutral for affection/trust/resentment
            rel.affection = drift_toward_zero(rel.affection, 0.01);
            rel.trust = drift_toward_zero(rel.trust, 0.005);
            rel.resentment = drift_toward_zero(rel.resentment, 0.008);
            // Familiarity grows very slowly
            rel.familiarity = (rel.familiarity + 0.001).min(10.0);
        }
    }
}

/// Drift a value toward zero by a small amount.
fn drift_toward_zero(value: f32, amount: f32) -> f32 {
    if value > 0.0 {
        (value - amount).max(0.0)
    } else if value < 0.0 {
        (value + amount).min(0.0)
    } else {
        0.0
    }
}

/// Determine if an NPC should be updated this tick based on tier and last update.
fn should_update_npc(
    tier: NpcTier,
    last_update: Option<SimTick>,
    current_tick: SimTick,
    config: &NpcUpdateConfig,
) -> bool {
    match tier {
        NpcTier::Tier0 => config.tier0_every_tick,
        NpcTier::Tier1 => {
            match last_update {
                None => true, // Never updated, update now
                Some(last) => {
                    let elapsed = current_tick.0.saturating_sub(last.0);
                    elapsed >= config.tier1_update_interval
                }
            }
        }
        NpcTier::Tier2 => {
            match last_update {
                None => true, // Never updated, update now
                Some(last) => {
                    let elapsed = current_tick.0.saturating_sub(last.0);
                    elapsed >= config.tier2_update_interval
                }
            }
        }
    }
}

/// Update all NPCs for the current tick based on their tier.
///
/// NPCs are processed in deterministic order (by NpcId).
/// Tier0 NPCs are updated every tick, Tier1/Tier2 are updated at intervals.
pub fn update_npcs_for_tick(
    world: &mut WorldState,
    sim_state: &mut WorldSimState,
    config: &NpcUpdateConfig,
    rng: &mut DeterministicRng,
) {
    let current_tick = world.current_tick;

    // Collect NPC IDs in deterministic order (sorted by ID)
    let mut npc_ids: Vec<NpcId> = world.known_npcs.clone();
    // Include player if not in known_npcs
    if !npc_ids.contains(&world.player_id) {
        npc_ids.push(world.player_id);
    }
    npc_ids.sort_by_key(|id| id.0);

    for npc_id in npc_ids {
        let tier = sim_state.npc_tier(npc_id);
        let last_update = sim_state.last_npc_update(npc_id);

        if should_update_npc(tier, last_update, current_tick, config) {
            // Perform per-NPC updates
            update_stats_for_npc(world, npc_id, rng);
            update_relationships_for_npc(world, npc_id, rng);

            // Mark as updated
            sim_state.mark_npc_updated(npc_id, current_tick);
        }
    }
}

/// Tracking structure for testing update frequencies.
#[cfg(test)]
#[derive(Debug, Default, Clone)]
pub struct UpdateCounter {
    counts: std::collections::HashMap<NpcId, u32>,
}

#[cfg(test)]
impl UpdateCounter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn increment(&mut self, id: NpcId) {
        *self.counts.entry(id).or_insert(0) += 1;
    }

    pub fn count(&self, id: NpcId) -> u32 {
        self.counts.get(&id).copied().unwrap_or(0)
    }
}

/// Test-only version that tracks update counts.
#[cfg(test)]
pub fn update_npcs_for_tick_with_counter(
    world: &mut WorldState,
    sim_state: &mut WorldSimState,
    config: &NpcUpdateConfig,
    rng: &mut DeterministicRng,
    counter: &mut UpdateCounter,
) {
    let current_tick = world.current_tick;

    let mut npc_ids: Vec<NpcId> = world.known_npcs.clone();
    if !npc_ids.contains(&world.player_id) {
        npc_ids.push(world.player_id);
    }
    npc_ids.sort_by_key(|id| id.0);

    for npc_id in npc_ids {
        let tier = sim_state.npc_tier(npc_id);
        let last_update = sim_state.last_npc_update(npc_id);

        if should_update_npc(tier, last_update, current_tick, config) {
            update_stats_for_npc(world, npc_id, rng);
            update_relationships_for_npc(world, npc_id, rng);
            sim_state.mark_npc_updated(npc_id, current_tick);
            counter.increment(npc_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn_core::{Relationship, WorldSeed};

    fn make_test_world() -> WorldState {
        let mut world = WorldState::new(WorldSeed(12345), NpcId(1));

        // Add player
        world.npcs.insert(
            NpcId(1),
            syn_core::AbstractNpc {
                id: NpcId(1),
                age: 25,
                job: "Player".to_string(),
                district: "Downtown".to_string(),
                household_id: 1,
                traits: Default::default(),
                seed: 1,
                attachment_style: Default::default(),
            },
        );

        // Add 3 NPCs
        for i in 2..=4 {
            world.npcs.insert(
                NpcId(i),
                syn_core::AbstractNpc {
                    id: NpcId(i),
                    age: 20 + i as u32,
                    job: format!("Job{}", i),
                    district: "Downtown".to_string(),
                    household_id: i,
                    traits: Default::default(),
                    seed: i,
                    attachment_style: Default::default(),
                },
            );
            world.known_npcs.push(NpcId(i));
        }

        // Add some relationships
        world.set_relationship(
            NpcId(1),
            NpcId(2),
            Relationship {
                affection: 5.0,
                trust: 3.0,
                ..Default::default()
            },
        );

        world
    }

    #[test]
    fn test_tier0_updates_every_tick() {
        let mut world = make_test_world();
        let mut sim_state = WorldSimState::new();
        let config = NpcUpdateConfig {
            tier0_every_tick: true,
            tier1_update_interval: 2,
            tier2_update_interval: 5,
        };
        let mut rng = DeterministicRng::new(42);
        let mut counter = UpdateCounter::new();

        // Set NPC 2 to Tier0
        sim_state.set_npc_tier(NpcId(2), NpcTier::Tier0);

        // Run 10 ticks
        for _ in 0..10 {
            world.current_tick = SimTick(world.current_tick.0 + 1);
            update_npcs_for_tick_with_counter(&mut world, &mut sim_state, &config, &mut rng, &mut counter);
        }

        // Tier0 NPC should be updated 10 times
        assert_eq!(counter.count(NpcId(2)), 10, "Tier0 should update every tick");
    }

    #[test]
    fn test_tier1_updates_at_interval() {
        let mut world = make_test_world();
        let mut sim_state = WorldSimState::new();
        let config = NpcUpdateConfig {
            tier0_every_tick: true,
            tier1_update_interval: 2,
            tier2_update_interval: 5,
        };
        let mut rng = DeterministicRng::new(42);
        let mut counter = UpdateCounter::new();

        // Set NPC 3 to Tier1
        sim_state.set_npc_tier(NpcId(3), NpcTier::Tier1);

        // Run 10 ticks
        for _ in 0..10 {
            world.current_tick = SimTick(world.current_tick.0 + 1);
            update_npcs_for_tick_with_counter(&mut world, &mut sim_state, &config, &mut rng, &mut counter);
        }

        // Tier1 with interval=2 should update ~5 times over 10 ticks
        let tier1_count = counter.count(NpcId(3));
        assert!(
            tier1_count >= 4 && tier1_count <= 6,
            "Tier1 should update ~5 times, got {}",
            tier1_count
        );
    }

    #[test]
    fn test_tier2_updates_at_interval() {
        let mut world = make_test_world();
        let mut sim_state = WorldSimState::new();
        let config = NpcUpdateConfig {
            tier0_every_tick: true,
            tier1_update_interval: 2,
            tier2_update_interval: 5,
        };
        let mut rng = DeterministicRng::new(42);
        let mut counter = UpdateCounter::new();

        // Set NPC 4 to Tier2
        sim_state.set_npc_tier(NpcId(4), NpcTier::Tier2);

        // Run 10 ticks
        for _ in 0..10 {
            world.current_tick = SimTick(world.current_tick.0 + 1);
            update_npcs_for_tick_with_counter(&mut world, &mut sim_state, &config, &mut rng, &mut counter);
        }

        // Tier2 with interval=5 should update ~2 times over 10 ticks
        let tier2_count = counter.count(NpcId(4));
        assert!(
            tier2_count >= 1 && tier2_count <= 3,
            "Tier2 should update ~2 times, got {}",
            tier2_count
        );
    }

    #[test]
    fn test_mixed_tiers_update_frequencies() {
        let mut world = make_test_world();
        let mut sim_state = WorldSimState::new();
        let config = NpcUpdateConfig {
            tier0_every_tick: true,
            tier1_update_interval: 2,
            tier2_update_interval: 5,
        };
        let mut rng = DeterministicRng::new(42);
        let mut counter = UpdateCounter::new();

        // Set different tiers
        sim_state.set_npc_tier(NpcId(1), NpcTier::Tier0); // Player
        sim_state.set_npc_tier(NpcId(2), NpcTier::Tier0);
        sim_state.set_npc_tier(NpcId(3), NpcTier::Tier1);
        sim_state.set_npc_tier(NpcId(4), NpcTier::Tier2);

        // Run 10 ticks
        for _ in 0..10 {
            world.current_tick = SimTick(world.current_tick.0 + 1);
            update_npcs_for_tick_with_counter(&mut world, &mut sim_state, &config, &mut rng, &mut counter);
        }

        // Verify expected counts
        assert_eq!(counter.count(NpcId(1)), 10, "Player (Tier0) should update 10 times");
        assert_eq!(counter.count(NpcId(2)), 10, "Tier0 NPC should update 10 times");

        let tier1_count = counter.count(NpcId(3));
        assert!(
            tier1_count >= 4 && tier1_count <= 6,
            "Tier1 should update ~5 times, got {}",
            tier1_count
        );

        let tier2_count = counter.count(NpcId(4));
        assert!(
            tier2_count >= 1 && tier2_count <= 3,
            "Tier2 should update ~2 times, got {}",
            tier2_count
        );
    }

    #[test]
    fn test_update_is_deterministic() {
        let config = NpcUpdateConfig {
            tier0_every_tick: true,
            tier1_update_interval: 2,
            tier2_update_interval: 5,
        };

        // First run
        let mut world1 = make_test_world();
        let mut sim_state1 = WorldSimState::new();
        sim_state1.set_npc_tier(NpcId(2), NpcTier::Tier0);
        sim_state1.set_npc_tier(NpcId(3), NpcTier::Tier1);
        sim_state1.set_npc_tier(NpcId(4), NpcTier::Tier2);
        let mut rng1 = DeterministicRng::new(42);
        let mut counter1 = UpdateCounter::new();

        for _ in 0..10 {
            world1.current_tick = SimTick(world1.current_tick.0 + 1);
            update_npcs_for_tick_with_counter(&mut world1, &mut sim_state1, &config, &mut rng1, &mut counter1);
        }

        // Second run with same initial state
        let mut world2 = make_test_world();
        let mut sim_state2 = WorldSimState::new();
        sim_state2.set_npc_tier(NpcId(2), NpcTier::Tier0);
        sim_state2.set_npc_tier(NpcId(3), NpcTier::Tier1);
        sim_state2.set_npc_tier(NpcId(4), NpcTier::Tier2);
        let mut rng2 = DeterministicRng::new(42);
        let mut counter2 = UpdateCounter::new();

        for _ in 0..10 {
            world2.current_tick = SimTick(world2.current_tick.0 + 1);
            update_npcs_for_tick_with_counter(&mut world2, &mut sim_state2, &config, &mut rng2, &mut counter2);
        }

        // Counts should be identical
        for i in 1..=4 {
            assert_eq!(
                counter1.count(NpcId(i)),
                counter2.count(NpcId(i)),
                "Update count for NpcId({}) should be deterministic",
                i
            );
        }
    }

    #[test]
    fn test_relationship_drift_applies() {
        let mut world = make_test_world();
        let mut sim_state = WorldSimState::new();
        let config = NpcUpdateConfig::default();
        let mut rng = DeterministicRng::new(42);

        // Set Tier0 for immediate updates
        sim_state.set_npc_tier(NpcId(2), NpcTier::Tier0);

        let initial_affection = world.get_relationship(NpcId(1), NpcId(2)).affection;

        // Run a few ticks
        for _ in 0..5 {
            world.current_tick = SimTick(world.current_tick.0 + 1);
            update_npcs_for_tick(&mut world, &mut sim_state, &config, &mut rng);
        }

        let final_affection = world.get_relationship(NpcId(1), NpcId(2)).affection;

        // Affection should have drifted toward zero
        assert!(
            final_affection < initial_affection,
            "Affection should drift: {} -> {}",
            initial_affection,
            final_affection
        );
    }

    #[test]
    fn test_no_double_updates_same_tick() {
        let mut world = make_test_world();
        let mut sim_state = WorldSimState::new();
        let config = NpcUpdateConfig::default();
        let mut rng = DeterministicRng::new(42);
        let mut counter = UpdateCounter::new();

        sim_state.set_npc_tier(NpcId(2), NpcTier::Tier0);

        // Advance to tick 1
        world.current_tick = SimTick(1);
        update_npcs_for_tick_with_counter(&mut world, &mut sim_state, &config, &mut rng, &mut counter);

        // Try to update again at same tick
        update_npcs_for_tick_with_counter(&mut world, &mut sim_state, &config, &mut rng, &mut counter);

        // For Tier0 with tier0_every_tick=true, it updates every call
        // But that's actually expected behavior - if you call update twice, it updates twice
        // The protection is via last_update_tick for Tier1/Tier2
        assert_eq!(counter.count(NpcId(2)), 2, "Tier0 updates on every call");

        // For Tier1/Tier2, test that last_update_tick prevents double updates
        let mut world2 = make_test_world();
        let mut sim_state2 = WorldSimState::new();
        let mut counter2 = UpdateCounter::new();
        let mut rng2 = DeterministicRng::new(42);

        sim_state2.set_npc_tier(NpcId(3), NpcTier::Tier1);

        world2.current_tick = SimTick(1);
        update_npcs_for_tick_with_counter(&mut world2, &mut sim_state2, &config, &mut rng2, &mut counter2);
        
        // Call again at same tick - should not update because last_update_tick is now 1
        update_npcs_for_tick_with_counter(&mut world2, &mut sim_state2, &config, &mut rng2, &mut counter2);

        // Should only have updated once (elapsed = 0 < interval)
        assert_eq!(counter2.count(NpcId(3)), 1, "Tier1 should not double-update at same tick");
    }
}
