use serde_json;
use syn_content::StoryletOutcome;
use syn_core::relationship_model::RelationshipAxis;

#[test]
fn storylet_outcome_deserializes_relationship_impacts_into_relationship_deltas() {
    let json = r#"
    {
        "relationship_impacts": [
            {
                "actor_id": 1,
                "target_id": 42,
                "axis": "Affection",
                "delta": 3.5,
                "source": "unit-test"
            },
            {
                "actor_id": 1,
                "target_id": 42,
                "axis": "Resentment",
                "delta": -2.0,
                "source": "unit-test"
            }
        ]
    }
    "#;

    let outcome: StoryletOutcome = serde_json::from_str(json)
        .expect("Failed to deserialize StoryletOutcome");

    assert_eq!(outcome.relationship_deltas.len(), 2);
    assert_eq!(outcome.relationship_deltas[0].actor_id, 1);
    assert_eq!(outcome.relationship_deltas[0].target_id, 42);
    assert_eq!(outcome.relationship_deltas[0].axis, RelationshipAxis::Affection);
    assert!((outcome.relationship_deltas[0].delta - 3.5).abs() < f32::EPSILON);
    assert_eq!(outcome.relationship_deltas[1].axis, RelationshipAxis::Resentment);
}
