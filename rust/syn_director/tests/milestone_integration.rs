use std::collections::HashMap;

use syn_core::relationship_milestones::RelationshipMilestoneKind;
use syn_core::relationship_model::{RelationshipAxis, RelationshipDelta};
use syn_core::{NpcId, Relationship, RelationshipState, SimTick, WorldSeed, WorldState};
use syn_director::{
    apply_storylet_outcome_with_memory, Storylet, StoryletOutcome, StoryletPrerequisites,
    StoryletRole,
};
use syn_memory::{MemoryEntry, MemorySystem};

#[test]
fn storylet_outcome_records_friend_to_rival_milestone() {
    let mut world = WorldState::new(WorldSeed(1), NpcId(1));
    world.relationships.insert(
        (NpcId(1), NpcId(2)),
        Relationship {
            affection: 6.0,
            trust: 6.0,
            attraction: 0.0,
            familiarity: 0.0,
            resentment: 0.0,
            state: RelationshipState::Stranger,
        },
    );
    world.relationship_milestones.record_role_for_pair(
        1,
        2,
        syn_core::relationship_model::RelationshipRole::Friend,
    );

    let mut memory = MemorySystem::new();
    let mut mem_entry = MemoryEntry::new(
        "mem_1".into(),
        "event_betrayal".into(),
        NpcId(1),
        SimTick(0),
        -0.8,
    );
    mem_entry.tags = vec!["betrayal".into()];
    mem_entry.participants = vec![1, 2];
    memory.record_memory(mem_entry);

    let storylet = Storylet {
        id: "story".into(),
        name: "Story".into(),
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
            relationship_prereqs: vec![syn_director::RelationshipPrereq {
                actor_id: Some(1),
                target_id: 2,
                axis: RelationshipAxis::Resentment,
                min_value: None,
                max_value: None,
                min_band: None,
                max_band: None,
            }],
            allowed_life_stages: vec![],
            time_and_location: None,
            digital_legacy_prereq: None,
        },
        heat: 1.0,
        weight: 1.0,
        cooldown_ticks: 0,
        roles: vec![StoryletRole {
            name: "target".into(),
            npc_id: NpcId(2),
        }],
        max_uses: None,
        choices: vec![],
        heat_category: None,
        actors: None,
        interaction_tone: None,
    };

    let outcome = StoryletOutcome {
        relationship_deltas: vec![RelationshipDelta {
            actor_id: 1,
            target_id: 2,
            axis: RelationshipAxis::Resentment,
            delta: 9.0,
            source: Some("test".into()),
        }],
        ..Default::default()
    };

    apply_storylet_outcome_with_memory(&mut world, &mut memory, &storylet, &outcome, SimTick(0));

    let event = world
        .relationship_milestones
        .pop_next()
        .expect("expected a milestone event");
    assert_eq!(event.kind, RelationshipMilestoneKind::FriendToRival);
    assert_eq!(event.actor_id, 1);
    assert_eq!(event.target_id, 2);
    assert_eq!(event.from_role, "Friend");
    assert_eq!(event.to_role, "Rival");
}
