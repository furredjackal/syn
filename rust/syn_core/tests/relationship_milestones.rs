use syn_core::relationship_milestones::{
    RelationshipMilestoneEvent, RelationshipMilestoneKind, RelationshipMilestoneState,
};
use syn_core::relationship_model::{RelationshipRole, RelationshipVector};

#[test]
fn detect_friend_to_rival_with_betrayal() {
    let tags = vec!["betrayal".to_string()];
    let kind = RelationshipMilestoneState::detect_milestone(
        RelationshipRole::Friend,
        RelationshipRole::Rival,
        &tags,
    )
    .expect("expected a milestone");
    assert!(matches!(kind, RelationshipMilestoneKind::FriendToRival));
}

#[test]
fn enqueue_event_with_roles_and_reason() {
    let mut state = RelationshipMilestoneState::default();
    state.record_role_for_pair(1, 2, RelationshipRole::Friend);

    let rel = RelationshipVector {
        affection: -8.0,
        trust: -6.0,
        attraction: 0.0,
        familiarity: 0.0,
        resentment: 9.0,
    };
    let tags = vec!["betrayal".to_string()];

    state.evaluate_and_record_milestones_for_pair(1, 2, &rel, &tags, Some("test".into()), Some(10));

    let event: RelationshipMilestoneEvent = state.pop_next().expect("expected event in queue");
    assert_eq!(event.actor_id, 1);
    assert_eq!(event.target_id, 2);
    assert_eq!(event.from_role, RelationshipRole::Friend.to_string());
    assert_eq!(event.to_role, rel.role().to_string());
    assert_eq!(event.kind, RelationshipMilestoneKind::FriendToRival);
    assert_eq!(event.source.as_deref(), Some("test"));
    assert_eq!(event.tick, Some(10));
}
