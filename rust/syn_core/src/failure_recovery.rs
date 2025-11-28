//! Failure & Recovery Systems (GDD §11)
//!
//! Implements:
//! - Spiral events when Mood < -5 (anxiety, addiction, withdrawal)
//! - Recovery thresholds (Health + Social/Support)
//! - Cooldowns to prevent looping trauma
//!
//! A "spiral" is a cascading negative state triggered by low mood that can
//! worsen stats and trigger special storylets. Recovery requires meeting
//! health and social support thresholds.

use crate::rng::DeterministicRng;
use crate::stats::StatKind;
use crate::types::Stats;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The mood threshold below which spirals can trigger.
pub const SPIRAL_MOOD_THRESHOLD: f32 = -5.0;

/// Minimum ticks between spiral events of the same type (prevents looping trauma).
pub const DEFAULT_SPIRAL_COOLDOWN: u64 = 168; // 1 week in ticks (hours)

/// Types of spiral events that can occur during emotional crisis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpiralType {
    /// Generalized anxiety spiral - worry loops, avoidance behavior
    Anxiety,
    /// Depression spiral - isolation, anhedonia, fatigue
    Depression,
    /// Addiction relapse or new dependency formation
    Addiction,
    /// Social withdrawal - avoiding relationships
    Withdrawal,
    /// Anger spiral - lashing out, damaging relationships
    Anger,
    /// Panic spiral - acute distress episodes
    Panic,
    /// Self-destructive behavior spiral
    SelfDestructive,
}

impl SpiralType {
    /// Get all spiral types.
    pub fn all() -> &'static [SpiralType] {
        &[
            SpiralType::Anxiety,
            SpiralType::Depression,
            SpiralType::Addiction,
            SpiralType::Withdrawal,
            SpiralType::Anger,
            SpiralType::Panic,
            SpiralType::SelfDestructive,
        ]
    }

    /// Base probability of this spiral type occurring when mood < -5.
    /// Actual probability is modified by traits and circumstances.
    pub fn base_probability(&self) -> f32 {
        match self {
            SpiralType::Anxiety => 0.15,
            SpiralType::Depression => 0.12,
            SpiralType::Addiction => 0.08,
            SpiralType::Withdrawal => 0.10,
            SpiralType::Anger => 0.10,
            SpiralType::Panic => 0.05,
            SpiralType::SelfDestructive => 0.03,
        }
    }

    /// Stat effects while this spiral is active (per tick).
    pub fn stat_effects(&self) -> Vec<(StatKind, f32)> {
        match self {
            SpiralType::Anxiety => vec![
                (StatKind::Energy, -0.05),
                (StatKind::Health, -0.02),
            ],
            SpiralType::Depression => vec![
                (StatKind::Energy, -0.08),
                (StatKind::Mood, -0.03),
                (StatKind::Health, -0.01),
            ],
            SpiralType::Addiction => vec![
                (StatKind::Health, -0.05),
                (StatKind::Wealth, -0.10),
                (StatKind::Reputation, -0.02),
            ],
            SpiralType::Withdrawal => vec![
                (StatKind::Charisma, -0.05),
                (StatKind::Reputation, -0.01),
            ],
            SpiralType::Anger => vec![
                (StatKind::Reputation, -0.08),
                (StatKind::Charisma, -0.03),
            ],
            SpiralType::Panic => vec![
                (StatKind::Energy, -0.15),
                (StatKind::Health, -0.03),
            ],
            SpiralType::SelfDestructive => vec![
                (StatKind::Health, -0.10),
                (StatKind::Reputation, -0.05),
                (StatKind::Wealth, -0.05),
            ],
        }
    }

    /// Recovery thresholds: (min_health, min_social_support)
    /// Recovery requires meeting BOTH thresholds.
    pub fn recovery_thresholds(&self) -> (f32, f32) {
        match self {
            SpiralType::Anxiety => (40.0, 30.0),
            SpiralType::Depression => (50.0, 40.0),
            SpiralType::Addiction => (60.0, 50.0),
            SpiralType::Withdrawal => (30.0, 50.0),
            SpiralType::Anger => (40.0, 40.0),
            SpiralType::Panic => (50.0, 35.0),
            SpiralType::SelfDestructive => (70.0, 60.0),
        }
    }

    /// Minimum ticks this spiral must last before recovery is possible.
    pub fn minimum_duration(&self) -> u64 {
        match self {
            SpiralType::Anxiety => 24,        // 1 day
            SpiralType::Depression => 72,     // 3 days
            SpiralType::Addiction => 168,     // 1 week
            SpiralType::Withdrawal => 48,     // 2 days
            SpiralType::Anger => 12,          // 12 hours
            SpiralType::Panic => 6,           // 6 hours
            SpiralType::SelfDestructive => 96, // 4 days
        }
    }

    /// Cooldown before this spiral type can trigger again.
    pub fn cooldown(&self) -> u64 {
        match self {
            SpiralType::Anxiety => 168,       // 1 week
            SpiralType::Depression => 336,    // 2 weeks
            SpiralType::Addiction => 720,     // ~1 month
            SpiralType::Withdrawal => 168,    // 1 week
            SpiralType::Anger => 72,          // 3 days
            SpiralType::Panic => 48,          // 2 days
            SpiralType::SelfDestructive => 504, // 3 weeks
        }
    }

    /// Human-readable name for UI.
    pub fn display_name(&self) -> &'static str {
        match self {
            SpiralType::Anxiety => "Anxiety Spiral",
            SpiralType::Depression => "Depressive Episode",
            SpiralType::Addiction => "Addiction Relapse",
            SpiralType::Withdrawal => "Social Withdrawal",
            SpiralType::Anger => "Anger Spiral",
            SpiralType::Panic => "Panic Episodes",
            SpiralType::SelfDestructive => "Self-Destructive Phase",
        }
    }
}

/// An active spiral event affecting an entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveSpiral {
    /// Type of spiral.
    pub spiral_type: SpiralType,
    /// Tick when this spiral started.
    pub started_tick: u64,
    /// Current severity (0.0 to 1.0, can exceed 1.0 for severe cases).
    pub severity: f32,
    /// Whether the entity is currently in recovery (meeting thresholds).
    pub in_recovery: bool,
    /// Ticks spent in recovery state.
    pub recovery_ticks: u64,
    /// Number of times this spiral has triggered for this entity.
    pub recurrence_count: u32,
}

impl ActiveSpiral {
    /// Create a new active spiral.
    pub fn new(spiral_type: SpiralType, started_tick: u64, severity: f32) -> Self {
        Self {
            spiral_type,
            started_tick,
            severity,
            in_recovery: false,
            recovery_ticks: 0,
            recurrence_count: 1,
        }
    }

    /// Check if minimum duration has passed (recovery can begin).
    pub fn can_recover(&self, current_tick: u64) -> bool {
        let elapsed = current_tick.saturating_sub(self.started_tick);
        elapsed >= self.spiral_type.minimum_duration()
    }

    /// Calculate stat effects for this tick based on severity.
    pub fn tick_effects(&self) -> Vec<(StatKind, f32)> {
        let base_effects = self.spiral_type.stat_effects();
        let severity_mult = if self.in_recovery { 0.5 } else { self.severity };
        
        base_effects
            .into_iter()
            .map(|(stat, delta)| (stat, delta * severity_mult))
            .collect()
    }
}

/// Tracks spiral state for a single entity (player or NPC).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpiralState {
    /// Currently active spirals.
    pub active_spirals: Vec<ActiveSpiral>,
    /// Cooldown timers: spiral_type → tick when cooldown expires.
    pub cooldowns: HashMap<SpiralType, u64>,
    /// Historical count of each spiral type experienced.
    pub history: HashMap<SpiralType, u32>,
    /// Total recovery events (successfully exited spirals).
    pub total_recoveries: u32,
}

impl SpiralState {
    /// Create new empty spiral state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if any spiral is currently active.
    pub fn has_active_spiral(&self) -> bool {
        !self.active_spirals.is_empty()
    }

    /// Check if a specific spiral type is active.
    pub fn is_spiral_active(&self, spiral_type: SpiralType) -> bool {
        self.active_spirals.iter().any(|s| s.spiral_type == spiral_type)
    }

    /// Check if a spiral type is on cooldown.
    pub fn is_on_cooldown(&self, spiral_type: SpiralType, current_tick: u64) -> bool {
        self.cooldowns
            .get(&spiral_type)
            .map(|&expires| current_tick < expires)
            .unwrap_or(false)
    }

    /// Get active spiral of a specific type.
    pub fn get_spiral(&self, spiral_type: SpiralType) -> Option<&ActiveSpiral> {
        self.active_spirals.iter().find(|s| s.spiral_type == spiral_type)
    }

    /// Get mutable reference to active spiral.
    pub fn get_spiral_mut(&mut self, spiral_type: SpiralType) -> Option<&mut ActiveSpiral> {
        self.active_spirals.iter_mut().find(|s| s.spiral_type == spiral_type)
    }

    /// Start a new spiral.
    pub fn start_spiral(&mut self, spiral_type: SpiralType, current_tick: u64, severity: f32) {
        // Check if already active
        if self.is_spiral_active(spiral_type) {
            // Worsen existing spiral
            if let Some(spiral) = self.get_spiral_mut(spiral_type) {
                spiral.severity = (spiral.severity + severity * 0.5).min(2.0);
                spiral.recurrence_count += 1;
            }
            return;
        }

        // Check cooldown
        if self.is_on_cooldown(spiral_type, current_tick) {
            return;
        }

        // Create new spiral
        let mut spiral = ActiveSpiral::new(spiral_type, current_tick, severity);
        
        // Increase severity based on history (repeated spirals are harder)
        let past_count = *self.history.get(&spiral_type).unwrap_or(&0);
        spiral.severity *= 1.0 + (past_count as f32 * 0.1).min(0.5);

        self.active_spirals.push(spiral);
        *self.history.entry(spiral_type).or_insert(0) += 1;
    }

    /// End a spiral and start cooldown.
    pub fn end_spiral(&mut self, spiral_type: SpiralType, current_tick: u64) {
        self.active_spirals.retain(|s| s.spiral_type != spiral_type);
        
        // Set cooldown
        let cooldown_expires = current_tick + spiral_type.cooldown();
        self.cooldowns.insert(spiral_type, cooldown_expires);
        
        self.total_recoveries += 1;
    }

    /// Get total active spiral count.
    pub fn active_count(&self) -> usize {
        self.active_spirals.len()
    }

    /// Get combined severity of all active spirals.
    pub fn total_severity(&self) -> f32 {
        self.active_spirals.iter().map(|s| s.severity).sum()
    }
}

/// Configuration for the failure/recovery system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureRecoveryConfig {
    /// Mood threshold below which spirals can trigger.
    pub spiral_mood_threshold: f32,
    /// Base probability multiplier for spiral events.
    pub spiral_probability_mult: f32,
    /// How many ticks of meeting recovery thresholds needed to exit spiral.
    pub recovery_ticks_required: u64,
    /// Whether to enable spiral system.
    pub spirals_enabled: bool,
    /// Maximum simultaneous spirals.
    pub max_concurrent_spirals: usize,
}

impl Default for FailureRecoveryConfig {
    fn default() -> Self {
        Self {
            spiral_mood_threshold: SPIRAL_MOOD_THRESHOLD,
            spiral_probability_mult: 1.0,
            recovery_ticks_required: 24, // 1 day of stable stats
            spirals_enabled: true,
            max_concurrent_spirals: 3,
        }
    }
}

/// Main failure/recovery system manager.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FailureRecoverySystem {
    /// Per-entity spiral states. Key is NpcId or 0 for player.
    pub entity_states: HashMap<u64, SpiralState>,
    /// Configuration.
    pub config: FailureRecoveryConfig,
    /// Global event log: (tick, entity_id, event_description).
    pub event_log: Vec<(u64, u64, String)>,
}

impl FailureRecoverySystem {
    /// Create new system with default config.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with custom config.
    pub fn with_config(config: FailureRecoveryConfig) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    /// Get or create spiral state for an entity.
    pub fn get_state(&mut self, entity_id: u64) -> &SpiralState {
        self.entity_states.entry(entity_id).or_default()
    }

    /// Get mutable state for an entity.
    pub fn get_state_mut(&mut self, entity_id: u64) -> &mut SpiralState {
        self.entity_states.entry(entity_id).or_default()
    }

    /// Calculate social support score from stats.
    /// Uses Charisma as proxy for social connections/support network.
    pub fn calculate_social_support(stats: &Stats) -> f32 {
        // Combination of charisma (social skill) and reputation (social standing)
        stats.charisma * 0.6 + stats.reputation.max(0.0) * 0.4
    }

    /// Check if entity meets recovery thresholds for a spiral.
    pub fn meets_recovery_thresholds(stats: &Stats, spiral_type: SpiralType) -> bool {
        let (health_threshold, social_threshold) = spiral_type.recovery_thresholds();
        let social_support = Self::calculate_social_support(stats);
        
        stats.health >= health_threshold && social_support >= social_threshold
    }

    /// Main tick function for an entity.
    /// Returns stat deltas to apply and any events that occurred.
    pub fn tick_entity(
        &mut self,
        entity_id: u64,
        stats: &Stats,
        current_tick: u64,
        seed: u64,
    ) -> (Vec<(StatKind, f32)>, Vec<String>) {
        if !self.config.spirals_enabled {
            return (vec![], vec![]);
        }

        let mut stat_deltas = Vec::new();
        let mut events = Vec::new();

        // Check for new spiral triggers (mood below threshold)
        if stats.mood < self.config.spiral_mood_threshold {
            let new_events = self.check_spiral_triggers(entity_id, stats, current_tick, seed);
            events.extend(new_events);
        }

        // Process active spirals
        let state = self.entity_states.entry(entity_id).or_default();
        
        // Collect spirals to end (can't modify while iterating)
        let mut spirals_to_end = Vec::new();

        for spiral in &mut state.active_spirals {
            // Apply stat effects
            stat_deltas.extend(spiral.tick_effects());

            // Check recovery
            if spiral.can_recover(current_tick) {
                if Self::meets_recovery_thresholds(stats, spiral.spiral_type) {
                    spiral.in_recovery = true;
                    spiral.recovery_ticks += 1;

                    if spiral.recovery_ticks >= self.config.recovery_ticks_required {
                        spirals_to_end.push(spiral.spiral_type);
                        events.push(format!(
                            "Recovered from {}",
                            spiral.spiral_type.display_name()
                        ));
                    }
                } else {
                    // Lost recovery progress
                    spiral.in_recovery = false;
                    spiral.recovery_ticks = spiral.recovery_ticks.saturating_sub(2);
                }
            }
        }

        // End completed spirals
        for spiral_type in spirals_to_end {
            state.end_spiral(spiral_type, current_tick);
        }

        // Log events
        for event in &events {
            self.event_log.push((current_tick, entity_id, event.clone()));
        }

        (stat_deltas, events)
    }

    /// Check and potentially trigger new spirals.
    fn check_spiral_triggers(
        &mut self,
        entity_id: u64,
        stats: &Stats,
        current_tick: u64,
        seed: u64,
    ) -> Vec<String> {
        let mut events = Vec::new();
        let mut rng = DeterministicRng::with_domain(seed, current_tick, "spiral_check");

        let state = self.entity_states.entry(entity_id).or_default();

        // Don't exceed max concurrent spirals
        if state.active_count() >= self.config.max_concurrent_spirals {
            return events;
        }

        // Check each spiral type
        for &spiral_type in SpiralType::all() {
            // Skip if active or on cooldown
            if state.is_spiral_active(spiral_type) || state.is_on_cooldown(spiral_type, current_tick) {
                continue;
            }

            // Calculate trigger probability
            let base_prob = spiral_type.base_probability() * self.config.spiral_probability_mult;
            
            // Modify by mood severity (worse mood = higher chance)
            let mood_factor = (self.config.spiral_mood_threshold - stats.mood).max(0.0) / 5.0;
            let adjusted_prob = base_prob * (1.0 + mood_factor);

            // Roll for trigger
            if rng.gen_f32() < adjusted_prob {
                // Calculate initial severity based on mood
                let severity = 0.5 + ((-stats.mood - 5.0) / 10.0).clamp(0.0, 0.5);
                
                state.start_spiral(spiral_type, current_tick, severity);
                events.push(format!(
                    "Entered {} (severity: {:.0}%)",
                    spiral_type.display_name(),
                    severity * 100.0
                ));
            }
        }

        events
    }

    /// Force trigger a specific spiral (for storylet outcomes).
    pub fn trigger_spiral(
        &mut self,
        entity_id: u64,
        spiral_type: SpiralType,
        current_tick: u64,
        severity: f32,
    ) {
        let state = self.entity_states.entry(entity_id).or_default();
        state.start_spiral(spiral_type, current_tick, severity);
        
        self.event_log.push((
            current_tick,
            entity_id,
            format!("Triggered {} (forced)", spiral_type.display_name()),
        ));
    }

    /// Force end a spiral (for storylet outcomes, therapy, etc.).
    pub fn force_recovery(
        &mut self,
        entity_id: u64,
        spiral_type: SpiralType,
        current_tick: u64,
    ) {
        if let Some(state) = self.entity_states.get_mut(&entity_id) {
            if state.is_spiral_active(spiral_type) {
                state.end_spiral(spiral_type, current_tick);
                self.event_log.push((
                    current_tick,
                    entity_id,
                    format!("Force recovered from {}", spiral_type.display_name()),
                ));
            }
        }
    }

    /// Get summary of entity's mental state.
    pub fn get_entity_summary(&self, entity_id: u64) -> SpiralSummary {
        let state = self.entity_states.get(&entity_id);
        
        match state {
            Some(s) => SpiralSummary {
                active_spirals: s.active_spirals.iter().map(|sp| sp.spiral_type).collect(),
                total_severity: s.total_severity(),
                any_in_recovery: s.active_spirals.iter().any(|sp| sp.in_recovery),
                lifetime_spirals: s.history.values().sum(),
                lifetime_recoveries: s.total_recoveries,
            },
            None => SpiralSummary::default(),
        }
    }

    /// Clear expired cooldowns (housekeeping).
    pub fn clear_expired_cooldowns(&mut self, current_tick: u64) {
        for state in self.entity_states.values_mut() {
            state.cooldowns.retain(|_, &mut expires| current_tick < expires);
        }
    }

    /// Trim event log to last N entries.
    pub fn trim_event_log(&mut self, max_entries: usize) {
        if self.event_log.len() > max_entries {
            let drain_count = self.event_log.len() - max_entries;
            self.event_log.drain(0..drain_count);
        }
    }
}

/// Summary of an entity's spiral state for API/UI.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpiralSummary {
    /// Currently active spiral types.
    pub active_spirals: Vec<SpiralType>,
    /// Combined severity of all active spirals.
    pub total_severity: f32,
    /// Whether any spiral is in recovery phase.
    pub any_in_recovery: bool,
    /// Total number of spirals ever entered.
    pub lifetime_spirals: u32,
    /// Total number of spirals successfully recovered from.
    pub lifetime_recoveries: u32,
}

impl SpiralSummary {
    /// Check if entity is in crisis (high severity or multiple spirals).
    pub fn is_in_crisis(&self) -> bool {
        self.total_severity > 1.5 || self.active_spirals.len() >= 2
    }

    /// Get a human-readable status.
    pub fn status_label(&self) -> &'static str {
        if self.active_spirals.is_empty() {
            "Stable"
        } else if self.any_in_recovery {
            "Recovering"
        } else if self.is_in_crisis() {
            "In Crisis"
        } else {
            "Struggling"
        }
    }
}

/// Trait modifier influences on spiral probability.
/// Higher stability = lower spiral chance, higher impulsivity = higher chance, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitSpiralModifiers {
    /// Stability trait (0-100): reduces all spiral chances.
    pub stability_factor: f32,
    /// Impulsivity trait (0-100): increases addiction/anger spirals.
    pub impulsivity_factor: f32,
    /// Sociability trait (0-100): reduces withdrawal spiral chance.
    pub sociability_factor: f32,
}

impl Default for TraitSpiralModifiers {
    fn default() -> Self {
        Self {
            stability_factor: 50.0,
            impulsivity_factor: 50.0,
            sociability_factor: 50.0,
        }
    }
}

impl TraitSpiralModifiers {
    /// Calculate probability modifier for a spiral type based on traits.
    pub fn get_modifier(&self, spiral_type: SpiralType) -> f32 {
        let stability_mod = 1.0 - (self.stability_factor - 50.0) / 100.0; // High stability = lower chance
        
        let specific_mod = match spiral_type {
            SpiralType::Addiction | SpiralType::Anger | SpiralType::SelfDestructive => {
                1.0 + (self.impulsivity_factor - 50.0) / 100.0 // High impulsivity = higher chance
            }
            SpiralType::Withdrawal => {
                1.0 - (self.sociability_factor - 50.0) / 100.0 // High sociability = lower chance
            }
            _ => 1.0,
        };

        (stability_mod * specific_mod).max(0.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spiral_state_new() {
        let state = SpiralState::new();
        assert!(!state.has_active_spiral());
        assert_eq!(state.active_count(), 0);
    }

    #[test]
    fn test_spiral_trigger() {
        let mut state = SpiralState::new();
        state.start_spiral(SpiralType::Anxiety, 100, 0.7);

        assert!(state.has_active_spiral());
        assert!(state.is_spiral_active(SpiralType::Anxiety));
        assert!(!state.is_spiral_active(SpiralType::Depression));
        assert_eq!(state.active_count(), 1);
    }

    #[test]
    fn test_spiral_cooldown() {
        let mut state = SpiralState::new();
        state.start_spiral(SpiralType::Anxiety, 100, 0.7);
        
        // End spiral
        state.end_spiral(SpiralType::Anxiety, 200);
        
        // Should be on cooldown
        assert!(state.is_on_cooldown(SpiralType::Anxiety, 200));
        assert!(state.is_on_cooldown(SpiralType::Anxiety, 300));
        
        // After cooldown expires
        let cooldown = SpiralType::Anxiety.cooldown();
        assert!(!state.is_on_cooldown(SpiralType::Anxiety, 200 + cooldown + 1));
    }

    #[test]
    fn test_spiral_recovery_thresholds() {
        let mut stats = Stats::default();
        stats.health = 60.0;
        stats.charisma = 50.0;
        stats.reputation = 30.0;

        // Anxiety needs (40, 30) - should meet
        assert!(FailureRecoverySystem::meets_recovery_thresholds(&stats, SpiralType::Anxiety));

        // Self-destructive needs (70, 60) - should not meet
        assert!(!FailureRecoverySystem::meets_recovery_thresholds(&stats, SpiralType::SelfDestructive));
    }

    #[test]
    fn test_system_tick_no_spiral() {
        let mut system = FailureRecoverySystem::new();
        let mut stats = Stats::default();
        stats.mood = 0.0; // Above threshold

        let (deltas, events) = system.tick_entity(1, &stats, 100, 12345);

        assert!(deltas.is_empty());
        assert!(events.is_empty());
    }

    #[test]
    fn test_system_tick_with_active_spiral() {
        let mut system = FailureRecoverySystem::new();
        
        // Manually add a spiral
        system.trigger_spiral(1, SpiralType::Anxiety, 100, 0.8);

        let stats = Stats::default();
        let (deltas, _events) = system.tick_entity(1, &stats, 110, 12345);

        // Should have stat effects
        assert!(!deltas.is_empty());
        
        // Check for expected stat types
        assert!(deltas.iter().any(|(kind, _)| *kind == StatKind::Energy));
    }

    #[test]
    fn test_spiral_severity_increases_with_history() {
        let mut state = SpiralState::new();
        
        // First occurrence
        state.start_spiral(SpiralType::Anxiety, 100, 0.5);
        let first_severity = state.get_spiral(SpiralType::Anxiety).unwrap().severity;
        state.end_spiral(SpiralType::Anxiety, 500);

        // Clear cooldown for test
        state.cooldowns.clear();

        // Second occurrence
        state.start_spiral(SpiralType::Anxiety, 1000, 0.5);
        let second_severity = state.get_spiral(SpiralType::Anxiety).unwrap().severity;

        // Second should be more severe due to history
        assert!(second_severity > first_severity);
    }

    #[test]
    fn test_trait_modifiers() {
        let stable_traits = TraitSpiralModifiers {
            stability_factor: 80.0,
            impulsivity_factor: 20.0,
            sociability_factor: 70.0,
        };

        let unstable_traits = TraitSpiralModifiers {
            stability_factor: 20.0,
            impulsivity_factor: 80.0,
            sociability_factor: 30.0,
        };

        // Stable person should have lower modifier
        let stable_mod = stable_traits.get_modifier(SpiralType::Anxiety);
        let unstable_mod = unstable_traits.get_modifier(SpiralType::Anxiety);
        
        assert!(stable_mod < unstable_mod);
    }

    #[test]
    fn test_summary_status() {
        let summary = SpiralSummary::default();
        assert_eq!(summary.status_label(), "Stable");

        let recovering = SpiralSummary {
            active_spirals: vec![SpiralType::Anxiety],
            total_severity: 0.5,
            any_in_recovery: true,
            lifetime_spirals: 1,
            lifetime_recoveries: 0,
        };
        assert_eq!(recovering.status_label(), "Recovering");

        let crisis = SpiralSummary {
            active_spirals: vec![SpiralType::Anxiety, SpiralType::Depression],
            total_severity: 2.0,
            any_in_recovery: false,
            lifetime_spirals: 2,
            lifetime_recoveries: 0,
        };
        assert_eq!(crisis.status_label(), "In Crisis");
        assert!(crisis.is_in_crisis());
    }
}
