use syn_core::npc::PersonalityVector;
use syn_core::npc_behavior::{compute_behavior_intents, compute_needs_from_state, choose_best_intent, BehaviorKind};
use syn_core::relationship_model::RelationshipVector;
use syn_core::types::Stats;

#[test]
fn computes_needs_within_bounds_and_intents_nonempty() {
    let stats = Stats {
        health: 40.0,
        intelligence: 60.0,
        charisma: 55.0,
        wealth: 20.0,
        mood: -6.0,
        appearance: 50.0,
        reputation: -20.0,
        wisdom: 25.0,
        curiosity: Some(50.0),
        energy: Some(50.0),
        libido: None,
    };

    let personality = PersonalityVector {
        warmth: 0.5,
        dominance: 0.3,
        volatility: 0.2,
        conscientiousness: 0.6,
        openness: 0.7,
    };

    let rel = RelationshipVector { affection: 4.0, trust: 2.0, attraction: 1.0, familiarity: 3.0, resentment: 0.0 };
    let needs = compute_needs_from_state(&stats, &personality, Some(&rel));

    // All needs should be within 0.0..=1.5
    for v in [needs.social, needs.security, needs.recognition, needs.comfort, needs.autonomy] {
        assert!(v >= 0.0 && v <= 1.5, "need out of bounds: {}", v);
    }

    let intents = compute_behavior_intents(&needs, &personality);
    assert!(!intents.is_empty());
    assert!(intents.iter().any(|i| i.utility > 0.0));

    let best = choose_best_intent(&intents);
    // Best must be one of the defined kinds
    match best.kind {
        BehaviorKind::SeekSocial
        | BehaviorKind::SeekSecurity
        | BehaviorKind::SeekRecognition
        | BehaviorKind::SeekComfort
        | BehaviorKind::SeekAutonomy
        | BehaviorKind::Idle => {}
    }
}
