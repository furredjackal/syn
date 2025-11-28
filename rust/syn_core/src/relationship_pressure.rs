//! Relationship Pressure System
//!
//! Tracks band transitions in relationships to trigger narrative events.
//! When a relationship axis crosses a band threshold (e.g., Trust goes from
//! "Wary" to "Trusted"), a pressure event is generated that storylets can react to.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

use crate::relationship_model::{
    AffectionBand, AttractionBand, RelationshipVector, ResentmentBand, TrustBand,
};

/// Snapshot of all relationship bands at a point in time.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RelationshipBandSnapshot {
    /// Current affection band.
    pub affection: AffectionBand,
    /// Current trust band.
    pub trust: TrustBand,
    /// Current attraction band.
    pub attraction: AttractionBand,
    /// Current resentment band.
    pub resentment: ResentmentBand,
    // Familiarity could become a band in the future.
}

impl RelationshipBandSnapshot {
    /// Create a snapshot from a relationship vector.
    pub fn from_vector(rel: &RelationshipVector) -> Self {
        Self {
            affection: rel.affection_band(),
            trust: rel.trust_band(),
            attraction: rel.attraction_band(),
            resentment: rel.resentment_band(),
        }
    }
}

/// Types of relationship band change events.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RelationshipEventKind {
    /// Affection band crossed a threshold.
    AffectionBandChanged,
    /// Trust band crossed a threshold.
    TrustBandChanged,
    /// Attraction band crossed a threshold.
    AttractionBandChanged,
    /// Resentment band crossed a threshold.
    ResentmentBandChanged,
}

/// A relationship pressure event (band transition).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RelationshipPressureEvent {
    /// NPC whose relationship changed.
    pub actor_id: u64,
    /// NPC the relationship is with.
    pub target_id: u64,
    /// Which axis changed.
    pub kind: RelationshipEventKind,
    /// Previous band label.
    pub old_band: String,
    /// New band label.
    pub new_band: String,
    /// Optional label indicating the source of the change, e.g. "storylet:<id>" or "drift".
    #[serde(default)]
    pub source: Option<String>,
    /// Optional tick index or time-step for ordering/debugging.
    #[serde(default)]
    pub tick: Option<u64>,
}

/// State for tracking relationship pressure events.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct RelationshipPressureState {
    /// Last known band snapshot for each (actor, target) pair.
    #[serde(default)]
    pub last_bands: HashMap<(u64, u64), RelationshipBandSnapshot>,

    /// FIFO queue of recent band change events.
    #[serde(default)]
    pub queue: VecDeque<RelationshipPressureEvent>,

    /// Legacy/simple tracking of changed pairs (kept for compatibility with prior logic).
    #[serde(default)]
    pub changed_pairs: Vec<(u64, u64)>,
}

impl RelationshipPressureState {
    /// Update tracking for a relationship pair, generating events if bands changed.
    pub fn update_for_pair(
        &mut self,
        actor_id: u64,
        target_id: u64,
        rel: &RelationshipVector,
        source: Option<String>,
        tick: Option<u64>,
    ) {
        use RelationshipEventKind::*;

        let new_snapshot = RelationshipBandSnapshot::from_vector(rel);

        let key = (actor_id, target_id);

        if let Some(old_snapshot) = self.last_bands.get(&key) {
            if old_snapshot.affection != new_snapshot.affection {
                self.queue.push_back(RelationshipPressureEvent {
                    actor_id,
                    target_id,
                    kind: AffectionBandChanged,
                    old_band: old_snapshot.affection.to_string(),
                    new_band: new_snapshot.affection.to_string(),
                    source: source.clone(),
                    tick,
                });
            }

            if old_snapshot.trust != new_snapshot.trust {
                self.queue.push_back(RelationshipPressureEvent {
                    actor_id,
                    target_id,
                    kind: TrustBandChanged,
                    old_band: old_snapshot.trust.to_string(),
                    new_band: new_snapshot.trust.to_string(),
                    source: source.clone(),
                    tick,
                });
            }

            if old_snapshot.attraction != new_snapshot.attraction {
                self.queue.push_back(RelationshipPressureEvent {
                    actor_id,
                    target_id,
                    kind: AttractionBandChanged,
                    old_band: old_snapshot.attraction.to_string(),
                    new_band: new_snapshot.attraction.to_string(),
                    source: source.clone(),
                    tick,
                });
            }

            if old_snapshot.resentment != new_snapshot.resentment {
                self.queue.push_back(RelationshipPressureEvent {
                    actor_id,
                    target_id,
                    kind: ResentmentBandChanged,
                    old_band: old_snapshot.resentment.to_string(),
                    new_band: new_snapshot.resentment.to_string(),
                    source,
                    tick,
                });
            }
        }

        // Keep simple changed_pairs tracking for legacy consumers.
        if !self.changed_pairs.contains(&key) {
            self.changed_pairs.push(key);
        }

        self.last_bands.insert(key, new_snapshot);
    }

    /// Pop the next pressure event from the queue.
    pub fn pop_next_event(&mut self) -> Option<RelationshipPressureEvent> {
        self.queue.pop_front()
    }

    /// Peek at the next pressure event without removing it.
    pub fn peek_next_event(&self) -> Option<&RelationshipPressureEvent> {
        self.queue.front()
    }

    /// Decay the queue by removing old events and enforcing size limits.
    ///
    /// This prevents unbounded queue growth when no matching storylets fire.
    ///
    /// # Arguments
    /// * `current_tick` - The current simulation tick
    /// * `max_age_ticks` - Events older than this are removed (default: 168 = 7 days)
    /// * `max_queue_size` - Maximum events to keep (oldest dropped first)
    pub fn decay_queue(&mut self, current_tick: u64, max_age_ticks: u64, max_queue_size: usize) {
        // Remove events older than max_age_ticks
        self.queue.retain(|event| {
            event.tick.map_or(true, |t| current_tick.saturating_sub(t) <= max_age_ticks)
        });

        // Enforce max queue size (drop oldest events)
        while self.queue.len() > max_queue_size {
            self.queue.pop_front();
        }

        // Also clean up stale changed_pairs (keep only recent pairs in queue)
        let active_pairs: std::collections::HashSet<(u64, u64)> = self
            .queue
            .iter()
            .map(|e| (e.actor_id, e.target_id))
            .collect();
        self.changed_pairs.retain(|pair| active_pairs.contains(pair));
    }

    /// Check if there are any pending pressure events.
    pub fn has_pending_events(&self) -> bool {
        !self.queue.is_empty()
    }

    /// Get the number of pending pressure events.
    pub fn pending_count(&self) -> usize {
        self.queue.len()
    }
}
