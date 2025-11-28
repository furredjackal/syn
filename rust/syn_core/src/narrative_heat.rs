//! Narrative Heat System
//!
//! Per GDD §6.2, narrative heat controls pacing:
//! - **Low** (0-25): Quiet moments, slice-of-life events
//! - **Medium** (25-50): Building tension, subplots develop
//! - **High** (50-80): Intense drama, major decisions
//! - **Critical** (80+): Climax moments, life-changing events
//!
//! Heat decays naturally over time to prevent permanent drama.

use serde::{Deserialize, Serialize};

use crate::life_stage::LifeStageStatProfile;
use crate::relationship_model::RelationshipVector;
use crate::Stats;

/// Scalar narrative heat value (0-100).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct NarrativeHeat(pub f32);

/// Narrative heat band for event eligibility and UI.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum NarrativeHeatBand {
    /// Quiet moments (0-25).
    Low,
    /// Building tension (25-50).
    Medium,
    /// Intense drama (50-80).
    High,
    /// Climax moments (80+).
    Critical,
}

impl NarrativeHeat {
    /// Create a new heat value (clamped to 0-100).
    pub fn new(value: f32) -> Self {
        let mut h = Self(value);
        h.clamp();
        h
    }

    /// Get the current heat value.
    pub fn value(&self) -> f32 {
        self.0
    }

    /// Set the heat value (clamped to 0-100).
    pub fn set(&mut self, value: f32) {
        self.0 = value;
        self.clamp();
    }

    /// Add a delta to the heat value.
    pub fn add(&mut self, delta: f32) {
        self.0 += delta;
        self.clamp();
    }

    /// Clamp to valid range.
    fn clamp(&mut self) {
        if self.0 < 0.0 {
            self.0 = 0.0;
        } else if self.0 > 100.0 {
            self.0 = 100.0;
        }
    }

    /// Get the current heat band.
    pub fn band(&self) -> NarrativeHeatBand {
        let v = self.0;
        if v < 25.0 {
            NarrativeHeatBand::Low
        } else if v < 50.0 {
            NarrativeHeatBand::Medium
        } else if v < 80.0 {
            NarrativeHeatBand::High
        } else {
            NarrativeHeatBand::Critical
        }
    }

    /// Per-tick decay toward a baseline (e.g. 10).
    /// This is used to prevent permanent “stuck at Critical”.
    pub fn decay_toward(&mut self, baseline: f32, amount: f32) {
        if amount <= 0.0 {
            return;
        }

        if self.0 > baseline {
            self.0 = (self.0 - amount).max(baseline);
        } else if self.0 < baseline {
            self.0 = (self.0 + amount).min(baseline);
        }
        self.clamp();
    }
}

impl Default for NarrativeHeat {
    fn default() -> Self {
        Self::new(10.0)
    }
}

impl std::fmt::Display for NarrativeHeatBand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            NarrativeHeatBand::Low => "Low",
            NarrativeHeatBand::Medium => "Medium",
            NarrativeHeatBand::High => "High",
            NarrativeHeatBand::Critical => "Critical",
        };
        write!(f, "{s}")
    }
}

/// Inputs used to compute heat deltas per tick.
pub struct NarrativeHeatInputs<'a> {
    /// Player's current stats.
    pub player_stats: &'a Stats,
    /// All relationship vectors (for resentment checks).
    pub relationships: &'a [(&'a (u64, u64), &'a RelationshipVector)],
    /// Whether player had recent trauma.
    pub has_recent_trauma: bool,
    /// Whether player was recently betrayed.
    pub has_recent_betrayal: bool,
    /// Whether player had a major win.
    pub has_recent_major_win: bool,
    /// Life stage stat profile for weight adjustments.
    pub stat_profile: Option<&'a LifeStageStatProfile>,
}

/// Configuration for heat computation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeHeatConfig {
    /// Target heat for decay.
    pub base_decay_toward: f32,
    /// Decay amount per tick.
    pub decay_per_tick: f32,
    /// Weight for extreme stat values.
    pub extreme_stat_weight: f32,
    /// Weight for resentment in relationships.
    pub resentment_weight: f32,
    /// Weight for economic stress.
    pub economic_stress_weight: f32,
    /// Weight for trauma events.
    pub trauma_weight: f32,
    /// Weight for major wins.
    pub win_weight: f32,
}

impl Default for NarrativeHeatConfig {
    fn default() -> Self {
        Self {
            base_decay_toward: 10.0,
            decay_per_tick: 0.5,
            extreme_stat_weight: 2.5,
            resentment_weight: 1.5,
            economic_stress_weight: 1.0,
            trauma_weight: 4.0,
            win_weight: 1.5,
        }
    }
}

/// Compute the heat delta for a single tick based on world state.
pub fn compute_heat_delta(inputs: &NarrativeHeatInputs<'_>, config: &NarrativeHeatConfig) -> f32 {
    let mut delta = 0.0;

    let profile = inputs.stat_profile;

    let weight = |w: fn(&LifeStageStatProfile) -> f32, default: f32| -> f32 {
        profile.map(w).unwrap_or(default)
    };

    let mood = inputs.player_stats.mood; // -10..10
    let health = inputs.player_stats.health; // 0..100
    let wealth = inputs.player_stats.wealth; // 0..100

    let mood_extreme = (mood.abs() / 10.0).clamp(0.0, 1.0);
    let health_low = ((50.0_f32 - health) / 50.0_f32).clamp(0.0, 1.0);
    let wealth_low = ((50.0_f32 - wealth) / 50.0_f32).clamp(0.0, 1.0);

    let mood_weight = weight(|p| p.mood_weight, 1.0);
    let health_weight = weight(|p| p.health_weight, 1.0);
    let wealth_weight = weight(|p| p.wealth_weight, 1.0);
    let reputation_weight = weight(|p| p.reputation_weight, 1.0);

    delta +=
        config.extreme_stat_weight * ((mood_extreme * mood_weight) + (health_low * health_weight));
    delta += config.economic_stress_weight * (wealth_low * wealth_weight);

    let mut avg_resentment = 0.0;
    let mut resent_count = 0.0;
    for (_key, rel) in inputs.relationships {
        if rel.resentment != 0.0 {
            avg_resentment += rel.resentment.abs();
            resent_count += 1.0;
        }
    }
    if resent_count > 0.0 {
        avg_resentment /= resent_count * 10.0;
        delta += config.resentment_weight * (avg_resentment.clamp(0.0, 1.0) * reputation_weight);
    }

    if inputs.has_recent_trauma {
        delta += config.trauma_weight;
    }
    if inputs.has_recent_betrayal {
        delta += config.trauma_weight * 0.7;
    }
    if inputs.has_recent_major_win {
        delta += config.win_weight;
    }

    delta
}
