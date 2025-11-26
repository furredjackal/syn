//! Deterministic scoring module for storylet selection.
//!
//! This module provides a structured scoring system that:
//! - Produces detailed score breakdowns for debugging and transparency
//! - Supports weighted candidate selection with deterministic tie-breaking
//! - Integrates with the pacing, pressure, and milestone systems
//!
//! # Determinism
//! All scoring and tie-breaking uses deterministic RNG seeded from world seed
//! and tick count. Given the same inputs, the same storylet will always be selected.

use crate::config::{DirectorConfig, PacingConfig, ScoringConfig};
use crate::pacing;
use crate::state::DirectorState;
use syn_core::rng::DeterministicRng;
use syn_core::WorldState;
use syn_storylets::library::{CompiledStorylet, StoryletKey};
use serde::{Deserialize, Serialize};

/// A storylet candidate with detailed score breakdown.
///
/// Provides full visibility into why a storylet received its final score,
/// useful for debugging, balancing, and player-facing UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredCandidate {
    /// The storylet key.
    pub key: StoryletKey,
    
    /// Base weight from the storylet definition.
    pub base_weight: f32,
    
    /// Multiplier from narrative heat alignment.
    /// Higher when storylet heat matches current phase.
    pub heat_alignment: f32,
    
    /// Bonus from matching context (player personality, traits, etc.).
    pub context_bonus: f32,
    
    /// Bonus from addressing active narrative pressures.
    pub pressure_bonus: f32,
    
    /// Bonus from advancing toward narrative milestones.
    pub milestone_bonus: f32,
    
    /// Penalty from pacing constraints (recency, variety).
    /// This is subtracted, not multiplied.
    pub pacing_penalty: f32,
    
    /// Deterministic jitter for tie-breaking.
    /// Small random value derived from (key, tick, seed) for stable ordering.
    pub jitter: f32,
    
    /// Final computed score (before jitter).
    /// `total_score = base_weight * heat_alignment * (1 + context_bonus + pressure_bonus + milestone_bonus) - pacing_penalty`
    pub total_score: f32,
    
    /// Final score including jitter for selection.
    /// `selection_score = total_score + jitter`
    pub selection_score: f32,
}

impl ScoredCandidate {
    /// Create a new scored candidate with all components.
    pub fn new(
        key: StoryletKey,
        base_weight: f32,
        heat_alignment: f32,
        context_bonus: f32,
        pressure_bonus: f32,
        milestone_bonus: f32,
        pacing_penalty: f32,
        jitter: f32,
    ) -> Self {
        let total_score = Self::compute_total(
            base_weight,
            heat_alignment,
            context_bonus,
            pressure_bonus,
            milestone_bonus,
            pacing_penalty,
        );
        let selection_score = total_score + jitter;
        
        ScoredCandidate {
            key,
            base_weight,
            heat_alignment,
            context_bonus,
            pressure_bonus,
            milestone_bonus,
            pacing_penalty,
            jitter,
            total_score,
            selection_score,
        }
    }
    
    /// Compute the total score from components.
    fn compute_total(
        base_weight: f32,
        heat_alignment: f32,
        context_bonus: f32,
        pressure_bonus: f32,
        milestone_bonus: f32,
        pacing_penalty: f32,
    ) -> f32 {
        let multiplied = base_weight * heat_alignment * (1.0 + context_bonus + pressure_bonus + milestone_bonus);
        (multiplied - pacing_penalty).max(0.0)
    }
    
    /// Check if this candidate has a viable score for selection.
    pub fn is_viable(&self, min_score: f32) -> bool {
        self.total_score >= min_score
    }
}

/// Results from scoring a batch of candidates.
#[derive(Debug, Clone)]
pub struct ScoringResults {
    /// All candidates with their scores.
    pub candidates: Vec<ScoredCandidate>,
    
    /// Candidates filtered to only viable scores.
    pub viable_candidates: Vec<ScoredCandidate>,
    
    /// The selected candidate (if any).
    pub selected: Option<ScoredCandidate>,
    
    /// Summary statistics for debugging.
    pub stats: ScoringStats,
}

/// Statistics about the scoring process.
#[derive(Debug, Clone, Default)]
pub struct ScoringStats {
    /// Total candidates scored.
    pub total_scored: usize,
    
    /// Candidates that met minimum viable score.
    pub viable_count: usize,
    
    /// Candidates eliminated by pacing penalty.
    pub pacing_eliminated: usize,
    
    /// Average score of viable candidates.
    pub avg_viable_score: f32,
    
    /// Maximum score (before jitter).
    pub max_score: f32,
    
    /// Score of selected candidate (if any).
    pub selected_score: Option<f32>,
}

/// Engine for scoring storylet candidates.
///
/// Takes eligible storylet keys and produces scored candidates
/// ready for weighted selection.
pub struct ScoringEngine<'a> {
    scoring_config: &'a ScoringConfig,
    pacing_config: &'a PacingConfig,
    state: &'a DirectorState,
    world_seed: u64,
}

impl<'a> ScoringEngine<'a> {
    /// Create a new scoring engine.
    pub fn new(
        scoring_config: &'a ScoringConfig,
        pacing_config: &'a PacingConfig,
        state: &'a DirectorState,
        world_seed: u64,
    ) -> Self {
        ScoringEngine {
            scoring_config,
            pacing_config,
            state,
            world_seed,
        }
    }
    
    /// Score all candidates and perform weighted selection.
    ///
    /// Returns full scoring results including breakdown and selection.
    pub fn score_and_select(
        &self,
        storylets: &[&CompiledStorylet],
        world: &WorldState,
    ) -> ScoringResults {
        let mut candidates = Vec::with_capacity(storylets.len());
        
        for storylet in storylets {
            let scored = self.score_storylet(storylet, world);
            candidates.push(scored);
        }
        
        // Compute stats
        let total_scored = candidates.len();
        let max_score = candidates.iter()
            .map(|c| c.total_score)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        
        // Filter to viable candidates
        let min_viable = self.scoring_config.min_viable_weight;
        let viable_candidates: Vec<ScoredCandidate> = candidates.iter()
            .filter(|c| c.is_viable(min_viable))
            .cloned()
            .collect();
        
        let viable_count = viable_candidates.len();
        let avg_viable_score = if viable_count > 0 {
            viable_candidates.iter().map(|c| c.total_score).sum::<f32>() / viable_count as f32
        } else {
            0.0
        };
        
        let pacing_eliminated = candidates.iter()
            .filter(|c| c.pacing_penalty > 0.0 && !c.is_viable(min_viable))
            .count();
        
        // Select from viable candidates
        let selected = self.pick_from_scored(&viable_candidates);
        let selected_score = selected.as_ref().map(|s| s.total_score);
        
        ScoringResults {
            candidates,
            viable_candidates,
            selected,
            stats: ScoringStats {
                total_scored,
                viable_count,
                pacing_eliminated,
                avg_viable_score,
                max_score,
                selected_score,
            },
        }
    }
    
    /// Score a single storylet.
    pub fn score_storylet(
        &self,
        storylet: &CompiledStorylet,
        world: &WorldState,
    ) -> ScoredCandidate {
        // 1. Base weight
        let base_weight = storylet.weight * self.scoring_config.base_weight_multiplier;
        
        // 2. Heat alignment
        let heat_alignment = self.compute_heat_alignment(storylet);
        
        // 3. Context bonus (personality, world flags, etc.)
        let context_bonus = self.compute_context_bonus(storylet, world);
        
        // 4. Pressure bonus (addresses active pressures)
        let pressure_bonus = self.compute_pressure_bonus(storylet);
        
        // 5. Milestone bonus (advances narrative milestones)
        let milestone_bonus = self.compute_milestone_bonus(storylet);
        
        // 6. Pacing penalty (recency, variety)
        let pacing_penalty = self.compute_pacing_penalty(storylet);
        
        // 7. Deterministic jitter for tie-breaking
        let jitter = self.compute_jitter(storylet.key);
        
        ScoredCandidate::new(
            storylet.key,
            base_weight,
            heat_alignment,
            context_bonus,
            pressure_bonus,
            milestone_bonus,
            pacing_penalty,
            jitter,
        )
    }
    
    /// Compute heat alignment multiplier.
    ///
    /// Returns a value typically between 0.5 and 2.0 based on how well
    /// the storylet's heat matches the current narrative phase.
    fn compute_heat_alignment(&self, storylet: &CompiledStorylet) -> f32 {
        let storylet_heat = storylet.heat as f32;
        pacing::heat_alignment_factor(self.state, self.pacing_config, storylet_heat)
    }
    
    /// Compute context bonus from world state.
    ///
    /// Checks for personality matches, world flags, etc.
    fn compute_context_bonus(&self, _storylet: &CompiledStorylet, _world: &WorldState) -> f32 {
        // TODO: Implement when personality/trait matching is added
        // For now, return 0 (no bonus)
        0.0
    }
    
    /// Compute pressure bonus from active narrative pressures.
    ///
    /// Storylets that address active pressures get a bonus.
    fn compute_pressure_bonus(&self, storylet: &CompiledStorylet) -> f32 {
        let mut bonus = 0.0;
        
        // Check if storylet addresses any active pressure
        for pressure in self.state.active_pressures.active_pressures() {
            // Check domain match via PressureKind
            if let Some(pressure_domain) = pressure.kind.to_domain() {
                if pressure_domain == storylet.domain {
                    // Higher severity = higher bonus
                    bonus += self.scoring_config.pressure_match_bonus * 0.5 * pressure.severity;
                }
            }
            
            // Check tag overlap
            for tag in &storylet.tags {
                if pressure.tags.contains(tag) {
                    bonus += self.scoring_config.pressure_match_bonus * 0.25 * pressure.severity;
                    break; // Only count once per storylet
                }
            }
        }
        
        // Cap at 2x the base pressure bonus
        bonus.min(self.scoring_config.pressure_match_bonus * 2.0)
    }
    
    /// Compute milestone bonus.
    ///
    /// Storylets that advance toward tracked milestones get a bonus.
    fn compute_milestone_bonus(&self, storylet: &CompiledStorylet) -> f32 {
        let mut bonus: f32 = 0.0;
        
        // Check if storylet advances any active milestone
        for milestone in self.state.milestones.active_milestones() {
            // Domain match gives bonus
            if milestone.kind.primary_domain() == storylet.domain {
                bonus += 0.3;
            }
            
            // Tag match gives bonus
            for tag in &storylet.tags {
                if milestone.advancing_tags.contains(tag) {
                    bonus += 0.2;
                    break;
                }
            }
        }
        
        bonus.min(1.0) // Cap at +100%
    }
    
    /// Compute pacing penalty.
    ///
    /// Penalizes storylets that:
    /// - Were recently fired (recency)
    /// - Are from overrepresented domains (variety)
    fn compute_pacing_penalty(&self, storylet: &CompiledStorylet) -> f32 {
        let mut penalty = 0.0;
        
        // Recency penalty
        if let Some(last_tick) = self.state.last_fired.last_tick_for_storylet(storylet.key) {
            let ticks_since = self.state.tick.0.saturating_sub(last_tick.0);
            if ticks_since < self.scoring_config.recency_decay_ticks {
                // Linear decay: full penalty at 0 ticks, 0 penalty at recency_decay_ticks
                let decay_progress = ticks_since as f32 / self.scoring_config.recency_decay_ticks as f32;
                penalty += self.scoring_config.recency_penalty * (1.0 - decay_progress);
            }
        }
        
        // Domain variety penalty (if domain fired recently)
        if let Some(last_domain_tick) = self.state.last_fired.last_tick_for_domain(storylet.domain) {
            let ticks_since = self.state.tick.0.saturating_sub(last_domain_tick.0);
            if ticks_since < 12 { // Within ~12 hours
                penalty += 0.2 * (1.0 - ticks_since as f32 / 12.0);
            }
        }
        
        penalty
    }
    
    /// Compute deterministic jitter for tie-breaking.
    ///
    /// Uses a hash of (key, tick, seed) to produce a small consistent value
    /// that breaks ties deterministically.
    fn compute_jitter(&self, key: StoryletKey) -> f32 {
        // Create a deterministic value from key + tick + seed
        let hash_input = key.0 as u64
            ^ (self.state.tick.0.wrapping_mul(0x9E37_79B9))
            ^ self.world_seed.wrapping_mul(0x7F4A_7C15);
        
        // Use a simple hash to get a value in [0, 0.01)
        // This is small enough to not affect actual scoring but large enough
        // to break ties consistently
        let hash = hash_input.wrapping_mul(0x517c_c1b7_2722_0a95);
        let normalized = (hash as f32) / (u64::MAX as f32);
        
        normalized * 0.01 // Jitter range: [0, 0.01)
    }
    
    /// Pick a storylet from scored candidates using weighted selection.
    ///
    /// Uses selection_score (which includes jitter) for deterministic ordering.
    /// Higher scores are more likely to be selected, but not guaranteed.
    pub fn pick_from_scored(&self, candidates: &[ScoredCandidate]) -> Option<ScoredCandidate> {
        if candidates.is_empty() {
            return None;
        }
        
        // Use deterministic RNG seeded from world seed + tick
        let seed = self.world_seed ^ (self.state.tick.0.wrapping_mul(0xDEAD_BEEF_CAFE_BABE));
        let mut rng = DeterministicRng::new(seed);
        
        // Compute total weight using selection_score
        let total_weight: f32 = candidates.iter()
            .map(|c| c.selection_score.max(0.0))
            .sum();
        
        if total_weight <= 0.0 {
            // All scores are zero or negative, return first (deterministic)
            return Some(candidates[0].clone());
        }
        
        // Weighted random selection
        let mut r = rng.gen_range_f32(0.0, total_weight);
        for candidate in candidates {
            let weight = candidate.selection_score.max(0.0);
            if r < weight {
                return Some(candidate.clone());
            }
            r -= weight;
        }
        
        // Fallback to last candidate (shouldn't happen but deterministic)
        Some(candidates.last().unwrap().clone())
    }
    
    /// Pick the top-scoring candidate deterministically.
    ///
    /// Uses selection_score which includes jitter for consistent tie-breaking.
    /// This is a simpler alternative to weighted selection when you want
    /// the "best" candidate rather than probabilistic selection.
    pub fn pick_top_scored(&self, candidates: &[ScoredCandidate]) -> Option<ScoredCandidate> {
        candidates.iter()
            .max_by(|a, b| {
                a.selection_score.partial_cmp(&b.selection_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned()
    }
}

/// Convenience function to score candidates without creating an engine.
///
/// Creates a temporary ScoringEngine and scores all candidates.
pub fn score_candidates(
    storylets: &[&CompiledStorylet],
    state: &DirectorState,
    config: &DirectorConfig,
    world: &WorldState,
) -> ScoringResults {
    let engine = ScoringEngine::new(&config.scoring, &config.pacing, state, world.seed.0);
    engine.score_and_select(storylets, world)
}

/// Convenience function to pick a storylet from scored candidates.
///
/// Uses weighted selection with deterministic tie-breaking.
pub fn pick_storylet_from_scored(
    candidates: &[ScoredCandidate],
    state: &DirectorState,
    world_seed: u64,
) -> Option<ScoredCandidate> {
    if candidates.is_empty() {
        return None;
    }
    
    // Use deterministic RNG seeded from world seed + tick
    let seed = world_seed ^ (state.tick.0.wrapping_mul(0xDEAD_BEEF_CAFE_BABE));
    let mut rng = DeterministicRng::new(seed);
    
    // Compute total weight
    let total_weight: f32 = candidates.iter()
        .map(|c| c.selection_score.max(0.0))
        .sum();
    
    if total_weight <= 0.0 {
        return Some(candidates[0].clone());
    }
    
    // Weighted random selection
    let mut r = rng.gen_range_f32(0.0, total_weight);
    for candidate in candidates {
        let weight = candidate.selection_score.max(0.0);
        if r < weight {
            return Some(candidate.clone());
        }
        r -= weight;
    }
    
    Some(candidates.last().unwrap().clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::PhaseThresholds;
    use crate::NarrativePhase;
    use syn_core::{NpcId, SimTick};
    use syn_storylets::{Cooldowns, LifeStage, Outcome, Prerequisites, StoryDomain, StoryletId};

    fn create_mock_storylet(key: u32, weight: f32, heat: u8) -> CompiledStorylet {
        CompiledStorylet {
            id: StoryletId::new(format!("test.storylet_{}", key)),
            key: StoryletKey(key),
            name: format!("Test Storylet {}", key),
            description: None,
            tags: vec![],
            domain: StoryDomain::Romance,
            life_stage: LifeStage::Adult,
            heat,
            weight,
            roles: vec![],
            prerequisites: Prerequisites::default(),
            cooldowns: Cooldowns::default(),
            outcomes: Outcome::default(),
            follow_ups_resolved: vec![],
        }
    }

    fn create_test_state() -> DirectorState {
        DirectorState::new()
    }

    fn create_test_scoring_config() -> ScoringConfig {
        ScoringConfig::default()
    }
    
    fn create_test_pacing_config() -> PacingConfig {
        PacingConfig {
            min_heat: 0.0,
            max_heat: 100.0,
            heat_decay_per_tick: 1.0,
            heat_increase_per_event_factor: 1.0,
            phase_thresholds: PhaseThresholds::default(),
            min_phase_duration: 1,
            phase_match_bonus: 1.5,
        }
    }

    #[test]
    fn test_scored_candidate_creation() {
        let scored = ScoredCandidate::new(
            StoryletKey(1),
            1.0,  // base_weight
            1.5,  // heat_alignment
            0.2,  // context_bonus
            0.3,  // pressure_bonus
            0.1,  // milestone_bonus
            0.1,  // pacing_penalty
            0.005, // jitter
        );

        // total = 1.0 * 1.5 * (1 + 0.2 + 0.3 + 0.1) - 0.1 = 1.5 * 1.6 - 0.1 = 2.4 - 0.1 = 2.3
        assert!((scored.total_score - 2.3).abs() < 0.001);
        assert!((scored.selection_score - 2.305).abs() < 0.001);
    }

    #[test]
    fn test_scored_candidate_viability() {
        let viable = ScoredCandidate::new(
            StoryletKey(1), 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0
        );
        let not_viable = ScoredCandidate::new(
            StoryletKey(2), 0.05, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0
        );

        assert!(viable.is_viable(0.1));
        assert!(!not_viable.is_viable(0.1));
    }

    #[test]
    fn test_heat_alignment_lowkey_prefers_low_heat() {
        let state = create_test_state(); // LowKey by default
        let scoring_config = create_test_scoring_config();
        let pacing_config = create_test_pacing_config();
        let engine = ScoringEngine::new(&scoring_config, &pacing_config, &state, 12345);

        let low_heat = create_mock_storylet(1, 1.0, 2);
        let high_heat = create_mock_storylet(2, 1.0, 8);

        let low_alignment = engine.compute_heat_alignment(&low_heat);
        let high_alignment = engine.compute_heat_alignment(&high_heat);

        assert!(low_alignment > high_alignment, 
            "Low heat should score better in LowKey phase: {} vs {}", 
            low_alignment, high_alignment);
    }

    #[test]
    fn test_heat_alignment_peak_prefers_high_heat() {
        let mut state = create_test_state();
        state.narrative_phase = NarrativePhase::Peak;
        let scoring_config = create_test_scoring_config();
        let pacing_config = create_test_pacing_config();
        let engine = ScoringEngine::new(&scoring_config, &pacing_config, &state, 12345);

        let low_heat = create_mock_storylet(1, 1.0, 2);
        let high_heat = create_mock_storylet(2, 1.0, 9);

        let low_alignment = engine.compute_heat_alignment(&low_heat);
        let high_alignment = engine.compute_heat_alignment(&high_heat);

        assert!(high_alignment > low_alignment,
            "High heat should score better in Peak phase: {} vs {}",
            high_alignment, low_alignment);
    }

    #[test]
    fn test_jitter_is_deterministic() {
        let state = create_test_state();
        let scoring_config = create_test_scoring_config();
        let pacing_config = create_test_pacing_config();
        
        // Same seed should produce same jitter
        let engine1 = ScoringEngine::new(&scoring_config, &pacing_config, &state, 12345);
        let engine2 = ScoringEngine::new(&scoring_config, &pacing_config, &state, 12345);

        let jitter1 = engine1.compute_jitter(StoryletKey(42));
        let jitter2 = engine2.compute_jitter(StoryletKey(42));

        assert_eq!(jitter1, jitter2);
    }

    #[test]
    fn test_jitter_differs_by_key() {
        let state = create_test_state();
        let scoring_config = create_test_scoring_config();
        let pacing_config = create_test_pacing_config();
        let engine = ScoringEngine::new(&scoring_config, &pacing_config, &state, 12345);

        let jitter1 = engine.compute_jitter(StoryletKey(1));
        let jitter2 = engine.compute_jitter(StoryletKey(2));

        assert_ne!(jitter1, jitter2);
    }

    #[test]
    fn test_jitter_differs_by_tick() {
        let mut state1 = create_test_state();
        state1.tick = SimTick::new(100);
        
        let mut state2 = create_test_state();
        state2.tick = SimTick::new(200);

        let scoring_config = create_test_scoring_config();
        let pacing_config = create_test_pacing_config();
        let engine1 = ScoringEngine::new(&scoring_config, &pacing_config, &state1, 12345);
        let engine2 = ScoringEngine::new(&scoring_config, &pacing_config, &state2, 12345);

        let jitter1 = engine1.compute_jitter(StoryletKey(42));
        let jitter2 = engine2.compute_jitter(StoryletKey(42));

        assert_ne!(jitter1, jitter2);
    }

    #[test]
    fn test_jitter_is_small() {
        let state = create_test_state();
        let scoring_config = create_test_scoring_config();
        let pacing_config = create_test_pacing_config();
        let engine = ScoringEngine::new(&scoring_config, &pacing_config, &state, 12345);

        for key in 0..100 {
            let jitter = engine.compute_jitter(StoryletKey(key));
            assert!(jitter >= 0.0 && jitter < 0.01,
                "Jitter should be in [0, 0.01): {}", jitter);
        }
    }

    #[test]
    fn test_scoring_engine_scores_storylets() {
        let state = create_test_state();
        let scoring_config = create_test_scoring_config();
        let pacing_config = create_test_pacing_config();
        let engine = ScoringEngine::new(&scoring_config, &pacing_config, &state, 12345);

        let storylet = create_mock_storylet(1, 1.5, 3);
        let world = WorldState::new(syn_core::WorldSeed(12345), NpcId::new(1));

        let scored = engine.score_storylet(&storylet, &world);

        assert_eq!(scored.key, StoryletKey(1));
        assert!((scored.base_weight - 1.5).abs() < 0.001);
        assert!(scored.total_score > 0.0);
    }

    #[test]
    fn test_weighted_selection_prefers_higher_weights() {
        let state = create_test_state();
        let scoring_config = create_test_scoring_config();
        let pacing_config = create_test_pacing_config();
        let engine = ScoringEngine::new(&scoring_config, &pacing_config, &state, 12345);

        // Create candidates with very different weights
        let candidates = vec![
            ScoredCandidate::new(StoryletKey(1), 10.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.001),
            ScoredCandidate::new(StoryletKey(2), 0.1, 1.0, 0.0, 0.0, 0.0, 0.0, 0.002),
        ];

        // Run selection many times and count
        let mut counts = [0usize; 2];
        for i in 0..1000 {
            let mut test_state = create_test_state();
            test_state.tick = SimTick::new(i);
            let test_engine = ScoringEngine::new(&scoring_config, &pacing_config, &test_state, 12345 + i);
            
            if let Some(selected) = test_engine.pick_from_scored(&candidates) {
                if selected.key.0 == 1 {
                    counts[0] += 1;
                } else {
                    counts[1] += 1;
                }
            }
        }

        // High-weight candidate should be selected most of the time
        assert!(counts[0] > counts[1] * 5, 
            "High-weight candidate should dominate: {} vs {}", counts[0], counts[1]);
    }

    #[test]
    fn test_pick_top_scored() {
        let state = create_test_state();
        let scoring_config = create_test_scoring_config();
        let pacing_config = create_test_pacing_config();
        let engine = ScoringEngine::new(&scoring_config, &pacing_config, &state, 12345);

        let candidates = vec![
            ScoredCandidate::new(StoryletKey(1), 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.001),
            ScoredCandidate::new(StoryletKey(2), 5.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.002),
            ScoredCandidate::new(StoryletKey(3), 3.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.003),
        ];

        let top = engine.pick_top_scored(&candidates).unwrap();
        assert_eq!(top.key, StoryletKey(2));
    }

    #[test]
    fn test_selection_is_deterministic() {
        let state = create_test_state();
        let scoring_config = create_test_scoring_config();
        let pacing_config = create_test_pacing_config();

        let candidates = vec![
            ScoredCandidate::new(StoryletKey(1), 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.001),
            ScoredCandidate::new(StoryletKey(2), 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.002),
            ScoredCandidate::new(StoryletKey(3), 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.003),
        ];

        // Same seed should always produce same result
        let engine1 = ScoringEngine::new(&scoring_config, &pacing_config, &state, 99999);
        let engine2 = ScoringEngine::new(&scoring_config, &pacing_config, &state, 99999);

        let selected1 = engine1.pick_from_scored(&candidates).unwrap();
        let selected2 = engine2.pick_from_scored(&candidates).unwrap();

        assert_eq!(selected1.key, selected2.key);
    }

    #[test]
    fn test_recency_penalty_applied() {
        let mut state = create_test_state();
        state.tick = SimTick::new(100);
        // Record that storylet 1 was fired recently
        state.last_fired.record_fired(
            StoryletKey(1),
            StoryDomain::Romance,
            &[],
            SimTick::new(95),
        );

        let scoring_config = create_test_scoring_config();
        let pacing_config = create_test_pacing_config();
        let engine = ScoringEngine::new(&scoring_config, &pacing_config, &state, 12345);

        let recently_fired = create_mock_storylet(1, 1.0, 3);
        let fresh = create_mock_storylet(2, 1.0, 3);

        let penalty_recent = engine.compute_pacing_penalty(&recently_fired);
        let penalty_fresh = engine.compute_pacing_penalty(&fresh);

        assert!(penalty_recent > penalty_fresh,
            "Recently fired should have higher penalty: {} vs {}",
            penalty_recent, penalty_fresh);
    }

    #[test]
    fn test_score_and_select_integration() {
        let state = create_test_state();
        let config = DirectorConfig::default();
        let world = WorldState::new(syn_core::WorldSeed(12345), NpcId::new(1));

        let storylets: Vec<CompiledStorylet> = vec![
            create_mock_storylet(1, 1.0, 3),
            create_mock_storylet(2, 2.0, 3),
            create_mock_storylet(3, 1.5, 3),
        ];
        let storylet_refs: Vec<&CompiledStorylet> = storylets.iter().collect();

        let results = score_candidates(&storylet_refs, &state, &config, &world);

        assert_eq!(results.stats.total_scored, 3);
        assert!(results.stats.viable_count > 0);
        assert!(results.selected.is_some());
    }

    #[test]
    fn test_empty_candidates_returns_none() {
        let state = create_test_state();
        
        let result = pick_storylet_from_scored(&[], &state, 12345);
        assert!(result.is_none());
    }

    #[test]
    fn test_convenience_pick_function() {
        let state = create_test_state();

        let candidates = vec![
            ScoredCandidate::new(StoryletKey(1), 2.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.001),
            ScoredCandidate::new(StoryletKey(2), 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.002),
        ];

        let selected = pick_storylet_from_scored(&candidates, &state, 12345);
        assert!(selected.is_some());
    }
}
