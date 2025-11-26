//! Layered eligibility pipeline for storylet filtering.
//!
//! This module implements a multi-stage filtering pipeline that cleanly separates
//! different types of eligibility checks. Each stage progressively narrows down
//! the candidate set, making it easy to debug and tune filtering behavior.
//!
//! # Pipeline Stages
//!
//! 1. **Index Prefilter**: Uses tag/domain/life_stage indexes for fast initial filtering
//! 2. **Structural Prerequisites**: Stats, traits, relationships, world flags, memory
//! 3. **Cooldown & Repetition**: Cooldowns and variety constraints
//! 4. **Pacing Filter**: Phase-based heat constraints
//!
//! # Example
//!
//! ```ignore
//! let pipeline = EligibilityPipeline::new(&storylets, &state, &config);
//! let candidates = pipeline.run(&ctx);
//! println!("After index: {} candidates", candidates.after_index_prefilter.len());
//! println!("After prereq: {} candidates", candidates.after_prereq_filter.len());
//! println!("After cooldown: {} candidates", candidates.after_cooldown_filter.len());
//! println!("After pacing: {} candidates", candidates.after_pacing_filter.len());
//! ```

use crate::config::DirectorConfig;
use crate::eligibility::EligibilityContext;
use crate::pacing;
use crate::state::DirectorState;
use crate::storylet_source::StoryletSource;
use crate::EligibilityEngine;
use syn_core::SimTick;
use syn_storylets::library::{CompiledStorylet, StoryletKey};
use syn_storylets::{LifeStage, StoryDomain, Tag};
use serde::{Deserialize, Serialize};

/// Tracks candidates through each stage of the eligibility pipeline.
///
/// This struct provides full visibility into how the candidate set evolves
/// through each filtering stage, enabling debugging and tuning.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CandidateSet {
    /// All storylets before any filtering (from initial query).
    pub initial: Vec<StoryletKey>,
    
    /// After Stage 1: Index-based prefiltering (tags, domain, life_stage).
    pub after_index_prefilter: Vec<StoryletKey>,
    
    /// After Stage 2: Structural prerequisite checks (stats, traits, relationships, etc).
    pub after_prereq_filter: Vec<StoryletKey>,
    
    /// After Stage 3: Cooldown and repetition/variety filtering.
    pub after_cooldown_filter: Vec<StoryletKey>,
    
    /// After Stage 4: Pacing constraints (narrative phase, heat limits).
    pub after_pacing_filter: Vec<StoryletKey>,
}

impl CandidateSet {
    /// Create a new empty CandidateSet.
    pub fn new() -> Self {
        CandidateSet::default()
    }

    /// Get the final set of eligible candidates (after all filters).
    pub fn final_candidates(&self) -> &[StoryletKey] {
        &self.after_pacing_filter
    }

    /// Check if any candidates remain after all filtering.
    pub fn has_candidates(&self) -> bool {
        !self.after_pacing_filter.is_empty()
    }

    /// Get statistics about how many candidates were filtered at each stage.
    pub fn filter_stats(&self) -> PipelineStats {
        PipelineStats {
            initial_count: self.initial.len(),
            after_index: self.after_index_prefilter.len(),
            after_prereq: self.after_prereq_filter.len(),
            after_cooldown: self.after_cooldown_filter.len(),
            after_pacing: self.after_pacing_filter.len(),
        }
    }
}

/// Statistics about filtering at each pipeline stage.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PipelineStats {
    pub initial_count: usize,
    pub after_index: usize,
    pub after_prereq: usize,
    pub after_cooldown: usize,
    pub after_pacing: usize,
}

impl PipelineStats {
    /// Calculate how many candidates were filtered at each stage.
    pub fn filtered_at_each_stage(&self) -> (usize, usize, usize, usize) {
        (
            self.initial_count.saturating_sub(self.after_index),
            self.after_index.saturating_sub(self.after_prereq),
            self.after_prereq.saturating_sub(self.after_cooldown),
            self.after_cooldown.saturating_sub(self.after_pacing),
        )
    }
}

/// Optional filters for the index prefilter stage.
///
/// These allow callers to narrow down the initial candidate set
/// based on context-specific requirements.
#[derive(Debug, Clone, Default)]
pub struct IndexPrefilterParams {
    /// If set, only include storylets with ALL of these tags.
    pub required_tags: Vec<Tag>,
    
    /// If set, only include storylets in one of these domains.
    pub allowed_domains: Option<Vec<StoryDomain>>,
    
    /// If set, override the life stage filter (otherwise uses player's life stage).
    pub life_stage_override: Option<LifeStage>,
}

/// The multi-stage eligibility pipeline.
///
/// Runs storylets through progressive filtering stages:
/// 1. Index prefilter (fast, uses library indexes)
/// 2. Structural prerequisites (detailed condition checks)
/// 3. Cooldown & repetition (uses DirectorState)
/// 4. Pacing constraints (narrative phase alignment)
pub struct EligibilityPipeline<'a, S: StoryletSource> {
    /// Reference to the storylet library.
    pub storylets: &'a S,
    
    /// Reference to the current director state.
    pub state: &'a DirectorState,
    
    /// Reference to the director configuration.
    pub config: &'a DirectorConfig,
}

impl<'a, S: StoryletSource> EligibilityPipeline<'a, S> {
    /// Create a new eligibility pipeline.
    pub fn new(
        storylets: &'a S,
        state: &'a DirectorState,
        config: &'a DirectorConfig,
    ) -> Self {
        EligibilityPipeline {
            storylets,
            state,
            config,
        }
    }

    /// Run the full eligibility pipeline.
    ///
    /// Executes all four stages in order, populating the CandidateSet
    /// with results from each stage for debugging visibility.
    pub fn run(&self, ctx: &EligibilityContext<'_>) -> CandidateSet {
        self.run_with_params(ctx, &IndexPrefilterParams::default())
    }

    /// Run the pipeline with custom index prefilter parameters.
    pub fn run_with_params(
        &self,
        ctx: &EligibilityContext<'_>,
        params: &IndexPrefilterParams,
    ) -> CandidateSet {
        let mut candidates = CandidateSet::new();

        // Stage 1: Index Prefilter
        candidates.initial = self.collect_initial_candidates(ctx, params);
        candidates.after_index_prefilter = self.apply_index_prefilter(
            &candidates.initial,
            ctx,
            params,
        );

        // Stage 2: Structural Prerequisites
        candidates.after_prereq_filter = self.apply_prereq_filter(
            &candidates.after_index_prefilter,
            ctx,
        );

        // Stage 3: Cooldown & Repetition
        candidates.after_cooldown_filter = self.apply_cooldown_filter(
            &candidates.after_prereq_filter,
        );

        // Stage 4: Pacing Constraints
        candidates.after_pacing_filter = self.apply_pacing_filter(
            &candidates.after_cooldown_filter,
        );

        candidates
    }

    // =========================================================================
    // Stage 1: Index Prefilter
    // =========================================================================

    /// Collect initial candidates using library indexes.
    ///
    /// Uses life_stage index as the primary filter, then applies
    /// optional tag and domain constraints.
    fn collect_initial_candidates(
        &self,
        ctx: &EligibilityContext<'_>,
        params: &IndexPrefilterParams,
    ) -> Vec<StoryletKey> {
        // Determine life stage to filter by
        let life_stage = params.life_stage_override.unwrap_or_else(|| {
            convert_core_life_stage_to_storylet(ctx.world.player_life_stage)
        });

        // Get all candidates for this life stage
        self.storylets.candidates_for_life_stage(life_stage)
    }

    /// Apply index-level prefiltering (tags, domains).
    fn apply_index_prefilter(
        &self,
        candidates: &[StoryletKey],
        _ctx: &EligibilityContext<'_>,
        params: &IndexPrefilterParams,
    ) -> Vec<StoryletKey> {
        let mut filtered = candidates.to_vec();

        // Filter by required tags (all must be present)
        if !params.required_tags.is_empty() {
            filtered.retain(|&key| {
                if let Some(storylet) = self.storylets.get_storylet_by_key(key) {
                    params.required_tags.iter().all(|required_tag| {
                        storylet.tags.contains(required_tag)
                    })
                } else {
                    false
                }
            });
        }

        // Filter by allowed domains
        if let Some(ref allowed_domains) = params.allowed_domains {
            filtered.retain(|&key| {
                if let Some(storylet) = self.storylets.get_storylet_by_key(key) {
                    allowed_domains.contains(&storylet.domain)
                } else {
                    false
                }
            });
        }

        filtered
    }

    // =========================================================================
    // Stage 2: Structural Prerequisites
    // =========================================================================

    /// Apply structural prerequisite filtering.
    ///
    /// Uses the EligibilityEngine to check stats, traits, relationships,
    /// world flags, and memory prerequisites.
    fn apply_prereq_filter(
        &self,
        candidates: &[StoryletKey],
        ctx: &EligibilityContext<'_>,
    ) -> Vec<StoryletKey> {
        let engine = EligibilityEngine::new(self.storylets);
        
        candidates
            .iter()
            .copied()
            .filter(|&key| {
                if let Some(storylet) = self.storylets.get_storylet_by_key(key) {
                    engine.is_storylet_eligible_public(storylet, ctx)
                } else {
                    false
                }
            })
            .collect()
    }

    // =========================================================================
    // Stage 3: Cooldown & Repetition
    // =========================================================================

    /// Apply cooldown and variety constraint filtering.
    ///
    /// Filters out storylets that:
    /// - Are on global cooldown
    /// - Were fired too recently (variety constraints)
    /// - Would violate domain/tag repetition limits
    fn apply_cooldown_filter(&self, candidates: &[StoryletKey]) -> Vec<StoryletKey> {
        let current_tick = self.state.tick;

        candidates
            .iter()
            .copied()
            .filter(|&key| {
                // Check global cooldown
                if !self.state.cooldowns.is_globally_ready(key, current_tick) {
                    return false;
                }

                // Check storylet repetition interval
                if self.config.variety.min_storylet_repeat_interval > 0 {
                    let since = SimTick::new(
                        current_tick.0.saturating_sub(self.config.variety.min_storylet_repeat_interval)
                    );
                    if self.state.last_fired.storylet_fired_since(key, since) {
                        return false;
                    }
                }

                // Check domain repetition if storylet lookup succeeds
                if let Some(storylet) = self.storylets.get_storylet_by_key(key) {
                    if self.config.variety.min_domain_repeat_interval > 0 {
                        let since = SimTick::new(
                            current_tick.0.saturating_sub(self.config.variety.min_domain_repeat_interval)
                        );
                        if self.state.last_fired.domain_fired_since(storylet.domain, since) {
                            return false;
                        }
                    }

                    // Check tag repetition
                    if self.config.variety.min_tag_repeat_interval > 0 {
                        let since = SimTick::new(
                            current_tick.0.saturating_sub(self.config.variety.min_tag_repeat_interval)
                        );
                        for tag in &storylet.tags {
                            if self.state.last_fired.tag_fired_since(tag, since) {
                                return false;
                            }
                        }
                    }
                }

                true
            })
            .collect()
    }

    // =========================================================================
    // Stage 4: Pacing Filter
    // =========================================================================

    /// Apply pacing constraints based on narrative phase.
    ///
    /// Filters storylets based on whether their heat level is appropriate
    /// for the current narrative phase.
    fn apply_pacing_filter(&self, candidates: &[StoryletKey]) -> Vec<StoryletKey> {
        candidates
            .iter()
            .copied()
            .filter(|&key| {
                if let Some(storylet) = self.storylets.get_storylet_by_key(key) {
                    passes_pacing_constraints(self.state, self.config, storylet)
                } else {
                    false
                }
            })
            .collect()
    }
}

/// Check if a storylet passes pacing constraints for the current narrative phase.
///
/// This function uses the pacing module's `is_heat_appropriate` function to determine
/// whether a storylet's heat level is suitable for the current phase:
/// - LowKey: Prefer low-heat storylets, block very high heat unless pressured
/// - Rising: Allow medium-heat storylets
/// - Peak: Prefer high-heat storylets, allow all
/// - Fallout: Allow medium-high heat (consequences)
/// - Recovery: Prefer low-medium heat (healing)
pub fn passes_pacing_constraints(
    state: &DirectorState,
    _config: &DirectorConfig,
    storylet: &CompiledStorylet,
) -> bool {
    let heat = storylet.heat as f32;
    let has_active_pressures = state.active_pressures.has_active_pressures();
    
    // Check if storylet has a "forced" tag (bypass pacing)
    let is_forced = storylet.tags.iter().any(|t| t.0.as_str() == "forced");
    
    pacing::is_heat_appropriate(state, heat, has_active_pressures, is_forced)
}

/// Convert from syn_core::LifeStage to syn_storylets::LifeStage.
fn convert_core_life_stage_to_storylet(core_stage: syn_core::LifeStage) -> LifeStage {
    match core_stage {
        syn_core::LifeStage::PreSim => LifeStage::Child,
        syn_core::LifeStage::Child => LifeStage::Child,
        syn_core::LifeStage::Teen => LifeStage::Teen,
        syn_core::LifeStage::YoungAdult => LifeStage::YoungAdult,
        syn_core::LifeStage::Adult => LifeStage::Adult,
        syn_core::LifeStage::Elder => LifeStage::Elder,
        syn_core::LifeStage::Digital => LifeStage::Digital,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DirectorConfig;
    use crate::state::{DirectorState, NarrativePhase};
    use syn_core::{NpcId, WorldSeed, WorldState};
    use syn_memory::MemorySystem;
    use syn_storylets::library::{StoryletKey, StoryletLibrary};
    use syn_storylets::{Cooldowns, Outcome, Prerequisites, StoryDomain};

    /// Helper to create a test storylet with configurable properties.
    fn create_test_storylet(
        id: &str,
        key: u32,
        domain: StoryDomain,
        life_stage: LifeStage,
        heat: u8,
        tags: Vec<&str>,
    ) -> CompiledStorylet {
        use syn_storylets::StoryletId;
        
        CompiledStorylet {
            id: StoryletId::new(id),
            key: StoryletKey(key),
            name: id.to_string(),
            description: None,
            tags: tags.into_iter().map(Tag::new).collect(),
            domain,
            life_stage,
            heat,
            weight: 1.0,
            roles: vec![],
            prerequisites: Prerequisites::default(),
            cooldowns: Cooldowns::default(),
            outcomes: Outcome::default(),
            follow_ups_resolved: vec![],
        }
    }

    /// Helper to add a storylet to a library (for testing).
    fn add_storylet_to_library(library: &mut StoryletLibrary, storylet: CompiledStorylet) {
        let key = storylet.key;
        
        // Add to id_to_key index
        library.id_to_key.insert(storylet.id.clone(), key);
        
        // Add to life_stage index
        library.life_stage_index
            .entry(storylet.life_stage)
            .or_default()
            .push(key);
        
        // Add to domain index
        library.domain_index
            .entry(storylet.domain)
            .or_default()
            .push(key);
        
        // Add to tag indexes
        for tag in &storylet.tags {
            library.tag_index
                .entry(tag.clone())
                .or_default()
                .push(key);
        }
        
        // Add to storylets vec (ensure it's at the right index)
        while library.storylets.len() <= key.0 as usize {
            // Placeholder - shouldn't happen with proper key assignment
            library.storylets.push(storylet.clone());
        }
        library.storylets[key.0 as usize] = storylet;
        
        library.total_count = library.storylets.len() as u32;
    }

    /// Create a test library with diverse storylets.
    fn create_test_library() -> StoryletLibrary {
        let mut library = StoryletLibrary::new();
        
        // Low heat romance for adults
        add_storylet_to_library(&mut library, create_test_storylet(
            "romance.casual_chat",
            0,
            StoryDomain::Romance,
            LifeStage::Adult,
            2,
            vec!["romance", "casual"],
        ));
        
        // Medium heat romance for adults
        add_storylet_to_library(&mut library, create_test_storylet(
            "romance.first_date",
            1,
            StoryDomain::Romance,
            LifeStage::Adult,
            5,
            vec!["romance", "dating"],
        ));
        
        // High heat conflict for adults
        add_storylet_to_library(&mut library, create_test_storylet(
            "conflict.major_argument",
            2,
            StoryDomain::Conflict,
            LifeStage::Adult,
            8,
            vec!["conflict", "drama"],
        ));
        
        // Low heat career for adults
        add_storylet_to_library(&mut library, create_test_storylet(
            "career.daily_routine",
            3,
            StoryDomain::Career,
            LifeStage::Adult,
            1,
            vec!["career", "routine"],
        ));
        
        // Teen-only storylet
        add_storylet_to_library(&mut library, create_test_storylet(
            "school.homework",
            4,
            StoryDomain::Career,
            LifeStage::Teen,
            2,
            vec!["school", "teen"],
        ));
        
        library
    }

    fn create_test_context<'a>(
        world: &'a WorldState,
        memory: &'a MemorySystem,
        tick: SimTick,
    ) -> EligibilityContext<'a> {
        EligibilityContext {
            world,
            memory,
            current_tick: tick,
        }
    }

    /// Create a world with Adult life stage for testing.
    fn create_adult_world() -> WorldState {
        let mut world = WorldState::new(WorldSeed(42), NpcId(1));
        world.player_life_stage = syn_core::LifeStage::Adult;
        world
    }

    #[test]
    fn test_pipeline_basic_flow() {
        let library = create_test_library();
        let state = DirectorState::new();
        let config = DirectorConfig::for_testing();
        
        let world = create_adult_world();
        let memory = MemorySystem::new();
        let ctx = create_test_context(&world, &memory, SimTick::new(0));
        
        let pipeline = EligibilityPipeline::new(&library, &state, &config);
        let candidates = pipeline.run(&ctx);
        
        // Should have adult storylets (4 adult storylets in library)
        assert!(!candidates.initial.is_empty());
        assert!(candidates.after_index_prefilter.len() <= candidates.initial.len());
    }

    #[test]
    fn test_life_stage_filtering() {
        let library = create_test_library();
        let state = DirectorState::new();
        let config = DirectorConfig::for_testing();
        
        // Create world with Teen player
        let mut world = WorldState::new(WorldSeed(42), NpcId(1));
        world.player_life_stage = syn_core::LifeStage::Teen;
        let memory = MemorySystem::new();
        let ctx = create_test_context(&world, &memory, SimTick::new(0));
        
        let pipeline = EligibilityPipeline::new(&library, &state, &config);
        let candidates = pipeline.run(&ctx);
        
        // Should only have teen storylet
        assert_eq!(candidates.initial.len(), 1);
        assert_eq!(candidates.initial[0], StoryletKey(4)); // school.homework
    }

    #[test]
    fn test_domain_filtering() {
        let library = create_test_library();
        let state = DirectorState::new();
        let config = DirectorConfig::for_testing();
        
        let world = create_adult_world();
        let memory = MemorySystem::new();
        let ctx = create_test_context(&world, &memory, SimTick::new(0));
        
        let params = IndexPrefilterParams {
            allowed_domains: Some(vec![StoryDomain::Romance]),
            ..Default::default()
        };
        
        let pipeline = EligibilityPipeline::new(&library, &state, &config);
        let candidates = pipeline.run_with_params(&ctx, &params);
        
        // Should only have romance storylets (2 of them)
        assert_eq!(candidates.after_index_prefilter.len(), 2);
    }

    #[test]
    fn test_tag_filtering() {
        let library = create_test_library();
        let state = DirectorState::new();
        let config = DirectorConfig::for_testing();
        
        let world = create_adult_world();
        let memory = MemorySystem::new();
        let ctx = create_test_context(&world, &memory, SimTick::new(0));
        
        let params = IndexPrefilterParams {
            required_tags: vec![Tag::new("dating")],
            ..Default::default()
        };
        
        let pipeline = EligibilityPipeline::new(&library, &state, &config);
        let candidates = pipeline.run_with_params(&ctx, &params);
        
        // Should only have the first_date storylet
        assert_eq!(candidates.after_index_prefilter.len(), 1);
    }

    #[test]
    fn test_cooldown_filtering() {
        let library = create_test_library();
        let mut state = DirectorState::with_tick(SimTick::new(100));
        let config = DirectorConfig::default(); // Uses real variety settings
        
        // Put storylet 0 on cooldown
        state.cooldowns.mark_global_cooldown(StoryletKey(0), 50, SimTick::new(80));
        
        let world = create_adult_world();
        let memory = MemorySystem::new();
        let ctx = create_test_context(&world, &memory, state.tick);
        
        let pipeline = EligibilityPipeline::new(&library, &state, &config);
        let candidates = pipeline.run(&ctx);
        
        // Storylet 0 should be filtered out due to cooldown (until tick 130)
        assert!(!candidates.after_cooldown_filter.contains(&StoryletKey(0)));
    }

    #[test]
    fn test_pacing_filter_lowkey_blocks_high_heat() {
        let library = create_test_library();
        let mut state = DirectorState::new();
        state.narrative_phase = NarrativePhase::LowKey;
        let config = DirectorConfig::for_testing();
        
        let world = create_adult_world();
        let memory = MemorySystem::new();
        let ctx = create_test_context(&world, &memory, SimTick::new(0));
        
        let pipeline = EligibilityPipeline::new(&library, &state, &config);
        let candidates = pipeline.run(&ctx);
        
        // High heat conflict (heat=8) should be filtered out in LowKey phase
        assert!(!candidates.after_pacing_filter.contains(&StoryletKey(2)));
        
        // Low heat storylets should remain
        assert!(candidates.after_pacing_filter.contains(&StoryletKey(0)) || 
                candidates.after_pacing_filter.contains(&StoryletKey(3)));
    }

    #[test]
    fn test_pacing_filter_peak_allows_high_heat() {
        let library = create_test_library();
        let mut state = DirectorState::new();
        state.narrative_phase = NarrativePhase::Peak;
        state.narrative_heat = 70.0;
        let config = DirectorConfig::for_testing();
        
        let world = create_adult_world();
        let memory = MemorySystem::new();
        let ctx = create_test_context(&world, &memory, SimTick::new(0));
        
        let pipeline = EligibilityPipeline::new(&library, &state, &config);
        let candidates = pipeline.run(&ctx);
        
        // High heat conflict should be allowed in Peak phase
        assert!(candidates.after_pacing_filter.contains(&StoryletKey(2)));
    }

    #[test]
    fn test_candidate_set_stats() {
        let library = create_test_library();
        let mut state = DirectorState::with_tick(SimTick::new(100));
        state.narrative_phase = NarrativePhase::LowKey;
        
        // Add some filtering conditions
        state.cooldowns.mark_global_cooldown(StoryletKey(1), 50, SimTick::new(80));
        
        let config = DirectorConfig::for_testing();
        
        let world = create_adult_world();
        let memory = MemorySystem::new();
        let ctx = create_test_context(&world, &memory, state.tick);
        
        let pipeline = EligibilityPipeline::new(&library, &state, &config);
        let candidates = pipeline.run(&ctx);
        
        let stats = candidates.filter_stats();
        
        // Verify stats are calculated correctly
        assert!(stats.initial_count >= stats.after_index);
        assert!(stats.after_index >= stats.after_prereq);
        assert!(stats.after_prereq >= stats.after_cooldown);
        assert!(stats.after_cooldown >= stats.after_pacing);
    }

    #[test]
    fn test_passes_pacing_constraints_recovery() {
        let mut state = DirectorState::new();
        state.narrative_phase = NarrativePhase::Recovery;
        let config = DirectorConfig::default();
        
        // Very high heat storylet
        let high_heat = create_test_storylet(
            "crisis.explosion",
            99,
            StoryDomain::Conflict,
            LifeStage::Adult,
            9,
            vec!["crisis"],
        );
        
        // Low heat storylet
        let low_heat = create_test_storylet(
            "calm.meditation",
            98,
            StoryDomain::Family,
            LifeStage::Adult,
            2,
            vec!["calm"],
        );
        
        // In Recovery, high heat should be blocked
        assert!(!passes_pacing_constraints(&state, &config, &high_heat));
        
        // Low heat should pass
        assert!(passes_pacing_constraints(&state, &config, &low_heat));
    }

    #[test]
    fn test_variety_domain_filtering() {
        let library = create_test_library();
        let mut state = DirectorState::with_tick(SimTick::new(100));
        let config = DirectorConfig::default();
        
        // Record that Romance domain fired recently
        state.last_fired.last_by_domain.insert(
            StoryDomain::Romance,
            SimTick::new(99), // Just 1 tick ago
        );
        
        let world = create_adult_world();
        let memory = MemorySystem::new();
        let ctx = create_test_context(&world, &memory, state.tick);
        
        let pipeline = EligibilityPipeline::new(&library, &state, &config);
        let candidates = pipeline.run(&ctx);
        
        // Romance storylets should be filtered due to variety constraints
        assert!(!candidates.after_cooldown_filter.contains(&StoryletKey(0)));
        assert!(!candidates.after_cooldown_filter.contains(&StoryletKey(1)));
    }
}
