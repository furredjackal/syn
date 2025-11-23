use serde_json;
use syn_content::StoryletPrerequisites;
use syn_core::relationship_model::RelationshipAxis;

#[test]
fn can_deserialize_relationship_prereqs_from_json() {
    let json = r#"
    {
        "min_relationship_affection": null,
        "min_relationship_resentment": null,
        "stat_conditions": {},
        "life_stages": [],
        "tags": [],
        "relationship_states": [],
        "memory_tags_required": [],
        "memory_tags_forbidden": [],
        "memory_recency_ticks": null,
        "relationship_prereqs": [
            {
                "actor_id": 1,
                "target_id": 42,
                "axis": "Affection",
                "min_value": 2.5,
                "max_value": 10.0,
                "min_band": "Friendly",
                "max_band": "Devoted"
            }
        ]
    }
    "#;

    let pre: StoryletPrerequisites =
        serde_json::from_str(json).expect("Failed to deserialize StoryletPrerequisites");

    assert_eq!(pre.relationship_prereqs.len(), 1);
    let r = &pre.relationship_prereqs[0];
    assert_eq!(r.actor_id, Some(1));
    assert_eq!(r.target_id, 42);
    assert_eq!(r.axis, RelationshipAxis::Affection);
    assert_eq!(r.min_value, Some(2.5));
    assert_eq!(r.max_value, Some(10.0));
    assert_eq!(r.min_band.as_deref(), Some("Friendly"));
    assert_eq!(r.max_band.as_deref(), Some("Devoted"));
    assert!(pre.allowed_life_stages.is_empty());
}
