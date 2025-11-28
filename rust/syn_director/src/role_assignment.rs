//! Role assignment engine for multi-actor narrative casting.
//!
//! This module handles the deterministic assignment of NPCs to storylet roles.
//! It uses a multi-factor scoring system based on:
//! - Relationship vectors (affection, trust, resentment, attraction)
//! - NPC traits/personality
//! - Current mood
//! - Contextual factors (e.g., district/cluster membership)
//!
//! All scoring is deterministic, using seeded RNG derived from world seed, tick, storylet,
//! and role name to ensure reproducible casting decisions.

use std::collections::{HashMap, HashSet};

use syn_core::{
    deterministic_rng_from_world, NpcId, SimTick, StatKind, WorldState,
};
use syn_storylets::library::{CompiledStorylet, StoryletKey};

use crate::eligibility::EligibilityContext;
use syn_memory::MemorySystem;

/// Result of assigning actors to storylet roles.
#[derive(Debug, Clone)]
pub struct RoleAssignments {
    /// The storylet being cast.
    pub storylet_key: StoryletKey,
    /// Mapping from role name to assigned actor ID.
    pub mapping: HashMap<String, NpcId>,
}

/// A candidate NPC for a specific role with computed score.
#[derive(Debug, Clone)]
pub struct RoleCandidate {
    /// The NPC being considered.
    pub actor_id: NpcId,
    /// Computed suitability score for this role.
    pub score: f32,
}

/// Role assignment engine for deterministic narrative casting.
///
/// Uses multi-factor scoring based on:
/// - Relationship vectors (affection, trust, resentment, attraction)
/// - NPC memories (tags like "betrayal", "support", "jealousy" boost role affinity)
/// - NPC traits/personality
/// - Current mood
///
/// All scoring is deterministic, using seeded RNG derived from world seed, tick, storylet,
/// and role name to ensure reproducible casting decisions.
pub struct RoleAssignmentEngine<'a> {
    world: &'a WorldState,
    memory: &'a MemorySystem,
    current_tick: SimTick,
}

impl<'a> RoleAssignmentEngine<'a> {
    /// Create a new role assignment engine from an eligibility context.
    pub fn from_context(ctx: &'a EligibilityContext<'a>) -> Self {
        RoleAssignmentEngine {
            world: ctx.world,
            memory: ctx.memory,
            current_tick: ctx.current_tick,
        }
    }

    /// Attempt to assign roles for a storylet given available candidates.
    ///
    /// Returns `Some(RoleAssignments)` if all required roles can be filled.
    /// Returns `None` if any required role cannot be filled (no suitable candidate).
    ///
    /// # Algorithm
    /// 1. Filter candidate pool (player + known NPCs)
    /// 2. For each role (required first):
    ///    - Score all available candidates based on relationship, traits, mood
    ///    - Select highest scoring candidate (with seeded tie-breaking)
    ///    - Remove selected candidate from future consideration
    /// 3. If any required role cannot be filled, return None
    /// 4. Return complete mapping
    pub fn assign_roles_for_storylet(
        &self,
        storylet: &CompiledStorylet,
        candidate_npcs: Option<&[NpcId]>,
    ) -> Option<RoleAssignments> {
        // Gather candidate pool: player + specified NPCs or all known NPCs
        let mut candidates = vec![self.world.player_id];
        if let Some(npcs) = candidate_npcs {
            candidates.extend_from_slice(npcs);
        } else {
            candidates.extend_from_slice(&self.world.known_npcs);
        }

        // Remove duplicates
        let candidates: Vec<NpcId> = {
            let mut unique = HashSet::new();
            candidates.into_iter().filter(|id| unique.insert(*id)).collect()
        };

        // Build role assignments
        let mut assignments = HashMap::new();
        let mut used_actors = HashSet::new();

        // Required roles first
        let required_roles: Vec<_> = storylet
            .roles
            .iter()
            .filter(|r| r.required)
            .collect();

        for role in required_roles {
            let scored = self.score_candidates_for_role(
                &role.name,
                &candidates,
                &used_actors,
                storylet.key,
            );

            if scored.is_empty() {
                return None; // Required role cannot be filled
            }

            // Select best candidate with deterministic tie-breaking
            let best = self.select_candidate_deterministically(
                &scored,
                storylet.key,
                &role.name,
            );

            used_actors.insert(best.actor_id);
            assignments.insert(role.name.clone(), best.actor_id);
        }

        // Optional roles (best effort)
        let optional_roles: Vec<_> = storylet
            .roles
            .iter()
            .filter(|r| !r.required)
            .collect();

        for role in optional_roles {
            let scored = self.score_candidates_for_role(
                &role.name,
                &candidates,
                &used_actors,
                storylet.key,
            );

            if !scored.is_empty() {
                let best = self.select_candidate_deterministically(
                    &scored,
                    storylet.key,
                    &role.name,
                );
                used_actors.insert(best.actor_id);
                assignments.insert(role.name.clone(), best.actor_id);
            }
            // Optional role left unfilled is acceptable
        }

        Some(RoleAssignments {
            storylet_key: storylet.key,
            mapping: assignments,
        })
    }

    /// Score all candidates for a specific role.
    ///
    /// Scoring factors:
    /// - **Relationship vectors**: Affection, trust, attraction, resentment
    /// - **Traits**: Empathy, impulsivity, dominance, creativity
    /// - **Mood**: Current emotional state
    /// - **Role context**: "rival", "friend", "romance", "manager", etc.
    fn score_candidates_for_role(
        &self,
        role_name: &str,
        candidates: &[NpcId],
        already_used: &HashSet<NpcId>,
        storylet_key: StoryletKey,
    ) -> Vec<RoleCandidate> {
        candidates
            .iter()
            .filter(|id| !already_used.contains(id))
            .map(|&actor_id| {
                let score = self.compute_role_score(
                    role_name,
                    actor_id,
                    storylet_key,
                );
                RoleCandidate { actor_id, score }
            })
            .filter(|c| c.score > -f32::INFINITY) // Filter out invalid candidates
            .collect()
    }

    /// Compute suitability score for an actor in a role.
    ///
    /// Returns a score where higher values indicate better fit.
    /// Returns `-INFINITY` for candidates that cannot fill the role.
    fn compute_role_score(
        &self,
        role_name: &str,
        actor_id: NpcId,
        _storylet_key: StoryletKey,
    ) -> f32 {
        // Normalize role name to compare against standard types
        let normalized_role = role_name.to_lowercase();

        // Get relationship from player to candidate
        let rel = self.world.get_relationship(self.world.player_id, actor_id);

        // Base score from relationship
        let mut score = 0.0;

        // Role-specific scoring
        if normalized_role.contains("rival") || normalized_role.contains("antagonist") {
            // High resentment + low trust = good rival
            score += rel.resentment * 2.0;
            score -= rel.trust * 1.5;
            score -= rel.affection * 1.0;
        } else if normalized_role.contains("friend") || normalized_role.contains("ally") {
            // High affection + high familiarity + high trust
            score += rel.affection * 2.0;
            score += rel.familiarity * 1.5;
            score += rel.trust * 1.5;
            score -= rel.resentment * 2.0;
        } else if normalized_role.contains("romance") || normalized_role.contains("love") {
            // High affection + high attraction
            score += rel.affection * 2.5;
            score += rel.attraction * 2.5;
            score += rel.trust * 1.0;
            score -= rel.resentment * 2.5;
        } else if normalized_role.contains("mentor") || normalized_role.contains("guide") {
            // High trust + moderate affection
            score += rel.trust * 2.0;
            score += rel.affection * 1.0;
            score += rel.familiarity * 1.5;
        } else if normalized_role.contains("manager") || normalized_role.contains("boss") {
            // Moderate trust + respect (represented by trust) - familiarity less important
            score += rel.trust * 1.5;
            score -= rel.resentment * 1.5;
        } else if normalized_role.contains("stranger") {
            // Low familiarity + low relationship intensity
            score -= rel.affection.abs() * 0.5;
            score -= rel.trust.abs() * 0.5;
        } else {
            // Generic role - prefer positive relationships
            score += rel.affection * 1.0;
            score += rel.trust * 1.0;
            score -= rel.resentment * 1.5;
        }

        // Mood factor: happier people better for light roles, troubled for dark
        // For now, generic boost for presence (can be specialized per role)
        score += self.world.player_stats.get(StatKind::Mood) * 0.1;

        // Memory-aware scoring: NPCs with relevant memory tags score higher for matching roles
        score += self.compute_memory_score(&normalized_role, actor_id);

        score
    }

    /// Compute memory-based score contribution for an actor in a role.
    ///
    /// Maps memory tags to role affinities. For example:
    /// - "betrayal" → boosts "accuser", "rival", "antagonist" scores
    /// - "support" → boosts "friend", "ally", "mentor" scores
    /// - "jealousy" → boosts "rival", "antagonist" scores
    /// - "confession" → boosts "romance", "love_interest" scores
    ///
    /// Only considers memories between the player and the candidate NPC.
    fn compute_memory_score(&self, normalized_role: &str, actor_id: NpcId) -> f32 {
        let player_id = self.world.player_id.0;
        let actor_id_raw = actor_id.0;

        // Get the NPC's journal if it exists
        let Some(journal) = self.memory.get_journal(actor_id) else {
            return 0.0;
        };

        // Only consider memories involving both player and this NPC
        let relevant_memories: Vec<_> = journal
            .entries
            .iter()
            .filter(|m| {
                m.participants.contains(&player_id) && m.participants.contains(&actor_id_raw)
            })
            .collect();

        if relevant_memories.is_empty() {
            return 0.0;
        }

        let mut memory_score = 0.0;

        // Define role-tag affinity mappings
        // Each (role_pattern, positive_tags, negative_tags)
        let role_affinities: &[(&str, &[&str], &[&str])] = &[
            // Antagonistic roles: boosted by betrayal, jealousy, conflict, resentment
            ("rival", &["betrayal", "jealousy", "conflict", "resentment", "argument"], &["support", "reconciliation"]),
            ("antagonist", &["betrayal", "jealousy", "conflict", "resentment", "trauma"], &["support", "reconciliation"]),
            ("accuser", &["betrayal", "witness", "caught", "scandal"], &["forgiveness"]),
            
            // Friendly roles: boosted by support, help, shared experiences
            ("friend", &["support", "help", "shared_moment", "trust", "bonding"], &["betrayal", "conflict"]),
            ("ally", &["support", "teamwork", "loyalty", "trust"], &["betrayal", "conflict"]),
            
            // Romantic roles: boosted by confession, intimacy, attraction signals
            ("romance", &["confession", "intimacy", "flirt", "attraction", "date"], &["rejection", "betrayal"]),
            ("love", &["confession", "intimacy", "attraction", "passion"], &["rejection"]),
            
            // Mentor roles: boosted by guidance, teaching moments
            ("mentor", &["guidance", "advice", "teaching", "wisdom"], &["condescension"]),
            ("guide", &["guidance", "help", "teaching"], &[]),
            
            // Witness/observer roles: anyone who was present at key events
            ("witness", &["witnessed", "present", "observed", "saw"], &[]),
            
            // Confidant: trusted with secrets
            ("confidant", &["secret", "trust", "confession", "private"], &["betrayal", "gossip"]),
            
            // Victim: suffered negative events
            ("victim", &["trauma", "hurt", "loss", "abuse"], &[]),
        ];

        // Find matching role affinity
        for (role_pattern, positive_tags, negative_tags) in role_affinities {
            if normalized_role.contains(role_pattern) {
                // Count positive tag matches
                for memory in &relevant_memories {
                    let lower_tags: Vec<String> = memory.tags.iter().map(|t| t.to_lowercase()).collect();
                    
                    for positive_tag in *positive_tags {
                        if lower_tags.iter().any(|t| t.contains(positive_tag)) {
                            // Stronger memories have more impact
                            let intensity_factor = 1.0 + memory.emotional_intensity.abs();
                            memory_score += 5.0 * intensity_factor;
                        }
                    }
                    
                    for negative_tag in *negative_tags {
                        if lower_tags.iter().any(|t| t.contains(negative_tag)) {
                            let intensity_factor = 1.0 + memory.emotional_intensity.abs();
                            memory_score -= 1.5 * intensity_factor;
                        }
                    }
                }
            }
        }

        // Recency bonus: more recent memories have stronger influence
        let recency_window = 168u64; // 7 days in ticks (24 ticks/day)
        let current = self.current_tick.0;
        for memory in &relevant_memories {
            let age = current.saturating_sub(memory.sim_tick.0);
            if age < recency_window {
                // Scale bonus by recency (newer = stronger)
                let recency_factor = 1.0 - (age as f32 / recency_window as f32);
                memory_score += recency_factor * memory.emotional_intensity.abs() * 2.0;
            }
        }

        memory_score
    }

    /// Deterministically select the best candidate using seeded RNG for tie-breaking.
    ///
    /// This ensures that if multiple candidates have the same score, the selection
    /// is deterministic based on (world_seed, tick, storylet_key, role_name).
    fn select_candidate_deterministically(
        &self,
        scored: &[RoleCandidate],
        storylet_key: StoryletKey,
        role_name: &str,
    ) -> RoleCandidate {
        if scored.is_empty() {
            panic!("Cannot select from empty candidate list");
        }

        // Find max score
        let max_score = scored.iter().map(|c| c.score).fold(f32::NEG_INFINITY, f32::max);

        // Get all candidates with max score
        let best_candidates: Vec<_> = scored
            .iter()
            .filter(|c| (c.score - max_score).abs() < 1e-6) // Floating point tolerance
            .collect();

        if best_candidates.len() == 1 {
            return best_candidates[0].clone();
        }

        // Multiple candidates tied - use seeded RNG for deterministic selection
        // Create a new world state with modified seed for this specific role/storylet
        let mut temp_world = self.world.clone();
        let seed_mod = self.derive_seed_for_role(storylet_key, role_name);
        temp_world.seed = syn_core::WorldSeed(temp_world.seed.0 ^ (seed_mod as u64));

        let mut rng = deterministic_rng_from_world(&temp_world);
        let idx = rng.gen_range_i32(0, best_candidates.len() as i32) as usize;

        best_candidates[idx].clone()
    }

    /// Derive a seed component for tie-breaking based on role and storylet.
    fn derive_seed_for_role(&self, storylet_key: StoryletKey, role_name: &str) -> u32 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        use std::hash::{Hash, Hasher};

        self.current_tick.0.hash(&mut hasher);
        storylet_key.0.hash(&mut hasher);
        role_name.hash(&mut hasher);

        (hasher.finish() as u32).wrapping_mul(0x9e3779b9)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn_memory::MemorySystem;
    use syn_core::RelationshipState;
    use syn_storylets::library::CompiledStorylet;
    use syn_storylets::{Cooldowns, Outcome, Prerequisites, RoleSlot, StoryDomain, LifeStage, StoryletId};

    struct TestSetup {
        world: WorldState,
        memory: MemorySystem,
    }

    impl TestSetup {
        fn new() -> Self {
            let world = WorldState::new(syn_core::WorldSeed(42), NpcId(1));
            let memory = MemorySystem::new();
            TestSetup { world, memory }
        }

        fn with_npc_relationship(
            mut self,
            from: NpcId,
            to: NpcId,
            affection: f32,
            trust: f32,
            attraction: f32,
            resentment: f32,
        ) -> Self {
            let rel = syn_core::Relationship {
                affection,
                trust,
                attraction,
                familiarity: 5.0,
                resentment,
                state: RelationshipState::Acquaintance,
            };
            self.world.relationships.insert((from, to), rel);
            self
        }
    }

    fn make_test_storylet(name: &str, roles: Vec<RoleSlot>) -> CompiledStorylet {
        CompiledStorylet {
            id: StoryletId(name.to_string()),
            key: StoryletKey(0),
            name: name.to_string(),
            description: None,
            tags: vec![],
            domain: StoryDomain::Friendship,
            life_stage: LifeStage::Adult,
            heat: 5,
            weight: 1.0,
            prerequisites: Prerequisites::default(),
            cooldowns: Cooldowns::default(),
            outcomes: Outcome::default(),
            follow_ups_resolved: vec![],
            roles,
        }
    }

    #[test]
    fn test_friend_role_scoring() {
        let setup = TestSetup::new().with_npc_relationship(
            NpcId(1),
            NpcId(2),
            8.0,  // High affection
            7.0,  // High trust
            6.0,  // Decent familiarity
            -1.0, // Low resentment
        );

        let engine = RoleAssignmentEngine {
            world: &setup.world,
            memory: &setup.memory,
            current_tick: SimTick(0),
        };

        let friend_role = RoleSlot {
            name: "friend".to_string(),
            required: true,
            constraints: None,
        };

        let storylet = make_test_storylet("meet_friend", vec![friend_role]);

        let result = engine.assign_roles_for_storylet(&storylet, Some(&[NpcId(2), NpcId(3)]));

        assert!(result.is_some());
        let assignments = result.unwrap();
        assert_eq!(assignments.mapping.get("friend"), Some(&NpcId(2)));
    }

    #[test]
    fn test_rival_role_scoring() {
        let setup = TestSetup::new().with_npc_relationship(
            NpcId(1),
            NpcId(2),
            -2.0, // Low affection
            -3.0, // Low trust
            0.0,  // No attraction
            8.0,  // High resentment
        );

        let engine = RoleAssignmentEngine {
            world: &setup.world,
            memory: &setup.memory,
            current_tick: SimTick(0),
        };

        let rival_role = RoleSlot {
            name: "rival".to_string(),
            required: true,
            constraints: None,
        };

        let storylet = make_test_storylet("confrontation", vec![rival_role]);

        let result = engine.assign_roles_for_storylet(&storylet, Some(&[NpcId(2), NpcId(3)]));

        assert!(result.is_some());
        let assignments = result.unwrap();
        assert_eq!(assignments.mapping.get("rival"), Some(&NpcId(2)));
    }

    #[test]
    fn test_deterministic_tie_breaking() {
        let setup = TestSetup::new()
            .with_npc_relationship(NpcId(1), NpcId(2), 5.0, 5.0, 0.0, 0.0)
            .with_npc_relationship(NpcId(1), NpcId(3), 5.0, 5.0, 0.0, 0.0);

        let engine = RoleAssignmentEngine {
            world: &setup.world,
            memory: &setup.memory,
            current_tick: SimTick(0),
        };

        let generic_role = RoleSlot {
            name: "participant".to_string(),
            required: true,
            constraints: None,
        };

        let storylet = make_test_storylet("event", vec![generic_role]);

        // Run assignment twice with same context
        let result1 = engine.assign_roles_for_storylet(&storylet, Some(&[NpcId(2), NpcId(3)]));
        let result2 = engine.assign_roles_for_storylet(&storylet, Some(&[NpcId(2), NpcId(3)]));

        assert!(result1.is_some());
        assert!(result2.is_some());

        // Should get same assignment both times
        assert_eq!(
            result1.unwrap().mapping.get("participant"),
            result2.unwrap().mapping.get("participant")
        );
    }

    #[test]
    fn test_required_role_missing_candidate() {
        let setup = TestSetup::new();

        let engine = RoleAssignmentEngine {
            world: &setup.world,
            memory: &setup.memory,
            current_tick: SimTick(0),
        };

        let friend_role = RoleSlot {
            name: "friend".to_string(),
            required: true,
            constraints: None,
        };

        let storylet = make_test_storylet("meet_friend", vec![friend_role]);

        // No candidate NPCs provided - player is only candidate
        let result = engine.assign_roles_for_storylet(&storylet, Some(&[]));

        // Should succeed - player can fill their own role
        assert!(result.is_some());
    }

    #[test]
    fn test_optional_role_unfilled() {
        let setup = TestSetup::new();

        let engine = RoleAssignmentEngine {
            world: &setup.world,
            memory: &setup.memory,
            current_tick: SimTick(0),
        };

        let required_role = RoleSlot {
            name: "protagonist".to_string(),
            required: true,
            constraints: None,
        };

        let optional_role = RoleSlot {
            name: "witness".to_string(),
            required: false,
            constraints: None,
        };

        let storylet = make_test_storylet("solo_event", vec![required_role, optional_role]);

        let result = engine.assign_roles_for_storylet(&storylet, Some(&[]));

        // Should succeed with only required role filled
        assert!(result.is_some());
        let assignments = result.unwrap();
        assert!(assignments.mapping.contains_key("protagonist"));
        assert!(!assignments.mapping.contains_key("witness")); // Optional unfilled
    }

    #[test]
    fn test_no_reuse_of_assigned_actors() {
        let setup = TestSetup::new()
            .with_npc_relationship(NpcId(1), NpcId(2), 8.0, 8.0, 0.0, 0.0)
            .with_npc_relationship(NpcId(1), NpcId(3), 5.0, 5.0, 0.0, 0.0);

        let engine = RoleAssignmentEngine {
            world: &setup.world,
            memory: &setup.memory,
            current_tick: SimTick(0),
        };

        let role1 = RoleSlot {
            name: "primary".to_string(),
            required: true,
            constraints: None,
        };

        let role2 = RoleSlot {
            name: "secondary".to_string(),
            required: true,
            constraints: None,
        };

        let storylet = make_test_storylet("two_person", vec![role1, role2]);

        let result = engine.assign_roles_for_storylet(&storylet, Some(&[NpcId(2), NpcId(3)]));

        assert!(result.is_some());
        let assignments = result.unwrap();

        // NpcId(2) should be primary (higher score)
        assert_eq!(assignments.mapping.get("primary"), Some(&NpcId(2)));
        // NpcId(3) should be secondary (couldn't reuse NpcId(2))
        assert_eq!(assignments.mapping.get("secondary"), Some(&NpcId(3)));
    }

    #[test]
    fn test_memory_boosts_rival_role() {
        use syn_memory::MemoryEntry;

        // NpcId(2) has neutral relationship but betrayal memory
        // NpcId(3) has higher resentment but no betrayal memory
        let mut setup = TestSetup::new()
            .with_npc_relationship(NpcId(1), NpcId(2), 0.0, 0.0, 0.0, 2.0)  // Low resentment
            .with_npc_relationship(NpcId(1), NpcId(3), 0.0, 0.0, 0.0, 5.0); // Higher resentment

        // Add betrayal memory for NpcId(2) involving player (NpcId(1))
        let mut entry = MemoryEntry::new(
            "mem_betrayal".to_string(),
            "player_betrayed_npc".to_string(),
            NpcId(2),
            SimTick(50),
            -0.9, // Highly negative emotional intensity
        );
        entry.tags = vec!["betrayal".to_string(), "trust_broken".to_string()];
        entry.participants = vec![1, 2]; // Player (1) and NPC (2)
        setup.memory.record_memory(entry);

        let engine = RoleAssignmentEngine {
            world: &setup.world,
            memory: &setup.memory,
            current_tick: SimTick(100),
        };

        let rival_role = RoleSlot {
            name: "rival".to_string(),
            required: true,
            constraints: None,
        };

        let storylet = make_test_storylet("confrontation", vec![rival_role]);

        let result = engine.assign_roles_for_storylet(&storylet, Some(&[NpcId(2), NpcId(3)]));

        assert!(result.is_some());
        let assignments = result.unwrap();

        // NpcId(2) should be chosen as rival due to betrayal memory boost
        // despite NpcId(3) having higher resentment
        assert_eq!(
            assignments.mapping.get("rival"),
            Some(&NpcId(2)),
            "NPC with betrayal memory should be cast as rival over one with just higher resentment"
        );
    }

    #[test]
    fn test_memory_boosts_friend_role() {
        use syn_memory::MemoryEntry;

        // NpcId(2) has lower affection but support memories
        // NpcId(3) has higher affection but no memories
        let mut setup = TestSetup::new()
            .with_npc_relationship(NpcId(1), NpcId(2), 3.0, 3.0, 0.0, 0.0)  // Lower affection
            .with_npc_relationship(NpcId(1), NpcId(3), 6.0, 6.0, 0.0, 0.0); // Higher affection

        // Add support memory for NpcId(2)
        let mut entry = MemoryEntry::new(
            "mem_support".to_string(),
            "npc_helped_player".to_string(),
            NpcId(2),
            SimTick(50),
            0.8, // Positive emotional intensity
        );
        entry.tags = vec!["support".to_string(), "help".to_string(), "bonding".to_string()];
        entry.participants = vec![1, 2];
        setup.memory.record_memory(entry);

        // Add another support memory to boost further
        let mut entry2 = MemoryEntry::new(
            "mem_support2".to_string(),
            "shared_moment".to_string(),
            NpcId(2),
            SimTick(80),
            0.7,
        );
        entry2.tags = vec!["shared_moment".to_string(), "trust".to_string()];
        entry2.participants = vec![1, 2];
        setup.memory.record_memory(entry2);

        let engine = RoleAssignmentEngine {
            world: &setup.world,
            memory: &setup.memory,
            current_tick: SimTick(100),
        };

        let friend_role = RoleSlot {
            name: "friend".to_string(),
            required: true,
            constraints: None,
        };

        let storylet = make_test_storylet("hangout", vec![friend_role]);

        let result = engine.assign_roles_for_storylet(&storylet, Some(&[NpcId(2), NpcId(3)]));

        assert!(result.is_some());
        let assignments = result.unwrap();

        // NpcId(2) should be chosen as friend due to support memory boosts
        assert_eq!(
            assignments.mapping.get("friend"),
            Some(&NpcId(2)),
            "NPC with support memories should be cast as friend over one with just higher affection"
        );
    }

    #[test]
    fn test_memory_recency_bonus() {
        use syn_memory::MemoryEntry;

        // Both NPCs have equal relationships with moderate resentment (good for rival)
        // But NpcId(2) has more recent memory
        let mut setup = TestSetup::new()
            .with_npc_relationship(NpcId(1), NpcId(2), 0.0, 0.0, 0.0, 4.0) // Moderate resentment
            .with_npc_relationship(NpcId(1), NpcId(3), 0.0, 0.0, 0.0, 4.0); // Equal resentment

        // Old betrayal memory for NpcId(3) - outside recency window
        let mut old_entry = MemoryEntry::new(
            "mem_old".to_string(),
            "old_betrayal".to_string(),
            NpcId(3),
            SimTick(10), // Very old (outside 168 tick window at tick 200)
            -0.8,
        );
        old_entry.tags = vec!["betrayal".to_string()];
        old_entry.participants = vec![1, 3];
        setup.memory.record_memory(old_entry);

        // Recent betrayal memory for NpcId(2)
        let mut recent_entry = MemoryEntry::new(
            "mem_recent".to_string(),
            "recent_betrayal".to_string(),
            NpcId(2),
            SimTick(195), // Very recent (within 168 tick window at tick 200)
            -0.8,
        );
        recent_entry.tags = vec!["betrayal".to_string()];
        recent_entry.participants = vec![1, 2];
        setup.memory.record_memory(recent_entry);

        let engine = RoleAssignmentEngine {
            world: &setup.world,
            memory: &setup.memory,
            current_tick: SimTick(200),
        };

        let rival_role = RoleSlot {
            name: "rival".to_string(),
            required: true,
            constraints: None,
        };

        let storylet = make_test_storylet("argument", vec![rival_role]);

        let result = engine.assign_roles_for_storylet(&storylet, Some(&[NpcId(2), NpcId(3)]));

        assert!(result.is_some());
        let assignments = result.unwrap();

        // NpcId(2) should be preferred due to recency bonus
        assert_eq!(
            assignments.mapping.get("rival"),
            Some(&NpcId(2)),
            "NPC with more recent betrayal memory should score higher for rival role"
        );
    }

    #[test]
    fn test_memory_irrelevant_to_wrong_role() {
        use syn_memory::MemoryEntry;

        // NpcId(2) has betrayal memory (good for rival, irrelevant for romance)
        // NpcId(3) has higher attraction
        let mut setup = TestSetup::new()
            .with_npc_relationship(NpcId(1), NpcId(2), 5.0, 5.0, 3.0, 0.0)  // Lower attraction
            .with_npc_relationship(NpcId(1), NpcId(3), 5.0, 5.0, 7.0, 0.0); // Higher attraction

        // Betrayal memory for NpcId(2) - irrelevant for romance role
        let mut entry = MemoryEntry::new(
            "mem_betrayal".to_string(),
            "betrayal".to_string(),
            NpcId(2),
            SimTick(50),
            -0.9,
        );
        entry.tags = vec!["betrayal".to_string()];
        entry.participants = vec![1, 2];
        setup.memory.record_memory(entry);

        let engine = RoleAssignmentEngine {
            world: &setup.world,
            memory: &setup.memory,
            current_tick: SimTick(100),
        };

        let romance_role = RoleSlot {
            name: "romance_interest".to_string(),
            required: true,
            constraints: None,
        };

        let storylet = make_test_storylet("date", vec![romance_role]);

        let result = engine.assign_roles_for_storylet(&storylet, Some(&[NpcId(2), NpcId(3)]));

        assert!(result.is_some());
        let assignments = result.unwrap();

        // NpcId(3) should be chosen - betrayal memory doesn't help romance role
        // and NpcId(3) has higher attraction
        assert_eq!(
            assignments.mapping.get("romance_interest"),
            Some(&NpcId(3)),
            "Betrayal memory should not boost romance role - higher attraction NPC should win"
        );
    }
}
