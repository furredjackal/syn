use syn_core::npc::{NpcPrototype, NpcRoleTag, PersonalityVector};
use syn_core::{LifeStage, NpcId, Stats, WorldSeed, WorldState};
use syn_sim::{instantiate_simulated_npc_from_prototype, npc_registry::NpcRegistry, NpcLod};

fn make_world_with_proto(id: NpcId) -> WorldState {
    let mut world = WorldState::new(WorldSeed(99), NpcId(1));
    let proto = NpcPrototype {
        id,
        display_name: "Proto Test".to_string(),
        role_label: None,
        role_tags: vec![NpcRoleTag::Peer],
        personality: PersonalityVector {
            warmth: 0.2,
            dominance: 0.1,
            volatility: 0.0,
            conscientiousness: 0.8,
            openness: 0.6,
        },
        base_stats: Stats::default(),
        active_stages: vec![LifeStage::Teen, LifeStage::Adult],
    };
    world.npc_prototypes.insert(id, proto);
    world
}

#[test]
fn test_registry_ensure_instance_and_lod() {
    let npc_id = NpcId(123);
    let world = make_world_with_proto(npc_id);
    let mut registry = NpcRegistry::default();

    registry.ensure_npc_instance(&world, npc_id, NpcLod::Tier2Active, 0);

    let inst = registry.get(npc_id).expect("instance should exist");
    assert_eq!(inst.id, npc_id);
    assert!(matches!(inst.lod, NpcLod::Tier2Active));
    // SimulatedNpc should have baseline stats initialized
    assert_eq!(inst.sim.stats.get(syn_core::StatKind::Mood), 0.0);
}
