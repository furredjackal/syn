use syn_core::npc::{PersonalityVector, NpcPrototype, NpcRoleTag};
use syn_core::{WorldState, WorldSeed, NpcId, Stats, LifeStage};
use std::collections::HashMap;

#[test]
fn test_personality_clamp_and_world_prototype_lookup() {
    // Personality clamp
    let mut p = PersonalityVector {
        warmth: 2.5,
        dominance: -2.0,
        volatility: 0.75,
        conscientiousness: 1.5,
        openness: -0.5,
    };
    p.clamp();
    assert!(p.warmth <= 1.0 && p.warmth >= -1.0);
    assert!(p.dominance <= 1.0 && p.dominance >= -1.0);
    assert!(p.volatility <= 1.0 && p.volatility >= -1.0);
    assert!((0.0..=1.0).contains(&p.conscientiousness));
    assert!((0.0..=1.0).contains(&p.openness));

    // Prototype and world lookup
    let id = NpcId(1001);
    let proto = NpcPrototype {
        id,
        display_name: "Test NPC".to_string(),
        role_label: Some("Childhood Friend".to_string()),
        role_tags: vec![NpcRoleTag::Peer],
        personality: p,
        base_stats: Stats::default(),
        active_stages: vec![LifeStage::Child, LifeStage::Teen],
    };

    let mut world = WorldState::new(WorldSeed(42), NpcId(1));
    world.npc_prototypes.insert(id, proto.clone());

    let fetched = world.npc_prototype(id).expect("prototype should exist");
    assert_eq!(fetched.display_name, proto.display_name);

    // ensure_npc_known
    assert!(world.known_npcs.is_empty());
    world.ensure_npc_known(id);
    assert!(world.known_npcs.contains(&id));
}
