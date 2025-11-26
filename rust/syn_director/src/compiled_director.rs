//! New generic Event Director with consolidated state and config.
//!
//! This module provides `CompiledEventDirector<S>`, a refactored version of the Event Director
//! that uses the generic `StoryletSource` trait and consolidates all runtime state into
//! `DirectorState` and all configuration into `DirectorConfig`.
//!
//! The old `EventDirector` in lib.rs is preserved for backward compatibility with
//! the legacy `Storylet` type. This new director works exclusively with compiled
//! storylets from `syn_storylets`.

use crate::config::DirectorConfig;
use crate::pacing;
use crate::pipeline::{CandidateSet, EligibilityPipeline, IndexPrefilterParams};
use crate::queue::{QueuedEvent, QueueSource};
use crate::scoring::{ScoredCandidate, ScoringEngine, ScoringResults};
use crate::state::{DirectorState, NarrativePhase};
use crate::storylet_source::StoryletSource;
use crate::{EligibilityContext, EligibilityEngine, RoleAssignmentEngine};
use syn_core::rng::DeterministicRng;
use syn_core::{SimTick, WorldState};
use syn_memory::MemorySystem;
use syn_storylets::library::{CompiledStorylet, StoryletKey};
use syn_storylets::StoryletId;

/// A modern Event Director that works with compiled storylets via the `StoryletSource` trait.
///
/// This director follows a clean architecture with:
/// - `storylets`: Read-only access to the storylet library via trait
/// - `state`: All mutable runtime state (tick, heat, cooldowns, queues, etc.)
/// - `config`: Immutable tuning parameters
///
/// # Type Parameter
/// - `S`: Any type implementing `StoryletSource` (in-memory library, memory-mapped, etc.)
///
/// # Determinism
/// All selection and tie-breaking uses deterministic RNG seeded from world state.
/// Given the same world seed and tick, the same storylet will always be selected.
pub struct CompiledEventDirector<S: StoryletSource> {
    /// The storylet library (read-only reference via trait).
    storylets: S,
    
    /// All mutable runtime state for the director.
    state: DirectorState,
    
    /// Immutable configuration parameters.
    config: DirectorConfig,
}

impl<S: StoryletSource> CompiledEventDirector<S> {
    /// Create a new CompiledEventDirector with the given storylets and config.
    ///
    /// Initializes DirectorState with sensible defaults (tick 0, no heat, LowKey phase).
    pub fn new(storylets: S, config: DirectorConfig) -> Self {
        CompiledEventDirector {
            storylets,
            state: DirectorState::new(),
            config,
        }
    }

    /// Create a new director with default configuration.
    pub fn with_defaults(storylets: S) -> Self {
        Self::new(storylets, DirectorConfig::default())
    }

    /// Create a new director optimized for testing.
    pub fn for_testing(storylets: S) -> Self {
        Self::new(storylets, DirectorConfig::for_testing())
    }

    /// Restore a director from a saved snapshot.
    ///
    /// This allows loading a previously saved director state, enabling
    /// save/load functionality across game sessions.
    ///
    /// # Arguments
    /// - `storylets`: The storylet source (must match what was used when saving)
    /// - `config`: Configuration to use (may differ from when saved)
    /// - `snapshot`: The saved director snapshot
    ///
    /// # Determinism
    /// Given the same world seed and tick, a restored director will produce
    /// identical storylet selections as the original.
    pub fn restore_from_snapshot(
        storylets: S,
        config: DirectorConfig,
        snapshot: crate::persistence::DirectorSnapshot,
    ) -> Self {
        // Note: We could validate snapshot.config_version here if needed
        CompiledEventDirector {
            storylets,
            state: snapshot.state,
            config,
        }
    }

    /// Create a snapshot of the current director state for persistence.
    ///
    /// The snapshot captures all mutable director state, allowing the director
    /// to be restored to this exact point later.
    ///
    /// # Example
    /// ```ignore
    /// let snapshot = director.snapshot();
    /// let bytes = serialize_snapshot(&snapshot)?;
    /// // Save bytes to disk or database
    /// ```
    pub fn snapshot(&self) -> crate::persistence::DirectorSnapshot {
        crate::persistence::DirectorSnapshot::new(self.state.clone())
    }

    /// Create a snapshot with an explicit config version marker.
    ///
    /// Useful if you want to track which config version was active when
    /// the snapshot was created.
    pub fn snapshot_with_config_version(&self, config_version: u32) -> crate::persistence::DirectorSnapshot {
        crate::persistence::DirectorSnapshot::with_config_version(self.state.clone(), config_version)
    }

    /// Get a reference to the current director state.
    pub fn state(&self) -> &DirectorState {
        &self.state
    }

    /// Get a mutable reference to the director state.
    pub fn state_mut(&mut self) -> &mut DirectorState {
        &mut self.state
    }

    /// Get a reference to the director configuration.
    pub fn config(&self) -> &DirectorConfig {
        &self.config
    }

    /// Get a reference to the storylet source.
    pub fn storylets(&self) -> &S {
        &self.storylets
    }

    /// Called when the simulation tick advances.
    ///
    /// Updates internal state to reflect the new tick:
    /// - Applies natural heat decay
    /// - Updates narrative phase based on heat thresholds
    /// - (Future: Process pressure changes)
    pub fn on_tick_advance(&mut self, tick: SimTick) {
        self.state.tick = tick;
        
        // Apply pacing engine updates
        pacing::on_tick_start(&mut self.state, &self.config.pacing);
        
        // TODO: In future prompts, add:
        // - Pressure decay
    }

    // =========================================================================
    // Queue Management
    // =========================================================================

    /// Dequeue all events ready to fire at the current tick.
    ///
    /// Returns events in deterministic order (by scheduled_tick, then priority, then key).
    /// These events should be processed before normal storylet selection.
    pub fn dequeue_ready_events(&mut self) -> Vec<QueuedEvent> {
        let now = self.state.tick;
        self.state.pending_queue.pop_ready(now)
    }

    /// Dequeue only forced events ready to fire.
    ///
    /// Forced events bypass normal pacing and take precedence over regular selection.
    /// Use this when you want forced events to pre-empt everything.
    pub fn dequeue_forced_events(&mut self) -> Vec<QueuedEvent> {
        let now = self.state.tick;
        self.state.pending_queue.pop_forced_ready(now)
    }

    /// Check if there are any forced events ready to fire.
    pub fn has_forced_ready(&self) -> bool {
        self.state.pending_queue.has_forced_ready(self.state.tick)
    }

    /// Check if there are any events ready to fire.
    pub fn has_ready_events(&self) -> bool {
        self.state.pending_queue.has_ready(self.state.tick)
    }

    /// Schedule a follow-up event from a storylet outcome.
    ///
    /// This is the primary way storylet outcomes schedule future events.
    pub fn schedule_follow_up(
        &mut self,
        storylet_key: StoryletKey,
        delay_ticks: u64,
        priority: i32,
        forced: bool,
    ) {
        let event = QueuedEvent::delayed_follow_up(
            storylet_key,
            self.state.tick,
            delay_ticks,
            priority,
            forced,
        );
        self.state.pending_queue.push(event, self.config.queue.max_queue_size());
    }

    /// Schedule an immediate follow-up event (fires next tick).
    pub fn schedule_immediate_follow_up(&mut self, storylet_key: StoryletKey, forced: bool) {
        let event = QueuedEvent::immediate_follow_up(storylet_key, self.state.tick, forced);
        self.state.pending_queue.push(event, self.config.queue.max_queue_size());
    }

    /// Schedule a milestone event.
    pub fn schedule_milestone(
        &mut self,
        storylet_key: StoryletKey,
        scheduled_tick: SimTick,
        priority: i32,
    ) {
        let event = QueuedEvent::milestone(storylet_key, scheduled_tick, priority);
        self.state.pending_queue.push(event, self.config.queue.max_queue_size());
    }

    /// Schedule a pressure relief event.
    pub fn schedule_pressure_relief(
        &mut self,
        storylet_key: StoryletKey,
        scheduled_tick: SimTick,
        priority: i32,
        forced: bool,
    ) {
        let event = QueuedEvent::pressure_relief(storylet_key, scheduled_tick, priority, forced);
        self.state.pending_queue.push(event, self.config.queue.max_queue_size());
    }

    /// Get the number of queued events.
    pub fn queue_len(&self) -> usize {
        self.state.pending_queue.len()
    }

    /// Find all eligible storylets for the current world state.
    ///
    /// Uses the `EligibilityEngine` to filter storylets based on:
    /// - Life stage compatibility
    /// - Stat/trait thresholds
    /// - Relationship conditions
    /// - Memory prerequisites
    /// - World flags
    /// - Cooldowns
    pub fn find_eligible<'a>(
        &'a self,
        world: &'a WorldState,
        memory: &'a MemorySystem,
    ) -> Vec<StoryletKey> {
        let ctx = EligibilityContext {
            world,
            memory,
            current_tick: self.state.tick,
        };
        let engine = EligibilityEngine::new(&self.storylets);
        let mut eligible = engine.find_eligible_storylets(&ctx);

        // Apply cooldown filtering from our state
        eligible.retain(|&key| {
            self.state.cooldowns.is_globally_ready(key, self.state.tick)
        });

        // Apply variety constraints
        if self.config.variety.min_storylet_repeat_interval > 0 {
            let since = SimTick::new(
                self.state.tick.0.saturating_sub(self.config.variety.min_storylet_repeat_interval)
            );
            eligible.retain(|&key| {
                !self.state.last_fired.storylet_fired_since(key, since)
            });
        }

        eligible
    }

    /// Compute candidates using the layered eligibility pipeline.
    ///
    /// This method provides full visibility into how candidates are filtered
    /// at each stage of the pipeline:
    /// 1. Index prefilter (tags, domain, life_stage)
    /// 2. Structural prerequisites (stats, traits, relationships, etc.)
    /// 3. Cooldown & repetition constraints
    /// 4. Pacing constraints (narrative phase alignment)
    ///
    /// Use this for debugging or when you need to understand why certain
    /// storylets are being filtered out.
    pub fn compute_candidates<'a>(
        &'a self,
        ctx: &EligibilityContext<'a>,
    ) -> CandidateSet {
        let pipeline = EligibilityPipeline::new(
            &self.storylets,
            &self.state,
            &self.config,
        );
        pipeline.run(ctx)
    }

    /// Compute candidates with custom index prefilter parameters.
    ///
    /// Allows callers to specify required tags, allowed domains, or
    /// override the life stage filter.
    pub fn compute_candidates_with_params<'a>(
        &'a self,
        ctx: &EligibilityContext<'a>,
        params: &IndexPrefilterParams,
    ) -> CandidateSet {
        let pipeline = EligibilityPipeline::new(
            &self.storylets,
            &self.state,
            &self.config,
        );
        pipeline.run_with_params(ctx, params)
    }

    /// Score a storylet for selection priority.
    ///
    /// Combines multiple factors:
    /// - Base weight from the storylet
    /// - Heat alignment with current narrative phase
    /// - Pressure matching (if storylet addresses active pressure)
    /// - Variety bonus (if domain is underrepresented)
    /// - Recency penalty (if recently fired)
    pub fn score_storylet(
        &self,
        storylet: &CompiledStorylet,
        world: &WorldState,
    ) -> f32 {
        let mut score = storylet.weight * self.config.scoring.base_weight_multiplier;

        // Heat alignment: prefer storylets matching current phase
        score *= self.compute_heat_alignment(storylet);

        // Apply narrative heat multiplier from world
        score *= world.heat_multiplier();

        // Recency penalty
        if let Some(last_tick) = self.state.last_fired.last_tick_for_storylet(storylet.key) {
            let ticks_since = self.state.tick.0.saturating_sub(last_tick.0);
            if ticks_since < self.config.scoring.recency_decay_ticks {
                let decay_factor = ticks_since as f32 / self.config.scoring.recency_decay_ticks as f32;
                score *= self.config.scoring.recency_penalty + 
                    (1.0 - self.config.scoring.recency_penalty) * decay_factor;
            }
        }

        // Variety bonus if domain hasn't fired recently
        if !self.state.last_fired.domain_fired_since(
            storylet.domain,
            SimTick::new(self.state.tick.0.saturating_sub(self.config.variety.min_domain_repeat_interval)),
        ) {
            score *= self.config.scoring.variety_bonus;
        }

        score.max(0.0)
    }

    /// Compute heat alignment factor for a storylet.
    ///
    /// Returns a multiplier based on how well the storylet's heat
    /// matches the current narrative phase.
    fn compute_heat_alignment(&self, storylet: &CompiledStorylet) -> f32 {
        let storylet_heat = storylet.heat as f32;
        
        match self.state.narrative_phase {
            NarrativePhase::LowKey => {
                // Prefer low-heat storylets during calm periods
                if storylet_heat <= 3.0 {
                    self.config.pacing.phase_match_bonus
                } else {
                    1.0 / (1.0 + (storylet_heat - 3.0) * 0.1)
                }
            }
            NarrativePhase::Rising => {
                // Prefer medium-heat storylets
                if storylet_heat >= 3.0 && storylet_heat <= 6.0 {
                    self.config.pacing.phase_match_bonus
                } else {
                    1.0
                }
            }
            NarrativePhase::Peak => {
                // Prefer high-heat storylets
                if storylet_heat >= 7.0 {
                    self.config.pacing.phase_match_bonus
                } else {
                    0.8
                }
            }
            NarrativePhase::Fallout => {
                // Prefer medium-high heat (consequences)
                if storylet_heat >= 4.0 && storylet_heat <= 7.0 {
                    self.config.pacing.phase_match_bonus
                } else {
                    1.0
                }
            }
            NarrativePhase::Recovery => {
                // Prefer low-medium heat (healing)
                if storylet_heat <= 4.0 {
                    self.config.pacing.phase_match_bonus
                } else {
                    0.9
                }
            }
        }
    }

    /// Select the best storylet from eligible candidates.
    ///
    /// Uses weighted selection with deterministic RNG for reproducibility.
    /// When scores are tied (within epsilon), uses RNG for tie-breaking.
    pub fn select_storylet(
        &self,
        eligible_keys: &[StoryletKey],
        world: &WorldState,
        memory: &MemorySystem,
    ) -> Option<StoryletKey> {
        if eligible_keys.is_empty() {
            return None;
        }

        // Build weighted candidates list
        let mut candidates: Vec<(StoryletKey, f32)> = Vec::new();
        let ctx = EligibilityContext {
            world,
            memory,
            current_tick: self.state.tick,
        };

        for &key in eligible_keys {
            if let Some(storylet) = self.storylets.get_storylet_by_key(key) {
                // Verify role assignment is possible
                let role_engine = RoleAssignmentEngine::from_context(&ctx);
                if role_engine.assign_roles_for_storylet(storylet, None).is_some() {
                    let score = self.score_storylet(storylet, world);
                    if score >= self.config.scoring.min_viable_weight {
                        candidates.push((key, score));
                    }
                }
            }
        }

        if candidates.is_empty() {
            return None;
        }

        // Deterministic weighted selection
        let mut rng = DeterministicRng::new(
            world.seed.0 ^ (self.state.tick.0.wrapping_mul(0x9E37_79B9_7F4A_7C15))
        );
        self.weighted_select(&candidates, &mut rng)
    }

    /// Weighted random selection from candidates using deterministic RNG.
    fn weighted_select(
        &self,
        candidates: &[(StoryletKey, f32)],
        rng: &mut DeterministicRng,
    ) -> Option<StoryletKey> {
        if candidates.is_empty() {
            return None;
        }

        let total_weight: f32 = candidates.iter().map(|(_, w)| w).sum();
        if total_weight <= 0.0 {
            return Some(candidates[0].0);
        }

        let mut r = rng.gen_range_f32(0.0, total_weight);
        for (key, weight) in candidates {
            if r < *weight {
                return Some(*key);
            }
            r -= *weight;
        }

        Some(candidates.last().unwrap().0)
    }

    /// Record that a storylet was fired.
    ///
    /// Updates cooldowns, last-fired tracking, and narrative heat.
    pub fn record_fired(
        &mut self,
        storylet: &CompiledStorylet,
        actor_id: Option<u64>,
    ) {
        let tick = self.state.tick;
        
        // Update last-fired tracking
        self.state.last_fired.record_fired(
            storylet.key,
            storylet.domain,
            &storylet.tags,
            tick,
        );

        // Apply cooldowns from storylet config
        if let Some(global_ticks) = storylet.cooldowns.global_cooldown_ticks {
            if global_ticks > 0 {
                self.state.cooldowns.mark_global_cooldown(
                    storylet.key,
                    global_ticks,
                    tick,
                );
            }
        }

        if let Some(actor) = actor_id {
            if let Some(actor_ticks) = storylet.cooldowns.per_actor_cooldown_ticks {
                if actor_ticks > 0 {
                    self.state.cooldowns.mark_actor_cooldown(
                        storylet.key,
                        actor,
                        actor_ticks,
                        tick,
                    );
                }
            }
        }

        // Update narrative heat via pacing engine
        pacing::on_event_fired(&mut self.state, &self.config.pacing, storylet.heat as f32);
    }

    /// Select a storylet with full scoring breakdown.
    ///
    /// This method provides maximum visibility into the selection process:
    /// 1. Runs the eligibility pipeline to find candidates
    /// 2. Scores all candidates with detailed breakdown
    /// 3. Performs weighted selection with deterministic tie-breaking
    ///
    /// Returns `ScoringResults` containing:
    /// - All scored candidates
    /// - Viable candidates (above minimum threshold)
    /// - The selected candidate (if any)
    /// - Statistics about the scoring process
    ///
    /// Use this when you need to understand why a particular storylet was chosen,
    /// or for debugging content balance issues.
    pub fn select_with_breakdown<'a>(
        &'a self,
        world: &'a WorldState,
        memory: &'a MemorySystem,
    ) -> ScoringResults {
        let ctx = EligibilityContext {
            world,
            memory,
            current_tick: self.state.tick,
        };
        
        // Run the eligibility pipeline
        let candidate_set = self.compute_candidates(&ctx);
        
        // Get storylet references for eligible candidates
        let storylets: Vec<&CompiledStorylet> = candidate_set.after_pacing_filter
            .iter()
            .filter_map(|key| self.storylets.get_storylet_by_key(*key))
            .collect();
        
        // Score and select
        let engine = ScoringEngine::new(&self.config.scoring, &self.config.pacing, &self.state, world.seed.0);
        engine.score_and_select(&storylets, world)
    }

    /// Select a storylet with breakdown, returning just the selected candidate.
    ///
    /// Convenience method that wraps `select_with_breakdown` and extracts
    /// just the selected candidate.
    pub fn select_storylet_scored<'a>(
        &'a self,
        world: &'a WorldState,
        memory: &'a MemorySystem,
    ) -> Option<ScoredCandidate> {
        self.select_with_breakdown(world, memory).selected
    }

    /// Select an event, checking the queue first.
    ///
    /// This method integrates the queue into the selection flow:
    /// 1. Check for forced queued events (fire immediately, bypass normal selection)
    /// 2. If no forced events, proceed with normal selection
    /// 3. Queued non-forced events compete with fresh candidates
    ///
    /// Returns the selected storylet key (if any) along with whether it was from the queue.
    pub fn select_with_queue<'a>(
        &'a mut self,
        world: &'a WorldState,
        memory: &'a MemorySystem,
    ) -> SelectionResult {
        // First, check for forced events
        let forced_events = self.state.pending_queue.pop_forced_ready(self.state.tick);
        if !forced_events.is_empty() {
            // Return the first forced event (highest priority)
            let event = &forced_events[0];
            return SelectionResult {
                selected_key: Some(event.storylet_key),
                from_queue: true,
                forced: true,
                source: Some(event.source),
            };
        }

        // Get regular queued events (non-forced)
        let queued_events = self.state.pending_queue.pop_ready(self.state.tick);
        
        // If we have queued events, they compete with fresh candidates
        if !queued_events.is_empty() {
            // For now, return the highest-priority queued event
            // In a more sophisticated implementation, queued events would be scored
            // alongside fresh candidates
            let event = &queued_events[0];
            return SelectionResult {
                selected_key: Some(event.storylet_key),
                from_queue: true,
                forced: false,
                source: Some(event.source),
            };
        }

        // No queued events, proceed with normal selection
        let selected = self.select_storylet_scored(world, memory);
        SelectionResult {
            selected_key: selected.map(|s| s.key),
            from_queue: false,
            forced: false,
            source: None,
        }
    }

    /// Get the storylet ID string for a key (useful for debugging/logging).
    pub fn storylet_id_for_key(&self, key: StoryletKey) -> Option<StoryletId> {
        self.storylets.get_storylet_by_key(key).map(|s| s.id.clone())
    }
    
    // =========================================================================
    // Step API - Primary Pipeline Entry Point
    // =========================================================================
    
    /// Execute one director step, selecting a storylet if possible.
    ///
    /// This is the primary entry point for the simulation engine. It wraps
    /// the entire Phase 2 pipeline into a single deterministic call:
    ///
    /// 1. Updates director time, pacing, and pressures
    /// 2. Drains ready queued events
    /// 3. Runs the eligibility pipeline for fresh candidates
    /// 4. Combines queued and fresh candidates
    /// 5. Scores and selects a storylet deterministically
    /// 6. Updates state (heat, cooldowns, last_fired, pressures/milestones)
    /// 7. Returns a compact result for the simulation engine to apply
    ///
    /// # Arguments
    /// - `tick`: The current simulation tick
    /// - `ctx`: Eligibility context containing world state and memory
    ///
    /// # Returns
    /// A `DirectorStepResult` containing:
    /// - The fired storylet (if any) with scoring breakdown
    /// - Statistics about the step
    ///
    /// # Determinism
    /// Given the same world seed, tick, and state, this method will always
    /// return the same result. All randomness is derived from the world seed.
    ///
    /// # Example
    /// ```ignore
    /// let ctx = EligibilityContext::new(&world, &memory, tick);
    /// let result = director.step(tick, &ctx);
    ///
    /// if let Some(fired) = result.fired {
    ///     // Apply storylet outcomes...
    ///     apply_outcomes(&mut world, &storylets.get(fired.key));
    /// }
    /// ```
    pub fn step<'a>(
        &mut self, 
        tick: SimTick, 
        ctx: &'a EligibilityContext<'a>
    ) -> crate::api::DirectorStepResult {
        use crate::api::{DirectorStepResult, FiredStorylet, StepStats};
        use crate::pipeline::EligibilityPipeline;
        use crate::pressure;
        use crate::scoring::ScoringEngine;
        
        // 1. Update tick and pacing (on_tick_advance calls pacing::on_tick_start internally)
        self.on_tick_advance(tick);
        
        // 2. Tick pressures and milestones
        pressure::tick_pressures(&mut self.state, &self.config.pressure, tick);
        
        // 3. Check for pressure crises and queue forced events
        let crisis_events = pressure::check_pressure_crises(&self.state, &self.config.pressure, tick);
        for event in crisis_events {
            self.state.pending_queue.push(event, self.config.queue.max_size);
        }
        
        // 4. Check for milestone climaxes and queue events
        let climax_events = pressure::check_milestone_climaxes(&mut self.state.milestones, tick);
        for event in climax_events {
            self.state.pending_queue.push(event, self.config.queue.max_size);
        }
        
        // 5. Dequeue ready events
        let ready_from_queue = self.state.pending_queue.pop_ready(tick);
        
        // 6. Run the eligibility pipeline for fresh candidates
        let pipeline = EligibilityPipeline::new(&self.storylets, &self.state, &self.config);
        let candidate_set = pipeline.run(ctx);
        let fresh_keys = candidate_set.final_candidates();
        
        // 7. Merge queued and fresh candidates
        let mut merged_keys: Vec<StoryletKey> = Vec::with_capacity(
            ready_from_queue.len() + fresh_keys.len()
        );
        
        // Add queue storylets (they have priority in scoring)
        for qe in &ready_from_queue {
            if !merged_keys.contains(&qe.storylet_key) {
                merged_keys.push(qe.storylet_key);
            }
        }
        
        // Add fresh candidates not already in queue
        for &key in fresh_keys {
            if !merged_keys.contains(&key) {
                merged_keys.push(key);
            }
        }
        
        // 8. Early exit if no candidates
        if merged_keys.is_empty() {
            return DirectorStepResult {
                fired: None,
                #[cfg(feature = "debug_candidates")]
                debug_candidates: None,
                stats: StepStats {
                    queue_ready_count: ready_from_queue.len(),
                    fresh_candidate_count: fresh_keys.len(),
                    merged_candidate_count: 0,
                    viable_candidate_count: 0,
                    narrative_heat: self.state.narrative_heat,
                    narrative_phase: format!("{:?}", self.state.narrative_phase),
                },
            };
        }
        
        // 9. Get compiled storylets for scoring
        let storylets: Vec<_> = merged_keys.iter()
            .filter_map(|&key| self.storylets.get_storylet_by_key(key))
            .collect();
        
        // 10. Score candidates
        let scoring_engine = ScoringEngine::new(
            &self.config.scoring,
            &self.config.pacing,
            &self.state,
            ctx.world.seed.0,
        );
        let scoring_results = scoring_engine.score_and_select(&storylets, ctx.world);
        
        // 11. Pick the winning candidate
        let selected = match &scoring_results.selected {
            Some(candidate) => candidate.clone(),
            None => {
                // No viable candidates
                return DirectorStepResult {
                    fired: None,
                    #[cfg(feature = "debug_candidates")]
                    debug_candidates: Some(scoring_results.candidates),
                    stats: StepStats {
                        queue_ready_count: ready_from_queue.len(),
                        fresh_candidate_count: fresh_keys.len(),
                        merged_candidate_count: merged_keys.len(),
                        viable_candidate_count: scoring_results.stats.viable_count,
                        narrative_heat: self.state.narrative_heat,
                        narrative_phase: format!("{:?}", self.state.narrative_phase),
                    },
                };
            }
        };
        
        let chosen_key = selected.key;
        
        // 12. Get the compiled storylet for state updates
        let compiled = match self.storylets.get_storylet_by_key(chosen_key) {
            Some(s) => s,
            None => {
                // Shouldn't happen, but handle gracefully
                return DirectorStepResult::none();
            }
        };
        
        // Cache values we need before mutable borrows
        let storylet_heat = compiled.heat as f32;
        let storylet_domain = compiled.domain;
        let storylet_tags = compiled.tags.clone();
        let global_cooldown = compiled.cooldowns.global_cooldown_ticks;
        
        // 13. Update state for the fired event
        
        // 13a. Update narrative heat
        pacing::on_event_fired(&mut self.state, &self.config.pacing, storylet_heat);
        
        // 13b. Record when this storylet last fired
        self.state.last_fired.record_fired(
            chosen_key,
            storylet_domain,
            &storylet_tags,
            tick,
        );
        
        // 13c. Apply cooldowns
        if let Some(cooldown_ticks) = global_cooldown {
            self.state.cooldowns.mark_global_cooldown(chosen_key, cooldown_ticks, tick);
        }
        
        // 13d. Update milestone progress
        pressure::update_milestone_progress(
            &mut self.state,
            &self.config.milestone,
            chosen_key,
            storylet_domain,
            &storylet_tags,
            tick,
        );
        
        // 13e. Check if this storylet resolves any pressures
        let pressures_to_resolve: Vec<_> = self.state.active_pressures
            .active_pressures()
            .filter(|p| p.resolution_storylet == Some(chosen_key))
            .map(|p| p.id)
            .collect();
        for pressure_id in pressures_to_resolve {
            pressure::resolve_pressure(&mut self.state, pressure_id);
        }
        
        // 14. Determine if this came from the queue
        let is_from_queue = ready_from_queue.iter().any(|qe| qe.storylet_key == chosen_key);
        let queue_source = ready_from_queue
            .iter()
            .find(|qe| qe.storylet_key == chosen_key)
            .map(|qe| qe.source);
        
        // 15. Build the result
        let fired = FiredStorylet::new(
            chosen_key,
            selected,
            is_from_queue,
            queue_source,
        );
        
        let stats = StepStats {
            queue_ready_count: ready_from_queue.len(),
            fresh_candidate_count: fresh_keys.len(),
            merged_candidate_count: merged_keys.len(),
            viable_candidate_count: scoring_results.stats.viable_count,
            narrative_heat: self.state.narrative_heat,
            narrative_phase: format!("{:?}", self.state.narrative_phase),
        };
        
        DirectorStepResult::with_fired(fired, stats)
    }
}

/// Result of the selection process, including queue information.
#[derive(Debug, Clone)]
pub struct SelectionResult {
    /// The selected storylet key, if any.
    pub selected_key: Option<StoryletKey>,
    /// Whether the selection came from the queue.
    pub from_queue: bool,
    /// Whether the selected event was forced.
    pub forced: bool,
    /// The source of the queued event (if from queue).
    pub source: Option<QueueSource>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn_storylets::library::StoryletLibrary;

    fn create_test_library() -> StoryletLibrary {
        StoryletLibrary::new()
    }

    #[test]
    fn test_director_new_initializes_defaults() {
        let library = create_test_library();
        let config = DirectorConfig::default();
        let director = CompiledEventDirector::new(library, config);

        assert_eq!(director.state().tick.0, 0);
        assert_eq!(director.state().narrative_heat, 0.0);
        assert_eq!(director.state().narrative_phase, NarrativePhase::LowKey);
        assert!(director.state().pending_queue.is_empty());
    }

    #[test]
    fn test_director_with_defaults() {
        let library = create_test_library();
        let director = CompiledEventDirector::with_defaults(library);

        assert_eq!(director.config().base_heat_target, 30.0);
        assert_eq!(director.state().tick.0, 0);
    }

    #[test]
    fn test_director_for_testing() {
        let library = create_test_library();
        let director = CompiledEventDirector::for_testing(library);

        assert_eq!(director.config().min_ticks_between_events, 0);
        assert_eq!(director.config().pacing.min_phase_duration, 1);
    }

    #[test]
    fn test_on_tick_advance_updates_tick() {
        let library = create_test_library();
        let mut director = CompiledEventDirector::with_defaults(library);

        assert_eq!(director.state().tick.0, 0);

        director.on_tick_advance(SimTick::new(10));
        assert_eq!(director.state().tick.0, 10);

        director.on_tick_advance(SimTick::new(25));
        assert_eq!(director.state().tick.0, 25);
    }

    #[test]
    fn test_on_tick_advance_preserves_other_state() {
        let library = create_test_library();
        let mut director = CompiledEventDirector::with_defaults(library);

        // Modify some state
        director.state_mut().narrative_heat = 50.0;
        director.state_mut().narrative_phase = NarrativePhase::Peak;

        // Advance tick
        director.on_tick_advance(SimTick::new(100));

        // Tick should advance
        assert_eq!(director.state().tick.0, 100);
        // Heat should decay (per pacing engine), not stay the same
        let expected_heat = 50.0 - director.config().pacing.heat_decay_per_tick;
        assert!((director.state().narrative_heat - expected_heat).abs() < 0.001);
        // Phase should still be Peak (we're above peak threshold)
        assert_eq!(director.state().narrative_phase, NarrativePhase::Peak);
    }

    #[test]
    fn test_state_mut_allows_modification() {
        let library = create_test_library();
        let mut director = CompiledEventDirector::with_defaults(library);

        director.state_mut().narrative_heat = 75.0;
        director.state_mut().narrative_phase = NarrativePhase::Rising;

        assert_eq!(director.state().narrative_heat, 75.0);
        assert_eq!(director.state().narrative_phase, NarrativePhase::Rising);
    }

    #[test]
    fn test_compute_heat_alignment_lowkey() {
        let library = create_test_library();
        let director = CompiledEventDirector::with_defaults(library);

        // Create a mock low-heat storylet
        let mut low_heat = create_mock_storylet();
        low_heat.heat = 2;

        let mut high_heat = create_mock_storylet();
        high_heat.heat = 9;

        // In LowKey phase, low heat should score better
        let low_score = director.compute_heat_alignment(&low_heat);
        let high_score = director.compute_heat_alignment(&high_heat);

        assert!(low_score > high_score);
    }

    fn create_mock_storylet() -> CompiledStorylet {
        use syn_storylets::{Cooldowns, LifeStage, Outcome, Prerequisites, StoryDomain};
        
        CompiledStorylet {
            id: StoryletId::new("test.storylet"),
            key: StoryletKey(0),
            name: "Test Storylet".to_string(),
            description: None,
            tags: vec![],
            domain: StoryDomain::Romance,
            life_stage: LifeStage::Adult,
            heat: 5,
            weight: 1.0,
            roles: vec![],
            prerequisites: Prerequisites::default(),
            cooldowns: Cooldowns::default(),
            outcomes: Outcome::default(),
            follow_ups_resolved: vec![],
        }
    }

    // =========================================================================
    // Queue Integration Tests
    // =========================================================================

    #[test]
    fn test_schedule_follow_up() {
        let library = create_test_library();
        let mut director = CompiledEventDirector::for_testing(library);
        
        director.on_tick_advance(SimTick::new(100));
        director.schedule_follow_up(StoryletKey(42), 10, 5, false);

        assert_eq!(director.queue_len(), 1);
        
        // Event should not be ready yet
        assert!(!director.has_ready_events());
        
        // Advance to the scheduled tick
        director.on_tick_advance(SimTick::new(110));
        assert!(director.has_ready_events());
        
        let ready = director.dequeue_ready_events();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].storylet_key, StoryletKey(42));
    }

    #[test]
    fn test_schedule_immediate_follow_up() {
        let library = create_test_library();
        let mut director = CompiledEventDirector::for_testing(library);
        
        director.on_tick_advance(SimTick::new(50));
        director.schedule_immediate_follow_up(StoryletKey(99), true);

        assert_eq!(director.queue_len(), 1);
        
        // Should be ready next tick
        director.on_tick_advance(SimTick::new(51));
        assert!(director.has_forced_ready());
        
        let forced = director.dequeue_forced_events();
        assert_eq!(forced.len(), 1);
        assert_eq!(forced[0].storylet_key, StoryletKey(99));
        assert!(forced[0].forced);
    }

    #[test]
    fn test_forced_events_dequeue_separately() {
        let library = create_test_library();
        let mut director = CompiledEventDirector::for_testing(library);
        
        director.on_tick_advance(SimTick::new(100));
        
        // Schedule both forced and non-forced events for the same tick
        director.schedule_follow_up(StoryletKey(1), 0, 10, true);  // Forced, high priority
        director.schedule_follow_up(StoryletKey(2), 0, 20, false); // Not forced, higher priority
        
        director.on_tick_advance(SimTick::new(100));
        
        // Forced events should be available
        assert!(director.has_forced_ready());
        
        // Dequeue forced first
        let forced = director.dequeue_forced_events();
        assert_eq!(forced.len(), 1);
        assert_eq!(forced[0].storylet_key, StoryletKey(1));
        
        // Non-forced should still be in queue
        assert!(director.has_ready_events());
        let ready = director.dequeue_ready_events();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].storylet_key, StoryletKey(2));
    }

    #[test]
    fn test_queue_respects_max_size() {
        let library = create_test_library();
        let mut config = DirectorConfig::for_testing();
        config.queue.max_size = 3;
        let mut director = CompiledEventDirector::new(library, config);
        
        director.on_tick_advance(SimTick::new(0));
        
        // Schedule more events than max_size allows
        director.schedule_follow_up(StoryletKey(1), 10, 5, false);
        director.schedule_follow_up(StoryletKey(2), 20, 5, false);
        director.schedule_follow_up(StoryletKey(3), 30, 5, false);
        director.schedule_follow_up(StoryletKey(4), 40, 5, false); // Should evict
        
        // Should only have max_size events
        assert_eq!(director.queue_len(), 3);
    }

    #[test]
    fn test_schedule_milestone() {
        let library = create_test_library();
        let mut director = CompiledEventDirector::for_testing(library);
        
        director.schedule_milestone(StoryletKey(77), SimTick::new(500), 10);
        
        assert_eq!(director.queue_len(), 1);
        
        // Not ready until tick 500
        director.on_tick_advance(SimTick::new(499));
        assert!(!director.has_ready_events());
        
        director.on_tick_advance(SimTick::new(500));
        assert!(director.has_ready_events());
    }

    #[test]
    fn test_schedule_pressure_relief() {
        let library = create_test_library();
        let mut director = CompiledEventDirector::for_testing(library);
        
        director.schedule_pressure_relief(StoryletKey(88), SimTick::new(200), 5, true);
        
        assert_eq!(director.queue_len(), 1);
        
        director.on_tick_advance(SimTick::new(200));
        assert!(director.has_forced_ready());
        
        let events = director.dequeue_forced_events();
        assert_eq!(events[0].source, QueueSource::PressureRelief);
    }

    #[test]
    fn test_dequeue_preserves_order() {
        let library = create_test_library();
        let mut director = CompiledEventDirector::for_testing(library);
        
        director.on_tick_advance(SimTick::new(0));
        
        // Schedule events with different priorities, all for same tick
        director.schedule_follow_up(StoryletKey(3), 10, 1, false);  // Low priority
        director.schedule_follow_up(StoryletKey(1), 10, 10, false); // High priority
        director.schedule_follow_up(StoryletKey(2), 10, 5, false);  // Medium priority
        
        director.on_tick_advance(SimTick::new(10));
        let ready = director.dequeue_ready_events();
        
        // Should be ordered by priority (descending)
        assert_eq!(ready[0].storylet_key, StoryletKey(1)); // priority 10
        assert_eq!(ready[1].storylet_key, StoryletKey(2)); // priority 5
        assert_eq!(ready[2].storylet_key, StoryletKey(3)); // priority 1
    }
}
