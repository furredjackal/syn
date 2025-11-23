use std::collections::HashMap;

use syn_core::relationship_model::RelationshipAxis;
use syn_core::relationship_pressure::{RelationshipEventKind, RelationshipPressureEvent};
use syn_core::{NpcId, Relationship, RelationshipState, SimTick, WorldSeed, WorldState};
use syn_director::{EventDirector, Storylet, StoryletOutcome, StoryletPrerequisites, RelationshipPrereq};
use syn_memory::MemorySystem;

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

    world.relationship_pressure.queue.push_back(RelationshipPressureEvent {
        actor_id: 1,
        target_id: 2,
        kind: RelationshipEventKind::AffectionBandChanged,
        old_band: "Friendly".to_string(),
        new_band: "Close".to_string(),
        source: Some("test".to_string()),
        tick: Some(1),
    });

    let storylet_a = Storylet {
        id: "generic".into(),
        name: "Generic".into(),
        tags: vec![],
        prerequisites: StoryletPrerequisites {
            min_relationship_affection: None,
            min_relationship_resentment: None,
            stat_conditions: HashMap::new(),
            life_stages: vec![],
            tags: vec![],
            relationship_states: vec![],
            memory_tags_required: vec![],
            memory_tags_forbidden: vec![],
            memory_recency_ticks: None,
            relationship_prereqs: vec![],
            allowed_life_stages: vec![],
        },
        heat: 50.0,
        weight: 1.0,
        cooldown_ticks: 0,
        roles: vec![],
        heat_category: None,
    };

    let storylet_b = Storylet {
        id: "targeted".into(),
        name: "Targeted".into(),
        tags: vec![],
        prerequisites: StoryletPrerequisites {
            min_relationship_affection: None,
            min_relationship_resentment: None,
            stat_conditions: HashMap::new(),
            life_stages: vec![],
            tags: vec![],
            relationship_states: vec![],
            memory_tags_required: vec![],
            memory_tags_forbidden: vec![],
            memory_recency_ticks: None,
            relationship_prereqs: vec![RelationshipPrereq {
                actor_id: Some(1),
                target_id: 2,
                axis: RelationshipAxis::Affection,
                min_value: None,
                max_value: None,
                min_band: None,
                max_band: None,
            }],
            allowed_life_stages: vec![],
        },
        heat: 50.0,
        weight: 1.0,
        cooldown_ticks: 0,
        roles: vec![],
        heat_category: None,
    };

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
