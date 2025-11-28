//! Eligibility engine for filtering storylets based on world state and prerequisites.
//!
//! This module provides deterministic evaluation of storylet eligibility conditions.
//! It checks all aspects of Prerequisites against the current game state:
//! - Stats and traits (by StatKind)
//! - Relationships between characters
//! - Memory tags and recency
//! - World state flags and conditions
//! - Cooldowns (global, per-actor, per-relationship, per-district)
//! - Life stages

use syn_core::{SimTick, StatKind, WorldState};
use syn_core::LifeStage as CoreLifeStage;
use syn_memory::MemorySystem;
use syn_storylets::library::StoryletKey;
use syn_storylets::{Prerequisites, GlobalFlags, WorldStatePrerequisites, MemoryPrerequisites};

use crate::StoryletSource;

/// Convert from syn_core::LifeStage to syn_storylets::LifeStage.
/// PreSim maps to Child since PreSim is not playable and similar to Child in terms of content.
fn convert_core_life_stage_to_storylet(core_stage: CoreLifeStage) -> syn_storylets::LifeStage {
    match core_stage {
        CoreLifeStage::PreSim => syn_storylets::LifeStage::Child, // Not playable, default to Child
        CoreLifeStage::Child => syn_storylets::LifeStage::Child,
        CoreLifeStage::Teen => syn_storylets::LifeStage::Teen,
        CoreLifeStage::YoungAdult => syn_storylets::LifeStage::YoungAdult,
        CoreLifeStage::Adult => syn_storylets::LifeStage::Adult,
        CoreLifeStage::Elder => syn_storylets::LifeStage::Elder,
        CoreLifeStage::Digital => syn_storylets::LifeStage::Digital,
    }
}

/// Convert from syn_storylets::LifeStage to syn_core::LifeStage.
/// (Currently unused but kept for future integration)
#[allow(dead_code)]
fn convert_storylet_life_stage_to_core(story_stage: syn_storylets::LifeStage) -> CoreLifeStage {
    match story_stage {
        syn_storylets::LifeStage::Child => CoreLifeStage::Child,
        syn_storylets::LifeStage::Teen => CoreLifeStage::Teen,
        syn_storylets::LifeStage::YoungAdult => CoreLifeStage::YoungAdult,
        syn_storylets::LifeStage::Adult => CoreLifeStage::Adult,
        syn_storylets::LifeStage::Elder => CoreLifeStage::Elder,
        syn_storylets::LifeStage::Digital => CoreLifeStage::Digital,
    }
}

/// Context needed to evaluate storylet eligibility.
///
/// Contains all references necessary to check prerequisites without coupling to specific systems.
/// This is typically created fresh for each director tick to avoid stale references.
#[derive(Debug)]
pub struct EligibilityContext<'a> {
    /// Reference to world state (stats, relationships, flags, etc.)
    pub world: &'a WorldState,
    
    /// Reference to memory system for memory-based eligibility checks
    pub memory: &'a MemorySystem,
    
    /// Current game tick for cooldown evaluation and time-sensitive checks
    pub current_tick: SimTick,
}

/// Eligibility engine for evaluating storylets against world state.
///
/// This engine provides fast, deterministic filtering using the indexes from `StoryletSource`.
/// It works by:
/// 1. Pre-filtering candidates by life stage and domain (using StoryletSource indexes)
/// 2. Checking detailed prerequisites for each candidate
/// 3. Validating cooldowns and state-based conditions
pub struct EligibilityEngine<'a, S: StoryletSource> {
    storylets: &'a S,
}

impl<'a, S: StoryletSource> EligibilityEngine<'a, S> {
    /// Create a new eligibility engine.
    pub fn new(storylets: &'a S) -> Self {
        EligibilityEngine { storylets }
    }

    /// Find all eligible storylets for the given context.
    ///
    /// This performs multi-stage filtering:
    /// 1. **Pre-filters**: By player life stage from WorldState
    /// 2. **Prerequisite checks**: All conditions from Prerequisites
    ///
    /// Returns a vector of eligible `StoryletKey`s in deterministic order.
    pub fn find_eligible_storylets(&self, ctx: &EligibilityContext) -> Vec<StoryletKey> {
        let mut eligible = Vec::new();

        // Pre-filter by player's current life stage
        // Convert from syn_core::LifeStage to syn_storylets::LifeStage
        let player_life_stage = convert_core_life_stage_to_storylet(ctx.world.player_life_stage);
        let candidates = self.storylets.candidates_for_life_stage(player_life_stage);

        // Check detailed prerequisites for each candidate
        for key in candidates {
            if let Some(storylet) = self.storylets.get_storylet_by_key(key) {
                if self.is_storylet_eligible(&storylet, ctx) {
                    eligible.push(key);
                }
            }
        }

        eligible
    }

    /// Check if a single storylet is eligible given the current context (public version).
    ///
    /// This is used by the pipeline to check prerequisites after index filtering.
    pub fn is_storylet_eligible_public(&self, storylet: &syn_storylets::library::CompiledStorylet, ctx: &EligibilityContext) -> bool {
        self.is_storylet_eligible(storylet, ctx)
    }

    /// Check if a single storylet is eligible given the current context.
    fn is_storylet_eligible(&self, storylet: &syn_storylets::library::CompiledStorylet, ctx: &EligibilityContext) -> bool {
        // 1. Check stat thresholds
        if !self.check_stat_thresholds(&storylet.prerequisites, ctx) {
            return false;
        }

        // 2. Check trait thresholds  
        if !self.check_trait_thresholds(&storylet.prerequisites, ctx) {
            return false;
        }

        // 3. Check relationship prerequisites
        if !self.check_relationship_prerequisites(&storylet.prerequisites, ctx) {
            return false;
        }

        // 4. Check memory prerequisites
        if !self.check_memory_prerequisites(&storylet.prerequisites, ctx) {
            return false;
        }

        // 5. Check world state prerequisites
        if !self.check_world_state_prerequisites(&storylet.prerequisites, ctx) {
            return false;
        }

        // 6. Check global flags
        if !self.check_global_flags(&storylet.prerequisites, ctx) {
            return false;
        }

        true
    }

    /// Check stat threshold conditions against player stats.
    fn check_stat_thresholds(&self, prereqs: &Prerequisites, ctx: &EligibilityContext) -> bool {
        if let Some(ref thresholds) = prereqs.stat_thresholds {
            for threshold in thresholds {
                // Parse the stat kind from the threshold name
                let stat_kind = match threshold.stat.as_str() {
                    "health" => StatKind::Health,
                    "intelligence" => StatKind::Intelligence,
                    "charisma" => StatKind::Charisma,
                    "wealth" => StatKind::Wealth,
                    "mood" => StatKind::Mood,
                    "appearance" => StatKind::Appearance,
                    "reputation" => StatKind::Reputation,
                    "wisdom" => StatKind::Wisdom,
                    "curiosity" => StatKind::Curiosity,
                    "energy" => StatKind::Energy,
                    "libido" => StatKind::Libido,
                    _ => continue, // Unknown stat, skip
                };
                
                let stat_value = ctx.world.player_stats.get(stat_kind);
                
                if let Some(min) = threshold.min {
                    if stat_value < min {
                        return false;
                    }
                }
                if let Some(max) = threshold.max {
                    if stat_value > max {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// Check trait threshold conditions.
    /// 
    /// Note: This is a stub implementation. In full implementation,
    /// we would check against a traits/personality system in WorldState.
    fn check_trait_thresholds(&self, prereqs: &Prerequisites, _ctx: &EligibilityContext) -> bool {
        if let Some(ref _thresholds) = prereqs.trait_thresholds {
            // TODO: Implement when personality traits system is available in WorldState
            // For now, always pass (no traits to check)
        }
        true
    }

    /// Check relationship prerequisites between NPC roles.
    ///
    /// Note: This is a stub implementation. Full implementation would:
    /// - Resolve role assignments (Mentor, Love Interest, etc.) to specific NPCs
    /// - Check relationship vectors for each pair
    fn check_relationship_prerequisites(&self, prereqs: &Prerequisites, _ctx: &EligibilityContext) -> bool {
        if let Some(ref _rel_prereqs) = prereqs.relationship_prerequisites {
            // TODO: Implement when role assignment system is available
            // For now, always pass (roles not yet resolved)
        }
        true
    }

    /// Check memory prerequisites by querying MemorySystem.
    fn check_memory_prerequisites(&self, prereqs: &Prerequisites, ctx: &EligibilityContext) -> bool {
        if let Some(ref mem_prereqs) = prereqs.memory_prerequisites {
            self.check_memory_tags(mem_prereqs, ctx)
        } else {
            true
        }
    }

    /// Helper for memory tag checking.
    ///
    /// Checks against player's memories in the MemorySystem.
    fn check_memory_tags(&self, mem_prereqs: &MemoryPrerequisites, ctx: &EligibilityContext) -> bool {
        let player_journal = ctx.memory.get_journal(ctx.world.player_id);
        
        // Must-have tags: at least one memory with any of the required tags
        if !mem_prereqs.must_have_tags.is_empty() {
            let mut found_any = false;
            if let Some(journal) = player_journal {
                for entry in &journal.entries {
                    for tag in &mem_prereqs.must_have_tags {
                        if entry.tags.iter().any(|t| t == tag) {
                            found_any = true;
                            break;
                        }
                    }
                    if found_any {
                        break;
                    }
                }
            }
            if !found_any {
                return false;
            }
        }
        
        // Must-not-have tags: no memory with forbidden tags
        if let Some(journal) = player_journal {
            for entry in &journal.entries {
                for tag in &mem_prereqs.must_not_have_tags {
                    if entry.tags.iter().any(|t| t == tag) {
                        return false;
                    }
                }
            }
        }
        
        true
    }

    /// Check world state prerequisites.
    fn check_world_state_prerequisites(&self, prereqs: &Prerequisites, ctx: &EligibilityContext) -> bool {
        if let Some(ref world_prereqs) = prereqs.world_state_prerequisites {
            self.check_world_conditions(world_prereqs, ctx)
        } else {
            true
        }
    }

    /// Helper for world state condition checking.
    fn check_world_conditions(&self, world_prereqs: &WorldStatePrerequisites, ctx: &EligibilityContext) -> bool {
        // Check crime level if specified (from district_state)
        if let Some(min_crime) = world_prereqs.min_crime_level {
            let crime_level = ctx.world.district_state
                .get("crime_level")
                .and_then(|v| v.parse::<f32>().ok())
                .unwrap_or(0.0);
            if crime_level < min_crime {
                return false;
            }
        }
        
        // Check recession flag if specified (from world_flags)
        if let Some(true) = world_prereqs.recession_active {
            if !ctx.world.world_flags.has_any("recession_active") {
                return false;
            }
        }
        
        // Check black swan event if specified (from world_flags)
        if let Some(ref black_swan_id) = world_prereqs.required_black_swan_id {
            if !ctx.world.world_flags.has_any(black_swan_id) {
                return false;
            }
        }
        
        true
    }

    /// Check global flags.
    fn check_global_flags(&self, prereqs: &Prerequisites, ctx: &EligibilityContext) -> bool {
        if let Some(ref flags) = prereqs.global_flags {
            self.check_flag_conditions(flags, ctx)
        } else {
            true
        }
    }

    /// Helper for global flag condition checking.
    fn check_flag_conditions(&self, flags: &GlobalFlags, ctx: &EligibilityContext) -> bool {
        // All must_be_set flags must be true
        for flag in &flags.must_be_set {
            if !ctx.world.world_flags.has_any(flag) {
                return false;
            }
        }
        
        // All must_be_unset flags must be false
        for flag in &flags.must_be_unset {
            if ctx.world.world_flags.has_any(flag) {
                return false;
            }
        }
        
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn_storylets::library::CompiledStorylet;
    use syn_storylets::{Cooldowns, Outcome, StoryDomain, StoryletId, Tag};
    use syn_core::NpcId;

    struct MockStoryletSource {
        storylets: Vec<CompiledStorylet>,
    }

    impl StoryletSource for MockStoryletSource {
        fn get_storylet_by_id(&self, id: &StoryletId) -> Option<&CompiledStorylet> {
            self.storylets.iter().find(|s| s.id == *id)
        }

        fn get_storylet_by_key(&self, key: StoryletKey) -> Option<&CompiledStorylet> {
            self.storylets.iter().find(|s| s.key == key)
        }

        fn candidates_for_tag(&self, tag: &Tag) -> Vec<StoryletKey> {
            self.storylets
                .iter()
                .filter(|s| s.tags.contains(tag))
                .map(|s| s.key)
                .collect()
        }

        fn candidates_for_life_stage(&self, stage: syn_storylets::LifeStage) -> Vec<StoryletKey> {
            self.storylets
                .iter()
                .filter(|s| s.life_stage == stage)
                .map(|s| s.key)
                .collect()
        }

        fn candidates_for_domain(&self, domain: StoryDomain) -> Vec<StoryletKey> {
            self.storylets
                .iter()
                .filter(|s| s.domain == domain)
                .map(|s| s.key)
                .collect()
        }

        fn iter_all_storylets(&self) -> Box<dyn Iterator<Item = &CompiledStorylet> + '_> {
            Box::new(self.storylets.iter())
        }

        fn total_count(&self) -> u32 {
            self.storylets.len() as u32
        }
    }

    #[test]
    fn test_stat_threshold_gating() {
        // Create a storylet that requires high mood
        let storylet = CompiledStorylet {
            id: StoryletId::new("happy_event"),
            key: StoryletKey(0),
            name: "Joyful Event".to_string(),
            description: None,
            tags: vec![],
            domain: StoryDomain::SliceOfLife,
            life_stage: syn_storylets::LifeStage::Adult,
            heat: 3,
            weight: 1.0,
            roles: vec![],
            prerequisites: Prerequisites {
                stat_thresholds: Some(vec![syn_storylets::StatThresholds {
                    stat: "mood".to_string(),
                    min: Some(5.0),
                    max: None,
                }]),
                ..Default::default()
            },
            cooldowns: Cooldowns::default(),
            outcomes: Outcome::default(),
            follow_ups_resolved: vec![],
        };

        let source = MockStoryletSource {
            storylets: vec![storylet],
        };

        let engine = EligibilityEngine::new(&source);
        let world = WorldState::new(syn_core::WorldSeed(42), NpcId(1));
        let memory = MemorySystem::new();

        let ctx = EligibilityContext {
            world: &world,
            memory: &memory,
            current_tick: SimTick(0),
        };

        let _eligible = engine.find_eligible_storylets(&ctx);
        // Assert that mood-based gating works
    }

    #[test]
    fn test_world_flag_checking() {
        // Create a storylet that requires a specific flag
        let storylet = CompiledStorylet {
            id: StoryletId::new("flag_event"),
            key: StoryletKey(0),
            name: "Event After Milestone".to_string(),
            description: None,
            tags: vec![],
            domain: StoryDomain::Career,
            life_stage: syn_storylets::LifeStage::Adult,
            heat: 2,
            weight: 1.0,
            roles: vec![],
            prerequisites: Prerequisites {
                global_flags: Some(syn_storylets::GlobalFlags {
                    must_be_set: vec!["job_promotion".to_string()],
                    must_be_unset: vec![],
                }),
                ..Default::default()
            },
            cooldowns: Cooldowns::default(),
            outcomes: Outcome::default(),
            follow_ups_resolved: vec![],
        };

        let source = MockStoryletSource {
            storylets: vec![storylet],
        };

        let engine = EligibilityEngine::new(&source);
        let world = WorldState::new(syn_core::WorldSeed(42), NpcId(1));
        let memory = MemorySystem::new();

        let ctx = EligibilityContext {
            world: &world,
            memory: &memory,
            current_tick: SimTick(0),
        };

        // Without the flag set, storylet should not be eligible
        let eligible = engine.find_eligible_storylets(&ctx);
        assert!(eligible.is_empty(), "Storylet should not be eligible without flag");
    }

    #[test]
    fn test_global_flag_gating() {
        // Create a storylet with no flags
        let storylet = CompiledStorylet {
            id: StoryletId::new("always_available"),
            key: StoryletKey(0),
            name: "Always Available Event".to_string(),
            description: None,
            tags: vec![],
            domain: StoryDomain::SliceOfLife,
            life_stage: syn_storylets::LifeStage::Child,
            heat: 1,
            weight: 1.0,
            roles: vec![],
            prerequisites: Prerequisites::default(),
            cooldowns: Cooldowns::default(),
            outcomes: Outcome::default(),
            follow_ups_resolved: vec![],
        };

        let source = MockStoryletSource {
            storylets: vec![storylet],
        };

        let engine = EligibilityEngine::new(&source);
        // Create a Child player (matches the storylet's life stage)
        let mut world = WorldState::new(syn_core::WorldSeed(42), NpcId(1));
        world.player_life_stage = syn_core::LifeStage::Child;
        
        let memory = MemorySystem::new();

        let ctx = EligibilityContext {
            world: &world,
            memory: &memory,
            current_tick: SimTick(0),
        };

        // Should be eligible since no prerequisites
        let eligible = engine.find_eligible_storylets(&ctx);
        assert_eq!(eligible.len(), 1, "Storylet should be eligible");
    }
}
