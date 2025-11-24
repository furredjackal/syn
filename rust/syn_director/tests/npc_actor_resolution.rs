use syn_core::npc::{NpcPrototype, NpcRoleTag, PersonalityVector};
use syn_core::{LifeStage, NpcId, Stats, WorldSeed, WorldState};
use syn_director::{
    prepare_storylet_execution, resolve_actor_ref_to_npc, StoryActorRef, Storylet, StoryletActors,
    StoryletCooldown, StoryletOutcomeSet, StoryletPrerequisites, StoryletRole, StoryletRoles,
    TagBitset,
};
use syn_sim::NpcRegistry;

fn make_world_with_known_tag(id: NpcId, tag: NpcRoleTag) -> WorldState {
    let mut world = WorldState::new(WorldSeed(7), NpcId(1));
    let proto = NpcPrototype {
        id,
        display_name: "Tagged NPC".to_string(),
        role_label: None,
        role_tags: vec![tag],
        personality: PersonalityVector {
            warmth: 0.0,
            dominance: 0.0,
            volatility: 0.0,
            conscientiousness: 0.5,
            openness: 0.5,
        },
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
    let found =
        resolve_actor_ref_to_npc(&world, &registry, &actor_ref).expect("should resolve to NPC");
    assert_eq!(found, id);
}

#[test]
fn test_prepare_storylet_execution_focuses_npc() {
    let id = NpcId(77);
    let mut world = make_world_with_known_tag(id, NpcRoleTag::Family);
    let mut registry = NpcRegistry::default();

    let mut outcomes = StoryletOutcomeSet::default();
    outcomes.actors = Some(StoryletActors {
        primary: Some(StoryActorRef::RoleTag(NpcRoleTag::Family)),
        secondary: None,
    });
    let storylet = Storylet {
        id: "test".into(),
        name: "Test Story".into(),
        tags: TagBitset::default(),
        prerequisites: StoryletPrerequisites::default(),
        roles: StoryletRoles::from(vec![StoryletRole {
            name: "primary".into(),
            npc_id: id,
        }]),
        heat: 0,
        triggers: Default::default(),
        outcomes: outcomes,
        cooldown: StoryletCooldown { ticks: 0 },
        weight: 1.0,
    };

    prepare_storylet_execution(&mut world, &mut registry, &storylet, 0);

    let inst = registry
        .get(id)
        .expect("NPC should be instantiated and focused");
    assert!(matches!(inst.lod, syn_sim::NpcLod::Tier2Active));
}
