use serde::{Deserialize, Serialize};

use crate::narrative_heat::NarrativeHeatConfig;
use crate::LifeStage;

/// Which stats are “foregrounded” and how strongly they matter in this stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifeStageStatProfile {
    pub mood_weight: f32,
    pub health_weight: f32,
    pub wealth_weight: f32,
    pub charisma_weight: f32,
    pub reputation_weight: f32,
    pub wisdom_weight: f32,
}

/// Which stats are visible/emphasized in UI for this stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifeStageStatVisibility {
    pub show_wealth: bool,
    pub show_reputation: bool,
    pub show_wisdom: bool,
    pub show_karma: bool,
}

/// Config bundle for a given stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifeStageConfig {
    pub stat_profile: LifeStageStatProfile,
    pub visibility: LifeStageStatVisibility,
    pub heat_config: NarrativeHeatConfig,
    pub min_age: u32,
    pub max_age: u32,
}

impl LifeStage {
    pub fn config(self) -> LifeStageConfig {
        match self {
            LifeStage::PreSim | LifeStage::Child => LifeStageConfig {
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
