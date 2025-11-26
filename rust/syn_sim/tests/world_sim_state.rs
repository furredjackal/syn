use syn_core::{NpcId, SimTick};
use syn_sim::{NpcTier, WorldSimState};

#[test]
fn test_world_sim_state_new() {
    let state = WorldSimState::new();
    assert_eq!(state.npc_count(), 0);
}

#[test]
fn test_npc_tier_defaults_to_tier2() {
    let state = WorldSimState::new();
    // Unknown NPC should default to Tier2
    assert_eq!(state.npc_tier(NpcId(999)), NpcTier::Tier2);
}

#[test]
fn test_set_and_get_npc_tier() {
    let mut state = WorldSimState::new();
    let npc1 = NpcId(1);
    let npc2 = NpcId(2);
    let npc3 = NpcId(3);

    // Set different tiers
    state.set_npc_tier(npc1, NpcTier::Tier0);
    state.set_npc_tier(npc2, NpcTier::Tier1);
    state.set_npc_tier(npc3, NpcTier::Tier2);

    // Verify each NPC has correct tier
    assert_eq!(state.npc_tier(npc1), NpcTier::Tier0);
    assert_eq!(state.npc_tier(npc2), NpcTier::Tier1);
    assert_eq!(state.npc_tier(npc3), NpcTier::Tier2);

    // Change tier
    state.set_npc_tier(npc1, NpcTier::Tier2);
    assert_eq!(state.npc_tier(npc1), NpcTier::Tier2);
}

#[test]
fn test_mark_and_get_last_update() {
    let mut state = WorldSimState::new();
    let npc1 = NpcId(1);
    let npc2 = NpcId(2);

    // No update recorded initially
    assert!(state.last_npc_update(npc1).is_none());

    // Mark updates
    state.mark_npc_updated(npc1, SimTick::new(10));
    state.mark_npc_updated(npc2, SimTick::new(20));

    // Verify update ticks
    assert_eq!(state.last_npc_update(npc1), Some(SimTick::new(10)));
    assert_eq!(state.last_npc_update(npc2), Some(SimTick::new(20)));

    // Update again
    state.mark_npc_updated(npc1, SimTick::new(50));
    assert_eq!(state.last_npc_update(npc1), Some(SimTick::new(50)));
}

#[test]
fn test_register_npc() {
    let mut state = WorldSimState::new();
    let npc1 = NpcId(1);

    state.register_npc(npc1, SimTick::new(0));

    // Should have Tier2 by default and initial tick recorded
    assert_eq!(state.npc_tier(npc1), NpcTier::Tier2);
    assert_eq!(state.last_npc_update(npc1), Some(SimTick::new(0)));
    assert_eq!(state.npc_count(), 1);

    // Re-registering shouldn't overwrite existing values
    state.set_npc_tier(npc1, NpcTier::Tier0);
    state.mark_npc_updated(npc1, SimTick::new(100));
    state.register_npc(npc1, SimTick::new(0));

    assert_eq!(state.npc_tier(npc1), NpcTier::Tier0);
    assert_eq!(state.last_npc_update(npc1), Some(SimTick::new(100)));
}

#[test]
fn test_remove_npc() {
    let mut state = WorldSimState::new();
    let npc1 = NpcId(1);

    state.set_npc_tier(npc1, NpcTier::Tier0);
    state.mark_npc_updated(npc1, SimTick::new(50));
    assert_eq!(state.npc_count(), 1);

    state.remove_npc(npc1);

    // After removal, defaults apply
    assert_eq!(state.npc_tier(npc1), NpcTier::Tier2);
    assert!(state.last_npc_update(npc1).is_none());
    assert_eq!(state.npc_count(), 0);
}

#[test]
fn test_iter_tiers() {
    let mut state = WorldSimState::new();
    state.set_npc_tier(NpcId(1), NpcTier::Tier0);
    state.set_npc_tier(NpcId(2), NpcTier::Tier1);
    state.set_npc_tier(NpcId(3), NpcTier::Tier2);

    let tiers: Vec<_> = state.iter_tiers().collect();
    assert_eq!(tiers.len(), 3);

    // Verify all expected entries are present
    let has_tier0 = tiers.iter().any(|(id, tier)| **id == NpcId(1) && **tier == NpcTier::Tier0);
    let has_tier1 = tiers.iter().any(|(id, tier)| **id == NpcId(2) && **tier == NpcTier::Tier1);
    let has_tier2 = tiers.iter().any(|(id, tier)| **id == NpcId(3) && **tier == NpcTier::Tier2);

    assert!(has_tier0, "Expected NpcId(1) with Tier0");
    assert!(has_tier1, "Expected NpcId(2) with Tier1");
    assert!(has_tier2, "Expected NpcId(3) with Tier2");
}

#[test]
fn test_npc_tier_enum_traits() {
    // Test Clone
    let tier = NpcTier::Tier0;
    let cloned = tier.clone();
    assert_eq!(tier, cloned);

    // Test Copy
    let tier2: NpcTier = tier;
    assert_eq!(tier, tier2);

    // Test Default
    let default_tier = NpcTier::default();
    assert_eq!(default_tier, NpcTier::Tier2);

    // Test Debug
    let debug_str = format!("{:?}", NpcTier::Tier1);
    assert!(debug_str.contains("Tier1"));

    // Test Hash (implicit via HashMap usage in WorldSimState)
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(NpcTier::Tier0);
    set.insert(NpcTier::Tier1);
    set.insert(NpcTier::Tier2);
    assert_eq!(set.len(), 3);
}
