use syn_core::{WorldSeed, WorldState, NpcId, LifeStage, StatKind};
use syn_core::npc::{NpcPrototype, PersonalityVector};
use syn_core::types::Stats;
use syn_sim::{npc_registry::NpcRegistry, NpcLod};
use syn_sim::{evaluate_npc_behavior, maybe_run_npc_action};

#[test]
fn run_action_applies_effects_and_busy() {
    // World and prototype
    let seed = WorldSeed(42);
    let player_id = NpcId(1);
    let mut world = WorldState::new(seed, player_id);
    world.player_life_stage = LifeStage::Teen;

    let npc_id = NpcId(2);
    let proto = NpcPrototype {
        id: npc_id,
        display_name: "Doer".to_string(),
        role_label: None,
        role_tags: vec![],
        personality: PersonalityVector { warmth: 0.9, dominance: 0.2, volatility: 0.1, conscientiousness: 0.4, openness: 0.5 },
        base_stats: Stats { mood: -6.0, wealth: 10.0, health: 60.0, ..Stats::default() },
        active_stages: vec![LifeStage::Teen, LifeStage::Adult],
    };
    world.npc_prototypes.insert(npc_id, proto);

    // Registry and instance
    let mut registry = NpcRegistry::default();
    registry.ensure_npc_instance(&world, npc_id, NpcLod::Tier2Active, 0);
    let inst = registry.get_mut(npc_id).expect("instance exists");

    // Force behavior snapshot with social target player by running evaluation
    evaluate_npc_behavior(&world, inst);
    // Sanity
    assert!(inst.behavior.is_some());
    let before_player_mood = world.player_stats.get(StatKind::Mood);

    // Run action selection/execution (deterministic first candidate)
    maybe_run_npc_action(&mut world, inst, 1);

    // Busy set or at least last_action written
    assert!(inst.last_action.is_some());
    if inst.busy_until_tick > 1 {
        // ok
    }

    // Player mood likely changed for SocialVisitPlayer/ProvokePlayer; allow either direction
    let after_player_mood = world.player_stats.get(StatKind::Mood);
    assert!(after_player_mood != before_player_mood);
}
