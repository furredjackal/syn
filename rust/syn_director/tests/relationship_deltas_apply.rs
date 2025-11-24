use syn_core::relationship_model::{RelationshipAxis, RelationshipDelta};
use syn_core::stats::{StatDelta, StatKind};
use syn_core::{NpcId, SimTick, WorldSeed, WorldState};
use syn_director::{
    apply_storylet_outcome_with_memory, Storylet, StoryletCooldown, StoryletOutcome,
    StoryletOutcomeSet, StoryletPrerequisites, StoryletRoles, TagBitset,
};
use syn_memory::MemorySystem;

#[test]
fn storylet_outcome_applies_new_relationship_deltas() {
    let mut world = WorldState::new(WorldSeed(1), NpcId(1));
    let mut memory = MemorySystem::new();

    let storylet = Storylet {
        id: "test".into(),
        name: "test".into(),
        tags: TagBitset::default(),
        prerequisites: StoryletPrerequisites::default(),
        roles: StoryletRoles::default(),
        heat: 0,
        triggers: Default::default(),
        outcomes: StoryletOutcomeSet::default(),
        cooldown: StoryletCooldown { ticks: 0 },
        weight: 1.0,
    };

    let outcome = StoryletOutcome {
        stat_deltas: vec![StatDelta {
            kind: StatKind::Mood,
            delta: 0.0,
            source: None,
        }],
        relationship_deltas: vec![RelationshipDelta {
            actor_id: 1,
            target_id: 2,
            axis: RelationshipAxis::Affection,
            delta: 4.0,
            source: Some("test".into()),
        }],
        ..Default::default()
    };

    apply_storylet_outcome_with_memory(&mut world, &mut memory, &storylet, &outcome, SimTick(0));

    let rel = world.relationships.get(&(NpcId(1), NpcId(2))).unwrap();
    assert!((rel.affection - 4.0).abs() < f32::EPSILON);
}
