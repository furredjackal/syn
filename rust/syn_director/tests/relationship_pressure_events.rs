use std::collections::HashMap;

use syn_core::relationship_model::{RelationshipAxis, RelationshipDelta};
use syn_core::relationship_pressure::RelationshipEventKind;
use syn_core::{NpcId, Relationship, RelationshipState, SimTick, WorldSeed, WorldState};
use syn_director::{
    apply_storylet_outcome_with_memory, next_hot_relationship, Storylet, StoryletOutcome,
    StoryletPrerequisites, StoryletRole,
};
use syn_memory::MemorySystem;

#[test]
fn storylet_outcomes_enqueue_relationship_pressure_events() {
    let mut world = WorldState::new(WorldSeed(1), NpcId(1));
    world.relationships.insert(
        (NpcId(1), NpcId(2)),
        Relationship {
            affection: 0.0,
            trust: 0.0,
            attraction: 0.0,
            familiarity: 0.0,
            resentment: 0.0,
            state: RelationshipState::Stranger,
        },
    );
    let mut memory = MemorySystem::default();

    let storylet = Storylet {
        id: "story_1".to_string(),
        name: "Test Storylet".to_string(),
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
            time_and_location: None,
        },
        heat: 0.0,
        weight: 1.0,
        cooldown_ticks: 0,
        roles: vec![StoryletRole {
            name: "target".to_string(),
            npc_id: NpcId(2),
        }],
        max_uses: None,
        choices: vec![],
        heat_category: None,
    };

    let outcome = StoryletOutcome {
        relationship_deltas: vec![RelationshipDelta {
            actor_id: 1,
            target_id: 2,
            axis: RelationshipAxis::Affection,
            delta: 7.0,
            source: None,
        }],
        ..Default::default()
    };

    apply_storylet_outcome_with_memory(&mut world, &mut memory, &storylet, &outcome, SimTick(0));

    let event = next_hot_relationship(&mut world).expect("expected a relationship pressure event");

    assert_eq!(event.actor_id, 1);
    assert_eq!(event.target_id, 2);
    assert_eq!(event.kind, RelationshipEventKind::AffectionBandChanged);
    assert_ne!(event.old_band, event.new_band);
}
