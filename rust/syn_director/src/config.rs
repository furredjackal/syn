//! Director configuration module.
//!
//! This module contains all immutable configuration parameters for the Event Director.
//! Separating config from state allows:
//! - Easy tuning without touching runtime state
//! - Clear documentation of all tuning knobs
//! - Potential for config hot-reloading in development

use serde::{Deserialize, Serialize};

/// Master configuration for the Event Director.
///
/// Controls all aspects of storylet selection, pacing, queuing, and persistence.
/// All fields have sensible defaults but can be tuned per-game or per-mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectorConfig {
    /// Target narrative heat level (0.0 = calm, 100.0 = crisis).
    /// The director will try to guide heat toward this target over time.
    pub base_heat_target: f32,
    
    /// Maximum number of events that can be queued.
    pub max_queue_size: usize,
    
    /// Minimum ticks between any two storylet events.
    /// Prevents event spam and gives player breathing room.
    pub min_ticks_between_events: u64,
    
    /// Pacing engine configuration.
    pub pacing: PacingConfig,
    
    /// Scoring engine configuration.
    pub scoring: ScoringConfig,
    
    /// Event queue configuration.
    pub queue: QueueConfig,
    
    /// Pressure system configuration.
    pub pressure: PressureConfig,
    
    /// Persistence configuration.
    pub persistence: PersistenceConfig,
    
    /// Variety/repetition prevention configuration.
    pub variety: VarietyConfig,
    
    /// Milestone system configuration.
    pub milestone: MilestoneConfig,
}

impl DirectorConfig {
    /// Create a new DirectorConfig with sensible gameplay defaults.
    pub fn new() -> Self {
        DirectorConfig {
            base_heat_target: 30.0,
            max_queue_size: 10,
            min_ticks_between_events: 4, // ~4 hours game time
            pacing: PacingConfig::default(),
            scoring: ScoringConfig::default(),
            queue: QueueConfig::default(),
            pressure: PressureConfig::default(),
            persistence: PersistenceConfig::default(),
            variety: VarietyConfig::default(),
            milestone: MilestoneConfig::default(),
        }
    }

    /// Create a config optimized for testing (faster pacing, no delays).
    pub fn for_testing() -> Self {
        DirectorConfig {
            base_heat_target: 50.0,
            max_queue_size: 5,
            min_ticks_between_events: 0,
            pacing: PacingConfig::for_testing(),
            scoring: ScoringConfig::default(),
            queue: QueueConfig::default(),
            pressure: PressureConfig::default(),
            persistence: PersistenceConfig::default(),
            variety: VarietyConfig::for_testing(),
            milestone: MilestoneConfig::default(),
        }
    }
}

impl Default for DirectorConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for the pacing engine.
///
/// Controls how the director modulates narrative intensity over time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacingConfig {
    /// Minimum heat floor (heat won't decay below this).
    pub min_heat: f32,
    
    /// Maximum heat ceiling (heat won't rise above this).
    pub max_heat: f32,
    
    /// How much heat decays per tick (absolute value).
    pub heat_decay_per_tick: f32,
    
    /// Multiplier for event heat contribution.
    /// story_heat * heat_increase_per_event_factor = heat added.
    pub heat_increase_per_event_factor: f32,
    
    /// Thresholds for phase transitions.
    pub phase_thresholds: PhaseThresholds,
    
    /// Minimum ticks to stay in each narrative phase before transitioning.
    pub min_phase_duration: u64,
    
    /// Bonus weight for storylets matching current phase.
    pub phase_match_bonus: f32,
}

/// Thresholds for transitioning between narrative phases.
///
/// Phase transitions follow a state machine:
/// - LowKey → Rising: when heat rises above `lowkey_to_rising`
/// - Rising → Peak: when heat rises above `rising_to_peak`
/// - Peak → Fallout: when heat drops below `peak_to_fallout`
/// - Fallout → Recovery: when heat drops below `fallout_to_recovery`
/// - Recovery → LowKey: when heat drops below `recovery_to_lowkey`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseThresholds {
    /// Heat level to transition from LowKey to Rising.
    pub lowkey_to_rising: f32,
    
    /// Heat level to transition from Rising to Peak.
    pub rising_to_peak: f32,
    
    /// Heat level to transition from Peak to Fallout (on descent).
    pub peak_to_fallout: f32,
    
    /// Heat level to transition from Fallout to Recovery.
    pub fallout_to_recovery: f32,
    
    /// Heat level to transition from Recovery to LowKey.
    pub recovery_to_lowkey: f32,
}

impl Default for PhaseThresholds {
    fn default() -> Self {
        PhaseThresholds {
            lowkey_to_rising: 25.0,
            rising_to_peak: 60.0,
            peak_to_fallout: 45.0,
            fallout_to_recovery: 25.0,
            recovery_to_lowkey: 15.0,
        }
    }
}

impl PacingConfig {
    /// Create testing config with faster phase transitions.
    pub fn for_testing() -> Self {
        PacingConfig {
            min_phase_duration: 1,
            heat_decay_per_tick: 1.0, // Faster decay for testing
            ..Self::default()
        }
    }
    
    /// Get the rising threshold (convenience accessor).
    pub fn rising_threshold(&self) -> f32 {
        self.phase_thresholds.lowkey_to_rising
    }
    
    /// Get the peak threshold (convenience accessor).
    pub fn peak_threshold(&self) -> f32 {
        self.phase_thresholds.rising_to_peak
    }
}

impl Default for PacingConfig {
    fn default() -> Self {
        PacingConfig {
            min_heat: 0.0,
            max_heat: 100.0,
            heat_decay_per_tick: 0.5,
            heat_increase_per_event_factor: 1.0,
            phase_thresholds: PhaseThresholds::default(),
            min_phase_duration: 8, // ~8 hours / half a day
            phase_match_bonus: 1.5,
        }
    }
}

/// Configuration for the scoring engine.
///
/// Controls how storylets are weighted for selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringConfig {
    /// Base weight multiplier (applied to all storylets).
    pub base_weight_multiplier: f32,
    
    /// Weight bonus for storylets matching active pressures.
    pub pressure_match_bonus: f32,
    
    /// Weight bonus for storylets matching player personality.
    pub personality_match_bonus: f32,
    
    /// Weight penalty for recently fired storylets.
    pub recency_penalty: f32,
    
    /// How many ticks before recency penalty fully decays.
    pub recency_decay_ticks: u64,
    
    /// Weight bonus for storylets in underrepresented domains.
    pub variety_bonus: f32,
    
    /// Minimum weight below which storylets are excluded.
    pub min_viable_weight: f32,
}

impl Default for ScoringConfig {
    fn default() -> Self {
        ScoringConfig {
            base_weight_multiplier: 1.0,
            pressure_match_bonus: 2.0,
            personality_match_bonus: 1.3,
            recency_penalty: 0.5,
            recency_decay_ticks: 48, // ~2 days game time
            variety_bonus: 1.2,
            min_viable_weight: 0.1,
        }
    }
}

/// Configuration for the event queue.
///
/// Controls how events are scheduled and prioritized.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueConfig {
    /// Maximum number of events in the queue.
    /// When exceeded, lowest-priority farthest-future events are dropped.
    pub max_size: usize,
    
    /// Whether to allow follow-up events to skip the queue.
    pub follow_ups_skip_queue: bool,
    
    /// Maximum delay (in ticks) for scheduled events.
    pub max_schedule_delay: u64,
    
    /// Whether high-priority events can preempt queued events.
    pub allow_preemption: bool,
    
    /// Priority threshold for preemption.
    pub preemption_threshold: f32,
}

impl Default for QueueConfig {
    fn default() -> Self {
        QueueConfig {
            max_size: 50,
            follow_ups_skip_queue: true,
            max_schedule_delay: 168, // ~1 week game time
            allow_preemption: true,
            preemption_threshold: 80.0,
        }
    }
}

impl QueueConfig {
    /// Get the maximum queue size.
    pub fn max_queue_size(&self) -> usize {
        self.max_size
    }
}

/// Configuration for the pressure system.
///
/// Controls how relationship and life pressures affect storylet selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PressureConfig {
    /// How quickly pressure decays if not addressed (per tick).
    pub pressure_decay_rate: f32,
    
    /// Pressure level that triggers urgent storylets (0.0-1.0 scale).
    pub urgency_threshold: f32,
    
    /// Pressure level that triggers crisis storylets (0.0-1.0 scale).
    pub crisis_threshold: f32,
    
    /// Maximum pressure before automatic resolution.
    pub max_pressure: f32,
    
    /// Weight bonus for storylets that address active pressure.
    pub addressing_bonus: f32,
    
    /// Base severity increase per tick for active pressures.
    pub base_severity_increase: f32,
    
    /// Severity increase per tick when a deadline is overdue.
    pub overdue_severity_increase: f32,
    
    /// Multiplier for deadline urgency factor (0-1 based on time remaining).
    pub deadline_urgency_factor: f32,
    
    /// How many ticks to keep resolved pressures before cleanup.
    pub resolved_cleanup_ticks: u64,
}

impl Default for PressureConfig {
    fn default() -> Self {
        PressureConfig {
            pressure_decay_rate: 0.1,
            urgency_threshold: 0.5,  // 50% severity
            crisis_threshold: 0.8,   // 80% severity
            max_pressure: 100.0,
            addressing_bonus: 3.0,
            base_severity_increase: 0.01,      // 1% per tick
            overdue_severity_increase: 0.05,   // 5% per tick when overdue
            deadline_urgency_factor: 0.5,      // Scales with time remaining
            resolved_cleanup_ticks: 24,        // ~1 game day before cleanup
        }
    }
}

/// Configuration for the milestone system.
///
/// Controls how narrative arcs and milestones affect storylet selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneConfig {
    /// Progress increment for matching storylet tags.
    pub progress_per_event: f32,
    
    /// Weight bonus multiplier for storylets advancing hot milestones.
    pub hot_milestone_bonus: f32,
    
    /// Progress threshold to consider a milestone "hot" (mid-arc).
    pub hot_threshold: f32,
    
    /// Progress threshold to trigger climax storylet scheduling.
    pub climax_threshold: f32,
    
    /// Minimum ticks before climax can be scheduled.
    pub min_ticks_before_climax: u64,
    
    /// Progress increment when storylet domain matches milestone domain.
    pub domain_match_progress: f32,
    
    /// Progress increment when storylet tags match milestone advancing tags.
    pub tag_match_progress: f32,
    
    /// Maximum total milestone bonus for a single storylet.
    pub max_milestone_bonus: f32,
}

impl Default for MilestoneConfig {
    fn default() -> Self {
        MilestoneConfig {
            progress_per_event: 0.1,
            hot_milestone_bonus: 2.0,
            hot_threshold: 0.2,
            climax_threshold: 0.8,
            min_ticks_before_climax: 48, // ~2 game days
            domain_match_progress: 0.05,  // 5% progress on domain match
            tag_match_progress: 0.1,      // 10% progress per tag match
            max_milestone_bonus: 3.0,     // Max 300% bonus
        }
    }
}

/// Configuration for director state persistence.
///
/// Controls how and when director state is saved/loaded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceConfig {
    /// Whether to auto-save director state periodically.
    pub auto_save_enabled: bool,
    
    /// How often to auto-save (in ticks).
    pub auto_save_interval: u64,
    
    /// Whether to persist cooldown state.
    pub persist_cooldowns: bool,
    
    /// Whether to persist the event queue.
    pub persist_queue: bool,
    
    /// Whether to persist last-fired tracking.
    pub persist_last_fired: bool,
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        PersistenceConfig {
            auto_save_enabled: true,
            auto_save_interval: 24, // ~1 day game time
            persist_cooldowns: true,
            persist_queue: true,
            persist_last_fired: true,
        }
    }
}

/// Configuration for variety/repetition prevention.
///
/// Controls how the director ensures narrative variety.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VarietyConfig {
    /// Minimum ticks before same storylet can fire again.
    pub min_storylet_repeat_interval: u64,
    
    /// Minimum ticks before same domain can fire again.
    pub min_domain_repeat_interval: u64,
    
    /// Minimum ticks before same tag can fire again.
    pub min_tag_repeat_interval: u64,
    
    /// Maximum consecutive events in same domain.
    pub max_consecutive_same_domain: u32,
    
    /// Weight penalty for same-domain back-to-back.
    pub same_domain_penalty: f32,
}

impl VarietyConfig {
    /// Create testing config with no variety restrictions.
    pub fn for_testing() -> Self {
        VarietyConfig {
            min_storylet_repeat_interval: 0,
            min_domain_repeat_interval: 0,
            min_tag_repeat_interval: 0,
            max_consecutive_same_domain: 100,
            same_domain_penalty: 1.0, // no penalty
        }
    }
}

impl Default for VarietyConfig {
    fn default() -> Self {
        VarietyConfig {
            min_storylet_repeat_interval: 24,  // ~1 day
            min_domain_repeat_interval: 4,     // ~4 hours
            min_tag_repeat_interval: 2,        // ~2 hours
            max_consecutive_same_domain: 3,
            same_domain_penalty: 0.5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_director_config_default() {
        let config = DirectorConfig::default();
        assert_eq!(config.base_heat_target, 30.0);
        assert_eq!(config.max_queue_size, 10);
        assert!(config.min_ticks_between_events > 0);
    }

    #[test]
    fn test_director_config_for_testing() {
        let config = DirectorConfig::for_testing();
        assert_eq!(config.min_ticks_between_events, 0);
        assert_eq!(config.pacing.min_phase_duration, 1);
    }

    #[test]
    fn test_pacing_config_thresholds_order() {
        let pacing = PacingConfig::default();
        let thresholds = &pacing.phase_thresholds;
        // Thresholds should be in ascending order for phase progression
        assert!(thresholds.recovery_to_lowkey < thresholds.lowkey_to_rising);
        assert!(thresholds.lowkey_to_rising < thresholds.rising_to_peak);
    }
    
    #[test]
    fn test_phase_thresholds_default_values() {
        let thresholds = PhaseThresholds::default();
        assert_eq!(thresholds.lowkey_to_rising, 25.0);
        assert_eq!(thresholds.rising_to_peak, 60.0);
        assert_eq!(thresholds.peak_to_fallout, 45.0);
        assert_eq!(thresholds.fallout_to_recovery, 25.0);
        assert_eq!(thresholds.recovery_to_lowkey, 15.0);
    }

    #[test]
    fn test_scoring_config_bonuses_positive() {
        let scoring = ScoringConfig::default();
        assert!(scoring.pressure_match_bonus > 1.0);
        assert!(scoring.personality_match_bonus > 1.0);
        assert!(scoring.variety_bonus > 1.0);
    }

    #[test]
    fn test_variety_config_testing_mode() {
        let variety = VarietyConfig::for_testing();
        assert_eq!(variety.min_storylet_repeat_interval, 0);
        assert_eq!(variety.same_domain_penalty, 1.0);
    }
}
