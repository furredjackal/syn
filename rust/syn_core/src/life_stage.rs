//! Life Stage Configuration
//!
//! Defines stat weights, visibility settings, and narrative heat configs per life stage.
//! Each stage (Child, Teen, YoungAdult, Adult, Elder, Digital) has distinct gameplay focus.

use serde::{Deserialize, Serialize};

use crate::narrative_heat::NarrativeHeatConfig;
use crate::LifeStage;

/// Which stats are "foregrounded" and how strongly they matter in this stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifeStageStatProfile {
    /// Importance of mood in this life stage.
    pub mood_weight: f32,
    /// Importance of health in this life stage.
    pub health_weight: f32,
    /// Importance of wealth in this life stage.
    pub wealth_weight: f32,
    /// Importance of charisma in this life stage.
    pub charisma_weight: f32,
    /// Importance of reputation in this life stage.
    pub reputation_weight: f32,
    /// Importance of wisdom in this life stage.
    pub wisdom_weight: f32,
}

/// Which stats are visible/emphasized in UI for this stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifeStageStatVisibility {
    /// Show wealth stat in UI.
    pub show_wealth: bool,
    /// Show reputation stat in UI.
    pub show_reputation: bool,
    /// Show wisdom stat in UI.
    pub show_wisdom: bool,
    /// Show karma stat in UI.
    pub show_karma: bool,
}

/// Config bundle for a given stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifeStageConfig {
    /// Stat weights for this life stage.
    pub stat_profile: LifeStageStatProfile,
    /// UI visibility settings for this life stage.
    pub visibility: LifeStageStatVisibility,
    /// Narrative heat configuration for this life stage.
    pub heat_config: NarrativeHeatConfig,
    /// Minimum age for this life stage.
    pub min_age: u32,
    /// Maximum age for this life stage.
    pub max_age: u32,
}

impl LifeStage {
    /// Get the configuration for this life stage.
    pub fn config(self) -> LifeStageConfig {
        match self {
            LifeStage::PreSim => LifeStageConfig {
                stat_profile: LifeStageStatProfile {
                    mood_weight: 2.0,
                    health_weight: 1.5,
                    wealth_weight: 0.0,
                    charisma_weight: 0.5,
                    reputation_weight: 0.0,
                    wisdom_weight: 0.0,
                },
                visibility: LifeStageStatVisibility {
                    show_wealth: false,
                    show_reputation: false,
                    show_wisdom: false,
                    show_karma: false,
                },
                heat_config: NarrativeHeatConfig {
                    base_decay_toward: 10.0,
                    decay_per_tick: 0.5,
                    extreme_stat_weight: 1.0,
                    resentment_weight: 0.5,
                    economic_stress_weight: 0.0,
                    trauma_weight: 2.0,
                    win_weight: 1.0,
                },
                min_age: 0,
                max_age: 5,
            },
            LifeStage::Child => LifeStageConfig {
                stat_profile: LifeStageStatProfile {
                    mood_weight: 2.0,
                    health_weight: 1.5,
                    wealth_weight: 0.2,
                    charisma_weight: 1.0,
                    reputation_weight: 0.5,
                    wisdom_weight: 0.1,
                },
                visibility: LifeStageStatVisibility {
                    show_wealth: false,
                    show_reputation: false,
                    show_wisdom: false,
                    show_karma: false,
                },
                heat_config: NarrativeHeatConfig {
                    base_decay_toward: 15.0,
                    decay_per_tick: 0.6,
                    extreme_stat_weight: 2.5,
                    resentment_weight: 1.0,
                    economic_stress_weight: 0.4,
                    trauma_weight: 3.0,
                    win_weight: 2.0,
                },
                min_age: 6,
                max_age: 12,
            },
            LifeStage::Teen => LifeStageConfig {
                stat_profile: LifeStageStatProfile {
                    mood_weight: 3.0,
                    health_weight: 1.0,
                    wealth_weight: 0.5,
                    charisma_weight: 2.0,
                    reputation_weight: 2.0,
                    wisdom_weight: 0.5,
                },
                visibility: LifeStageStatVisibility {
                    show_wealth: false,
                    show_reputation: true,
                    show_wisdom: false,
                    show_karma: true,
                },
                heat_config: NarrativeHeatConfig {
                    base_decay_toward: 20.0,
                    decay_per_tick: 0.4,
                    extreme_stat_weight: 3.0,
                    resentment_weight: 2.5,
                    economic_stress_weight: 0.7,
                    trauma_weight: 4.0,
                    win_weight: 2.5,
                },
                min_age: 13,
                max_age: 18,
            },
            LifeStage::YoungAdult => LifeStageConfig {
                stat_profile: LifeStageStatProfile {
                    mood_weight: 2.5,
                    health_weight: 2.0,
                    wealth_weight: 2.0,
                    charisma_weight: 1.5,
                    reputation_weight: 1.5,
                    wisdom_weight: 1.0,
                },
                visibility: LifeStageStatVisibility {
                    show_wealth: true,
                    show_reputation: true,
                    show_wisdom: true,
                    show_karma: true,
                },
                heat_config: NarrativeHeatConfig {
                    base_decay_toward: 25.0,
                    decay_per_tick: 0.5,
                    extreme_stat_weight: 2.5,
                    resentment_weight: 2.0,
                    economic_stress_weight: 2.5,
                    trauma_weight: 3.5,
                    win_weight: 3.0,
                },
                min_age: 19,
                max_age: 30,
            },
            LifeStage::Adult => LifeStageConfig {
                stat_profile: LifeStageStatProfile {
                    mood_weight: 2.0,
                    health_weight: 2.5,
                    wealth_weight: 2.5,
                    charisma_weight: 1.0,
                    reputation_weight: 2.5,
                    wisdom_weight: 2.5,
                },
                visibility: LifeStageStatVisibility {
                    show_wealth: true,
                    show_reputation: true,
                    show_wisdom: true,
                    show_karma: true,
                },
                heat_config: NarrativeHeatConfig {
                    base_decay_toward: 30.0,
                    decay_per_tick: 0.6,
                    extreme_stat_weight: 2.0,
                    resentment_weight: 2.0,
                    economic_stress_weight: 2.0,
                    trauma_weight: 3.0,
                    win_weight: 2.0,
                },
                min_age: 31,
                max_age: 60,
            },
            LifeStage::Elder => LifeStageConfig {
                stat_profile: LifeStageStatProfile {
                    mood_weight: 1.5,
                    health_weight: 3.0,
                    wealth_weight: 1.5,
                    charisma_weight: 0.8,
                    reputation_weight: 2.0,
                    wisdom_weight: 3.0,
                },
                visibility: LifeStageStatVisibility {
                    show_wealth: true,
                    show_reputation: true,
                    show_wisdom: true,
                    show_karma: true,
                },
                heat_config: NarrativeHeatConfig {
                    base_decay_toward: 20.0,
                    decay_per_tick: 0.7,
                    extreme_stat_weight: 1.5,
                    resentment_weight: 1.5,
                    economic_stress_weight: 1.0,
                    trauma_weight: 2.0,
                    win_weight: 1.5,
                },
                min_age: 61,
                max_age: 90,
            },
            LifeStage::Digital => LifeStageConfig {
                stat_profile: LifeStageStatProfile {
                    mood_weight: 1.0,
                    health_weight: 0.0,
                    wealth_weight: 1.0,
                    charisma_weight: 1.0,
                    reputation_weight: 3.0,
                    wisdom_weight: 3.0,
                },
                visibility: LifeStageStatVisibility {
                    show_wealth: true,
                    show_reputation: true,
                    show_wisdom: true,
                    show_karma: true,
                },
                heat_config: NarrativeHeatConfig {
                    base_decay_toward: 15.0,
                    decay_per_tick: 0.8,
                    extreme_stat_weight: 1.0,
                    resentment_weight: 1.0,
                    economic_stress_weight: 0.5,
                    trauma_weight: 1.0,
                    win_weight: 2.0,
                },
                min_age: 91,
                max_age: 200,
            },
        }
    }

    /// Determine the life stage for a given age.
    pub fn stage_for_age(age: u32) -> LifeStage {
        for stage in [
            LifeStage::PreSim,
            LifeStage::Child,
            LifeStage::Teen,
            LifeStage::YoungAdult,
            LifeStage::Adult,
            LifeStage::Elder,
            LifeStage::Digital,
        ] {
            let cfg = stage.config();
            if age >= cfg.min_age && age <= cfg.max_age {
                return stage;
            }
        }
        LifeStage::YoungAdult
    }
}
