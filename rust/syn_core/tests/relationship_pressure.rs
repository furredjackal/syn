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

#[test]
fn decay_queue_removes_old_events() {
    let mut pressure = RelationshipPressureState::default();

    // We need to create band transitions, so first set up initial states
    let low = RelationshipVector {
        affection: 0.0,
        trust: 0.0,
        attraction: 0.0,
        familiarity: 0.0,
        resentment: 0.0,
    };
    let high = RelationshipVector {
        affection: 7.0,
        ..low
    };

    // Initialize tracking and then trigger band changes at different ticks
    // Pair (1,2) at tick 100 - will be old
    pressure.update_for_pair(1, 2, &low, None, Some(100));
    pressure.update_for_pair(1, 2, &high, Some("old".to_string()), Some(100));

    // Pair (3,4) at tick 300 - will be kept
    pressure.update_for_pair(3, 4, &low, None, Some(300));
    pressure.update_for_pair(3, 4, &high, Some("mid".to_string()), Some(300));

    // Pair (5,6) at tick 350 - will be kept
    pressure.update_for_pair(5, 6, &low, None, Some(350));
    pressure.update_for_pair(5, 6, &high, Some("new".to_string()), Some(350));

    assert_eq!(pressure.pending_count(), 3);

    // Decay with current_tick=400, max_age=150
    // Events older than tick 250 (400-150) should be removed
    // tick 100: 400-100=300 > 150 → removed
    // tick 300: 400-300=100 <= 150 → kept
    // tick 350: 400-350=50 <= 150 → kept
    pressure.decay_queue(400, 150, 10);

    assert_eq!(pressure.pending_count(), 2);

    // Remaining events should be from ticks 300 and 350
    let event1 = pressure.pop_next_event().unwrap();
    let event2 = pressure.pop_next_event().unwrap();
    assert_eq!(event1.tick, Some(300));
    assert_eq!(event2.tick, Some(350));
}

#[test]
fn decay_queue_enforces_max_size() {
    let mut pressure = RelationshipPressureState::default();

    let rel = RelationshipVector {
        affection: 0.0,
        trust: 0.0,
        attraction: 0.0,
        familiarity: 0.0,
        resentment: 0.0,
    };

    // Add 5 events
    for i in 0..5 {
        let actor = i * 2;
        let target = i * 2 + 1;
        pressure.update_for_pair(actor, target, &rel, None, Some(i as u64 * 10));

        let high = RelationshipVector {
            affection: 7.0,
            ..rel
        };
        pressure.update_for_pair(actor, target, &high, None, Some(i as u64 * 10));
    }

    assert_eq!(pressure.pending_count(), 5);

    // Decay with max_size=3 (should keep newest 3)
    pressure.decay_queue(100, 1000, 3);

    assert_eq!(pressure.pending_count(), 3);

    // Remaining events should be from ticks 20, 30, 40 (oldest dropped)
    let event1 = pressure.pop_next_event().unwrap();
    let event2 = pressure.pop_next_event().unwrap();
    let event3 = pressure.pop_next_event().unwrap();
    assert_eq!(event1.tick, Some(20));
    assert_eq!(event2.tick, Some(30));
    assert_eq!(event3.tick, Some(40));
}

#[test]
fn has_pending_events_and_pending_count() {
    let mut pressure = RelationshipPressureState::default();

    assert!(!pressure.has_pending_events());
    assert_eq!(pressure.pending_count(), 0);

    let rel = RelationshipVector {
        affection: 0.0,
        trust: 0.0,
        attraction: 0.0,
        familiarity: 0.0,
        resentment: 0.0,
    };
    pressure.update_for_pair(1, 2, &rel, None, Some(100));

    let high = RelationshipVector {
        affection: 7.0,
        ..rel
    };
    pressure.update_for_pair(1, 2, &high, None, Some(100));

    assert!(pressure.has_pending_events());
    assert_eq!(pressure.pending_count(), 1);

    pressure.pop_next_event();
    assert!(!pressure.has_pending_events());
    assert_eq!(pressure.pending_count(), 0);
}
