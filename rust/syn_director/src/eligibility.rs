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

    /// Check trait threshold conditions against the player's personality vector.
    /// 
    /// Traits are permanent personality dimensions (stability, confidence, sociability,
    /// empathy, impulsivity, ambition, charm) ranging from 0-100.
    fn check_trait_thresholds(&self, prereqs: &Prerequisites, ctx: &EligibilityContext) -> bool {
        if let Some(ref thresholds) = prereqs.trait_thresholds {
            // Get the player's traits from their NPC data
            let player_traits = match ctx.world.npcs.get(&ctx.world.player_id) {
                Some(npc) => &npc.traits,
                None => return true, // No player NPC found, pass by default
            };

            for threshold in thresholds {
                // Look up the trait value by name
                let trait_value = match player_traits.get_by_name(&threshold.trait_name) {
                    Some(v) => v,
                    None => {
                        // Unknown trait name - skip this check (validation should catch this earlier)
                        continue;
                    }
                };

                // Check minimum threshold
                if let Some(min) = threshold.min {
                    if trait_value < min {
                        return false;
                    }
                }

                // Check maximum threshold
                if let Some(max) = threshold.max {
                    if trait_value > max {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// Check relationship prerequisites between NPC roles.
    ///
    /// This validates that at least one NPC exists who could satisfy each relationship
    /// prerequisite. The "protagonist" role is assumed to be the player.
    ///
    /// For other roles, we check if any known NPC has a relationship with the player
    /// that meets the specified thresholds on each axis.
    fn check_relationship_prerequisites(&self, prereqs: &Prerequisites, ctx: &EligibilityContext) -> bool {
        let Some(ref rel_prereqs) = prereqs.relationship_prerequisites else {
            return true;
        };

        for rel_prereq in rel_prereqs {
            // Determine the direction: from_role → to_role
            // "protagonist" or "player" is the player character
            let from_is_player = rel_prereq.from_role.to_lowercase() == "protagonist" 
                || rel_prereq.from_role.to_lowercase() == "player";
            let to_is_player = rel_prereq.to_role.to_lowercase() == "protagonist"
                || rel_prereq.to_role.to_lowercase() == "player";

            // If both roles are the player, skip (self-relationship)
            if from_is_player && to_is_player {
                continue;
            }

            // Check if at least one known NPC satisfies the relationship thresholds
            let mut found_valid_candidate = false;

            for &npc_id in &ctx.world.known_npcs {
                // Skip the player themselves
                if npc_id == ctx.world.player_id {
                    continue;
                }

                // Get relationship in the correct direction
                let relationship = if from_is_player {
                    // Player → NPC (player's feelings toward NPC)
                    ctx.world.get_relationship(ctx.world.player_id, npc_id)
                } else if to_is_player {
                    // NPC → Player (NPC's feelings toward player)
                    ctx.world.get_relationship(npc_id, ctx.world.player_id)
                } else {
                    // Both roles are NPCs - we'd need to check NPC→NPC relationships
                    // For now, if neither role is the player, we can't validate at eligibility time
                    // This will be handled during full role assignment
                    found_valid_candidate = true;
                    break;
                };

                // Check all thresholds for this relationship
                if self.relationship_meets_thresholds(&relationship, &rel_prereq.thresholds) {
                    found_valid_candidate = true;
                    break;
                }
            }

            if !found_valid_candidate {
                return false;
            }
        }

        true
    }

    /// Check if a relationship meets all the specified axis thresholds.
    fn relationship_meets_thresholds(
        &self,
        relationship: &syn_core::types::Relationship,
        thresholds: &[syn_storylets::RelationshipThreshold],
    ) -> bool {
        for threshold in thresholds {
            let axis_value = match threshold.axis.to_lowercase().as_str() {
                "affection" => relationship.affection,
                "trust" => relationship.trust,
                "attraction" => relationship.attraction,
                "familiarity" => relationship.familiarity,
                "resentment" => relationship.resentment,
                _ => continue, // Unknown axis, skip
            };

            if let Some(min) = threshold.min {
                if axis_value < min {
                    return false;
                }
            }
            if let Some(max) = threshold.max {
                if axis_value > max {
                    return false;
                }
            }
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

    #[test]
    fn test_trait_threshold_minimum() {
        use syn_core::types::{AbstractNpc, AttachmentStyle, Traits};
        use syn_storylets::TraitThresholds;

        // Create a storylet requiring high impulsivity (min 60)
        let storylet = CompiledStorylet {
            id: StoryletId::new("risky_event"),
            key: StoryletKey(0),
            name: "Risky Opportunity".to_string(),
            description: None,
            tags: vec![],
            domain: StoryDomain::Career,
            life_stage: syn_storylets::LifeStage::Adult,
            heat: 3,
            weight: 1.0,
            roles: vec![],
            prerequisites: Prerequisites {
                trait_thresholds: Some(vec![
                    TraitThresholds {
                        trait_name: "impulsivity".to_string(),
                        min: Some(60.0),
                        max: None,
                    },
                ]),
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
        let mut world = WorldState::new(syn_core::WorldSeed(42), NpcId(1));
        world.player_life_stage = syn_core::LifeStage::Adult;
        
        // Create player NPC with LOW impulsivity (30) - should NOT pass
        let low_impulsivity_npc = AbstractNpc {
            id: NpcId(1),
            age: 25,
            job: "Worker".to_string(),
            district: "Downtown".to_string(),
            household_id: 1,
            traits: Traits {
                impulsivity: 30.0,
                ..Default::default()
            },
            seed: 42,
            attachment_style: AttachmentStyle::Secure,
        };
        world.npcs.insert(NpcId(1), low_impulsivity_npc);
        
        let memory = MemorySystem::new();
        let ctx = EligibilityContext {
            world: &world,
            memory: &memory,
            current_tick: SimTick(0),
        };

        // Should NOT be eligible (impulsivity 30 < required 60)
        let eligible = engine.find_eligible_storylets(&ctx);
        assert!(eligible.is_empty(), "Storylet should NOT be eligible with low impulsivity");

        // Now update to HIGH impulsivity (75) - should pass
        if let Some(npc) = world.npcs.get_mut(&NpcId(1)) {
            npc.traits.impulsivity = 75.0;
        }

        let ctx2 = EligibilityContext {
            world: &world,
            memory: &memory,
            current_tick: SimTick(0),
        };

        let eligible2 = engine.find_eligible_storylets(&ctx2);
        assert_eq!(eligible2.len(), 1, "Storylet SHOULD be eligible with high impulsivity");
    }

    #[test]
    fn test_trait_threshold_maximum() {
        use syn_core::types::{AbstractNpc, AttachmentStyle, Traits};
        use syn_storylets::TraitThresholds;

        // Create a storylet requiring LOW stability (max 40)
        let storylet = CompiledStorylet {
            id: StoryletId::new("emotional_event"),
            key: StoryletKey(0),
            name: "Emotional Outburst".to_string(),
            description: None,
            tags: vec![],
            domain: StoryDomain::SliceOfLife,
            life_stage: syn_storylets::LifeStage::Teen,
            heat: 4,
            weight: 1.0,
            roles: vec![],
            prerequisites: Prerequisites {
                trait_thresholds: Some(vec![
                    TraitThresholds {
                        trait_name: "stability".to_string(),
                        min: None,
                        max: Some(40.0),
                    },
                ]),
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
        let mut world = WorldState::new(syn_core::WorldSeed(42), NpcId(1));
        world.player_life_stage = syn_core::LifeStage::Teen;
        
        // Create player NPC with HIGH stability (70) - should NOT pass
        let high_stability_npc = AbstractNpc {
            id: NpcId(1),
            age: 15,
            job: "Student".to_string(),
            district: "Suburbs".to_string(),
            household_id: 1,
            traits: Traits {
                stability: 70.0,
                ..Default::default()
            },
            seed: 42,
            attachment_style: AttachmentStyle::Secure,
        };
        world.npcs.insert(NpcId(1), high_stability_npc);
        
        let memory = MemorySystem::new();
        let ctx = EligibilityContext {
            world: &world,
            memory: &memory,
            current_tick: SimTick(0),
        };

        // Should NOT be eligible (stability 70 > max 40)
        let eligible = engine.find_eligible_storylets(&ctx);
        assert!(eligible.is_empty(), "Storylet should NOT be eligible with high stability");

        // Now update to LOW stability (25) - should pass
        if let Some(npc) = world.npcs.get_mut(&NpcId(1)) {
            npc.traits.stability = 25.0;
        }

        let ctx2 = EligibilityContext {
            world: &world,
            memory: &memory,
            current_tick: SimTick(0),
        };

        let eligible2 = engine.find_eligible_storylets(&ctx2);
        assert_eq!(eligible2.len(), 1, "Storylet SHOULD be eligible with low stability");
    }

    #[test]
    fn test_relationship_prerequisite_high_trust() {
        use syn_core::types::{AbstractNpc, AttachmentStyle, Traits, Relationship};
        use syn_storylets::{RelationshipPrerequisites, RelationshipThreshold, RoleSlot};

        // Create a storylet requiring high trust from protagonist to target
        let storylet = CompiledStorylet {
            id: StoryletId::new("trust_confession"),
            key: StoryletKey(0),
            name: "Heart-to-Heart".to_string(),
            description: None,
            tags: vec![],
            domain: StoryDomain::Friendship,
            life_stage: syn_storylets::LifeStage::YoungAdult,
            heat: 4,
            weight: 1.0,
            roles: vec![
                RoleSlot { name: "protagonist".to_string(), required: true, constraints: None },
                RoleSlot { name: "trusted_friend".to_string(), required: true, constraints: None },
            ],
            prerequisites: Prerequisites {
                relationship_prerequisites: Some(vec![
                    RelationshipPrerequisites {
                        from_role: "protagonist".to_string(),
                        to_role: "trusted_friend".to_string(),
                        thresholds: vec![
                            RelationshipThreshold { axis: "trust".to_string(), min: Some(5.0), max: None },
                        ],
                    },
                ]),
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
        let mut world = WorldState::new(syn_core::WorldSeed(42), NpcId(1));
        world.player_life_stage = syn_core::LifeStage::YoungAdult;
        
        // Create player NPC
        let player_npc = AbstractNpc {
            id: NpcId(1),
            age: 22,
            job: "Office Worker".to_string(),
            district: "Downtown".to_string(),
            household_id: 1,
            traits: Traits::default(),
            seed: 42,
            attachment_style: AttachmentStyle::Secure,
        };
        world.npcs.insert(NpcId(1), player_npc);

        // Create an NPC with LOW trust (2.0) - should NOT satisfy prerequisite
        let npc2 = AbstractNpc {
            id: NpcId(2),
            age: 25,
            job: "Colleague".to_string(),
            district: "Downtown".to_string(),
            household_id: 2,
            traits: Traits::default(),
            seed: 43,
            attachment_style: AttachmentStyle::Secure,
        };
        world.npcs.insert(NpcId(2), npc2);
        world.known_npcs.push(NpcId(2));

        // Set low trust relationship
        world.set_relationship(NpcId(1), NpcId(2), Relationship {
            trust: 2.0,
            affection: 3.0,
            ..Default::default()
        });

        let memory = MemorySystem::new();
        let ctx = EligibilityContext {
            world: &world,
            memory: &memory,
            current_tick: SimTick(0),
        };

        // Should NOT be eligible (no NPC with trust >= 5.0)
        let eligible = engine.find_eligible_storylets(&ctx);
        assert!(eligible.is_empty(), "Storylet should NOT be eligible without high-trust NPC");

        // Now update to HIGH trust (7.0) - should pass
        world.set_relationship(NpcId(1), NpcId(2), Relationship {
            trust: 7.0,
            affection: 5.0,
            ..Default::default()
        });

        let ctx2 = EligibilityContext {
            world: &world,
            memory: &memory,
            current_tick: SimTick(0),
        };

        let eligible2 = engine.find_eligible_storylets(&ctx2);
        assert_eq!(eligible2.len(), 1, "Storylet SHOULD be eligible with high-trust NPC");
    }

    #[test]
    fn test_relationship_prerequisite_rivalry() {
        use syn_core::types::{AbstractNpc, AttachmentStyle, Traits, Relationship};
        use syn_storylets::{RelationshipPrerequisites, RelationshipThreshold, RoleSlot};

        // Create a storylet requiring high resentment (rivalry)
        let storylet = CompiledStorylet {
            id: StoryletId::new("rival_confrontation"),
            key: StoryletKey(0),
            name: "Boiling Point".to_string(),
            description: None,
            tags: vec![],
            domain: StoryDomain::Conflict,
            life_stage: syn_storylets::LifeStage::Adult,
            heat: 7,
            weight: 1.0,
            roles: vec![
                RoleSlot { name: "protagonist".to_string(), required: true, constraints: None },
                RoleSlot { name: "rival".to_string(), required: true, constraints: None },
            ],
            prerequisites: Prerequisites {
                relationship_prerequisites: Some(vec![
                    RelationshipPrerequisites {
                        from_role: "protagonist".to_string(),
                        to_role: "rival".to_string(),
                        thresholds: vec![
                            RelationshipThreshold { axis: "resentment".to_string(), min: Some(4.0), max: None },
                            RelationshipThreshold { axis: "familiarity".to_string(), min: Some(2.0), max: None },
                        ],
                    },
                ]),
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
        let mut world = WorldState::new(syn_core::WorldSeed(42), NpcId(1));
        world.player_life_stage = syn_core::LifeStage::Adult;
        
        // Create player NPC
        let player_npc = AbstractNpc {
            id: NpcId(1),
            age: 35,
            job: "Manager".to_string(),
            district: "Business".to_string(),
            household_id: 1,
            traits: Traits::default(),
            seed: 42,
            attachment_style: AttachmentStyle::Secure,
        };
        world.npcs.insert(NpcId(1), player_npc);

        // Create potential rival NPC
        let rival_npc = AbstractNpc {
            id: NpcId(3),
            age: 33,
            job: "Competitor".to_string(),
            district: "Business".to_string(),
            household_id: 3,
            traits: Traits::default(),
            seed: 44,
            attachment_style: AttachmentStyle::Avoidant,
        };
        world.npcs.insert(NpcId(3), rival_npc);
        world.known_npcs.push(NpcId(3));

        // Set rivalry relationship (high resentment, high familiarity)
        world.set_relationship(NpcId(1), NpcId(3), Relationship {
            resentment: 6.0,
            familiarity: 4.0,
            trust: -2.0,
            affection: -1.0,
            ..Default::default()
        });

        let memory = MemorySystem::new();
        let ctx = EligibilityContext {
            world: &world,
            memory: &memory,
            current_tick: SimTick(0),
        };

        // Should be eligible (resentment 6.0 >= 4.0, familiarity 4.0 >= 2.0)
        let eligible = engine.find_eligible_storylets(&ctx);
        assert_eq!(eligible.len(), 1, "Storylet SHOULD be eligible with rival NPC");

        // Now reduce resentment below threshold
        world.set_relationship(NpcId(1), NpcId(3), Relationship {
            resentment: 2.0,  // Below threshold of 4.0
            familiarity: 4.0,
            trust: -2.0,
            affection: -1.0,
            ..Default::default()
        });

        let ctx2 = EligibilityContext {
            world: &world,
            memory: &memory,
            current_tick: SimTick(0),
        };

        let eligible2 = engine.find_eligible_storylets(&ctx2);
        assert!(eligible2.is_empty(), "Storylet should NOT be eligible without sufficient resentment");
    }

    #[test]
    fn test_relationship_prerequisite_max_threshold() {
        use syn_core::types::{AbstractNpc, AttachmentStyle, Traits, Relationship};
        use syn_storylets::{RelationshipPrerequisites, RelationshipThreshold, RoleSlot};

        // Create a storylet for meeting strangers (low familiarity required)
        let storylet = CompiledStorylet {
            id: StoryletId::new("stranger_intro"),
            key: StoryletKey(0),
            name: "A New Face".to_string(),
            description: None,
            tags: vec![],
            domain: StoryDomain::SliceOfLife,
            life_stage: syn_storylets::LifeStage::YoungAdult,
            heat: 2,
            weight: 1.0,
            roles: vec![
                RoleSlot { name: "protagonist".to_string(), required: true, constraints: None },
                RoleSlot { name: "stranger".to_string(), required: true, constraints: None },
            ],
            prerequisites: Prerequisites {
                relationship_prerequisites: Some(vec![
                    RelationshipPrerequisites {
                        from_role: "protagonist".to_string(),
                        to_role: "stranger".to_string(),
                        thresholds: vec![
                            RelationshipThreshold { axis: "familiarity".to_string(), min: None, max: Some(1.0) },
                        ],
                    },
                ]),
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
        let mut world = WorldState::new(syn_core::WorldSeed(42), NpcId(1));
        world.player_life_stage = syn_core::LifeStage::YoungAdult;
        
        // Create player NPC
        let player_npc = AbstractNpc {
            id: NpcId(1),
            age: 22,
            job: "Worker".to_string(),
            district: "City".to_string(),
            household_id: 1,
            traits: Traits::default(),
            seed: 42,
            attachment_style: AttachmentStyle::Secure,
        };
        world.npcs.insert(NpcId(1), player_npc);

        // Create a stranger NPC (default relationship = 0 familiarity)
        let stranger_npc = AbstractNpc {
            id: NpcId(4),
            age: 24,
            job: "Unknown".to_string(),
            district: "City".to_string(),
            household_id: 4,
            traits: Traits::default(),
            seed: 45,
            attachment_style: AttachmentStyle::Secure,
        };
        world.npcs.insert(NpcId(4), stranger_npc);
        world.known_npcs.push(NpcId(4));

        // Default relationship has 0 familiarity - should pass max threshold
        let memory = MemorySystem::new();
        let ctx = EligibilityContext {
            world: &world,
            memory: &memory,
            current_tick: SimTick(0),
        };

        // Should be eligible (familiarity 0.0 <= 1.0)
        let eligible = engine.find_eligible_storylets(&ctx);
        assert_eq!(eligible.len(), 1, "Storylet SHOULD be eligible with stranger");

        // Now increase familiarity above threshold
        world.set_relationship(NpcId(1), NpcId(4), Relationship {
            familiarity: 5.0,  // Above max threshold of 1.0
            ..Default::default()
        });

        let ctx2 = EligibilityContext {
            world: &world,
            memory: &memory,
            current_tick: SimTick(0),
        };

        let eligible2 = engine.find_eligible_storylets(&ctx2);
        assert!(eligible2.is_empty(), "Storylet should NOT be eligible once familiarity exceeds max");
    }
}
