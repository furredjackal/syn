//! Public API module for the Event Director.
//!
//! This module provides the primary `step()` method that wraps the entire Phase 2 pipeline
//! into a single, deterministic call. The step method:
//!
//! 1. Updates director time, pacing, and pressures
//! 2. Drains ready queued events
//! 3. Runs the eligibility pipeline for fresh candidates
//! 4. Combines queued and fresh candidates
//! 5. Scores and selects a storylet deterministically
//! 6. Updates state (heat, cooldowns, last_fired, pressures/milestones, queue)
//! 7. Returns a compact result for the simulation engine to apply
//!
//! # Example
//!
//! ```ignore
//! let ctx = EligibilityContext::new(&world, &memory, tick);
//! let result = director.step(tick, &ctx);
//!
//! if let Some(fired) = result.fired {
//!     println!("Storylet {} fired!", fired.key.0);
//!     // Apply storylet outcomes to world state...
//! }
//! ```

use serde::{Deserialize, Serialize};
use syn_storylets::library::StoryletKey;

use crate::queue::QueueSource;
use crate::scoring::ScoredCandidate;

// ============================================================================
// Result Types
// ============================================================================

/// Information about a fired storylet.
///
/// Contains the storylet key, scoring details, and source information
/// for the simulation engine to apply.
#[derive(Debug, Clone)]
pub struct FiredStorylet {
    /// The key of the storylet that was selected.
    pub key: StoryletKey,

    /// Full scoring breakdown for the selected storylet.
    pub scored: ScoredCandidate,

    /// Whether this storylet came from the queue (vs. fresh pipeline).
    pub is_from_queue: bool,

    /// If from queue, what was the source (follow-up, milestone, etc.).
    pub queue_source: Option<QueueSource>,
}

impl FiredStorylet {
    /// Create a new FiredStorylet.
    pub fn new(
        key: StoryletKey,
        scored: ScoredCandidate,
        is_from_queue: bool,
        queue_source: Option<QueueSource>,
    ) -> Self {
        FiredStorylet {
            key,
            scored,
            is_from_queue,
            queue_source,
        }
    }
}

/// Result of a director step.
///
/// Contains the fired storylet (if any) and optional debug information
/// about the candidate pool.
#[derive(Debug, Clone, Default)]
pub struct DirectorStepResult {
    /// The storylet that was selected and fired, if any.
    pub fired: Option<FiredStorylet>,

    /// All scored candidates for debugging (only populated when debug_candidates feature enabled).
    /// Useful for understanding why certain storylets were or weren't selected.
    #[cfg(feature = "debug_candidates")]
    pub debug_candidates: Option<Vec<ScoredCandidate>>,

    /// Summary statistics about this step.
    pub stats: StepStats,
}

impl DirectorStepResult {
    /// Create a result with no storylet fired.
    pub fn none() -> Self {
        DirectorStepResult {
            fired: None,
            #[cfg(feature = "debug_candidates")]
            debug_candidates: None,
            stats: StepStats::default(),
        }
    }

    /// Create a result with a fired storylet.
    pub fn with_fired(fired: FiredStorylet, stats: StepStats) -> Self {
        DirectorStepResult {
            fired: Some(fired),
            #[cfg(feature = "debug_candidates")]
            debug_candidates: None,
            stats,
        }
    }

    /// Check if a storylet was fired this step.
    pub fn has_fired(&self) -> bool {
        self.fired.is_some()
    }

    /// Get the key of the fired storylet, if any.
    pub fn fired_key(&self) -> Option<StoryletKey> {
        self.fired.as_ref().map(|f| f.key)
    }
}

/// Statistics about a director step.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StepStats {
    /// Number of events that were ready in the queue.
    pub queue_ready_count: usize,

    /// Number of fresh candidates from the pipeline.
    pub fresh_candidate_count: usize,

    /// Number of merged candidates after combining queue + fresh.
    pub merged_candidate_count: usize,

    /// Number of candidates that passed minimum scoring threshold.
    pub viable_candidate_count: usize,

    /// Current narrative heat after this step.
    pub narrative_heat: f32,

    /// Current narrative phase after this step.
    pub narrative_phase: String,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_director_step_result_helpers() {
        let result = DirectorStepResult::none();
        assert!(!result.has_fired());
        assert!(result.fired_key().is_none());
    }

    #[test]
    fn test_step_stats_default() {
        let stats = StepStats::default();
        assert_eq!(stats.queue_ready_count, 0);
        assert_eq!(stats.fresh_candidate_count, 0);
        assert_eq!(stats.merged_candidate_count, 0);
    }
}
