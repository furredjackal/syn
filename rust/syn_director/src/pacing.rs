//! Pacing engine for narrative heat and phase management.
//!
//! This module provides the core pacing logic that maintains narrative rhythm:
//! - Tracks `narrative_heat` as a scalar intensity measure
//! - Manages `narrative_phase` transitions through the story arc
//! - Integrates with eligibility (filtering) and scoring
//!
//! # Narrative Heat
//!
//! Heat represents the current intensity level of the narrative (0.0 = calm, 100.0 = crisis).
//! It naturally decays each tick and increases when high-heat events fire.
//!
//! # Narrative Phases
//!
//! The director cycles through phases based on heat thresholds:
//! - **LowKey**: Slice-of-life, building blocks, quiet moments
//! - **Rising**: Tension building, foreshadowing, setup  
//! - **Peak**: High-drama, crisis moments, confrontations
//! - **Fallout**: Aftermath, consequences, emotional processing
//! - **Recovery**: Healing, reconciliation, new equilibrium
//!
//! # Usage
//!
//! ```ignore
//! // At the start of each tick:
//! pacing::on_tick_start(&mut state, &config.pacing);
//!
//! // After a storylet fires:
//! pacing::on_event_fired(&mut state, &config.pacing, storylet.heat as f32);
//! ```

use crate::config::PacingConfig;
use crate::state::{DirectorState, NarrativePhase};

/// Called at the start of each tick to apply natural heat decay.
///
/// This function:
/// 1. Decays heat toward the minimum (natural cooling)
/// 2. Clamps heat to configured bounds
/// 3. Updates the narrative phase if thresholds are crossed
///
/// Should be called before storylet selection each tick.
pub fn on_tick_start(state: &mut DirectorState, config: &PacingConfig) {
    // Apply natural heat decay
    state.narrative_heat -= config.heat_decay_per_tick;
    
    // Clamp to bounds
    state.clamp_heat(config.min_heat, config.max_heat);
    
    // Check for phase transitions
    state.update_phase(&config.phase_thresholds, config.min_phase_duration);
}

/// Called after an event fires to apply its heat contribution.
///
/// This function:
/// 1. Increases heat based on the event's heat value and config multiplier
/// 2. Clamps heat to configured bounds
/// 3. Updates the narrative phase if thresholds are crossed
///
/// Should be called after storylet selection and execution.
pub fn on_event_fired(state: &mut DirectorState, config: &PacingConfig, story_heat: f32) {
    // Add heat from the event
    state.narrative_heat += story_heat * config.heat_increase_per_event_factor;
    
    // Clamp to bounds
    state.clamp_heat(config.min_heat, config.max_heat);
    
    // Check for phase transitions
    state.update_phase(&config.phase_thresholds, config.min_phase_duration);
}

/// Get the heat contribution factor for a storylet based on current phase.
///
/// This is used in scoring to prefer storylets that match the current phase:
/// - In LowKey, low-heat storylets get a bonus
/// - In Peak, high-heat storylets get a bonus
/// - etc.
///
/// Returns a multiplier (typically 0.5 to 2.0).
pub fn heat_alignment_factor(
    state: &DirectorState,
    config: &PacingConfig,
    storylet_heat: f32,
) -> f32 {
    match state.narrative_phase {
        NarrativePhase::LowKey => {
            // Prefer low-heat storylets (1-3)
            if storylet_heat <= 3.0 {
                config.phase_match_bonus
            } else if storylet_heat <= 5.0 {
                1.0
            } else {
                // Penalize high heat during calm periods
                1.0 / (1.0 + (storylet_heat - 5.0) * 0.15)
            }
        }
        NarrativePhase::Rising => {
            // Prefer medium-heat storylets (3-6)
            if storylet_heat >= 3.0 && storylet_heat <= 6.0 {
                config.phase_match_bonus * 0.9 // Slight bonus
            } else if storylet_heat < 3.0 {
                0.85 // Slight penalty for too calm
            } else {
                1.1 // High heat is acceptable during rising
            }
        }
        NarrativePhase::Peak => {
            // Strongly prefer high-heat storylets (7+)
            if storylet_heat >= 7.0 {
                config.phase_match_bonus * 1.2
            } else if storylet_heat >= 5.0 {
                config.phase_match_bonus * 0.8
            } else {
                0.6 // Low heat during peak is discouraged
            }
        }
        NarrativePhase::Fallout => {
            // Prefer medium-high heat (4-7) - consequences
            if storylet_heat >= 4.0 && storylet_heat <= 7.0 {
                config.phase_match_bonus
            } else if storylet_heat > 7.0 {
                0.8 // Too intense during fallout
            } else {
                0.9 // Low heat is okay but not preferred
            }
        }
        NarrativePhase::Recovery => {
            // Prefer low-medium heat (1-4) - healing
            if storylet_heat <= 4.0 {
                config.phase_match_bonus
            } else if storylet_heat <= 6.0 {
                0.85
            } else {
                0.65 // High intensity during recovery is discouraged
            }
        }
    }
}

/// Check if a storylet's heat level is appropriate for the current phase.
///
/// This is a hard filter used in the eligibility pipeline.
/// Returns false to block storylets that are completely inappropriate.
///
/// Unlike `heat_alignment_factor` (which affects scoring), this is a
/// strict yes/no gate.
pub fn is_heat_appropriate(
    state: &DirectorState,
    storylet_heat: f32,
    has_active_pressures: bool,
    is_forced: bool,
) -> bool {
    // Forced events always pass
    if is_forced {
        return true;
    }
    
    match state.narrative_phase {
        NarrativePhase::LowKey => {
            // Block very high-heat events unless there's pressure
            if storylet_heat > 7.0 && !has_active_pressures {
                return false;
            }
            true
        }
        NarrativePhase::Rising => {
            // More permissive during rising tension
            // Only block extremely low heat if we're well into rising
            if storylet_heat < 1.0 && state.narrative_heat > 35.0 {
                return false;
            }
            true
        }
        NarrativePhase::Peak => {
            // Block very low heat during intense peak moments
            // but only if we're at extreme heat levels
            if storylet_heat < 2.0 && state.narrative_heat > 75.0 {
                return false;
            }
            true
        }
        NarrativePhase::Fallout => {
            // Fallout is permissive - consequences can come in many forms
            true
        }
        NarrativePhase::Recovery => {
            // Block very high heat to allow the narrative to cool down
            if storylet_heat > 8.0 && !has_active_pressures {
                return false;
            }
            true
        }
    }
}

/// Compute what phase we would be in given a heat level.
///
/// Useful for testing or for preview calculations.
pub fn compute_phase_for_heat(
    heat: f32,
    current_phase: NarrativePhase,
    thresholds: &crate::config::PhaseThresholds,
) -> NarrativePhase {
    // Use same logic as update_phase but without mutation
    match current_phase {
        NarrativePhase::LowKey => {
            if heat >= thresholds.lowkey_to_rising {
                NarrativePhase::Rising
            } else {
                NarrativePhase::LowKey
            }
        }
        NarrativePhase::Rising => {
            if heat >= thresholds.rising_to_peak {
                NarrativePhase::Peak
            } else if heat < thresholds.recovery_to_lowkey {
                NarrativePhase::LowKey
            } else {
                NarrativePhase::Rising
            }
        }
        NarrativePhase::Peak => {
            if heat < thresholds.peak_to_fallout {
                NarrativePhase::Fallout
            } else {
                NarrativePhase::Peak
            }
        }
        NarrativePhase::Fallout => {
            if heat < thresholds.fallout_to_recovery {
                NarrativePhase::Recovery
            } else if heat >= thresholds.rising_to_peak {
                NarrativePhase::Peak
            } else {
                NarrativePhase::Fallout
            }
        }
        NarrativePhase::Recovery => {
            if heat < thresholds.recovery_to_lowkey {
                NarrativePhase::LowKey
            } else if heat >= thresholds.lowkey_to_rising {
                NarrativePhase::Rising
            } else {
                NarrativePhase::Recovery
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::PhaseThresholds;
    use syn_core::SimTick;

    fn create_test_config() -> PacingConfig {
        PacingConfig {
            min_heat: 0.0,
            max_heat: 100.0,
            heat_decay_per_tick: 1.0,
            heat_increase_per_event_factor: 1.0,
            phase_thresholds: PhaseThresholds::default(),
            min_phase_duration: 1, // Fast transitions for testing
            phase_match_bonus: 1.5,
        }
    }

    fn create_test_state() -> DirectorState {
        DirectorState::new()
    }

    // =========================================================================
    // Heat Decay Tests
    // =========================================================================

    #[test]
    fn test_heat_decays_each_tick() {
        let config = create_test_config();
        let mut state = create_test_state();
        state.narrative_heat = 50.0;
        state.tick = SimTick::new(10);

        on_tick_start(&mut state, &config);

        assert!((state.narrative_heat - 49.0).abs() < 0.001,
            "Heat should decay by 1.0 per tick: {}", state.narrative_heat);
    }

    #[test]
    fn test_heat_does_not_decay_below_min() {
        let config = create_test_config();
        let mut state = create_test_state();
        state.narrative_heat = 0.5;
        state.tick = SimTick::new(10);

        on_tick_start(&mut state, &config);

        assert_eq!(state.narrative_heat, 0.0,
            "Heat should not go below min_heat");
    }

    #[test]
    fn test_heat_with_no_events_eventually_reaches_min() {
        let config = create_test_config();
        let mut state = create_test_state();
        state.narrative_heat = 50.0;

        // Simulate 100 ticks with no events
        for tick in 1..=100 {
            state.tick = SimTick::new(tick);
            on_tick_start(&mut state, &config);
        }

        assert_eq!(state.narrative_heat, 0.0,
            "Heat should reach min after many ticks with no events");
    }

    // =========================================================================
    // Heat Increase Tests
    // =========================================================================

    #[test]
    fn test_event_increases_heat() {
        let config = create_test_config();
        let mut state = create_test_state();
        state.narrative_heat = 10.0;

        on_event_fired(&mut state, &config, 5.0);

        assert!((state.narrative_heat - 15.0).abs() < 0.001,
            "Heat should increase by event heat: {}", state.narrative_heat);
    }

    #[test]
    fn test_event_heat_respects_factor() {
        let mut config = create_test_config();
        config.heat_increase_per_event_factor = 2.0;
        let mut state = create_test_state();
        state.narrative_heat = 10.0;

        on_event_fired(&mut state, &config, 5.0);

        assert!((state.narrative_heat - 20.0).abs() < 0.001,
            "Heat should increase by event heat * factor: {}", state.narrative_heat);
    }

    #[test]
    fn test_heat_does_not_exceed_max() {
        let config = create_test_config();
        let mut state = create_test_state();
        state.narrative_heat = 95.0;

        on_event_fired(&mut state, &config, 10.0);

        assert_eq!(state.narrative_heat, 100.0,
            "Heat should not exceed max_heat");
    }

    // =========================================================================
    // Phase Transition Tests
    // =========================================================================

    #[test]
    fn test_lowkey_to_rising_transition() {
        let config = create_test_config();
        let mut state = create_test_state();
        state.tick = SimTick::new(10);
        state.phase_started_at = SimTick::new(0); // Been in phase long enough

        // Fire enough events to cross threshold
        for _ in 0..30 {
            on_event_fired(&mut state, &config, 1.0);
        }

        assert_eq!(state.narrative_phase, NarrativePhase::Rising,
            "Should transition to Rising when heat >= 25.0");
    }

    #[test]
    fn test_rising_to_peak_transition() {
        let config = create_test_config();
        let mut state = create_test_state();
        state.narrative_phase = NarrativePhase::Rising;
        state.narrative_heat = 55.0;
        state.tick = SimTick::new(10);
        state.phase_started_at = SimTick::new(0);

        // Push heat above peak threshold
        on_event_fired(&mut state, &config, 10.0);

        assert_eq!(state.narrative_phase, NarrativePhase::Peak,
            "Should transition to Peak when heat >= 60.0");
    }

    #[test]
    fn test_peak_to_fallout_transition() {
        let config = create_test_config();
        let mut state = create_test_state();
        state.narrative_phase = NarrativePhase::Peak;
        state.narrative_heat = 50.0;
        state.tick = SimTick::new(20);
        state.phase_started_at = SimTick::new(10);

        // Let heat decay below fallout threshold
        for tick in 21..=30 {
            state.tick = SimTick::new(tick);
            on_tick_start(&mut state, &config);
        }

        assert_eq!(state.narrative_phase, NarrativePhase::Fallout,
            "Should transition to Fallout when heat < 45.0");
    }

    #[test]
    fn test_fallout_to_recovery_transition() {
        let config = create_test_config();
        let mut state = create_test_state();
        state.narrative_phase = NarrativePhase::Fallout;
        state.narrative_heat = 30.0;
        state.tick = SimTick::new(20);
        state.phase_started_at = SimTick::new(10);

        // Let heat decay below recovery threshold
        for tick in 21..=30 {
            state.tick = SimTick::new(tick);
            on_tick_start(&mut state, &config);
        }

        assert_eq!(state.narrative_phase, NarrativePhase::Recovery,
            "Should transition to Recovery when heat < 25.0");
    }

    #[test]
    fn test_recovery_to_lowkey_transition() {
        let config = create_test_config();
        let mut state = create_test_state();
        state.narrative_phase = NarrativePhase::Recovery;
        state.narrative_heat = 20.0;
        state.tick = SimTick::new(20);
        state.phase_started_at = SimTick::new(10);

        // Let heat decay below lowkey threshold
        for tick in 21..=30 {
            state.tick = SimTick::new(tick);
            on_tick_start(&mut state, &config);
        }

        assert_eq!(state.narrative_phase, NarrativePhase::LowKey,
            "Should transition to LowKey when heat < 15.0");
    }

    #[test]
    fn test_min_phase_duration_prevents_rapid_transitions() {
        let mut config = create_test_config();
        config.min_phase_duration = 10;
        let mut state = create_test_state();
        state.tick = SimTick::new(5);
        state.phase_started_at = SimTick::new(0);
        state.narrative_heat = 30.0; // Above rising threshold

        // Try to trigger transition before min_phase_duration
        on_tick_start(&mut state, &config);

        assert_eq!(state.narrative_phase, NarrativePhase::LowKey,
            "Should not transition before min_phase_duration");
    }

    #[test]
    fn test_phase_transition_after_min_duration() {
        let mut config = create_test_config();
        config.min_phase_duration = 10;
        let mut state = create_test_state();
        state.narrative_heat = 30.0; // Above rising threshold
        state.tick = SimTick::new(15);
        state.phase_started_at = SimTick::new(0);

        on_tick_start(&mut state, &config);

        assert_eq!(state.narrative_phase, NarrativePhase::Rising,
            "Should transition after min_phase_duration");
    }

    // =========================================================================
    // Full Cycle Tests
    // =========================================================================

    #[test]
    fn test_full_narrative_cycle() {
        let config = create_test_config();
        let mut state = create_test_state();
        let mut tick = 0u64;

        // Start in LowKey
        assert_eq!(state.narrative_phase, NarrativePhase::LowKey);

        // Fire high-heat events to reach Rising (need heat >= 25)
        for _ in 0..30 {
            tick += 1;
            state.tick = SimTick::new(tick);
            on_tick_start(&mut state, &config);
            on_event_fired(&mut state, &config, 2.0);
        }
        assert_eq!(state.narrative_phase, NarrativePhase::Rising,
            "Should be in Rising after sustained heat");

        // Fire more to reach Peak (need heat >= 60)
        for _ in 0..40 {
            tick += 1;
            state.tick = SimTick::new(tick);
            on_tick_start(&mut state, &config);
            on_event_fired(&mut state, &config, 2.0);
        }
        assert_eq!(state.narrative_phase, NarrativePhase::Peak,
            "Should be in Peak after high heat");

        // Let it decay to Fallout (heat < 45)
        // Starting from 71, need 27 ticks to get below 45
        for _ in 0..27 {
            tick += 1;
            state.tick = SimTick::new(tick);
            on_tick_start(&mut state, &config);
        }
        assert_eq!(state.narrative_phase, NarrativePhase::Fallout,
            "Should be in Fallout as heat decays from Peak");

        // Let it decay to Recovery (heat < 25)
        // At 44, need 20 ticks to get below 25 (44 - 20 = 24 < 25)
        for _ in 0..20 {
            tick += 1;
            state.tick = SimTick::new(tick);
            on_tick_start(&mut state, &config);
        }
        assert_eq!(state.narrative_phase, NarrativePhase::Recovery,
            "Should be in Recovery as heat continues to decay");

        // Let it decay back to LowKey (heat < 15)
        // At 24, need 10 ticks to get below 15 (24 - 10 = 14 < 15)
        for _ in 0..10 {
            tick += 1;
            state.tick = SimTick::new(tick);
            on_tick_start(&mut state, &config);
        }
        assert_eq!(state.narrative_phase, NarrativePhase::LowKey,
            "Should return to LowKey after full decay");
    }

    #[test]
    fn test_phase_transitions_are_deterministic() {
        let config = create_test_config();

        // Run the same sequence twice
        let mut state1 = create_test_state();
        let mut state2 = create_test_state();

        let events = vec![5.0, 3.0, 8.0, 2.0, 7.0, 1.0, 9.0, 4.0];

        for (tick, &heat) in events.iter().enumerate() {
            state1.tick = SimTick::new(tick as u64 + 1);
            state2.tick = SimTick::new(tick as u64 + 1);
            
            on_tick_start(&mut state1, &config);
            on_tick_start(&mut state2, &config);
            
            on_event_fired(&mut state1, &config, heat);
            on_event_fired(&mut state2, &config, heat);
        }

        assert_eq!(state1.narrative_heat, state2.narrative_heat,
            "Heat should be deterministic");
        assert_eq!(state1.narrative_phase, state2.narrative_phase,
            "Phase should be deterministic");
    }

    // =========================================================================
    // Heat Alignment Factor Tests
    // =========================================================================

    #[test]
    fn test_lowkey_prefers_low_heat() {
        let state = create_test_state();
        let config = create_test_config();

        let low_factor = heat_alignment_factor(&state, &config, 2.0);
        let high_factor = heat_alignment_factor(&state, &config, 8.0);

        assert!(low_factor > high_factor,
            "Low heat should score better in LowKey: {} vs {}", low_factor, high_factor);
    }

    #[test]
    fn test_peak_prefers_high_heat() {
        let mut state = create_test_state();
        state.narrative_phase = NarrativePhase::Peak;
        let config = create_test_config();

        let low_factor = heat_alignment_factor(&state, &config, 2.0);
        let high_factor = heat_alignment_factor(&state, &config, 9.0);

        assert!(high_factor > low_factor,
            "High heat should score better in Peak: {} vs {}", high_factor, low_factor);
    }

    #[test]
    fn test_recovery_prefers_low_heat() {
        let mut state = create_test_state();
        state.narrative_phase = NarrativePhase::Recovery;
        let config = create_test_config();

        let low_factor = heat_alignment_factor(&state, &config, 3.0);
        let high_factor = heat_alignment_factor(&state, &config, 8.0);

        assert!(low_factor > high_factor,
            "Low heat should score better in Recovery: {} vs {}", low_factor, high_factor);
    }

    // =========================================================================
    // Heat Appropriateness Filter Tests
    // =========================================================================

    #[test]
    fn test_lowkey_blocks_high_heat_without_pressure() {
        let state = create_test_state();

        assert!(!is_heat_appropriate(&state, 8.0, false, false),
            "High heat should be blocked in LowKey without pressure");
    }

    #[test]
    fn test_lowkey_allows_high_heat_with_pressure() {
        let state = create_test_state();

        assert!(is_heat_appropriate(&state, 8.0, true, false),
            "High heat should be allowed in LowKey with pressure");
    }

    #[test]
    fn test_forced_events_always_pass() {
        let state = create_test_state();

        assert!(is_heat_appropriate(&state, 10.0, false, true),
            "Forced events should always pass");
    }

    #[test]
    fn test_recovery_blocks_high_heat() {
        let mut state = create_test_state();
        state.narrative_phase = NarrativePhase::Recovery;

        assert!(!is_heat_appropriate(&state, 9.0, false, false),
            "Very high heat should be blocked in Recovery");
    }

    #[test]
    fn test_peak_allows_most_heat_levels() {
        let mut state = create_test_state();
        state.narrative_phase = NarrativePhase::Peak;
        state.narrative_heat = 70.0; // High but not extreme

        assert!(is_heat_appropriate(&state, 1.0, false, false),
            "Peak should allow low heat at moderate heat levels");
        assert!(is_heat_appropriate(&state, 5.0, false, false),
            "Peak should allow medium heat");
        assert!(is_heat_appropriate(&state, 9.0, false, false),
            "Peak should allow high heat");
    }

    #[test]
    fn test_peak_blocks_very_low_heat_at_extreme_levels() {
        let mut state = create_test_state();
        state.narrative_phase = NarrativePhase::Peak;
        state.narrative_heat = 80.0; // Extreme

        assert!(!is_heat_appropriate(&state, 1.0, false, false),
            "Very low heat should be blocked at extreme Peak");
    }
}
