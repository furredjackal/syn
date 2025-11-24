use syn_core::relationship_model::{RelationshipAxis, RelationshipDelta};
use syn_core::stats::{StatDelta, StatKind};
use syn_core::{NpcId, SimTick, WorldSeed, WorldState};
use syn_director::apply_storylet_outcome_with_memory;
use syn_director::{
    Storylet, StoryletCooldown, StoryletOutcome, StoryletOutcomeSet, StoryletPrerequisites,
    StoryletRoles, TagBitset,
};
use syn_memory::MemorySystem;

#[test]
fn apply_storylet_outcome_with_memory_applies_relationship_deltas() {
    let mut world = WorldState::new(WorldSeed(1), NpcId(1));
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
            delta: 5.0,
            source: Some("test".into()),
        }],
        ..Default::default()
    };

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

    apply_storylet_outcome_with_memory(
        &mut world,
        &mut MemorySystem::new(),
        &storylet,
        &outcome,
        SimTick(0),
    );

    let rel = world.get_relationship(NpcId(1), NpcId(2));
    assert!((rel.affection - 5.0).abs() < f32::EPSILON);
}
