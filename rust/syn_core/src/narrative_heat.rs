use serde::{Deserialize, Serialize};

use crate::relationship_model::RelationshipVector;
use crate::life_stage::LifeStageStatProfile;
use crate::Stats;

/// Scalar narrative heat value, typically clamped within [0, 100].
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct NarrativeHeat(pub f32);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum NarrativeHeatBand {
    Low,
    Medium,
    High,
    Critical,
}

impl NarrativeHeat {
    pub fn new(value: f32) -> Self {
        let mut h = Self(value);
        h.clamp();
        h
    }

    pub fn value(&self) -> f32 {
        self.0
    }

    pub fn set(&mut self, value: f32) {
        self.0 = value;
        self.clamp();
    }

    pub fn add(&mut self, delta: f32) {
        self.0 += delta;
        self.clamp();
    }

    fn clamp(&mut self) {
        if self.0 < 0.0 {
            self.0 = 0.0;
        } else if self.0 > 100.0 {
            self.0 = 100.0;
        }
    }

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
    pub player_stats: &'a Stats,
    pub relationships: &'a [(&'a (u64, u64), &'a RelationshipVector)],
    pub has_recent_trauma: bool,
    pub has_recent_betrayal: bool,
    pub has_recent_major_win: bool,
    pub stat_profile: Option<&'a LifeStageStatProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeHeatConfig {
    pub base_decay_toward: f32,
    pub decay_per_tick: f32,
    pub extreme_stat_weight: f32,
    pub resentment_weight: f32,
    pub economic_stress_weight: f32,
    pub trauma_weight: f32,
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

    delta += config.extreme_stat_weight * ((mood_extreme * mood_weight) + (health_low * health_weight));
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
