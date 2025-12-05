use syn_core::npc::PersonalityVector;
use syn_core::npc_behavior::{
    choose_best_intent, compute_behavior_intents, compute_needs_from_state, BehaviorKind,
};
use syn_core::relationship_model::RelationshipVector;
use syn_core::types::Stats;

fn make_test_personality() -> PersonalityVector {
    PersonalityVector {
        warmth: 0.5,
        dominance: 0.3,
        volatility: 0.2,
        conscientiousness: 0.6,
        openness: 0.7,
    }
}

fn make_test_stats() -> Stats {
    Stats {
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
    }
}

#[test]
fn computes_needs_within_bounds_and_intents_nonempty() {
    let stats = make_test_stats();
    let personality = make_test_personality();

    let rel = RelationshipVector {
        affection: 4.0,
        trust: 2.0,
        attraction: 1.0,
        familiarity: 3.0,
        resentment: 0.0,
    };
    let needs = compute_needs_from_state(&stats, &personality, Some(&rel));

    // All needs should be within 0.0..=1.5
    for v in [
        needs.social,
        needs.security,
        needs.recognition,
        needs.comfort,
        needs.autonomy,
    ] {
        assert!(v >= 0.0 && v <= 1.5, "need out of bounds: {}", v);
    }

    // Use baseline heat_multiplier of 1.0 for test (neutral pacing)
    let intents = compute_behavior_intents(&needs, &personality, 1.0);
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

#[test]
fn high_heat_boosts_risky_behaviors() {
    let personality = make_test_personality();
    let stats = make_test_stats();
    let needs = compute_needs_from_state(&stats, &personality, None);

    // Compare intents at low heat (0.5) vs high heat (2.0)
    let intents_low_heat = compute_behavior_intents(&needs, &personality, 0.5);
    let intents_high_heat = compute_behavior_intents(&needs, &personality, 2.0);

    // Find risky behaviors (Autonomy, Recognition)
    let risky_utility_low: f32 = intents_low_heat
        .iter()
        .filter(|i| {
            matches!(
                i.kind,
                BehaviorKind::SeekAutonomy | BehaviorKind::SeekRecognition
            )
        })
        .map(|i| i.utility)
        .sum();

    let risky_utility_high: f32 = intents_high_heat
        .iter()
        .filter(|i| {
            matches!(
                i.kind,
                BehaviorKind::SeekAutonomy | BehaviorKind::SeekRecognition
            )
        })
        .map(|i| i.utility)
        .sum();

    // High heat should boost risky behaviors
    assert!(
        risky_utility_high > risky_utility_low,
        "High heat should boost risky behaviors: high={}, low={}",
        risky_utility_high,
        risky_utility_low
    );

    // Find safety behaviors (Security, Comfort)
    let safety_utility_low: f32 = intents_low_heat
        .iter()
        .filter(|i| {
            matches!(
                i.kind,
                BehaviorKind::SeekSecurity | BehaviorKind::SeekComfort
            )
        })
        .map(|i| i.utility)
        .sum();

    let safety_utility_high: f32 = intents_high_heat
        .iter()
        .filter(|i| {
            matches!(
                i.kind,
                BehaviorKind::SeekSecurity | BehaviorKind::SeekComfort
            )
        })
        .map(|i| i.utility)
        .sum();

    // High heat should dampen safety behaviors
    assert!(
        safety_utility_high < safety_utility_low,
        "High heat should dampen safety behaviors: high={}, low={}",
        safety_utility_high,
        safety_utility_low
    );
}
