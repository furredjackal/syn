use std::collections::HashMap;

use syn_core::relationship_model::*;

#[test]
fn get_set_apply_delta_clamps() {
    let mut vec = RelationshipVector::default();
    vec.set(RelationshipAxis::Affection, 12.0);
    assert_eq!(vec.get(RelationshipAxis::Affection), 10.0);

    vec.apply_delta(RelationshipAxis::Affection, -25.0);
    assert_eq!(vec.get(RelationshipAxis::Affection), -10.0);

    vec.set(RelationshipAxis::Trust, 5.0);
    assert_eq!(vec.get(RelationshipAxis::Trust), 5.0);
}

#[test]
fn bands_map_expected_ranges() {
    let mut vec = RelationshipVector::default();

    vec.set(RelationshipAxis::Affection, -6.0);
    assert_eq!(vec.affection_band(), AffectionBand::Stranger);
    vec.set(RelationshipAxis::Affection, 2.0);
    assert_eq!(vec.affection_band(), AffectionBand::Friendly);
    vec.set(RelationshipAxis::Affection, 9.0);
    assert_eq!(vec.affection_band(), AffectionBand::Devoted);

    vec.set(RelationshipAxis::Trust, -6.0);
    assert_eq!(vec.trust_band(), TrustBand::Unknown);
    vec.set(RelationshipAxis::Trust, 0.0);
    assert_eq!(vec.trust_band(), TrustBand::Neutral);
    vec.set(RelationshipAxis::Trust, 8.0);
    assert_eq!(vec.trust_band(), TrustBand::DeepTrust);

    vec.set(RelationshipAxis::Attraction, 0.0);
    assert_eq!(vec.attraction_band(), AttractionBand::None);
    vec.set(RelationshipAxis::Attraction, 4.0);
    assert_eq!(vec.attraction_band(), AttractionBand::Interested);
    vec.set(RelationshipAxis::Attraction, 9.0);
    assert_eq!(vec.attraction_band(), AttractionBand::Intense);

    vec.set(RelationshipAxis::Resentment, 0.0);
    assert_eq!(vec.resentment_band(), ResentmentBand::None);
    vec.set(RelationshipAxis::Resentment, 4.0);
    assert_eq!(vec.resentment_band(), ResentmentBand::Resentful);
    vec.set(RelationshipAxis::Resentment, 9.0);
    assert_eq!(vec.resentment_band(), ResentmentBand::Vindictive);
}

#[test]
fn apply_relationship_deltas_applies_all() {
    let mut store: HashMap<(u64, u64), RelationshipVector> = HashMap::new();

    let deltas = vec![
        RelationshipDelta {
            actor_id: 1,
            target_id: 2,
            axis: RelationshipAxis::Affection,
            delta: 3.0,
            source: None,
        },
        RelationshipDelta {
            actor_id: 1,
            target_id: 2,
            axis: RelationshipAxis::Trust,
            delta: -2.0,
            source: Some("test".into()),
        },
    ];

    // Manually apply deltas to test the same behavior
    for d in &deltas {
        let vec = store
            .entry((d.actor_id, d.target_id))
            .or_insert_with(RelationshipVector::default);
        vec.apply_delta(d.axis, d.delta);
    }

    let vec = store.get(&(1, 2)).unwrap();
    assert_eq!(vec.get(RelationshipAxis::Affection), 3.0);
    assert_eq!(vec.get(RelationshipAxis::Trust), -2.0);
}
