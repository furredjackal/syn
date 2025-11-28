//! 5-Axis Relationship Model
//!
//! This module implements the core relationship vector system per GDD §5.
//! Relationships are modeled as 5-axis vectors:
//! - **Affection**: Emotional warmth (-10 to +10)
//! - **Trust**: Reliability, safety (-10 to +10)
//! - **Attraction**: Romantic/sexual pull (-10 to +10)
//! - **Familiarity**: Shared history (-10 to +10)
//! - **Resentment**: Hostility, grudges (-10 to +10)
//!
//! Axes are converted to bands for eligibility checking and UI display.

use serde::{Deserialize, Serialize};

/// Clamp a relationship axis value to valid range.
fn clamp_axis(value: f32) -> f32 {
    value.clamp(-10.0, 10.0)
}

/// The 5 axes of the relationship model.
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

/// High-level relationship roles derived from axis combinations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationshipRole {
    /// No meaningful connection.
    Stranger,
    /// Casual acquaintance.
    Acquaintance,
    /// Positive friendship.
    Friend,
    /// Conflicted, hostile.
    Rival,
    /// Trusted ally (not romantic).
    Ally,
    /// Romantic interest or partner.
    Romance,
    /// Family-like bond.
    Family,
}

impl std::fmt::Display for RelationshipRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            RelationshipRole::Stranger => "Stranger",
            RelationshipRole::Acquaintance => "Acquaintance",
            RelationshipRole::Friend => "Friend",
            RelationshipRole::Rival => "Rival",
            RelationshipRole::Ally => "Ally",
            RelationshipRole::Romance => "Crush",
            RelationshipRole::Family => "Family",
        };
        write!(f, "{s}")
    }
}

/// 5-axis relationship vector between two characters.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RelationshipVector {
    /// Emotional warmth (-10 to +10).
    pub affection: f32,
    /// Reliability, safety (-10 to +10).
    pub trust: f32,
    /// Romantic/sexual pull (-10 to +10).
    pub attraction: f32,
    /// Shared time, history (-10 to +10).
    pub familiarity: f32,
    /// Hostility, grudges (-10 to +10).
    pub resentment: f32,
}

impl RelationshipVector {
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

    /// Set the value of a specific axis (clamped to -10..+10).
    pub fn set(&mut self, axis: RelationshipAxis, value: f32) {
        let clamped = clamp_axis(value);
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
        self.set(axis, self.get(axis) + delta);
    }

    /// Get the affection band for this vector.
    pub fn affection_band(&self) -> AffectionBand {
        let v = self.affection;
        if v <= -5.0 {
            AffectionBand::Stranger
        } else if v < 1.0 {
            AffectionBand::Acquaintance
        } else if v < 5.0 {
            AffectionBand::Friendly
        } else if v < 8.0 {
            AffectionBand::Close
        } else {
            AffectionBand::Devoted
        }
    }

    /// Get the trust band for this vector.
    pub fn trust_band(&self) -> TrustBand {
        let v = self.trust;
        if v <= -5.0 {
            TrustBand::Unknown
        } else if v < -1.0 {
            TrustBand::Wary
        } else if v < 2.0 {
            TrustBand::Neutral
        } else if v < 7.0 {
            TrustBand::Trusted
        } else {
            TrustBand::DeepTrust
        }
    }

    /// Get the attraction band for this vector.
    pub fn attraction_band(&self) -> AttractionBand {
        let v = self.attraction;
        if v <= 0.0 {
            AttractionBand::None
        } else if v < 3.0 {
            AttractionBand::Curious
        } else if v < 6.0 {
            AttractionBand::Interested
        } else if v < 8.0 {
            AttractionBand::Strong
        } else {
            AttractionBand::Intense
        }
    }

    /// Get the resentment band for this vector.
    pub fn resentment_band(&self) -> ResentmentBand {
        let v = self.resentment;
        if v <= 0.0 {
            ResentmentBand::None
        } else if v < 3.0 {
            ResentmentBand::Irritated
        } else if v < 6.0 {
            ResentmentBand::Resentful
        } else if v < 8.0 {
            ResentmentBand::Hostile
        } else {
            ResentmentBand::Vindictive
        }
    }

    /// Derive the high-level relationship role from axis bands.
    pub fn role(&self) -> RelationshipRole {
        let aff = self.affection_band();
        let trust = self.trust_band();
        let attr = self.attraction_band();
        let resent = self.resentment_band();

        if matches!(resent, ResentmentBand::Hostile | ResentmentBand::Vindictive) {
            return RelationshipRole::Rival;
        }

        if matches!(attr, AttractionBand::Strong | AttractionBand::Intense)
            && matches!(
                aff,
                AffectionBand::Friendly | AffectionBand::Close | AffectionBand::Devoted
            )
        {
            return RelationshipRole::Romance;
        }

        if matches!(aff, AffectionBand::Devoted)
            && matches!(trust, TrustBand::Trusted | TrustBand::DeepTrust)
        {
            return RelationshipRole::Family;
        }

        if matches!(aff, AffectionBand::Close | AffectionBand::Friendly)
            && matches!(trust, TrustBand::Trusted | TrustBand::DeepTrust)
        {
            return RelationshipRole::Friend;
        }

        if matches!(aff, AffectionBand::Acquaintance | AffectionBand::Friendly) {
            return RelationshipRole::Acquaintance;
        }

        RelationshipRole::Stranger
    }
}

/// Derive a high-level role label from a relationship vector.
/// Returns labels like "Romance", "Friend", "Rival", "Acquaintance", "Stranger".
pub fn derive_role_label(rel: &RelationshipVector) -> String {
    rel.role().to_string()
}

/// Affection axis band (emotional warmth tier).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AffectionBand {
    /// Cold/distant (-10 to -5).
    Stranger,
    /// Casual (-5 to 1).
    Acquaintance,
    /// Warm (1 to 5).
    Friendly,
    /// Very warm (5 to 8).
    Close,
    /// Deeply bonded (8+).
    Devoted,
}

impl std::fmt::Display for AffectionBand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            AffectionBand::Stranger => "Stranger",
            AffectionBand::Acquaintance => "Acquaintance",
            AffectionBand::Friendly => "Friendly",
            AffectionBand::Close => "Close",
            AffectionBand::Devoted => "Devoted",
        };
        write!(f, "{}", s)
    }
}

/// Trust axis band (reliability tier).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrustBand {
    /// Completely unknown (-10 to -5).
    Unknown,
    /// Suspicious (-5 to -1).
    Wary,
    /// Default (-1 to 2).
    Neutral,
    /// Reliable (2 to 7).
    Trusted,
    /// Absolute trust (7+).
    DeepTrust,
}

impl std::fmt::Display for TrustBand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TrustBand::Unknown => "Unknown",
            TrustBand::Wary => "Wary",
            TrustBand::Neutral => "Neutral",
            TrustBand::Trusted => "Trusted",
            TrustBand::DeepTrust => "DeepTrust",
        };
        write!(f, "{}", s)
    }
}

/// Attraction axis band (romantic interest tier).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttractionBand {
    /// No attraction (0 or below).
    None,
    /// Slight interest (0 to 3).
    Curious,
    /// Definite interest (3 to 6).
    Interested,
    /// Strong attraction (6 to 8).
    Strong,
    /// Overwhelming attraction (8+).
    Intense,
}

impl std::fmt::Display for AttractionBand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            AttractionBand::None => "None",
            AttractionBand::Curious => "Curious",
            AttractionBand::Interested => "Interested",
            AttractionBand::Strong => "Strong",
            AttractionBand::Intense => "Intense",
        };
        write!(f, "{}", s)
    }
}

/// Resentment axis band (hostility tier).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResentmentBand {
    /// No resentment (0 or below).
    None,
    /// Mild annoyance (0 to 3).
    Irritated,
    /// Active dislike (3 to 6).
    Resentful,
    /// Hostile (6 to 8).
    Hostile,
    /// Seeking revenge (8+).
    Vindictive,
}

impl std::fmt::Display for ResentmentBand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ResentmentBand::None => "None",
            ResentmentBand::Irritated => "Irritated",
            ResentmentBand::Resentful => "Resentful",
            ResentmentBand::Hostile => "Hostile",
            ResentmentBand::Vindictive => "Vindictive",
        };
        write!(f, "{}", s)
    }
}

/// A pending change to a relationship axis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipDelta {
    /// NPC initiating the change.
    pub actor_id: u64,
    /// NPC receiving the change.
    pub target_id: u64,
    /// Which axis to modify.
    pub axis: RelationshipAxis,
    /// Amount to change (+/-).
    pub delta: f32,
    /// Optional source event/storylet.
    pub source: Option<String>,
}

/// Apply a batch of relationship deltas to vectors.
pub fn apply_relationship_deltas<'a, V>(get_rel: &mut V, deltas: &[RelationshipDelta])
where
    V: FnMut(u64, u64) -> &'a mut RelationshipVector,
{
    for d in deltas {
        let vec = get_rel(d.actor_id, d.target_id);
        vec.apply_delta(d.axis, d.delta);
    }
}
