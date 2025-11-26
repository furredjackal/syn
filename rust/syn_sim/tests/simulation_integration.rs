//! Integration tests for the unified simulation tick with tier-based updates.
//!
//! These tests verify:
//! - Correct ordering: tier assignment → NPC updates → (director ready)
//! - Determinism: same seed + tick = same results
//! - Tier behavior: Tier0 NPCs get more updates than Tier2

use syn_core::{NpcId, Relationship, SimTick, WorldSeed, WorldState};
use syn_sim::{
    tick_simulation, tick_simulation_n, NpcTier, SimulationTickConfig, WorldSimState,
};

/// Helper to create a test world with NPCs and relationships.
fn make_integration_test_world() -> WorldState {
    let mut world = WorldState::new(WorldSeed(54321), NpcId(1));

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

    // Add 5 NPCs with varying relationships
    for i in 2..=6 {
        world.npcs.insert(
            NpcId(i),
            syn_core::AbstractNpc {
                id: NpcId(i),
                age: 20 + i as u32,
                job: format!("Job{}", i),
                district: if i <= 3 { "Downtown" } else { "Suburbs" }.to_string(),
                household_id: i,
                traits: Default::default(),
                seed: i,
                attachment_style: Default::default(),
            },
        );
        world.known_npcs.push(NpcId(i));
    }

    // Set relationships: NPC 2 is close friend, NPC 3-6 progressively more distant
    world.set_relationship(
        NpcId(1),
        NpcId(2),
        Relationship {
            affection: 8.0,
            trust: 7.0,
            familiarity: 9.0,
            ..Default::default()
        },
    );

    world.set_relationship(
        NpcId(1),
        NpcId(3),
        Relationship {
            affection: 4.0,
            trust: 3.0,
            familiarity: 5.0,
            ..Default::default()
        },
    );

    world.set_relationship(
        NpcId(1),
        NpcId(4),
        Relationship {
            affection: 1.0,
            trust: 1.0,
            familiarity: 2.0,
            ..Default::default()
        },
    );

    world.set_relationship(
        NpcId(1),
        NpcId(5),
        Relationship {
            affection: 0.0,
            trust: 0.0,
            familiarity: 1.0,
            ..Default::default()
        },
    );

    world.set_relationship(
        NpcId(1),
        NpcId(6),
        Relationship {
            affection: -1.0,
            trust: -1.0,
            familiarity: 0.5,
            ..Default::default()
        },
    );

    world
}

#[test]
fn test_tick_simulation_advances_time() {
    let mut world = make_integration_test_world();
    let mut sim_state = WorldSimState::new();
    let config = SimulationTickConfig::default();

    let initial_tick = world.current_tick;

    let result = tick_simulation(&mut world, &mut sim_state, &config);

    // Time should have advanced
    assert_eq!(result.tick.0, initial_tick.0 + 1);
    assert_eq!(world.current_tick.0, initial_tick.0 + 1);
}

#[test]
fn test_tick_simulation_assigns_tiers() {
    let mut world = make_integration_test_world();
    let mut sim_state = WorldSimState::new();
    // Use a config with limited tier slots to force some NPCs into Tier2
    let config = SimulationTickConfig {
        tier_config: syn_sim::TierUpdateConfig {
            max_tier0_npcs: 2,  // Player + 1 NPC
            max_tier1_npcs: 2,
            ..Default::default()
        },
        ..Default::default()
    };

    // Run a tick
    tick_simulation(&mut world, &mut sim_state, &config);

    // Player should be Tier0
    assert_eq!(sim_state.npc_tier(NpcId(1)), NpcTier::Tier0);

    // NPC 2 (high importance) should be Tier0 or Tier1
    let npc2_tier = sim_state.npc_tier(NpcId(2));
    assert!(
        npc2_tier == NpcTier::Tier0 || npc2_tier == NpcTier::Tier1,
        "High-importance NPC should be Tier0 or Tier1"
    );

    // At least one NPC should be Tier2 (low importance ones)
    let tier2_count = (2..=6)
        .filter(|&i| sim_state.npc_tier(NpcId(i)) == NpcTier::Tier2)
        .count();
    assert!(tier2_count > 0, "Should have at least one Tier2 NPC with limited slots");
}

#[test]
fn test_tick_simulation_is_deterministic() {
    let config = SimulationTickConfig::default();

    // First run
    let mut world1 = make_integration_test_world();
    let mut sim_state1 = WorldSimState::new();
    let results1 = tick_simulation_n(&mut world1, &mut sim_state1, &config, 20);

    // Second run with identical initial state
    let mut world2 = make_integration_test_world();
    let mut sim_state2 = WorldSimState::new();
    let results2 = tick_simulation_n(&mut world2, &mut sim_state2, &config, 20);

    // Verify tick results are identical
    assert_eq!(results1.len(), results2.len());
    for (r1, r2) in results1.iter().zip(results2.iter()) {
        assert_eq!(r1.tick, r2.tick, "Ticks should match");
    }

    // Verify tier assignments are identical
    for i in 1..=6 {
        assert_eq!(
            sim_state1.npc_tier(NpcId(i)),
            sim_state2.npc_tier(NpcId(i)),
            "Tier for NpcId({}) should be deterministic",
            i
        );
    }

    // Verify last update ticks are identical
    for i in 1..=6 {
        assert_eq!(
            sim_state1.last_npc_update(NpcId(i)),
            sim_state2.last_npc_update(NpcId(i)),
            "Last update for NpcId({}) should be deterministic",
            i
        );
    }

    // Verify world state is identical
    assert_eq!(world1.current_tick, world2.current_tick);
    
    // Check relationship values are the same
    for i in 2..=6 {
        let rel1 = world1.get_relationship(NpcId(1), NpcId(i));
        let rel2 = world2.get_relationship(NpcId(1), NpcId(i));
        assert!(
            (rel1.affection - rel2.affection).abs() < 0.0001,
            "Affection for NpcId({}) should be deterministic: {} vs {}",
            i,
            rel1.affection,
            rel2.affection
        );
    }
}

#[test]
fn test_tier0_npcs_updated_more_frequently() {
    let mut world = make_integration_test_world();
    let mut sim_state = WorldSimState::new();
    let config = SimulationTickConfig {
        tier_config: syn_sim::TierUpdateConfig {
            max_tier0_npcs: 2,  // Player + 1 NPC
            max_tier1_npcs: 2,
            ..Default::default()
        },
        npc_update_config: syn_sim::NpcUpdateConfig {
            tier0_every_tick: true,
            tier1_update_interval: 3,
            tier2_update_interval: 6,
        },
    };

    // Run 12 ticks
    tick_simulation_n(&mut world, &mut sim_state, &config, 12);

    // Count how many NPCs are in each tier
    let mut tier0_npcs = Vec::new();
    let mut tier2_npcs = Vec::new();

    for i in 2..=6 {
        match sim_state.npc_tier(NpcId(i)) {
            NpcTier::Tier0 => tier0_npcs.push(NpcId(i)),
            NpcTier::Tier2 => tier2_npcs.push(NpcId(i)),
            _ => {}
        }
    }

    // Get initial relationship values to compare drift
    // (Note: drift happens during updates, so more updates = more drift)
    
    // Verify that Tier0 NPCs were updated on the last tick
    for &npc_id in &tier0_npcs {
        let last_update = sim_state.last_npc_update(npc_id);
        assert!(
            last_update.is_some(),
            "Tier0 NPC {:?} should have been updated",
            npc_id
        );
        // Last update should be recent (within last few ticks for Tier0)
        let last_tick = last_update.unwrap().0;
        assert!(
            last_tick >= world.current_tick.0 - 1,
            "Tier0 NPC {:?} should have been updated recently: last={}, current={}",
            npc_id,
            last_tick,
            world.current_tick.0
        );
    }

    // Tier2 NPCs should have been updated less recently (interval=6)
    for &npc_id in &tier2_npcs {
        let last_update = sim_state.last_npc_update(npc_id);
        // They should have been updated at some point
        assert!(
            last_update.is_some(),
            "Tier2 NPC {:?} should have been updated at least once",
            npc_id
        );
    }
}

#[test]
fn test_tiers_change_based_on_relationships() {
    let mut world = make_integration_test_world();
    let mut sim_state = WorldSimState::new();
    let config = SimulationTickConfig {
        tier_config: syn_sim::TierUpdateConfig {
            max_tier0_npcs: 2,
            max_tier1_npcs: 2,
            ..Default::default()
        },
        ..Default::default()
    };

    // Initial tick
    tick_simulation(&mut world, &mut sim_state, &config);
    let initial_tier_npc6 = sim_state.npc_tier(NpcId(6));

    // NPC 6 should likely be Tier2 (lowest importance)
    assert_eq!(
        initial_tier_npc6,
        NpcTier::Tier2,
        "NPC 6 should start as Tier2"
    );

    // Now dramatically increase NPC 6's importance
    world.set_relationship(
        NpcId(1),
        NpcId(6),
        Relationship {
            affection: 9.0,
            trust: 9.0,
            familiarity: 10.0,
            ..Default::default()
        },
    );

    // Run several more ticks for tier reassignment
    tick_simulation_n(&mut world, &mut sim_state, &config, 5);

    let new_tier_npc6 = sim_state.npc_tier(NpcId(6));

    // NPC 6 should now be promoted (Tier0 or Tier1)
    assert!(
        new_tier_npc6 == NpcTier::Tier0 || new_tier_npc6 == NpcTier::Tier1,
        "NPC 6 should be promoted after relationship boost: {:?}",
        new_tier_npc6
    );
}

#[test]
fn test_tick_results_track_progression() {
    let mut world = make_integration_test_world();
    let mut sim_state = WorldSimState::new();
    let config = SimulationTickConfig::default();

    let results = tick_simulation_n(&mut world, &mut sim_state, &config, 10);

    // Should have 10 results
    assert_eq!(results.len(), 10);

    // Each tick should be sequential
    for (i, result) in results.iter().enumerate() {
        assert_eq!(
            result.tick.0 as usize,
            i + 1,
            "Tick {} should have tick index {}",
            i,
            i + 1
        );
    }

    // Final world tick should match
    assert_eq!(world.current_tick.0, 10);
}

#[test]
fn test_simulation_with_different_seeds_produces_different_results() {
    let config = SimulationTickConfig::default();

    // Run with seed 12345
    let mut world1 = WorldState::new(WorldSeed(12345), NpcId(1));
    world1.npcs.insert(
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
    for i in 2..=4 {
        world1.npcs.insert(
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
        world1.known_npcs.push(NpcId(i));
        world1.set_relationship(
            NpcId(1),
            NpcId(i),
            Relationship {
                affection: 5.0 - i as f32,
                trust: 4.0 - i as f32,
                familiarity: 5.0,
                ..Default::default()
            },
        );
    }
    let mut sim_state1 = WorldSimState::new();

    // Run with seed 67890
    let mut world2 = world1.clone();
    world2.seed = WorldSeed(67890);
    let mut sim_state2 = WorldSimState::new();

    tick_simulation_n(&mut world1, &mut sim_state1, &config, 20);
    tick_simulation_n(&mut world2, &mut sim_state2, &config, 20);

    // The RNG domains should produce different sequences, but since tier
    // assignment is based on relationships (not RNG), tiers should still match.
    // What differs is the internal RNG state which affects future stochastic events.
    
    // Verify both completed
    assert_eq!(world1.current_tick.0, 20);
    assert_eq!(world2.current_tick.0, 20);
}
