use syn_core::{WorldSeed, WorldState, NpcId, LifeStage};
use syn_core::npc::{NpcPrototype, PersonalityVector};
use syn_core::types::Stats;
use syn_sim::{npc_registry::NpcRegistry, NpcLod};
use syn_sim::evaluate_npc_behavior;

#[test]
fn evaluate_behavior_sets_snapshot_for_active_npc() {
    // Build a minimal world with one NPC prototype
    let seed = WorldSeed(1234);
    let player_id = NpcId(1);
    let mut world = WorldState::new(seed, player_id);
    world.player_life_stage = LifeStage::Teen; // arbitrary

    let npc_id = NpcId(2);
    let proto = NpcPrototype {
        id: npc_id,
        display_name: "Test NPC".to_string(),
        role_label: None,
        role_tags: vec![],
        personality: PersonalityVector {
            warmth: 0.8,
            dominance: 0.1,
            volatility: 0.1,
            conscientiousness: 0.4,
            openness: 0.5,
        },
        base_stats: Stats { mood: -8.0, wealth: 20.0, health: 40.0, ..Stats::default() },
        active_stages: vec![LifeStage::Teen, LifeStage::Adult],
    };
    world.npc_prototypes.insert(npc_id, proto.clone());

    // Instantiate NPC in registry
    let mut registry = NpcRegistry::default();
    registry.ensure_npc_instance(&world, npc_id, NpcLod::Tier2Active, 0);
    let inst = registry.get_mut(npc_id).expect("instance exists");

    // Evaluate behavior
    evaluate_npc_behavior(&world, inst);

    let snap = inst.behavior.as_ref().expect("snapshot set");
    // With low mood and warm personality, social or comfort should be plausible winners
    use syn_core::npc_behavior::BehaviorKind::*;
    match snap.chosen_intent.kind {
        SeekSocial | SeekComfort | SeekSecurity | SeekRecognition | SeekAutonomy | Idle => {}
    }
}
