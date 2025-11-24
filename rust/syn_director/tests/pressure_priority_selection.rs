use syn_core::relationship_model::RelationshipAxis;
use syn_core::relationship_pressure::{RelationshipEventKind, RelationshipPressureEvent};
use syn_core::{NpcId, Relationship, RelationshipState, SimTick, WorldSeed, WorldState};
use syn_director::{
    EventDirector, RelationshipPrereq, Storylet, StoryletCooldown, StoryletOutcome,
    StoryletOutcomeSet, StoryletPrerequisites, StoryletRoles, TagBitset,
};
use syn_memory::MemorySystem;

fn build_storylet(id: &str, prereqs: StoryletPrerequisites) -> Storylet {
    Storylet {
        id: id.to_string(),
        name: id.to_string(),
        tags: TagBitset::default(),
        prerequisites: prereqs,
        roles: StoryletRoles::default(),
        heat: 50,
        triggers: Default::default(),
        outcomes: StoryletOutcomeSet::default(),
        cooldown: StoryletCooldown { ticks: 0 },
        weight: 1.0,
    }
}

#[test]
fn storylets_targeting_hot_pair_get_higher_priority() {
    let mut world = WorldState::new(WorldSeed(1), NpcId(1));
    world.relationships.insert(
        (NpcId(1), NpcId(2)),
        Relationship {
            affection: 7.0,
            trust: 5.0,
            attraction: 0.0,
            familiarity: 3.0,
            resentment: 0.0,
            state: RelationshipState::Stranger,
        },
    );

    world
        .relationship_pressure
        .queue
        .push_back(RelationshipPressureEvent {
            actor_id: 1,
            target_id: 2,
            kind: RelationshipEventKind::AffectionBandChanged,
            old_band: "Friendly".to_string(),
            new_band: "Close".to_string(),
            source: Some("test".to_string()),
            tick: Some(1),
        });

    let storylet_a = build_storylet("generic", StoryletPrerequisites::default());

    let prereqs = StoryletPrerequisites {
        relationship_prereqs: vec![RelationshipPrereq {
            actor_id: Some(1),
            target_id: 2,
            axis: RelationshipAxis::Affection,
            min_value: None,
            max_value: None,
            min_band: None,
            max_band: None,
        }],
        ..Default::default()
    };
    let storylet_b = build_storylet("targeted", prereqs);

    let mut director = EventDirector::new();
    director.register_storylet(storylet_a);
    director.register_storylet(storylet_b);

    let memory = MemorySystem::new();
    let chosen = director
        .select_next_event(&world, &memory, SimTick(0))
        .cloned()
        .expect("expected a storylet");

    assert_eq!(chosen.id, "targeted");

    let mut world = world;
    let mut memory = MemorySystem::new();
    let outcome = StoryletOutcome::default();
    director.fire_storylet(&chosen, &mut world, &mut memory, outcome, SimTick(0));

    assert!(
        world.relationship_pressure.queue.is_empty(),
        "Pressure event should be consumed when firing a matching storylet"
    );
}
