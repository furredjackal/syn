use syn_core::{WorldState, WorldSeed, NpcId, Stats, LifeStage};
use syn_core::npc::{NpcPrototype, PersonalityVector, NpcRoleTag};
use syn_director::{Storylet, StoryActorRef, StoryletActors, resolve_actor_ref_to_npc, prepare_storylet_execution, StoryletPrerequisites, StoryletRole};
use syn_sim::npc_registry::NpcRegistry;

fn make_world_with_known_tag(id: NpcId, tag: NpcRoleTag) -> WorldState {
    let mut world = WorldState::new(WorldSeed(7), NpcId(1));
    let proto = NpcPrototype {
        id,
        display_name: "Tagged NPC".to_string(),
        role_label: None,
        role_tags: vec![tag],
        personality: PersonalityVector { warmth: 0.0, dominance: 0.0, volatility: 0.0, conscientiousness: 0.5, openness: 0.5 },
        base_stats: Stats::default(),
        active_stages: vec![LifeStage::Teen, LifeStage::Adult],
        schedule: Default::default(),
    };
    world.npc_prototypes.insert(id, proto);
    world.ensure_npc_known(id);
    world
}

#[test]
fn test_resolve_actor_ref_role_tag() {
    let id = NpcId(55);
    let world = make_world_with_known_tag(id, NpcRoleTag::Peer);
    let registry = NpcRegistry::default();
    let actor_ref = StoryActorRef::RoleTag(NpcRoleTag::Peer);
    let found = resolve_actor_ref_to_npc(&world, &registry, &actor_ref).expect("should resolve to NPC");
    assert_eq!(found, id);
}

#[test]
fn test_prepare_storylet_execution_focuses_npc() {
    let id = NpcId(77);
    let mut world = make_world_with_known_tag(id, NpcRoleTag::Family);
    let mut registry = NpcRegistry::default();

    let storylet = Storylet {
        id: "test".into(),
        name: "Test Story".into(),
        tags: vec![],
        prerequisites: StoryletPrerequisites {
            min_relationship_affection: None,
            min_relationship_resentment: None,
            stat_conditions: Default::default(),
            life_stages: vec![],
            tags: vec![],
            relationship_states: vec![],
            memory_tags_required: vec![],
            memory_tags_forbidden: vec![],
            memory_recency_ticks: None,
            relationship_prereqs: vec![],
            allowed_life_stages: vec![],
            digital_legacy_prereq: None,
        },
        heat: 0.0,
        weight: 1.0,
        cooldown_ticks: 0,
        roles: vec![StoryletRole { name: "primary".into(), npc_id: id }],
        heat_category: None,
        actors: Some(StoryletActors { primary: Some(StoryActorRef::RoleTag(NpcRoleTag::Family)), secondary: None }),
    };

    prepare_storylet_execution(&mut world, &mut registry, &storylet, 0);

    let inst = registry.get(id).expect("NPC should be instantiated and focused");
    assert!(matches!(inst.lod, syn_sim::NpcLod::Tier2Active));
}
