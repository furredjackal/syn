//! Event queue management for the director.
//!
//! This module provides deterministic queue management for:
//! - Immediate events (fire this tick if selected)
//! - Delayed follow-ups (fire at tick T + N)
//! - Scheduled milestones (fire when conditions + tick thresholds are met)
//!
//! The queue maintains deterministic ordering and respects max size limits.

use serde::{Deserialize, Serialize};
use syn_core::SimTick;
use syn_storylets::library::StoryletKey;

/// Source of a queued event.
///
/// Different sources may have different priority handling or bypass rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QueueSource {
    /// Follow-up event from a previous storylet outcome.
    FollowUp,
    /// Milestone-triggered event when conditions are met.
    Milestone,
    /// Pressure relief event from relationship/life pressure.
    PressureRelief,
    /// Scripted event from external game logic.
    Scripted,
}

impl Default for QueueSource {
    fn default() -> Self {
        QueueSource::Scripted
    }
}

/// An event queued for future execution.
///
/// Queued events are sorted by (scheduled_tick, -priority, storylet_key)
/// for deterministic ordering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedEvent {
    /// The storylet to fire.
    pub storylet_key: StoryletKey,
    
    /// The tick at which this event should fire.
    pub scheduled_tick: SimTick,
    
    /// Priority for ordering (higher = fires first when multiple events are ready).
    /// Range: i32::MIN to i32::MAX, with 0 being "normal" priority.
    pub priority: i32,
    
    /// Whether this event bypasses some pacing/eligibility filters.
    pub forced: bool,
    
    /// Where this event came from.
    pub source: QueueSource,
}

impl QueuedEvent {
    /// Create a new queued event.
    pub fn new(
        storylet_key: StoryletKey,
        scheduled_tick: SimTick,
        priority: i32,
        forced: bool,
        source: QueueSource,
    ) -> Self {
        QueuedEvent {
            storylet_key,
            scheduled_tick,
            priority,
            forced,
            source,
        }
    }

    /// Create an immediate follow-up event (fires next tick).
    pub fn immediate_follow_up(storylet_key: StoryletKey, current_tick: SimTick, forced: bool) -> Self {
        QueuedEvent {
            storylet_key,
            scheduled_tick: SimTick::new(current_tick.0 + 1),
            priority: 0,
            forced,
            source: QueueSource::FollowUp,
        }
    }

    /// Create a delayed follow-up event.
    pub fn delayed_follow_up(
        storylet_key: StoryletKey,
        current_tick: SimTick,
        delay_ticks: u64,
        priority: i32,
        forced: bool,
    ) -> Self {
        QueuedEvent {
            storylet_key,
            scheduled_tick: SimTick::new(current_tick.0 + delay_ticks),
            priority,
            forced,
            source: QueueSource::FollowUp,
        }
    }

    /// Create a milestone-triggered event.
    pub fn milestone(
        storylet_key: StoryletKey,
        scheduled_tick: SimTick,
        priority: i32,
    ) -> Self {
        QueuedEvent {
            storylet_key,
            scheduled_tick,
            priority,
            forced: false,
            source: QueueSource::Milestone,
        }
    }

    /// Create a pressure relief event.
    pub fn pressure_relief(
        storylet_key: StoryletKey,
        scheduled_tick: SimTick,
        priority: i32,
        forced: bool,
    ) -> Self {
        QueuedEvent {
            storylet_key,
            scheduled_tick,
            priority,
            forced,
            source: QueueSource::PressureRelief,
        }
    }

    /// Compute the sort key for deterministic ordering.
    ///
    /// Events are sorted by:
    /// 1. scheduled_tick (ascending - earlier events first)
    /// 2. priority (descending - higher priority first, so we negate)
    /// 3. storylet_key (ascending - for deterministic tie-breaking)
    fn sort_key(&self) -> (u64, i32, u32) {
        // Negate priority so higher priority comes first when sorted ascending
        (self.scheduled_tick.0, -self.priority, self.storylet_key.0)
    }
}

impl PartialEq for QueuedEvent {
    fn eq(&self, other: &Self) -> bool {
        self.sort_key() == other.sort_key()
    }
}

impl Eq for QueuedEvent {}

impl PartialOrd for QueuedEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for QueuedEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.sort_key().cmp(&other.sort_key())
    }
}

/// Queue of scheduled and pending storylets.
///
/// The queue maintains events sorted by (scheduled_tick, -priority, storylet_key)
/// for deterministic processing order.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EventQueue {
    /// The queued events, maintained in sorted order.
    items: Vec<QueuedEvent>,
}

impl EventQueue {
    /// Create a new empty EventQueue.
    pub fn new() -> Self {
        EventQueue { items: Vec::new() }
    }

    /// Create an EventQueue with a pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        EventQueue {
            items: Vec::with_capacity(capacity),
        }
    }

    /// Push an event onto the queue, maintaining sorted order.
    ///
    /// If the queue would exceed `max_size`, the lowest-priority,
    /// farthest-future item is dropped.
    pub fn push(&mut self, event: QueuedEvent, max_size: usize) {
        // Insert in sorted order (binary search insertion)
        let pos = self.items.binary_search(&event).unwrap_or_else(|pos| pos);
        self.items.insert(pos, event);

        // If over capacity, drop the lowest priority item
        // (last in sorted order = farthest future, lowest priority)
        if self.items.len() > max_size && max_size > 0 {
            self.items.pop();
        }
    }

    /// Push an event without checking max_size (for internal use or testing).
    pub fn push_unchecked(&mut self, event: QueuedEvent) {
        let pos = self.items.binary_search(&event).unwrap_or_else(|pos| pos);
        self.items.insert(pos, event);
    }

    /// Pop and return all events ready to fire (scheduled_tick <= now).
    ///
    /// Returns events in deterministic order (sorted by priority, then key).
    pub fn pop_ready(&mut self, now: SimTick) -> Vec<QueuedEvent> {
        // Find the split point where scheduled_tick > now
        let split_pos = self.items.partition_point(|e| e.scheduled_tick.0 <= now.0);
        
        // Drain the ready events
        self.items.drain(..split_pos).collect()
    }

    /// Peek at events ready to fire without removing them.
    pub fn peek_ready(&self, now: SimTick) -> impl Iterator<Item = &QueuedEvent> {
        let now_tick = now.0;
        self.items.iter().take_while(move |e| e.scheduled_tick.0 <= now_tick)
    }

    /// Pop only forced events that are ready.
    ///
    /// Forced events bypass normal pacing and take precedence.
    pub fn pop_forced_ready(&mut self, now: SimTick) -> Vec<QueuedEvent> {
        let mut forced = Vec::new();
        let mut remaining = Vec::new();
        
        for event in self.items.drain(..) {
            if event.scheduled_tick.0 <= now.0 && event.forced {
                forced.push(event);
            } else {
                remaining.push(event);
            }
        }
        
        self.items = remaining;
        forced
    }

    /// Get the next event that will be ready (for preview/debugging).
    pub fn peek_next(&self) -> Option<&QueuedEvent> {
        self.items.first()
    }

    /// Get all queued events (for inspection/debugging).
    pub fn all_events(&self) -> &[QueuedEvent] {
        &self.items
    }

    /// Remove all events for a specific storylet.
    ///
    /// Useful when a storylet becomes invalid or is being replaced.
    pub fn remove_storylet(&mut self, key: StoryletKey) {
        self.items.retain(|e| e.storylet_key != key);
    }

    /// Remove all events from a specific source.
    pub fn remove_by_source(&mut self, source: QueueSource) {
        self.items.retain(|e| e.source != source);
    }

    /// Clear all queued events.
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Check if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get the number of queued events.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if there are any ready events.
    pub fn has_ready(&self, now: SimTick) -> bool {
        self.items.first().map(|e| e.scheduled_tick.0 <= now.0).unwrap_or(false)
    }

    /// Check if there are any forced events ready.
    pub fn has_forced_ready(&self, now: SimTick) -> bool {
        self.items.iter().any(|e| e.scheduled_tick.0 <= now.0 && e.forced)
    }

    /// Get count of events by source.
    pub fn count_by_source(&self, source: QueueSource) -> usize {
        self.items.iter().filter(|e| e.source == source).count()
    }

    /// Get the earliest scheduled tick in the queue.
    pub fn earliest_tick(&self) -> Option<SimTick> {
        self.items.first().map(|e| e.scheduled_tick)
    }

    /// Get the latest scheduled tick in the queue.
    pub fn latest_tick(&self) -> Option<SimTick> {
        self.items.last().map(|e| e.scheduled_tick)
    }
}

// ============================================================================
// Convenience functions for queue management
// ============================================================================

/// Schedule a follow-up event from a storylet outcome.
///
/// This is the primary way storylet outcomes schedule future events.
pub fn schedule_follow_up(
    queue: &mut EventQueue,
    storylet_key: StoryletKey,
    current_tick: SimTick,
    delay_ticks: u64,
    priority: i32,
    forced: bool,
    max_queue_size: usize,
) {
    let event = QueuedEvent::delayed_follow_up(
        storylet_key,
        current_tick,
        delay_ticks,
        priority,
        forced,
    );
    queue.push(event, max_queue_size);
}

/// Schedule a milestone event.
pub fn schedule_milestone(
    queue: &mut EventQueue,
    storylet_key: StoryletKey,
    scheduled_tick: SimTick,
    priority: i32,
    max_queue_size: usize,
) {
    let event = QueuedEvent::milestone(storylet_key, scheduled_tick, priority);
    queue.push(event, max_queue_size);
}

/// Schedule a pressure relief event.
pub fn schedule_pressure_relief(
    queue: &mut EventQueue,
    storylet_key: StoryletKey,
    scheduled_tick: SimTick,
    priority: i32,
    forced: bool,
    max_queue_size: usize,
) {
    let event = QueuedEvent::pressure_relief(storylet_key, scheduled_tick, priority, forced);
    queue.push(event, max_queue_size);
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_event(key: u32, tick: u64, priority: i32, forced: bool, source: QueueSource) -> QueuedEvent {
        QueuedEvent::new(
            StoryletKey(key),
            SimTick::new(tick),
            priority,
            forced,
            source,
        )
    }

    // =========================================================================
    // QueuedEvent Tests
    // =========================================================================

    #[test]
    fn test_queued_event_creation() {
        let event = QueuedEvent::new(
            StoryletKey(42),
            SimTick::new(100),
            5,
            true,
            QueueSource::FollowUp,
        );

        assert_eq!(event.storylet_key, StoryletKey(42));
        assert_eq!(event.scheduled_tick, SimTick::new(100));
        assert_eq!(event.priority, 5);
        assert!(event.forced);
        assert_eq!(event.source, QueueSource::FollowUp);
    }

    #[test]
    fn test_immediate_follow_up() {
        let current = SimTick::new(50);
        let event = QueuedEvent::immediate_follow_up(StoryletKey(1), current, true);

        assert_eq!(event.scheduled_tick, SimTick::new(51));
        assert_eq!(event.priority, 0);
        assert!(event.forced);
        assert_eq!(event.source, QueueSource::FollowUp);
    }

    #[test]
    fn test_delayed_follow_up() {
        let current = SimTick::new(50);
        let event = QueuedEvent::delayed_follow_up(StoryletKey(1), current, 10, 5, false);

        assert_eq!(event.scheduled_tick, SimTick::new(60));
        assert_eq!(event.priority, 5);
        assert!(!event.forced);
        assert_eq!(event.source, QueueSource::FollowUp);
    }

    #[test]
    fn test_event_ordering_by_tick() {
        let early = create_event(1, 10, 0, false, QueueSource::Scripted);
        let late = create_event(2, 20, 0, false, QueueSource::Scripted);

        assert!(early < late);
    }

    #[test]
    fn test_event_ordering_by_priority() {
        // Same tick, different priorities - higher priority should come first
        let high_priority = create_event(1, 10, 10, false, QueueSource::Scripted);
        let low_priority = create_event(2, 10, 5, false, QueueSource::Scripted);

        assert!(high_priority < low_priority);
    }

    #[test]
    fn test_event_ordering_by_key_tiebreaker() {
        // Same tick, same priority - lower key should come first
        let low_key = create_event(1, 10, 5, false, QueueSource::Scripted);
        let high_key = create_event(2, 10, 5, false, QueueSource::Scripted);

        assert!(low_key < high_key);
    }

    // =========================================================================
    // EventQueue Basic Tests
    // =========================================================================

    #[test]
    fn test_queue_new() {
        let queue = EventQueue::new();
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_queue_push_maintains_order() {
        let mut queue = EventQueue::new();

        // Push in random order
        queue.push_unchecked(create_event(3, 30, 0, false, QueueSource::Scripted));
        queue.push_unchecked(create_event(1, 10, 0, false, QueueSource::Scripted));
        queue.push_unchecked(create_event(2, 20, 0, false, QueueSource::Scripted));

        // Should be sorted by tick
        let events = queue.all_events();
        assert_eq!(events[0].storylet_key, StoryletKey(1));
        assert_eq!(events[1].storylet_key, StoryletKey(2));
        assert_eq!(events[2].storylet_key, StoryletKey(3));
    }

    #[test]
    fn test_queue_push_priority_ordering() {
        let mut queue = EventQueue::new();

        // Same tick, different priorities
        queue.push_unchecked(create_event(1, 10, 1, false, QueueSource::Scripted));
        queue.push_unchecked(create_event(2, 10, 10, false, QueueSource::Scripted)); // High priority
        queue.push_unchecked(create_event(3, 10, 5, false, QueueSource::Scripted));

        // Sorted by priority (descending, so high first)
        let events = queue.all_events();
        assert_eq!(events[0].storylet_key, StoryletKey(2)); // priority 10
        assert_eq!(events[1].storylet_key, StoryletKey(3)); // priority 5
        assert_eq!(events[2].storylet_key, StoryletKey(1)); // priority 1
    }

    #[test]
    fn test_queue_push_key_tiebreaker() {
        let mut queue = EventQueue::new();

        // Same tick, same priority, different keys
        queue.push_unchecked(create_event(3, 10, 5, false, QueueSource::Scripted));
        queue.push_unchecked(create_event(1, 10, 5, false, QueueSource::Scripted));
        queue.push_unchecked(create_event(2, 10, 5, false, QueueSource::Scripted));

        // Sorted by key (ascending)
        let events = queue.all_events();
        assert_eq!(events[0].storylet_key, StoryletKey(1));
        assert_eq!(events[1].storylet_key, StoryletKey(2));
        assert_eq!(events[2].storylet_key, StoryletKey(3));
    }

    // =========================================================================
    // Max Size Tests
    // =========================================================================

    #[test]
    fn test_queue_push_respects_max_size() {
        let mut queue = EventQueue::new();

        // Push events with max_size = 3
        queue.push(create_event(1, 10, 5, false, QueueSource::Scripted), 3);
        queue.push(create_event(2, 20, 5, false, QueueSource::Scripted), 3);
        queue.push(create_event(3, 30, 5, false, QueueSource::Scripted), 3);
        queue.push(create_event(4, 40, 5, false, QueueSource::Scripted), 3); // This should evict

        assert_eq!(queue.len(), 3);
        // The last item (key=4, tick=40) should have been evicted
        let events = queue.all_events();
        assert_eq!(events[0].storylet_key, StoryletKey(1));
        assert_eq!(events[1].storylet_key, StoryletKey(2));
        assert_eq!(events[2].storylet_key, StoryletKey(3));
    }

    #[test]
    fn test_queue_push_evicts_lowest_priority() {
        let mut queue = EventQueue::new();

        // Push events with different priorities, max_size = 2
        queue.push(create_event(1, 10, 10, false, QueueSource::Scripted), 2); // High priority
        queue.push(create_event(2, 20, 1, false, QueueSource::Scripted), 2);  // Low priority
        queue.push(create_event(3, 15, 5, false, QueueSource::Scripted), 2);  // Medium priority

        assert_eq!(queue.len(), 2);
        // Key 2 (lowest priority, farthest future) should be evicted
        let events = queue.all_events();
        assert_eq!(events[0].storylet_key, StoryletKey(1)); // priority 10, tick 10
        assert_eq!(events[1].storylet_key, StoryletKey(3)); // priority 5, tick 15
    }

    #[test]
    fn test_queue_max_size_zero_means_unbounded() {
        let mut queue = EventQueue::new();

        // max_size = 0 should not evict (acts as unbounded)
        queue.push(create_event(1, 10, 0, false, QueueSource::Scripted), 0);
        queue.push(create_event(2, 20, 0, false, QueueSource::Scripted), 0);

        assert_eq!(queue.len(), 2);
    }

    // =========================================================================
    // Pop Ready Tests
    // =========================================================================

    #[test]
    fn test_pop_ready_returns_ready_events() {
        let mut queue = EventQueue::new();

        queue.push_unchecked(create_event(1, 10, 0, false, QueueSource::Scripted));
        queue.push_unchecked(create_event(2, 20, 0, false, QueueSource::Scripted));
        queue.push_unchecked(create_event(3, 30, 0, false, QueueSource::Scripted));

        let now = SimTick::new(20);
        let ready = queue.pop_ready(now);

        assert_eq!(ready.len(), 2);
        assert_eq!(ready[0].storylet_key, StoryletKey(1));
        assert_eq!(ready[1].storylet_key, StoryletKey(2));

        // Only tick 30 event should remain
        assert_eq!(queue.len(), 1);
        assert_eq!(queue.all_events()[0].storylet_key, StoryletKey(3));
    }

    #[test]
    fn test_pop_ready_empty_when_none_ready() {
        let mut queue = EventQueue::new();

        queue.push_unchecked(create_event(1, 100, 0, false, QueueSource::Scripted));
        queue.push_unchecked(create_event(2, 200, 0, false, QueueSource::Scripted));

        let now = SimTick::new(50);
        let ready = queue.pop_ready(now);

        assert!(ready.is_empty());
        assert_eq!(queue.len(), 2);
    }

    #[test]
    fn test_pop_ready_includes_exact_tick() {
        let mut queue = EventQueue::new();

        queue.push_unchecked(create_event(1, 50, 0, false, QueueSource::Scripted));

        let now = SimTick::new(50);
        let ready = queue.pop_ready(now);

        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].storylet_key, StoryletKey(1));
    }

    #[test]
    fn test_pop_ready_preserves_order() {
        let mut queue = EventQueue::new();

        // Mix of ticks and priorities, all ready
        queue.push_unchecked(create_event(3, 10, 1, false, QueueSource::Scripted));
        queue.push_unchecked(create_event(1, 5, 10, false, QueueSource::Scripted));
        queue.push_unchecked(create_event(2, 5, 5, false, QueueSource::Scripted));

        let now = SimTick::new(20);
        let ready = queue.pop_ready(now);

        assert_eq!(ready.len(), 3);
        // Order: tick 5 priority 10, tick 5 priority 5, tick 10 priority 1
        assert_eq!(ready[0].storylet_key, StoryletKey(1));
        assert_eq!(ready[1].storylet_key, StoryletKey(2));
        assert_eq!(ready[2].storylet_key, StoryletKey(3));
    }

    // =========================================================================
    // Forced Events Tests
    // =========================================================================

    #[test]
    fn test_pop_forced_ready() {
        let mut queue = EventQueue::new();

        queue.push_unchecked(create_event(1, 10, 0, true, QueueSource::Scripted));   // Forced
        queue.push_unchecked(create_event(2, 10, 0, false, QueueSource::Scripted));  // Not forced
        queue.push_unchecked(create_event(3, 10, 0, true, QueueSource::Scripted));   // Forced
        queue.push_unchecked(create_event(4, 100, 0, true, QueueSource::Scripted));  // Forced but not ready

        let now = SimTick::new(50);
        let forced = queue.pop_forced_ready(now);

        assert_eq!(forced.len(), 2);
        assert!(forced.iter().all(|e| e.forced));

        // Non-forced ready event and future forced event should remain
        assert_eq!(queue.len(), 2);
        let remaining: Vec<_> = queue.all_events().iter().map(|e| e.storylet_key).collect();
        assert!(remaining.contains(&StoryletKey(2)));
        assert!(remaining.contains(&StoryletKey(4)));
    }

    #[test]
    fn test_has_forced_ready() {
        let mut queue = EventQueue::new();

        // No forced events
        queue.push_unchecked(create_event(1, 10, 0, false, QueueSource::Scripted));
        assert!(!queue.has_forced_ready(SimTick::new(50)));

        // Add forced event (but not ready)
        queue.push_unchecked(create_event(2, 100, 0, true, QueueSource::Scripted));
        assert!(!queue.has_forced_ready(SimTick::new(50)));

        // Add forced event (ready)
        queue.push_unchecked(create_event(3, 10, 0, true, QueueSource::Scripted));
        assert!(queue.has_forced_ready(SimTick::new(50)));
    }

    // =========================================================================
    // Removal Tests
    // =========================================================================

    #[test]
    fn test_remove_storylet() {
        let mut queue = EventQueue::new();

        queue.push_unchecked(create_event(1, 10, 0, false, QueueSource::Scripted));
        queue.push_unchecked(create_event(2, 20, 0, false, QueueSource::Scripted));
        queue.push_unchecked(create_event(1, 30, 0, false, QueueSource::Scripted)); // Same key, different tick

        queue.remove_storylet(StoryletKey(1));

        assert_eq!(queue.len(), 1);
        assert_eq!(queue.all_events()[0].storylet_key, StoryletKey(2));
    }

    #[test]
    fn test_remove_by_source() {
        let mut queue = EventQueue::new();

        queue.push_unchecked(create_event(1, 10, 0, false, QueueSource::FollowUp));
        queue.push_unchecked(create_event(2, 20, 0, false, QueueSource::Milestone));
        queue.push_unchecked(create_event(3, 30, 0, false, QueueSource::FollowUp));

        queue.remove_by_source(QueueSource::FollowUp);

        assert_eq!(queue.len(), 1);
        assert_eq!(queue.all_events()[0].source, QueueSource::Milestone);
    }

    #[test]
    fn test_clear() {
        let mut queue = EventQueue::new();

        queue.push_unchecked(create_event(1, 10, 0, false, QueueSource::Scripted));
        queue.push_unchecked(create_event(2, 20, 0, false, QueueSource::Scripted));

        queue.clear();

        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
    }

    // =========================================================================
    // Query Tests
    // =========================================================================

    #[test]
    fn test_count_by_source() {
        let mut queue = EventQueue::new();

        queue.push_unchecked(create_event(1, 10, 0, false, QueueSource::FollowUp));
        queue.push_unchecked(create_event(2, 20, 0, false, QueueSource::Milestone));
        queue.push_unchecked(create_event(3, 30, 0, false, QueueSource::FollowUp));
        queue.push_unchecked(create_event(4, 40, 0, false, QueueSource::PressureRelief));

        assert_eq!(queue.count_by_source(QueueSource::FollowUp), 2);
        assert_eq!(queue.count_by_source(QueueSource::Milestone), 1);
        assert_eq!(queue.count_by_source(QueueSource::PressureRelief), 1);
        assert_eq!(queue.count_by_source(QueueSource::Scripted), 0);
    }

    #[test]
    fn test_earliest_and_latest_tick() {
        let mut queue = EventQueue::new();

        assert!(queue.earliest_tick().is_none());
        assert!(queue.latest_tick().is_none());

        queue.push_unchecked(create_event(1, 50, 0, false, QueueSource::Scripted));
        queue.push_unchecked(create_event(2, 10, 0, false, QueueSource::Scripted));
        queue.push_unchecked(create_event(3, 30, 0, false, QueueSource::Scripted));

        assert_eq!(queue.earliest_tick(), Some(SimTick::new(10)));
        assert_eq!(queue.latest_tick(), Some(SimTick::new(50)));
    }

    // =========================================================================
    // Determinism Tests
    // =========================================================================

    #[test]
    fn test_queue_ordering_is_deterministic() {
        // Run the same sequence multiple times
        for _ in 0..10 {
            let mut queue = EventQueue::new();

            queue.push_unchecked(create_event(5, 10, 3, false, QueueSource::Scripted));
            queue.push_unchecked(create_event(2, 10, 5, true, QueueSource::FollowUp));
            queue.push_unchecked(create_event(8, 5, 1, false, QueueSource::Milestone));
            queue.push_unchecked(create_event(1, 10, 5, false, QueueSource::PressureRelief));
            queue.push_unchecked(create_event(3, 20, 10, true, QueueSource::Scripted));

            let events = queue.all_events();
            
            // Expected order:
            // 1. tick=5, priority=1, key=8
            // 2. tick=10, priority=5, key=1
            // 3. tick=10, priority=5, key=2
            // 4. tick=10, priority=3, key=5
            // 5. tick=20, priority=10, key=3
            assert_eq!(events[0].storylet_key, StoryletKey(8));
            assert_eq!(events[1].storylet_key, StoryletKey(1));
            assert_eq!(events[2].storylet_key, StoryletKey(2));
            assert_eq!(events[3].storylet_key, StoryletKey(5));
            assert_eq!(events[4].storylet_key, StoryletKey(3));
        }
    }

    #[test]
    fn test_pop_ready_is_deterministic() {
        for _ in 0..10 {
            let mut queue = EventQueue::new();

            queue.push_unchecked(create_event(3, 10, 1, false, QueueSource::Scripted));
            queue.push_unchecked(create_event(1, 10, 5, false, QueueSource::Scripted));
            queue.push_unchecked(create_event(2, 10, 5, false, QueueSource::Scripted));
            queue.push_unchecked(create_event(4, 5, 10, false, QueueSource::Scripted));

            let ready = queue.pop_ready(SimTick::new(10));

            // Deterministic order
            assert_eq!(ready[0].storylet_key, StoryletKey(4)); // tick 5
            assert_eq!(ready[1].storylet_key, StoryletKey(1)); // tick 10, priority 5, key 1
            assert_eq!(ready[2].storylet_key, StoryletKey(2)); // tick 10, priority 5, key 2
            assert_eq!(ready[3].storylet_key, StoryletKey(3)); // tick 10, priority 1
        }
    }

    // =========================================================================
    // Convenience Function Tests
    // =========================================================================

    #[test]
    fn test_schedule_follow_up() {
        let mut queue = EventQueue::new();

        schedule_follow_up(
            &mut queue,
            StoryletKey(42),
            SimTick::new(100),
            10,
            5,
            true,
            100,
        );

        assert_eq!(queue.len(), 1);
        let event = queue.peek_next().unwrap();
        assert_eq!(event.storylet_key, StoryletKey(42));
        assert_eq!(event.scheduled_tick, SimTick::new(110));
        assert_eq!(event.priority, 5);
        assert!(event.forced);
        assert_eq!(event.source, QueueSource::FollowUp);
    }

    #[test]
    fn test_schedule_milestone() {
        let mut queue = EventQueue::new();

        schedule_milestone(
            &mut queue,
            StoryletKey(99),
            SimTick::new(500),
            10,
            100,
        );

        let event = queue.peek_next().unwrap();
        assert_eq!(event.source, QueueSource::Milestone);
        assert!(!event.forced);
    }

    #[test]
    fn test_schedule_pressure_relief() {
        let mut queue = EventQueue::new();

        schedule_pressure_relief(
            &mut queue,
            StoryletKey(77),
            SimTick::new(200),
            3,
            true,
            100,
        );

        let event = queue.peek_next().unwrap();
        assert_eq!(event.source, QueueSource::PressureRelief);
        assert!(event.forced);
    }

    // =========================================================================
    // Integration Tests
    // =========================================================================

    #[test]
    fn test_forced_events_win_over_normal() {
        let mut queue = EventQueue::new();

        // Higher priority normal event
        queue.push_unchecked(create_event(1, 10, 100, false, QueueSource::Scripted));
        // Lower priority forced event
        queue.push_unchecked(create_event(2, 10, 1, true, QueueSource::FollowUp));

        // Pop forced first
        let forced = queue.pop_forced_ready(SimTick::new(10));
        assert_eq!(forced.len(), 1);
        assert_eq!(forced[0].storylet_key, StoryletKey(2));

        // Normal event still in queue
        assert_eq!(queue.len(), 1);
        assert_eq!(queue.all_events()[0].storylet_key, StoryletKey(1));
    }
}
