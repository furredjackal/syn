use serde::{Deserialize, Serialize};

use crate::types::{NpcId, WorldState};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationshipAxis {
    Affection,
    Trust,
    Attraction,
    Familiarity,
    Resentment,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RelationshipVector {
    pub affection: f32,
    pub trust: f32,
    pub attraction: f32,
    pub familiarity: f32,
    pub resentment: f32,
}

impl RelationshipVector {
    fn clamp_axis(axis: RelationshipAxis, value: f32) -> f32 {
        match axis {
            // All axes currently share the -10..10 range.
            _ => value.clamp(-10.0, 10.0),
        }
    }

    pub fn get(&self, axis: RelationshipAxis) -> f32 {
        match axis {
            RelationshipAxis::Affection => self.affection,
            RelationshipAxis::Trust => self.trust,
            RelationshipAxis::Attraction => self.attraction,
            RelationshipAxis::Familiarity => self.familiarity,
            RelationshipAxis::Resentment => self.resentment,
        }
    }

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

    pub fn apply_delta(&mut self, axis: RelationshipAxis, delta: f32) {
        let current = self.get(axis);
        self.set(axis, current + delta);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RelationshipDelta {
    pub target_id: NpcId,
    pub axis: RelationshipAxis,
    pub delta: f32,
    pub source: Option<String>,
}

pub trait RelationshipStore {
    fn apply_delta(&mut self, delta: &RelationshipDelta);
}

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
