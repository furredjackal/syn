use std::collections::HashMap;

use syn_core::{
    relationship_model::{RelationshipAxis, RelationshipDelta},
    NpcId, SimTick, StatDelta, StatKind, WorldSeed, WorldState,
};
use syn_director::{
    apply_choice_and_advance, tags_to_bitset, Storylet, StoryletChoice, StoryletCooldown,
    StoryletLibrary, StoryletOutcome, StoryletOutcomeSet, StoryletPrerequisites,
};
use syn_sim::SimState;

fn basic_prereqs() -> StoryletPrerequisites {
    StoryletPrerequisites {
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
        digital_legacy_prereq: None,
        time_and_location: None,
    }
}

#[test]
fn apply_choice_advances_time_and_applies_outcome() {
    let mut world = WorldState::new(WorldSeed(99), NpcId(1));
    let mut sim = SimState::new();

    let storylet = Storylet {
        id: "s1".to_string(),
        name: "Test Story".to_string(),
        tags: tags_to_bitset(&[]),
        prerequisites: basic_prereqs(),
        heat: 1,
        weight: 1.0,
        roles: vec![],
        outcomes: StoryletOutcomeSet {
            choices: vec![StoryletChoice {
                id: "c1".to_string(),
                label: "Proceed".to_string(),
                outcome: StoryletOutcome {
                    stat_deltas: vec![StatDelta {
                        kind: StatKind::Mood,
                        delta: 3.0,
                        source: None,
                    }],
                    relationship_deltas: vec![RelationshipDelta {
                        actor_id: 1,
                        target_id: 1,
                        axis: RelationshipAxis::Trust,
                        delta: 1.0,
                        source: None,
                    }],
                    karma_delta: Some(2.5),
                    ..Default::default()
                },
            }],
            ..Default::default()
        },
        cooldown: StoryletCooldown { ticks: 0 },
        ..Default::default()
    };

    let library = StoryletLibrary::from_storylets(vec![storylet]);

    let next_event = apply_choice_and_advance(&mut world, &mut sim, &library, "s1", "c1", 4)
        .expect("expected next event");

    assert_eq!(world.game_time.tick_index, 4);
    assert_eq!(world.player_stats.get(StatKind::Mood), 3.0);
    assert!((world.player_karma.0 - 2.5).abs() < f32::EPSILON);
    assert_eq!(world.get_relationship(NpcId(1), NpcId(1)).trust, 1.0);
    assert!(next_event.choices.len() >= 1);
    assert_eq!(next_event.storylet_id, "s1");
}
