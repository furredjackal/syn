//! Tier management system for NPC fidelity promotion/demotion.
//!
//! This module handles the deterministic assignment of NPCs to fidelity tiers
//! (Tier0, Tier1, Tier2) based on relationship importance, proximity to player,
//! and active pressure/milestone involvement.

use std::cmp::Ordering;

use syn_core::{DeterministicRng, NpcId, SimTick, WorldState};

use crate::{NpcTier, WorldSimState};

/// Configuration for tier update behavior.
#[derive(Debug, Clone)]
pub struct TierUpdateConfig {
    /// Maximum number of NPCs in Tier0 (high fidelity).
    pub max_tier0_npcs: usize,
    /// Maximum number of NPCs in Tier1 (active but batched).
    pub max_tier1_npcs: usize,
    /// Demote to Tier2 after this many ticks of inactivity.
    pub idle_demote_after: u64,
    /// Proximity promotion radius (for district/cluster-based promotion).
    /// NPCs in the same district as the player get a proximity bonus.
    pub proximity_promote_radius: u32,
}

impl Default for TierUpdateConfig {
    fn default() -> Self {
        Self {
            max_tier0_npcs: 5,
            max_tier1_npcs: 15,
            idle_demote_after: 48, // 2 days (48 ticks at 1 tick/hour)
            proximity_promote_radius: 1,
        }
    }
}

/// Computed score for an NPC used in tier assignment.
#[derive(Debug, Clone)]
struct NpcScore {
    npc_id: NpcId,
    /// Relationship importance score (higher = more important).
    relationship_importance: f32,
    /// Whether NPC is in the same district as player (1.0 bonus if true).
    proximity_bonus: f32,
    /// Whether NPC has active pressure/milestone events (1.0 bonus if true).
    has_active_events: bool,
    /// Recency score based on last update (higher = more recent).
    recency_score: f32,
    /// Whether this NPC should always be Tier0 (e.g., pinned NPCs).
    force_tier0: bool,
}

impl NpcScore {
    /// Compute composite score for sorting.
    fn composite_score(&self) -> f32 {
        if self.force_tier0 {
            return f32::MAX;
        }
        let event_bonus = if self.has_active_events { 10.0 } else { 0.0 };
        self.relationship_importance + self.proximity_bonus + event_bonus + self.recency_score
    }
}

/// Compute relationship importance from relationship vector.
/// Uses affection + trust as primary importance metrics.
fn compute_relationship_importance(world: &WorldState, player_id: NpcId, npc_id: NpcId) -> f32 {
    // Check both directions of relationship
    let rel = world
        .relationships
        .get(&(player_id, npc_id))
        .or_else(|| world.relationships.get(&(npc_id, player_id)));

    match rel {
        Some(r) => {
            // Importance = affection + trust, with familiarity as minor boost
            // Range roughly -20..+20 before familiarity
            let base = r.affection + r.trust;
            let familiarity_boost = r.familiarity * 0.1;
            base + familiarity_boost
        }
        None => 0.0, // Unknown NPC has zero importance
    }
}

/// Check if NPC is in the same district as the player.
fn is_same_district(world: &WorldState, player_id: NpcId, npc_id: NpcId) -> bool {
    let player_district = world
        .npcs
        .get(&player_id)
        .map(|npc| npc.district.as_str())
        .unwrap_or("");

    let npc_district = world
        .npcs
        .get(&npc_id)
        .map(|npc| npc.district.as_str())
        .unwrap_or("");

    !player_district.is_empty() && player_district == npc_district
}

/// Check if NPC has active pressure or milestone events in the queues.
fn has_active_pressure_or_milestone(world: &WorldState, npc_id: NpcId) -> bool {
    let npc_id_raw = npc_id.0;

    // Check pressure queue for events involving this NPC
    let has_pressure = world.relationship_pressure.queue.iter().any(|event| {
        event.actor_id == npc_id_raw || event.target_id == npc_id_raw
    });

    // Check milestone queue for events involving this NPC
    let has_milestone = world.relationship_milestones.queue.iter().any(|event| {
        event.actor_id == npc_id_raw || event.target_id == npc_id_raw
    });

    has_pressure || has_milestone
}

/// Compute recency score based on last update tick.
/// Returns higher values for more recently updated NPCs.
fn compute_recency_score(
    sim_state: &WorldSimState,
    npc_id: NpcId,
    current_tick: SimTick,
    idle_demote_after: u64,
) -> f32 {
    match sim_state.last_npc_update(npc_id) {
        Some(last_tick) => {
            let ticks_since = current_tick.0.saturating_sub(last_tick.0);
            if ticks_since >= idle_demote_after {
                -10.0 // Penalty for very idle NPCs
            } else {
                // Scale from 0 (just updated) to negative (approaching idle threshold)
                let freshness = 1.0 - (ticks_since as f32 / idle_demote_after as f32);
                freshness * 5.0 // Max 5 point bonus for very recently updated
            }
        }
        None => 0.0, // Never updated = neutral
    }
}

/// Build score entries for all known NPCs.
fn collect_npc_scores(
    world: &WorldState,
    sim_state: &WorldSimState,
    config: &TierUpdateConfig,
) -> Vec<NpcScore> {
    let player_id = world.player_id;
    let current_tick = world.current_tick;

    world
        .known_npcs
        .iter()
        .filter(|&&npc_id| npc_id != player_id) // Exclude player from NPC list
        .map(|&npc_id| {
            let relationship_importance = compute_relationship_importance(world, player_id, npc_id);
            let proximity_bonus = if is_same_district(world, player_id, npc_id) {
                5.0 // Significant bonus for same district
            } else {
                0.0
            };
            let has_active_events = has_active_pressure_or_milestone(world, npc_id);
            let recency_score = compute_recency_score(
                sim_state,
                npc_id,
                current_tick,
                config.idle_demote_after,
            );

            NpcScore {
                npc_id,
                relationship_importance,
                proximity_bonus,
                has_active_events,
                recency_score,
                force_tier0: false, // No pinning mechanism yet
            }
        })
        .collect()
}

/// Deterministic comparison for NPC scores.
/// Primary: composite score (descending)
/// Secondary: NpcId (ascending, for stable tie-breaking)
fn compare_npc_scores(a: &NpcScore, b: &NpcScore) -> Ordering {
    // Higher composite score = higher priority (descending)
    let score_cmp = b
        .composite_score()
        .partial_cmp(&a.composite_score())
        .unwrap_or(Ordering::Equal);

    if score_cmp != Ordering::Equal {
        return score_cmp;
    }

    // Tie-breaker: lower NpcId first (ascending)
    a.npc_id.0.cmp(&b.npc_id.0)
}

/// Update NPC tiers for the current tick.
///
/// This function assigns NPCs to Tier0, Tier1, or Tier2 based on:
/// - Relationship importance (affection + trust)
/// - Proximity to player (same district)
/// - Active pressure/milestone events
/// - Recency of last update
///
/// The player is always assigned Tier0 if represented as an NPC.
/// Results are deterministic given the same world state and RNG seed.
pub fn update_npc_tiers_for_tick(
    world: &WorldState,
    sim_state: &mut WorldSimState,
    config: &TierUpdateConfig,
    _rng: &mut DeterministicRng, // Reserved for future tie-breaking
) {
    let player_id = world.player_id;

    // 1. Always set player to Tier0
    sim_state.set_npc_tier(player_id, NpcTier::Tier0);

    // 2. Collect and score all known NPCs (excluding player)
    let mut scores = collect_npc_scores(world, sim_state, config);

    // 3. Sort deterministically by score and ID
    scores.sort_by(compare_npc_scores);

    // 4. Assign tiers based on sorted order
    // Player already takes one Tier0 slot if they're in known_npcs, but we excluded them
    let mut tier0_count = 1; // Player is always Tier0
    let mut tier1_count = 0;

    for score in scores {
        let npc_id = score.npc_id;

        // Force Tier0 for pinned NPCs
        if score.force_tier0 {
            sim_state.set_npc_tier(npc_id, NpcTier::Tier0);
            tier0_count += 1;
            continue;
        }

        // Assign based on remaining slots
        if tier0_count < config.max_tier0_npcs {
            sim_state.set_npc_tier(npc_id, NpcTier::Tier0);
            tier0_count += 1;
        } else if tier1_count < config.max_tier1_npcs {
            sim_state.set_npc_tier(npc_id, NpcTier::Tier1);
            tier1_count += 1;
        } else {
            sim_state.set_npc_tier(npc_id, NpcTier::Tier2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn_core::{Relationship, WorldSeed};

    fn make_test_world() -> WorldState {
        let mut world = WorldState::new(WorldSeed(12345), NpcId(1));

        // Add player to npcs
        world.npcs.insert(
            NpcId(1),
            syn_core::AbstractNpc {
                id: NpcId(1),
                age: 25,
                job: "Player".to_string(),
                district: "Downtown".to_string(),
                household_id: 1,
                traits: Default::default(),
                seed: 1,
                attachment_style: Default::default(),
            },
        );

        // Add 5 NPCs with varying characteristics
        for i in 2..=6 {
            world.npcs.insert(
                NpcId(i),
                syn_core::AbstractNpc {
                    id: NpcId(i),
                    age: 20 + i as u32,
                    job: format!("Job{}", i),
                    district: if i <= 3 { "Downtown" } else { "Suburbs" }.to_string(),
                    household_id: i,
                    traits: Default::default(),
                    seed: i,
                    attachment_style: Default::default(),
                },
            );
            world.known_npcs.push(NpcId(i));
        }

        // Set up relationships with varying importance
        // NPC 2: High importance (close friend)
        world.set_relationship(
            NpcId(1),
            NpcId(2),
            Relationship {
                affection: 8.0,
                trust: 7.0,
                familiarity: 9.0,
                ..Default::default()
            },
        );

        // NPC 3: Medium importance, same district
        world.set_relationship(
            NpcId(1),
            NpcId(3),
            Relationship {
                affection: 3.0,
                trust: 2.0,
                familiarity: 5.0,
                ..Default::default()
            },
        );

        // NPC 4: Low importance, different district
        world.set_relationship(
            NpcId(1),
            NpcId(4),
            Relationship {
                affection: 0.0,
                trust: 0.0,
                familiarity: 1.0,
                ..Default::default()
            },
        );

        // NPC 5: Very low importance, different district
        world.set_relationship(
            NpcId(1),
            NpcId(5),
            Relationship {
                affection: -2.0,
                trust: -1.0,
                familiarity: 0.5,
                ..Default::default()
            },
        );

        // NPC 6: Medium importance, different district
        world.set_relationship(
            NpcId(1),
            NpcId(6),
            Relationship {
                affection: 4.0,
                trust: 3.0,
                familiarity: 4.0,
                ..Default::default()
            },
        );

        world
    }

    #[test]
    fn test_player_always_tier0() {
        let world = make_test_world();
        let mut sim_state = WorldSimState::new();
        let config = TierUpdateConfig {
            max_tier0_npcs: 2,
            max_tier1_npcs: 2,
            ..Default::default()
        };
        let mut rng = DeterministicRng::new(42);

        update_npc_tiers_for_tick(&world, &mut sim_state, &config, &mut rng);

        assert_eq!(sim_state.npc_tier(NpcId(1)), NpcTier::Tier0);
    }

    #[test]
    fn test_high_importance_npcs_promoted() {
        let world = make_test_world();
        let mut sim_state = WorldSimState::new();
        let config = TierUpdateConfig {
            max_tier0_npcs: 2,
            max_tier1_npcs: 3,
            ..Default::default()
        };
        let mut rng = DeterministicRng::new(42);

        update_npc_tiers_for_tick(&world, &mut sim_state, &config, &mut rng);

        // NPC 2 has highest importance, should be Tier0
        assert_eq!(sim_state.npc_tier(NpcId(2)), NpcTier::Tier0);

        // NPC 3 and 6 have medium importance, should be Tier1
        // (NPC 3 has proximity bonus, NPC 6 has slightly higher raw importance)
        let npc3_tier = sim_state.npc_tier(NpcId(3));
        let npc6_tier = sim_state.npc_tier(NpcId(6));
        assert!(
            npc3_tier == NpcTier::Tier1 || npc6_tier == NpcTier::Tier1,
            "At least one medium-importance NPC should be Tier1"
        );
    }

    #[test]
    fn test_low_importance_npcs_demoted() {
        let world = make_test_world();
        let mut sim_state = WorldSimState::new();
        let config = TierUpdateConfig {
            max_tier0_npcs: 2,
            max_tier1_npcs: 2,
            ..Default::default()
        };
        let mut rng = DeterministicRng::new(42);

        update_npc_tiers_for_tick(&world, &mut sim_state, &config, &mut rng);

        // NPC 5 has lowest importance, should be Tier2
        assert_eq!(sim_state.npc_tier(NpcId(5)), NpcTier::Tier2);
    }

    #[test]
    fn test_tier_assignment_is_deterministic() {
        let world = make_test_world();
        let config = TierUpdateConfig {
            max_tier0_npcs: 2,
            max_tier1_npcs: 3,
            ..Default::default()
        };

        // Run twice with same initial state
        let mut sim_state1 = WorldSimState::new();
        let mut rng1 = DeterministicRng::new(42);
        update_npc_tiers_for_tick(&world, &mut sim_state1, &config, &mut rng1);

        let mut sim_state2 = WorldSimState::new();
        let mut rng2 = DeterministicRng::new(42);
        update_npc_tiers_for_tick(&world, &mut sim_state2, &config, &mut rng2);

        // Verify all tiers are identical
        for i in 1..=6 {
            let id = NpcId(i);
            assert_eq!(
                sim_state1.npc_tier(id),
                sim_state2.npc_tier(id),
                "Tier for NpcId({}) should be deterministic",
                i
            );
        }
    }

    #[test]
    fn test_proximity_bonus_affects_ranking() {
        let mut world = make_test_world();
        let mut sim_state = WorldSimState::new();
        let config = TierUpdateConfig {
            max_tier0_npcs: 2,
            max_tier1_npcs: 1,
            ..Default::default()
        };
        let mut rng = DeterministicRng::new(42);

        // Give NPC 4 (Suburbs) slightly higher importance than NPC 3 (Downtown)
        world.set_relationship(
            NpcId(1),
            NpcId(4),
            Relationship {
                affection: 3.5,
                trust: 2.5,
                familiarity: 5.0,
                ..Default::default()
            },
        );

        update_npc_tiers_for_tick(&world, &mut sim_state, &config, &mut rng);

        // NPC 3 should still rank higher due to proximity bonus (same district)
        let npc3_tier = sim_state.npc_tier(NpcId(3));
        let npc4_tier = sim_state.npc_tier(NpcId(4));

        // With only 1 Tier1 slot (after player+NPC2 take Tier0), NPC3 should get it
        assert!(
            npc3_tier <= npc4_tier,
            "NPC3 (same district) should rank >= NPC4 (different district)"
        );
    }

    #[test]
    fn test_respects_max_counts() {
        let world = make_test_world();
        let mut sim_state = WorldSimState::new();
        let config = TierUpdateConfig {
            max_tier0_npcs: 2, // Player + 1 NPC
            max_tier1_npcs: 2,
            ..Default::default()
        };
        let mut rng = DeterministicRng::new(42);

        update_npc_tiers_for_tick(&world, &mut sim_state, &config, &mut rng);

        // Count tiers
        let mut tier0_count = 0;
        let mut tier1_count = 0;
        let mut tier2_count = 0;

        for i in 1..=6 {
            match sim_state.npc_tier(NpcId(i)) {
                NpcTier::Tier0 => tier0_count += 1,
                NpcTier::Tier1 => tier1_count += 1,
                NpcTier::Tier2 => tier2_count += 1,
            }
        }

        assert!(tier0_count <= config.max_tier0_npcs);
        assert!(tier1_count <= config.max_tier1_npcs);
        // 6 total - max(2 tier0) - max(2 tier1) = at least 2 tier2
        assert!(tier2_count >= 2);
    }
}
