use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

use crate::relationship_model::{
    RelationshipVector, AffectionBand, TrustBand, AttractionBand, ResentmentBand,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipBandSnapshot {
    pub affection: AffectionBand,
    pub trust: TrustBand,
    pub attraction: AttractionBand,
    pub resentment: ResentmentBand,
    // Familiarity could become a band in the future.
}

impl RelationshipBandSnapshot {
    pub fn from_vector(rel: &RelationshipVector) -> Self {
        Self {
            affection: rel.affection_band(),
            trust: rel.trust_band(),
            attraction: rel.attraction_band(),
            resentment: rel.resentment_band(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RelationshipEventKind {
    AffectionBandChanged,
    TrustBandChanged,
    AttractionBandChanged,
    ResentmentBandChanged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipPressureEvent {
    pub actor_id: u64,
    pub target_id: u64,
    pub kind: RelationshipEventKind,
    pub old_band: String,
    pub new_band: String,
    /// Optional label indicating the source of the change, e.g. "storylet:<id>" or "drift".
    #[serde(default)]
    pub source: Option<String>,
    /// Optional tick index or time-step for ordering/debugging.
    #[serde(default)]
    pub tick: Option<u64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

    pub fn pop_next_event(&mut self) -> Option<RelationshipPressureEvent> {
        self.queue.pop_front()
    }

    pub fn peek_next_event(&self) -> Option<&RelationshipPressureEvent> {
        self.queue.front()
    }
}
