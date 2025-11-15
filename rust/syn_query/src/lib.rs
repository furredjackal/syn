//! syn_query: Read-only query helpers for world state.
//!
//! Provides efficient lookups and filters for NPCs, relationships, and events.
//! Used by syn_sim and syn_director to gather data for decisions.

#[allow(unused_imports)]
use syn_core::{AbstractNpc, NpcId, Relationship, Traits, WorldState};

/// Query builder for finding NPCs by various criteria.
#[derive(Default)]
pub struct NpcQuery {
    filters: Vec<Box<dyn Fn(&AbstractNpc) -> bool>>,
}

impl NpcQuery {
    pub fn new() -> Self {
        NpcQuery::default()
    }

    /// Filter NPCs by age range.
    pub fn with_age_range(mut self, min_age: u32, max_age: u32) -> Self {
        self.filters.push(Box::new(move |npc| npc.age >= min_age && npc.age <= max_age));
        self
    }

    /// Filter NPCs by job.
    pub fn with_job(mut self, job: String) -> Self {
        self.filters.push(Box::new(move |npc| npc.job == job));
        self
    }

    /// Filter NPCs by district.
    pub fn with_district(mut self, district: String) -> Self {
        self.filters.push(Box::new(move |npc| npc.district == district));
        self
    }

    /// Apply filters to NPCs in world state.
    pub fn execute<'a>(&self, world: &'a WorldState) -> Vec<&'a AbstractNpc> {
        world
            .npcs
            .values()
            .filter(|npc| self.filters.iter().all(|f| f(npc)))
            .collect()
    }
}

/// Query relationships for an NPC.
pub struct RelationshipQuery;

impl RelationshipQuery {
    /// Find all NPCs with high affection toward a target.
    pub fn find_close_relations(world: &WorldState, npc_id: NpcId, threshold: f32) -> Vec<NpcId> {
        world
            .relationships
            .iter()
            .filter(|((from, _to), rel)| from == &npc_id && rel.affection >= threshold)
            .map(|((_from, to), _rel)| *to)
            .collect()
    }

    /// Find all NPCs with high resentment toward a target.
    pub fn find_resentful_relations(world: &WorldState, npc_id: NpcId, threshold: f32) -> Vec<NpcId> {
        world
            .relationships
            .iter()
            .filter(|((from, _to), rel)| from == &npc_id && rel.resentment >= threshold)
            .map(|((_from, to), _rel)| *to)
            .collect()
    }

    /// Calculate mutual affection between two NPCs.
    pub fn mutual_affection(world: &WorldState, npc_a: NpcId, npc_b: NpcId) -> f32 {
        let affection_a = world.get_relationship(npc_a, npc_b).affection;
        let affection_b = world.get_relationship(npc_b, npc_a).affection;
        (affection_a + affection_b) / 2.0
    }

    /// Check if two NPCs have a "pressure point" (high-tension combination).
    /// Returns true if the relationship is likely to trigger conflict.
    pub fn has_pressure_point(world: &WorldState, npc_a: NpcId, npc_b: NpcId) -> bool {
        let rel_ab = world.get_relationship(npc_a, npc_b);
        let rel_ba = world.get_relationship(npc_b, npc_a);

        // High resentment + low stability = pressure
        let has_high_resentment = rel_ab.resentment > 5.0 || rel_ba.resentment > 5.0;
        let low_stability = world
            .npcs
            .get(&npc_a)
            .map(|n| n.traits.stability < 40.0)
            .unwrap_or(false)
            || world
                .npcs
                .get(&npc_b)
                .map(|n| n.traits.stability < 40.0)
                .unwrap_or(false);

        has_high_resentment && low_stability
    }
}

/// Stat-based queries.
pub struct StatQuery;

impl StatQuery {
    /// Find NPCs in critical stat ranges (likely to trigger events).
    pub fn find_at_risk(world: &WorldState) -> Vec<NpcId> {
        world
            .npcs
            .iter()
            .filter(|(_npc_id, npc)| {
                // Placeholder: would query actual instantiated stats in a full implementation
                // For now, use traits as proxy
                npc.traits.stability < 30.0 || npc.traits.ambition > 80.0
            })
            .map(|(id, _)| *id)
            .collect()
    }
}

/// Event/storylet matching queries.
pub struct StoryletQuery;

impl StoryletQuery {
    /// Get all NPCs matching a tag-based filter (e.g., "workplace rival").
    pub fn find_by_tags(world: &WorldState, tags: &[&str]) -> Vec<NpcId> {
        // Placeholder: in full implementation, this would query storylet prerequisites
        // For now, return NPCs based on simple criteria
        world
            .npcs
            .iter()
            .filter(|(_npc_id, npc)| {
                tags.iter().any(|tag| match *tag {
                    "ambitious" => npc.traits.ambition > 60.0,
                    "stable" => npc.traits.stability > 60.0,
                    "social" => npc.traits.sociability > 60.0,
                    _ => false,
                })
            })
            .map(|(id, _)| *id)
            .collect()
    }
}

/// Utility for finding neighbors or clustered NPCs.
pub struct ClusterQuery;

impl ClusterQuery {
    /// Find NPCs in the same district.
    pub fn find_in_district(world: &WorldState, district: &str) -> Vec<NpcId> {
        world
            .npcs
            .iter()
            .filter(|(_, npc)| npc.district == district)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Find NPCs in the same household.
    pub fn find_in_household(world: &WorldState, household_id: u64) -> Vec<NpcId> {
        world
            .npcs
            .iter()
            .filter(|(_, npc)| npc.household_id == household_id)
            .map(|(id, _)| *id)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn_core::{AttachmentStyle, NpcId, WorldSeed, WorldState};

    #[test]
    fn test_npc_query_age_range() {
        let mut world = WorldState::new(WorldSeed(42), NpcId(1));
        world.npcs.insert(
            NpcId(2),
            AbstractNpc {
                id: NpcId(2),
                age: 25,
                job: "Engineer".to_string(),
                district: "Downtown".to_string(),
                household_id: 1,
                traits: Traits::default(),
                seed: 123,
                attachment_style: AttachmentStyle::Secure,
            },
        );
        world.npcs.insert(
            NpcId(3),
            AbstractNpc {
                id: NpcId(3),
                age: 45,
                job: "Manager".to_string(),
                district: "Uptown".to_string(),
                household_id: 2,
                traits: Traits::default(),
                seed: 124,
                attachment_style: AttachmentStyle::Secure,
            },
        );

        let query = NpcQuery::new().with_age_range(20, 30);
        let results = query.execute(&world);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].age, 25);
    }

    #[test]
    fn test_relationship_mutual_affection() {
        let mut world = WorldState::new(WorldSeed(42), NpcId(1));
        let rel1 = Relationship {
            affection: 5.0,
            ..Default::default()
        };
        let rel2 = Relationship {
            affection: 7.0,
            ..Default::default()
        };
        world.set_relationship(NpcId(1), NpcId(2), rel1);
        world.set_relationship(NpcId(2), NpcId(1), rel2);

        let mutual = RelationshipQuery::mutual_affection(&world, NpcId(1), NpcId(2));
        assert_eq!(mutual, 6.0);
    }

    #[test]
    fn test_cluster_query_district() {
        let mut world = WorldState::new(WorldSeed(42), NpcId(1));
        world.npcs.insert(
            NpcId(2),
            AbstractNpc {
                id: NpcId(2),
                age: 25,
                job: "Engineer".to_string(),
                district: "Downtown".to_string(),
                household_id: 1,
                traits: Traits::default(),
                seed: 123,
                attachment_style: AttachmentStyle::Secure,
            },
        );
        world.npcs.insert(
            NpcId(3),
            AbstractNpc {
                id: NpcId(3),
                age: 30,
                job: "Artist".to_string(),
                district: "Downtown".to_string(),
                household_id: 2,
                traits: Traits::default(),
                seed: 124,
                attachment_style: AttachmentStyle::Secure,
            },
        );

        let in_district = ClusterQuery::find_in_district(&world, "Downtown");
        assert_eq!(in_district.len(), 2);
    }
}
