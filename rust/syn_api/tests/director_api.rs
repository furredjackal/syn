use std::collections::HashMap;

use syn_api::{
    api_choose_option, api_get_current_event, api_reset_runtime, tags_to_bitset, Storylet,
    StoryletChoice, StoryletCooldown, StoryletOutcome, StoryletOutcomeSet, WorldSeed, WorldState,
};
use syn_director::StoryletLibrary;
use syn_core::relationship_model::{RelationshipAxis, RelationshipDelta};
use syn_core::{NpcId, StatDelta, StatKind};
use syn_sim::SimState;

fn basic_prereqs() -> syn_director::StoryletPrerequisites {
    syn_director::StoryletPrerequisites {
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

fn sample_storylet() -> Storylet {
    Storylet {
        id: "story-api".to_string(),
        name: "API Story".to_string(),
        tags: tags_to_bitset(&[]),
        prerequisites: basic_prereqs(),
        heat: 1,
        weight: 1.0,
        roles: vec![],
        outcomes: StoryletOutcomeSet {
            choices: vec![StoryletChoice {
                id: "choice-api".to_string(),
                label: "Take it".to_string(),
                outcome: StoryletOutcome {
                    stat_deltas: vec![StatDelta {
                        kind: StatKind::Mood,
                        delta: 1.0,
                        source: None,
                    }],
                    relationship_deltas: vec![RelationshipDelta {
                        actor_id: 1,
                        target_id: 1,
                        axis: RelationshipAxis::Affection,
                        delta: 0.5,
                        source: None,
                    }],
                    ..Default::default()
                },
            }],
            ..Default::default()
        },
        cooldown: StoryletCooldown { ticks: 0 },
        ..Default::default()
    }
}

#[test]
fn api_flow_returns_events() {
    let world = WorldState::new(WorldSeed(5), NpcId(1));
    let sim = SimState::new();
    let library = StoryletLibrary::from_storylets(vec![sample_storylet()]);

    api_reset_runtime(world, sim, library);

    let event = api_get_current_event().expect("expected event");
    assert_eq!(event.storylet_id, "story-api");
    assert_eq!(event.choices.len(), 1);

    let next = api_choose_option(event.storylet_id, event.choices[0].id.clone(), 2);
    assert!(next.is_some());
}
