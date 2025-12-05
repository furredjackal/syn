//! NPC behavior and needs model: centralized enums and pure helpers.

use serde::{Deserialize, Serialize};

use crate::npc::PersonalityVector;
use crate::relationship_model::RelationshipVector;
use crate::{NpcId, Stats};

/// High-level needs used by NPC utility logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NeedKind {
    /// Connection, validation
    Social,
    /// Safety, stability, financial
    Security,
    /// Status, praise, being seen
    Recognition,
    /// Physical/mental ease
    Comfort,
    /// Control, independence
    Autonomy,
}

/// Behavior categories NPCs can intend to pursue.
/// These are not specific actions, just high-level intents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BehaviorKind {
    /// Seeks connection with player or NPC (chat, hang, support).
    SeekSocial,

    /// Seeks safety or resources (work, routine, withdrawing to recharge).
    SeekSecurity,

    /// Seeks praise/status (show off, confront, perform).
    SeekRecognition,

    /// Seeks comfort (rest, self-soothe, avoidance).
    SeekComfort,

    /// Seeks control/change (push boundaries, start conflict, assert self).
    SeekAutonomy,

    /// Passive/no strong intent this tick.
    Idle,
}

/// Intensity for each need.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct NeedVector {
    /// Desire for connection and validation.
    pub social: f32,
    /// Desire for safety and financial stability.
    pub security: f32,
    /// Desire for status and praise.
    pub recognition: f32,
    /// Desire for physical/mental ease.
    pub comfort: f32,
    /// Desire for control and independence.
    pub autonomy: f32,
}

impl NeedVector {
    /// Get the intensity of a specific need.
    pub fn get(&self, kind: NeedKind) -> f32 {
        match kind {
            NeedKind::Social => self.social,
            NeedKind::Security => self.security,
            NeedKind::Recognition => self.recognition,
            NeedKind::Comfort => self.comfort,
            NeedKind::Autonomy => self.autonomy,
        }
    }

    /// Set the intensity of a specific need (clamped to 0.0-1.5).
    pub fn set(&mut self, kind: NeedKind, value: f32) {
        let v = value.clamp(0.0, 1.5); // allow some overshoot, but bounded
        match kind {
            NeedKind::Social => self.social = v,
            NeedKind::Security => self.security = v,
            NeedKind::Recognition => self.recognition = v,
            NeedKind::Comfort => self.comfort = v,
            NeedKind::Autonomy => self.autonomy = v,
        }
    }
}

/// A single behavior choice with a utility score used for comparison.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorIntent {
    /// High-level behavior category.
    pub kind: BehaviorKind,
    /// Utility score for this behavior (higher = more desirable).
    pub utility: f32,
}

/// Short-lived snapshot: what this NPC is currently leaning toward.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorSnapshot {
    /// Current need intensities.
    pub needs: NeedVector,
    /// The chosen behavioral intent.
    pub chosen_intent: BehaviorIntent,
    /// True if target is the player.
    #[serde(default)]
    pub target_player: bool,
    /// Target NPC ID if not player.
    #[serde(default)]
    pub target_npc_id: Option<NpcId>,
}

/// Pure function: derive a NeedVector for an NPC based on:
/// - Their stats
/// - Their personality
/// - Their relationship with the player (if any)
pub fn compute_needs_from_state(
    stats: &Stats,
    personality: &PersonalityVector,
    rel_to_player: Option<&RelationshipVector>,
) -> NeedVector {
    // Normalize stats to 0..1 ranges where possible.
    let mood_norm = ((stats.mood + 10.0) / 20.0).clamp(0.0, 1.0); // -10..10
    let health_norm = (stats.health / 10.0).clamp(0.0, 1.0); // 0..10
    let wealth_norm = (stats.wealth / 10.0).clamp(0.0, 1.0); // 0..10
    let reputation_norm = ((stats.reputation + 100.0) / 200.0).clamp(0.0, 1.0);

    let warmth = personality.warmth;
    let dominance = personality.dominance;
    let volatility = personality.volatility;
    let conscientious = personality.conscientiousness;
    let openness = personality.openness;

    // Base social need: lower mood, positive warmth, low connection → higher social need.
    let mut social_need = 0.0;
    social_need += (1.0 - mood_norm) * 0.5;
    social_need += (warmth.max(0.0)) * 0.3; // warm people crave contact
    if let Some(rel) = rel_to_player {
        let aff = rel.affection_band();
        use crate::relationship_model::AffectionBand::*;
        let player_aff = match aff {
            Stranger => 0.0,
            Acquaintance => 0.2,
            Friendly => 0.5,
            Close => 0.8,
            Devoted => 1.0,
        };
        // If they care a lot about the player and mood is low, social need spikes.
        social_need += (1.0 - mood_norm) * player_aff * 0.5;
    }

    // Security need: low wealth / bad health / volatility.
    let mut security_need = 0.0;
    security_need += (1.0 - health_norm) * 0.5;
    security_need += (1.0 - wealth_norm) * 0.5;
    security_need += (volatility.max(0.0)) * 0.2;

    // Recognition need: high dominance, high openness, low reputation.
    let mut recognition_need = 0.0;
    recognition_need += (dominance.max(0.0)) * 0.5;
    recognition_need += openness * 0.3;
    recognition_need += (1.0 - reputation_norm) * 0.4;

    // Comfort need: low mood + low health + low conscientiousness.
    let mut comfort_need = 0.0;
    comfort_need += (1.0 - mood_norm) * 0.6;
    comfort_need += (1.0 - health_norm) * 0.4;
    comfort_need += (1.0 - conscientious) * 0.3;

    // Autonomy need: high dominance, high volatility.
    let mut autonomy_need = 0.0;
    autonomy_need += (dominance.max(0.0)) * 0.5;
    autonomy_need += (volatility.max(0.0)) * 0.5;

    let mut out = NeedVector::default();
    out.set(NeedKind::Social, social_need);
    out.set(NeedKind::Security, security_need);
    out.set(NeedKind::Recognition, recognition_need);
    out.set(NeedKind::Comfort, comfort_need);
    out.set(NeedKind::Autonomy, autonomy_need);

    out
}

/// Pure function: compute candidate BehaviorIntent values from needs + personality.
/// The `heat_multiplier` (typically 0.5..2.0) biases risky behaviors at high heat.
pub fn compute_behavior_intents(
    needs: &NeedVector,
    personality: &PersonalityVector,
    heat_multiplier: f32,
) -> Vec<BehaviorIntent> {
    let mut intents = Vec::new();

    let warmth = personality.warmth;
    let dominance = personality.dominance;
    let volatility = personality.volatility;
    let conscientious = personality.conscientiousness;

    // Heat affects risk-taking: high heat → NPCs pursue riskier behaviors
    // - SeekAutonomy and SeekRecognition are boosted (confrontation, showing off)
    // - SeekSecurity and SeekComfort are dampened (survival instincts override at extremes)
    let risk_boost = heat_multiplier.clamp(0.5, 2.5);
    let safety_dampen = (2.0 - heat_multiplier).clamp(0.5, 1.5);

    // SeekSocial: high social need, warm, mood-driven.
    // Slightly boosted at high heat (people reach out in crises)
    let social_u = needs.social * (1.0 + warmth.max(0.0) * 0.5) * (0.8 + risk_boost * 0.2);
    intents.push(BehaviorIntent {
        kind: BehaviorKind::SeekSocial,
        utility: social_u,
    });

    // SeekSecurity: security need boosted by conscientiousness.
    // Dampened at high heat (panic overrides rational safety-seeking)
    let sec_u = needs.security * (1.0 + conscientious * 0.5) * safety_dampen;
    intents.push(BehaviorIntent {
        kind: BehaviorKind::SeekSecurity,
        utility: sec_u,
    });

    // SeekRecognition: recognition need boosted by dominance.
    // Strongly boosted at high heat (people act out, show off, confront)
    let rec_u = needs.recognition * (1.0 + dominance.max(0.0) * 0.7) * risk_boost;
    intents.push(BehaviorIntent {
        kind: BehaviorKind::SeekRecognition,
        utility: rec_u,
    });

    // SeekComfort: comfort need boosted by low conscientiousness (avoidant).
    // Dampened at high heat (adrenaline overrides comfort-seeking)
    let comfort_u = needs.comfort * (1.0 + (1.0 - conscientious) * 0.5) * safety_dampen;
    intents.push(BehaviorIntent {
        kind: BehaviorKind::SeekComfort,
        utility: comfort_u,
    });

    // SeekAutonomy: autonomy need boosted by dominance + volatility.
    // Strongly boosted at high heat (NPCs assert themselves, start conflicts)
    let auto_u = needs.autonomy * (1.0 + dominance.max(0.0) * 0.5 + volatility.max(0.0) * 0.5) * risk_boost;
    intents.push(BehaviorIntent {
        kind: BehaviorKind::SeekAutonomy,
        utility: auto_u,
    });

    // Idle: baseline, dampened at high heat (less passive behavior)
    let idle_u = 0.1 * safety_dampen;
    intents.push(BehaviorIntent {
        kind: BehaviorKind::Idle,
        utility: idle_u,
    });

    intents
}

/// Choose the best intent by utility.
pub fn choose_best_intent(intents: &[BehaviorIntent]) -> BehaviorIntent {
    intents
        .iter()
        .cloned()
        .max_by(|a, b| {
            a.utility
                .partial_cmp(&b.utility)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or(BehaviorIntent {
            kind: BehaviorKind::Idle,
            utility: 0.0,
        })
}
