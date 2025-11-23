use serde::{Deserialize, Serialize};

fn clamp_axis(value: f32) -> f32 {
    value.clamp(-10.0, 10.0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationshipAxis {
    Affection,
    Trust,
    Attraction,
    Familiarity,
    Resentment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationshipRole {
    Stranger,
    Acquaintance,
    Friend,
    Rival,
    Ally,
    Romance,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RelationshipVector {
    pub affection: f32,
    pub trust: f32,
    pub attraction: f32,
    pub familiarity: f32,
    pub resentment: f32,
}

impl RelationshipVector {
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
        let clamped = clamp_axis(value);
        match axis {
            RelationshipAxis::Affection => self.affection = clamped,
            RelationshipAxis::Trust => self.trust = clamped,
            RelationshipAxis::Attraction => self.attraction = clamped,
            RelationshipAxis::Familiarity => self.familiarity = clamped,
            RelationshipAxis::Resentment => self.resentment = clamped,
        }
    }

    pub fn apply_delta(&mut self, axis: RelationshipAxis, delta: f32) {
        self.set(axis, self.get(axis) + delta);
    }

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AffectionBand {
    Stranger,
    Acquaintance,
    Friendly,
    Close,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrustBand {
    Unknown,
    Wary,
    Neutral,
    Trusted,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttractionBand {
    None,
    Curious,
    Interested,
    Strong,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResentmentBand {
    None,
    Irritated,
    Resentful,
    Hostile,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipDelta {
    pub actor_id: u64,
    pub target_id: u64,
    pub axis: RelationshipAxis,
    pub delta: f32,
    pub source: Option<String>,
}

pub fn apply_relationship_deltas<'a, V>(
    get_rel: &mut V,
    deltas: &[RelationshipDelta],
) where
    V: FnMut(u64, u64) -> &'a mut RelationshipVector,
{
    for d in deltas {
        let vec = get_rel(d.actor_id, d.target_id);
        vec.apply_delta(d.axis, d.delta);
    }
}
