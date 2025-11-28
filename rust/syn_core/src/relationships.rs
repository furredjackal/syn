//! Relationship system: axes, vectors, deltas, and storage.
//!
//! This module provides the core relationship mechanics:
//! - 5-axis relationship model (Affection, Trust, Attraction, Familiarity, Resentment)
//! - Delta application for storylet outcomes
//! - Store trait for different backend implementations

use serde::{Deserialize, Serialize};

use crate::types::{NpcId, WorldState};

/// The 5 axes of relationship (matches relationship_model.rs for compatibility).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationshipAxis {
    /// Emotional warmth and closeness.
    Affection,
    /// Reliability and safety.
    Trust,
    /// Romantic or sexual pull.
    Attraction,
    /// Shared time and history.
    Familiarity,
    /// Hostility and grudges.
    Resentment,
}

/// 5-axis relationship vector.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RelationshipVector {
    /// Emotional warmth (-10 to +10).
    pub affection: f32,
    /// Reliability (-10 to +10).
    pub trust: f32,
    /// Romantic pull (-10 to +10).
    pub attraction: f32,
    /// Shared history (-10 to +10).
    pub familiarity: f32,
    /// Hostility (-10 to +10).
    pub resentment: f32,
}

impl RelationshipVector {
    /// Clamp an axis value to valid range.
    fn clamp_axis(axis: RelationshipAxis, value: f32) -> f32 {
        match axis {
            // All axes currently share the -10..10 range.
            _ => value.clamp(-10.0, 10.0),
        }
    }

    /// Get the value of a specific axis.
    pub fn get(&self, axis: RelationshipAxis) -> f32 {
        match axis {
            RelationshipAxis::Affection => self.affection,
            RelationshipAxis::Trust => self.trust,
            RelationshipAxis::Attraction => self.attraction,
            RelationshipAxis::Familiarity => self.familiarity,
            RelationshipAxis::Resentment => self.resentment,
        }
    }

    /// Set the value of a specific axis (clamped).
    pub fn set(&mut self, axis: RelationshipAxis, value: f32) {
        let clamped = Self::clamp_axis(axis, value);
        match axis {
            RelationshipAxis::Affection => self.affection = clamped,
            RelationshipAxis::Trust => self.trust = clamped,
            RelationshipAxis::Attraction => self.attraction = clamped,
            RelationshipAxis::Familiarity => self.familiarity = clamped,
            RelationshipAxis::Resentment => self.resentment = clamped,
        }
    }

    /// Apply a delta to a specific axis.
    pub fn apply_delta(&mut self, axis: RelationshipAxis, delta: f32) {
        let current = self.get(axis);
        self.set(axis, current + delta);
    }
}

/// A pending change to a relationship axis.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RelationshipDelta {
    /// NPC whose relationship is being modified.
    pub target_id: NpcId,
    /// Which axis to modify.
    pub axis: RelationshipAxis,
    /// Amount to change (+/-).
    pub delta: f32,
    /// Optional source event/storylet.
    pub source: Option<String>,
}

/// Trait for types that can store and modify relationships.
pub trait RelationshipStore {
    /// Apply a relationship delta.
    fn apply_delta(&mut self, delta: &RelationshipDelta);
}

/// Apply a batch of relationship deltas to a store.
pub fn apply_relationship_deltas<R>(relations: &mut R, deltas: &[RelationshipDelta])
where
    R: RelationshipStore,
{
    for d in deltas {
        relations.apply_delta(d);
    }
}

impl RelationshipVector {
    /// Apply a delta directly on a vector (used by Relationship when delegating).
    pub fn apply_delta_axis(&mut self, axis: RelationshipAxis, delta: f32) {
        self.apply_delta(axis, delta);
    }
}

impl RelationshipStore for WorldState {
    fn apply_delta(&mut self, delta: &RelationshipDelta) {
        let mut current = self.get_relationship(self.player_id, delta.target_id);
        match delta.axis {
            RelationshipAxis::Affection => {
                current.affection = (current.affection + delta.delta).clamp(-10.0, 10.0)
            }
            RelationshipAxis::Trust => {
                current.trust = (current.trust + delta.delta).clamp(-10.0, 10.0)
            }
            RelationshipAxis::Attraction => {
                current.attraction = (current.attraction + delta.delta).clamp(-10.0, 10.0)
            }
            RelationshipAxis::Familiarity => {
                current.familiarity = (current.familiarity + delta.delta).clamp(-10.0, 10.0)
            }
            RelationshipAxis::Resentment => {
                current.resentment = (current.resentment + delta.delta).clamp(-10.0, 10.0)
            }
        }
        current.state = current.compute_next_state();
        self.set_relationship(self.player_id, delta.target_id, current);
    }
}
