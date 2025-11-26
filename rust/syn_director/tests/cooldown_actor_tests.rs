//! Tests for per-actor cooldown isolation.
//!
//! Verifies that per-actor cooldowns are correctly isolated between actors:
//! when a storylet fires for actor A, actor B can still trigger the same storylet.

use syn_core::SimTick;
use syn_director::state::CooldownState;
use syn_storylets::library::StoryletKey;

#[test]
fn test_per_actor_cooldown_isolates_between_actors() {
    let mut cooldowns = CooldownState::new();
    let storylet_key = StoryletKey(100);
    let actor_a = 1_u64;
    let actor_b = 2_u64;
    let current_tick = SimTick::new(100);

    // Initially, both actors should be ready
    assert!(
        cooldowns.is_actor_ready(storylet_key, actor_a, current_tick),
        "Actor A should initially be ready"
    );
    assert!(
        cooldowns.is_actor_ready(storylet_key, actor_b, current_tick),
        "Actor B should initially be ready"
    );

    // Fire storylet for actor A with 10-tick cooldown
    cooldowns.mark_actor_cooldown(storylet_key, actor_a, 10, current_tick);

    // Actor A should now be on cooldown
    let tick_during_cooldown = SimTick::new(105);
    assert!(
        !cooldowns.is_actor_ready(storylet_key, actor_a, tick_during_cooldown),
        "Actor A should be on cooldown at tick 105"
    );

    // Actor B should still be ready (cooldown is per-actor)
    assert!(
        cooldowns.is_actor_ready(storylet_key, actor_b, tick_during_cooldown),
        "Actor B should still be ready (different actor)"
    );
}

#[test]
fn test_per_actor_cooldown_expires_correctly() {
    let mut cooldowns = CooldownState::new();
    let storylet_key = StoryletKey(100);
    let actor_id = 42_u64;
    let current_tick = SimTick::new(100);

    // Mark cooldown for 10 ticks
    cooldowns.mark_actor_cooldown(storylet_key, actor_id, 10, current_tick);

    // Check various tick points
    assert!(
        !cooldowns.is_actor_ready(storylet_key, actor_id, SimTick::new(105)),
        "Should be on cooldown at tick 105"
    );
    assert!(
        !cooldowns.is_actor_ready(storylet_key, actor_id, SimTick::new(109)),
        "Should be on cooldown at tick 109"
    );
    assert!(
        cooldowns.is_actor_ready(storylet_key, actor_id, SimTick::new(110)),
        "Should be ready at tick 110 (cooldown expired)"
    );
    assert!(
        cooldowns.is_actor_ready(storylet_key, actor_id, SimTick::new(200)),
        "Should be ready well after cooldown"
    );
}

#[test]
fn test_different_storylets_have_independent_actor_cooldowns() {
    let mut cooldowns = CooldownState::new();
    let storylet_a = StoryletKey(1);
    let storylet_b = StoryletKey(2);
    let actor_id = 42_u64;
    let current_tick = SimTick::new(100);

    // Mark cooldown for storylet A only
    cooldowns.mark_actor_cooldown(storylet_a, actor_id, 10, current_tick);

    let check_tick = SimTick::new(105);

    // Storylet A should be on cooldown
    assert!(
        !cooldowns.is_actor_ready(storylet_a, actor_id, check_tick),
        "Storylet A should be on cooldown"
    );

    // Storylet B should still be ready (different storylet)
    assert!(
        cooldowns.is_actor_ready(storylet_b, actor_id, check_tick),
        "Storylet B should be ready (different storylet)"
    );
}

#[test]
fn test_global_vs_actor_cooldown_independence() {
    let mut cooldowns = CooldownState::new();
    let storylet_key = StoryletKey(100);
    let actor_id = 42_u64;
    let current_tick = SimTick::new(100);

    // Mark only a global cooldown
    cooldowns.mark_global_cooldown(storylet_key, 10, current_tick);

    let check_tick = SimTick::new(105);

    // Global cooldown blocks
    assert!(
        !cooldowns.is_globally_ready(storylet_key, check_tick),
        "Global cooldown should block"
    );

    // Per-actor is independent - no per-actor cooldown was set
    assert!(
        cooldowns.is_actor_ready(storylet_key, actor_id, check_tick),
        "Per-actor check should be ready (no actor cooldown set)"
    );
}

#[test]
fn test_multiple_actors_can_have_different_cooldown_expirations() {
    let mut cooldowns = CooldownState::new();
    let storylet_key = StoryletKey(100);
    let actor_a = 1_u64;
    let actor_b = 2_u64;
    let actor_c = 3_u64;
    let base_tick = SimTick::new(100);

    // Set different cooldowns for different actors
    cooldowns.mark_actor_cooldown(storylet_key, actor_a, 5, base_tick);  // Expires at 105
    cooldowns.mark_actor_cooldown(storylet_key, actor_b, 10, base_tick); // Expires at 110
    cooldowns.mark_actor_cooldown(storylet_key, actor_c, 20, base_tick); // Expires at 120

    // At tick 107: A ready, B not ready, C not ready
    let tick_107 = SimTick::new(107);
    assert!(
        cooldowns.is_actor_ready(storylet_key, actor_a, tick_107),
        "Actor A should be ready at 107"
    );
    assert!(
        !cooldowns.is_actor_ready(storylet_key, actor_b, tick_107),
        "Actor B should not be ready at 107"
    );
    assert!(
        !cooldowns.is_actor_ready(storylet_key, actor_c, tick_107),
        "Actor C should not be ready at 107"
    );

    // At tick 115: A ready, B ready, C not ready
    let tick_115 = SimTick::new(115);
    assert!(
        cooldowns.is_actor_ready(storylet_key, actor_a, tick_115),
        "Actor A should be ready at 115"
    );
    assert!(
        cooldowns.is_actor_ready(storylet_key, actor_b, tick_115),
        "Actor B should be ready at 115"
    );
    assert!(
        !cooldowns.is_actor_ready(storylet_key, actor_c, tick_115),
        "Actor C should not be ready at 115"
    );

    // At tick 125: all ready
    let tick_125 = SimTick::new(125);
    assert!(
        cooldowns.is_actor_ready(storylet_key, actor_a, tick_125),
        "Actor A should be ready at 125"
    );
    assert!(
        cooldowns.is_actor_ready(storylet_key, actor_b, tick_125),
        "Actor B should be ready at 125"
    );
    assert!(
        cooldowns.is_actor_ready(storylet_key, actor_c, tick_125),
        "Actor C should be ready at 125"
    );
}

#[test]
fn test_actor_cooldown_cleanup_removes_expired() {
    let mut cooldowns = CooldownState::new();
    let storylet_key = StoryletKey(100);
    let actor_a = 1_u64;
    let actor_b = 2_u64;

    // Set cooldowns
    cooldowns.mark_actor_cooldown(storylet_key, actor_a, 10, SimTick::new(100)); // Expires at 110
    cooldowns.mark_actor_cooldown(storylet_key, actor_b, 50, SimTick::new(100)); // Expires at 150

    // Verify both are tracked
    assert_eq!(cooldowns.actor_cooldowns.len(), 2);

    // Cleanup at tick 120 - should remove actor_a's cooldown
    cooldowns.cleanup_expired(SimTick::new(120));

    assert_eq!(
        cooldowns.actor_cooldowns.len(),
        1,
        "Should have removed expired actor_a cooldown"
    );

    // Actor B's cooldown should still be tracked
    assert!(
        !cooldowns.is_actor_ready(storylet_key, actor_b, SimTick::new(120)),
        "Actor B should still be on cooldown"
    );
}
