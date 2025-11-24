use syn_core::relationship_model::{RelationshipAxis, RelationshipDelta};
use syn_core::relationship_pressure::RelationshipEventKind;
use syn_core::{NpcId, Relationship, RelationshipState, SimTick, WorldSeed, WorldState};
use syn_director::{
    apply_storylet_outcome_with_memory, next_hot_relationship, Storylet, StoryletCooldown,
    StoryletOutcome, StoryletOutcomeSet, StoryletPrerequisites, StoryletRole, StoryletRoles,
    TagBitset,
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
        tags: TagBitset::default(),
        prerequisites: StoryletPrerequisites::default(),
        roles: StoryletRoles::from(vec![StoryletRole {
            name: "target".to_string(),
            npc_id: NpcId(2),
        }]),
        heat: 0,
        triggers: Default::default(),
        outcomes: StoryletOutcomeSet::default(),
        cooldown: StoryletCooldown { ticks: 0 },
        weight: 1.0,
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
