use syn_core::relationship_model::RelationshipVector;
use syn_core::relationship_pressure::{RelationshipEventKind, RelationshipPressureState};

#[test]
fn records_band_change_events() {
    let mut pressure = RelationshipPressureState::default();
    let actor_id = 1;
    let target_id = 2;

    let low = RelationshipVector {
        affection: 0.0,
        trust: 0.0,
        attraction: 0.0,
        familiarity: 0.0,
        resentment: 0.0,
    };
    let mut high = low.clone();

    pressure.update_for_pair(actor_id, target_id, &low, None, None);
    assert!(pressure.queue.is_empty());

    high.affection = 6.5; // Crosses Friendly -> Close
    pressure.update_for_pair(
        actor_id,
        target_id,
        &high,
        Some("test_source".to_string()),
        Some(42),
    );

    assert!(!pressure.queue.is_empty());
    let event = pressure
        .pop_next_event()
        .expect("expected a band change event");

    assert_eq!(event.actor_id, actor_id);
    assert_eq!(event.target_id, target_id);
    assert_eq!(event.kind, RelationshipEventKind::AffectionBandChanged);
    assert_eq!(event.old_band, low.affection_band().to_string());
    assert_eq!(event.new_band, high.affection_band().to_string());
    assert_eq!(event.source.as_deref(), Some("test_source"));
    assert_eq!(event.tick, Some(42));
}
