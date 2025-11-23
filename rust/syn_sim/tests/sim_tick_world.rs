use syn_core::{WorldState, WorldSeed, NpcId, Stats, LifeStage};
use syn_core::npc::{NpcPrototype, PersonalityVector, NpcRoleTag};
use syn_sim::{SimState, NpcLodTier, tick_world};

fn make_world_with_proto(id: NpcId) -> WorldState {
    let mut world = WorldState::new(WorldSeed(1234), NpcId(1));
    let proto = NpcPrototype {
        id,
        display_name: "Test NPC".to_string(),
        role_label: None,
        role_tags: vec![NpcRoleTag::Peer],
        personality: PersonalityVector { warmth: 0.5, dominance: 0.3, volatility: 0.2, conscientiousness: 0.6, openness: 0.4 },
        base_stats: Stats::default(),
        active_stages: vec![LifeStage::Teen, LifeStage::Adult],
    };
    world.npc_prototypes.insert(id, proto);
    world
}

#[test]
fn tick_world_advances_time_and_ticks_tiers() {
    let mut world = make_world_with_proto(NpcId(2));
    let mut sim = SimState::new();

    // Ensure instances via registry
    sim.npc_registry.ensure_npc_instance(&world, NpcId(2), syn_sim::NpcLod::Tier2Active, 0);
    // Add another NPC prototype
    let proto2 = NpcPrototype {
        id: NpcId(3),
        display_name: "Test NPC 2".to_string(),
        role_label: None,
        role_tags: vec![NpcRoleTag::Peer],
        personality: PersonalityVector { warmth: 0.5, dominance: 0.3, volatility: 0.2, conscientiousness: 0.6, openness: 0.4 },
        base_stats: Stats::default(),
        active_stages: vec![LifeStage::Teen, LifeStage::Adult],
    };
    world.npc_prototypes.insert(NpcId(3), proto2);
    sim.npc_registry.ensure_npc_instance(&world, NpcId(3), syn_sim::NpcLod::Tier2Active, 0);

    // Assign canonical tiers
    {
        let npc2 = sim.npc_registry.get_mut(NpcId(2)).unwrap();
        npc2.tier = NpcLodTier::Tier1Active;
        let npc3 = sim.npc_registry.get_mut(NpcId(3)).unwrap();
        npc3.tier = NpcLodTier::Tier2Background;
    }

    assert_eq!(world.game_time.tick_index, 0);

    tick_world(&mut world, &mut sim, 24);

    // Time advanced exactly 1 day
    assert_eq!(world.game_time.day(), 1);
    assert_eq!(world.game_time.hour_in_day(), 0);

    let n2 = sim.npc_registry.get(NpcId(2)).unwrap();
    let n3 = sim.npc_registry.get(NpcId(3)).unwrap();
    assert_eq!(n2.last_tick, world.game_time.tick_index);
    assert!(n3.last_tick <= world.game_time.tick_index);
}
