//! Tests for queue max_size overflow behavior.
//!
//! Verifies that when the queue exceeds max_size, the lowest-priority
//! (farthest future, lowest priority, highest key) event is evicted.

use syn_core::SimTick;
use syn_director::queue::{EventQueue, QueueSource, QueuedEvent};
use syn_storylets::library::StoryletKey;

/// Create a test event with specified parameters.
fn create_event(
    key: u32,
    tick: u64,
    priority: i32,
    forced: bool,
    source: QueueSource,
) -> QueuedEvent {
    QueuedEvent::new(
        StoryletKey(key),
        SimTick::new(tick),
        priority,
        forced,
        source,
    )
}

#[test]
fn test_queue_overflow_evicts_last_item() {
    let mut queue = EventQueue::new();

    // Push 3 events with max_size = 3
    queue.push(
        create_event(1, 10, 5, false, QueueSource::Scripted),
        3,
    );
    queue.push(
        create_event(2, 20, 5, false, QueueSource::Scripted),
        3,
    );
    queue.push(
        create_event(3, 30, 5, false, QueueSource::Scripted),
        3,
    );

    assert_eq!(queue.len(), 3, "Queue should have 3 items");

    // Push a 4th event - should evict the "worst" one
    // Sorting: (tick ascending, priority descending, key ascending)
    // So key=4, tick=40 is "worst" (farthest future)
    queue.push(
        create_event(4, 40, 5, false, QueueSource::Scripted),
        3,
    );

    assert_eq!(queue.len(), 3, "Queue should still have 3 items after overflow");

    // Verify which events remain
    let events = queue.all_events();
    let keys: Vec<u32> = events.iter().map(|e| e.storylet_key.0).collect();

    // Events should be sorted by tick (ascending)
    // The 4th event (tick=40) was just added and then evicted as it's "worst"
    assert_eq!(keys, vec![1, 2, 3], "Should keep events with earlier ticks");
}

#[test]
fn test_queue_overflow_evicts_lowest_priority_when_same_tick() {
    let mut queue = EventQueue::new();

    // All events at same tick, different priorities
    queue.push(
        create_event(1, 10, 10, false, QueueSource::Scripted), // High priority
        2,
    );
    queue.push(
        create_event(2, 10, 1, false, QueueSource::Scripted), // Low priority
        2,
    );

    assert_eq!(queue.len(), 2);

    // Add a medium priority event - should evict the lowest priority
    queue.push(
        create_event(3, 10, 5, false, QueueSource::Scripted), // Medium priority
        2,
    );

    assert_eq!(queue.len(), 2, "Should still have 2 items");

    let events = queue.all_events();
    let keys: Vec<u32> = events.iter().map(|e| e.storylet_key.0).collect();

    // key=2 (priority 1) should be evicted
    // Remaining should be key=1 (priority 10) and key=3 (priority 5)
    // Sorted by priority descending, so key=1 first, then key=3
    assert!(
        keys.contains(&1) && keys.contains(&3),
        "Should keep high and medium priority events, got {:?}",
        keys
    );
    assert!(
        !keys.contains(&2),
        "Should evict lowest priority event (key=2)"
    );
}

#[test]
fn test_queue_overflow_uses_key_as_tiebreaker() {
    let mut queue = EventQueue::new();

    // All events at same tick, same priority, different keys
    queue.push(
        create_event(3, 10, 5, false, QueueSource::Scripted),
        2,
    );
    queue.push(
        create_event(1, 10, 5, false, QueueSource::Scripted),
        2,
    );

    assert_eq!(queue.len(), 2);

    // Add another event with higher key
    queue.push(
        create_event(5, 10, 5, false, QueueSource::Scripted),
        2,
    );

    assert_eq!(queue.len(), 2);

    let events = queue.all_events();
    let keys: Vec<u32> = events.iter().map(|e| e.storylet_key.0).collect();

    // With same tick and priority, higher keys come last (ascending order)
    // So key=5 is "worst" and should be evicted
    assert_eq!(keys, vec![1, 3], "Should keep lower keys and evict highest key");
}

#[test]
fn test_queue_max_size_zero_allows_unbounded() {
    let mut queue = EventQueue::new();

    // max_size = 0 means unbounded
    queue.push(create_event(1, 10, 0, false, QueueSource::Scripted), 0);
    queue.push(create_event(2, 20, 0, false, QueueSource::Scripted), 0);
    queue.push(create_event(3, 30, 0, false, QueueSource::Scripted), 0);
    queue.push(create_event(4, 40, 0, false, QueueSource::Scripted), 0);

    assert_eq!(queue.len(), 4, "max_size=0 should not evict");
}

#[test]
fn test_queue_maintains_sorted_order_after_overflow() {
    let mut queue = EventQueue::new();

    // Push events in non-sorted order with varying priorities
    queue.push(
        create_event(3, 30, 1, false, QueueSource::Scripted), // Low priority, late
        3,
    );
    queue.push(
        create_event(1, 10, 10, false, QueueSource::Scripted), // High priority, early
        3,
    );
    queue.push(
        create_event(2, 20, 5, false, QueueSource::Scripted), // Medium
        3,
    );

    // Now overflow
    queue.push(
        create_event(4, 5, 3, false, QueueSource::Scripted), // Very early, low-ish priority
        3,
    );

    assert_eq!(queue.len(), 3);

    let events = queue.all_events();

    // Verify events are still sorted by (tick, -priority, key)
    for i in 0..events.len() - 1 {
        let current = &events[i];
        let next = &events[i + 1];
        assert!(
            current <= next,
            "Events should remain sorted: {:?} should be <= {:?}",
            current,
            next
        );
    }
}
