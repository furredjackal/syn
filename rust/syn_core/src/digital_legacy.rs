//! Digital Legacy / PostLife layer for SYN simulation.
//!
//! Compresses a player's life into a DigitalImprint containing:
//! - Stats & karma over time
//! - Relationship roles & milestones
//! - Memory echoes & tags (betrayal, support, impact)
//!
//! In LifeStage::Digital (PostLife), simulation & storylets can operate on
//! this imprint instead of physical stats.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::relationship_milestones::RelationshipMilestoneEvent;
use crate::relationship_model::{RelationshipRole, RelationshipVector};
use crate::types::{LifeStage, NpcId};
use crate::{Karma, Stats}; // Re-exported from types

/// Aggregated high-level legacy traits distilled from stats, karma, relationships, memories.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LegacyVector {
    /// Overall tendency toward kindness vs cruelty.
    pub compassion_vs_cruelty: f32, // -1.0 .. 1.0

    /// How obsessively the player pursued impact vs comfort.
    pub ambition_vs_comfort: f32, // -1.0 .. 1.0

    /// How relational vs solitary they were.
    pub connection_vs_isolation: f32, // -1.0 .. 1.0

    /// How stable vs chaotic their life arc was.
    pub stability_vs_chaos: f32, // -1.0 .. 1.0

    /// How "bright" vs "dark" their karmic footprint is.
    pub light_vs_shadow: f32, // -1.0 .. 1.0
}

/// Digital imprint: a compressed "ghost profile" of a player's life.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigitalImprint {
    /// Snapshot id; for now we keep a single primary imprint per player.
    pub id: u64,

    /// Life stage at which this imprint was created (e.g. Digital/PostLife).
    pub created_at_stage: LifeStage,

    /// Age when imprint was taken.
    pub created_at_age_years: u32,

    /// Final physical stats snapshot (for reference).
    pub final_stats: Stats,

    /// Final karma snapshot.
    pub final_karma: Karma,

    /// Legacy vector summarizing the arc.
    pub legacy_vector: LegacyVector,

    /// Summary of relationship roles by target id.
    /// e.g. "Friend", "Rival", "Romance", "Family".
    pub relationship_roles: HashMap<NpcId, RelationshipRole>,

    /// Relationship milestones that were part of this legacy.
    pub relationship_milestones: Vec<RelationshipMilestoneEvent>,

    /// Tagged counts of memory themes (e.g. "betrayal": 3, "support": 5).
    pub memory_tag_counts: HashMap<String, u32>,
}

/// Wrapper for world-level legacy state.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DigitalLegacyState {
    /// Primary imprint for the player; None until PostLife.
    #[serde(default)]
    pub primary_imprint: Option<DigitalImprint>,

    /// Future: other imprints (snapshots at key life stages).
    #[serde(default)]
    pub archived_imprints: Vec<DigitalImprint>,
}

/// Input bundle for computing legacy vector.
pub struct LegacyInputs<'a> {
    pub final_stats: &'a Stats,
    pub final_karma: &'a Karma,
    pub relationships: &'a [(&'a (NpcId, NpcId), &'a RelationshipVector)],
    pub relationship_milestones: &'a [RelationshipMilestoneEvent],
    /// Flattened memory tag counts, e.g. "betrayal" -> 3, "support" -> 5
    pub memory_tag_counts: &'a HashMap<String, u32>,
}

/// Compute legacy vector from life inputs (deterministic).
pub fn compute_legacy_vector(inputs: &LegacyInputs<'_>) -> LegacyVector {
    let mut vec = LegacyVector::default();

    // 1) Compassion vs cruelty from karma + memory tags
    let k = inputs.final_karma.0; // Karma.0 is f32
    let light_norm = (k + 100.0) / 200.0; // map -100..100 -> 0..1
    vec.light_vs_shadow = (light_norm * 2.0 - 1.0).clamp(-1.0, 1.0);

    let support_count = inputs
        .memory_tag_counts
        .get("support")
        .copied()
        .unwrap_or(0) as f32;
    let betrayal_count = inputs
        .memory_tag_counts
        .get("betrayal")
        .copied()
        .unwrap_or(0) as f32;
    let total_rel = support_count + betrayal_count + 1.0;

    let compassion_score = (support_count - betrayal_count) / total_rel; // -1..1 approx
    vec.compassion_vs_cruelty = ((vec.light_vs_shadow + compassion_score) * 0.5).clamp(-1.0, 1.0);

    // 2) Ambition vs comfort: wealth, reputation, major-win tags
    let wealth = inputs.final_stats.wealth;
    let reputation = inputs.final_stats.reputation;
    let ambition_tags = inputs
        .memory_tag_counts
        .get("ambition")
        .copied()
        .unwrap_or(0) as f32
        + inputs
            .memory_tag_counts
            .get("career_win")
            .copied()
            .unwrap_or(0) as f32;

    let wealth_norm = (wealth / 10.0).clamp(0.0, 1.0);
    let rep_norm = ((reputation + 100.0) / 200.0).clamp(0.0, 1.0);
    let ambition_norm = (ambition_tags / 10.0).min(1.0);

    let ambition_score = (wealth_norm + rep_norm + ambition_norm) / 3.0; // 0..1
    vec.ambition_vs_comfort = (ambition_score * 2.0 - 1.0).clamp(-1.0, 1.0);

    // 3) Connection vs isolation from relationship roles + tags
    let mut social_score = 0.0f32;
    let mut social_count = 0.0f32;

    for ((_a, _t), rel_vec) in inputs.relationships {
        if rel_vec.role() != RelationshipRole::Stranger {
            // Count non-stranger relationships as connection
            social_score += 1.0;
        }
        social_count += 1.0;
    }

    let conn_norm = if social_count > 0.0 {
        (social_score / social_count).clamp(0.0, 1.0)
    } else {
        0.0
    };

    // Punish if we have many "isolation" or "withdrawal" tags
    let isolation_tags = inputs
        .memory_tag_counts
        .get("isolation")
        .copied()
        .unwrap_or(0) as f32;

    let isolation_penalty = (isolation_tags / 10.0).min(1.0);
    let conn_score = (conn_norm - isolation_penalty).clamp(-1.0, 1.0);

    vec.connection_vs_isolation = conn_score;

    // 4) Stability vs chaos from milestones & mood extremes
    let mut chaotic_events = 0.0f32;
    for ev in inputs.relationship_milestones {
        use crate::relationship_milestones::RelationshipMilestoneKind::*;
        match ev.kind {
            FriendToRival | RomanceCollapse => chaotic_events += 1.0,
            _ => {}
        }
    }
    let crisis_tags = inputs.memory_tag_counts.get("crisis").copied().unwrap_or(0) as f32;

    let chaos_norm = ((chaotic_events + crisis_tags) / 10.0).min(1.0);
    vec.stability_vs_chaos = (1.0 - chaos_norm) * 2.0 - 1.0; // 1..-1

    vec
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_legacy_vector_default() {
        let lv = LegacyVector::default();
        assert_eq!(lv.compassion_vs_cruelty, 0.0);
        assert_eq!(lv.ambition_vs_comfort, 0.0);
        assert_eq!(lv.connection_vs_isolation, 0.0);
        assert_eq!(lv.stability_vs_chaos, 0.0);
        assert_eq!(lv.light_vs_shadow, 0.0);
    }

    #[test]
    fn test_compute_legacy_vector_neutral() {
        let stats = Stats::default();
        let karma = Karma(0.0);
        let relationships = vec![];
        let milestones = vec![];
        let memory_tags = HashMap::new();

        let inputs = LegacyInputs {
            final_stats: &stats,
            final_karma: &karma,
            relationships: &relationships,
            relationship_milestones: &milestones,
            memory_tag_counts: &memory_tags,
        };

        let lv = compute_legacy_vector(&inputs);
        // All should be close to neutral
        assert!(lv.light_vs_shadow.abs() < 0.1);
        assert!(lv.compassion_vs_cruelty.abs() < 0.6);
    }
}
