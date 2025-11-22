use serde_json;
use syn_core::stats::StatKind;
use syn_content::storylet::StoryletOutcome;

#[test]
fn storylet_outcome_deserializes_stat_impacts_into_stat_deltas() {
    let json = r#"
    {
        "stat_impacts": [
            { "kind": "Mood", "delta": -5.0, "source": "unit-test" },
            { "kind": "Reputation", "delta": 10.0, "source": "unit-test" }
        ]
    }
    "#;

    let outcome: StoryletOutcome =
        serde_json::from_str(json).expect("Failed to deserialize StoryletOutcome");

    assert_eq!(outcome.stat_deltas.len(), 2);
    assert_eq!(outcome.stat_deltas[0].kind, StatKind::Mood);
    assert!((outcome.stat_deltas[0].delta - -5.0).abs() < f32::EPSILON);
    assert_eq!(outcome.stat_deltas[1].kind, StatKind::Reputation);
    assert!((outcome.stat_deltas[1].delta - 10.0).abs() < f32::EPSILON);
}
