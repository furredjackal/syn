//! FRB v2 API Entrypoint Module
//!
//! This module provides thin wrappers around the core API functions in `lib.rs`
//! for flutter_rust_bridge code generation. All functions here delegate to the
//! existing implementations without reimplementing logic.
//!
//! FRB v2 requires a dedicated module for code scanning; this module acts as
//! the "public FFI surface" that the Dart codegen discovers.

use crate::{
    // Core API functions from lib.rs
    engine_new_game as engine_new_game_impl,
    engine_step as engine_step_impl,
    engine_choose_option as engine_choose_option_impl,
    get_game_state_snapshot as get_game_state_snapshot_impl,
    get_current_storylet as get_current_storylet_impl,
    get_available_choices as get_available_choices_impl,
    engine_tick as engine_tick_impl,
    engine_tick_many as engine_tick_many_impl,
    
    // API types used in function signatures
    ApiPlayerConfig,
    ApiSimpleGameState,
    ApiDirectorEventView,
    ApiDirectorChoiceView,
    ApiGameStateSnapshot,
};
use flutter_rust_bridge::frb;

// ==================== Game Initialization ====================

/// Initialize a new game with player configuration.
///
/// Combines character generation and world initialization into a single call.
/// Returns the initial game state snapshot for UI rendering.
///
/// # Arguments
/// * `seed` - World seed for deterministic generation
/// * `config` - Player configuration (name, archetype, difficulty, etc.)
#[frb(sync)]
pub fn engine_new_game(seed: u64, config: ApiPlayerConfig) -> Option<ApiSimpleGameState> {
    engine_new_game_impl(seed, config)
}

// ==================== Simulation Stepping ====================

/// Advance simulation by specified ticks and return updated state.
///
/// This is the main game loop function Flutter should call to progress time.
///
/// # Arguments
/// * `ticks` - Number of ticks to advance
#[frb(sync)]
pub fn engine_step(ticks: u32) -> Option<ApiSimpleGameState> {
    engine_step_impl(ticks)
}

/// Tick the engine by 1 tick (convenience wrapper).
#[frb(sync)]
pub fn engine_tick() {
    engine_tick_impl()
}

/// Advance the simulation by multiple ticks (convenience wrapper).
///
/// # Arguments
/// * `count` - Number of ticks to advance
#[frb(sync)]
pub fn engine_tick_many(count: u32) {
    engine_tick_many_impl(count)
}

// ==================== Event/Storylet Interaction ====================

/// Make a choice in the current event and advance simulation.
///
/// Applies the choice outcome and progresses time by the specified ticks.
///
/// # Arguments
/// * `storylet_id` - ID of the current storylet/event
/// * `choice_id` - ID of the selected choice
/// * `ticks` - Number of ticks to advance after applying the choice
#[frb(sync)]
pub fn engine_choose_option(
    storylet_id: String,
    choice_id: String,
    ticks: u32,
) -> Option<ApiSimpleGameState> {
    engine_choose_option_impl(storylet_id, choice_id, ticks)
}

/// Get current storylet/event card for UI display.
///
/// Returns the next eligible storylet, or None if no events are available.
#[frb(sync)]
pub fn get_current_storylet() -> Option<ApiDirectorEventView> {
    get_current_storylet_impl()
}

/// Get available choices for the current event.
///
/// Returns empty vector if no event is active.
#[frb(sync)]
pub fn get_available_choices() -> Vec<ApiDirectorChoiceView> {
    get_available_choices_impl()
}

// ==================== State Accessors ====================

/// Get unified game state snapshot for UI.
///
/// This is the primary comprehensive state accessor Flutter should use
/// for full UI rendering (includes narrative heat, karma bands, etc.).
#[frb(sync)]
pub fn get_game_state_snapshot() -> Option<ApiGameStateSnapshot> {
    get_game_state_snapshot_impl()
}
