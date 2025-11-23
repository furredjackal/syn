//! PostLife / Digital Legacy simulation helpers.
//!
//! Builds the DigitalImprint when entering LifeStage::Digital (PostLife).

use syn_core::{
    digital_legacy::{compute_legacy_vector, DigitalImprint, LegacyInputs},
    relationship_model::RelationshipVector,
    LifeStage, NpcId, WorldState,
};
use syn_memory::MemoryEntry;
use std::collections::HashMap;

/// Build a digital imprint from world state and memory entries.
pub fn build_digital_imprint(
    world: &WorldState,
    memory_entries: &[MemoryEntry],
) -> DigitalImprint {
    let player_id = world.player_id;

    // Build memory tag counts for player
    let tag_counts = syn_memory::tag_counts_for_actor(memory_entries, player_id.0);

    // Collect relationships slice for LegacyInputs
    let relationships: Vec<(&(NpcId, NpcId), &RelationshipVector)> = world
        .relationships
        .iter()
        .map(|(k, v)| {
            let rel_vec = RelationshipVector {
                affection: v.affection,
                trust: v.trust,
                attraction: v.attraction,
                familiarity: v.familiarity,
                resentment: v.resentment,
            };
            // SAFETY: we're creating a temporary RelationshipVector that lives
            // as long as this function, which is fine for LegacyInputs.
            // This is a workaround for the fact that WorldState uses the old
            // Relationship type, not RelationshipVector.
            // In production, you'd want to unify these types or refactor.
            // For now, we'll leak these vectors (small, one-time operation).
            let leaked: &'static RelationshipVector = Box::leak(Box::new(rel_vec));
            (k, leaked)
        })
        .collect();

    // Filter milestones involving the player
    let rel_milestones: Vec<_> = world
        .relationship_milestones
        .queue
        .iter()
        .cloned()
        .filter(|ev| ev.actor_id == player_id.0 || ev.target_id == player_id.0)
        .collect();

    let inputs = LegacyInputs {
        final_stats: &world.player_stats,
        final_karma: &world.player_karma,
        relationships: &relationships,
        relationship_milestones: &rel_milestones,
        memory_tag_counts: &tag_counts,
    };

    let legacy_vector = compute_legacy_vector(&inputs);

    let relationship_roles = relationships
        .iter()
        .filter_map(|((actor, target), rel)| {
            if *actor == player_id {
                Some((*target, rel.role()))
            } else {
                None
            }
        })
        .collect::<HashMap<_, _>>();

    DigitalImprint {
        id: 1, // simple constant for now; can be replaced by a sequence later.
        created_at_stage: world.player_life_stage,
        created_at_age_years: world.player_age_years,
        final_stats: world.player_stats,
        final_karma: world.player_karma,
        legacy_vector,
        relationship_roles,
        relationship_milestones: rel_milestones,
        memory_tag_counts: tag_counts,
    }
}

/// Ensure digital imprint is created when entering PostLife/Digital stage.
/// Should be called after life_stage has been updated.
pub fn ensure_digital_imprint_for_postlife(
    world: &mut WorldState,
    memory_entries: &[MemoryEntry],
) {
    // Only create if in Digital (PostLife) and no primary imprint yet.
    if !matches!(world.player_life_stage, LifeStage::Digital) {
        return;
    }
    if world.digital_legacy.primary_imprint.is_some() {
        return;
    }

    let imprint = build_digital_imprint(world, memory_entries);
    world.digital_legacy.primary_imprint = Some(imprint);
}

/// Optional PostLife drift: slowly smooths the legacy vector toward neutral.
pub fn tick_postlife_drift(world: &mut WorldState) {
    if !matches!(world.player_life_stage, LifeStage::Digital) {
        return;
    }
    if let Some(imprint) = &mut world.digital_legacy.primary_imprint {
        // Optional: slowly dampen extreme values so the ghost settles.
        let lerp = |v: &mut f32, target: f32, factor: f32| {
            *v = *v + (target - *v) * factor;
        };

        let lv = &mut imprint.legacy_vector;
        lerp(&mut lv.stability_vs_chaos, 0.0, 0.01); // drift toward neutral
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn_core::{Karma, NpcId, SimTick, Stats, Traits, WorldSeed, AttachmentStyle, AbstractNpc};
    use syn_memory::MemoryEntry;

    #[test]
    fn test_build_digital_imprint() {
        let mut world = WorldState::new(WorldSeed(42), NpcId(1));
        world.player_life_stage = LifeStage::Digital;
        world.player_age_years = 91;
        world.player_stats = Stats::default();
        world.player_karma = Karma(50.0);

        // Add an NPC for the player
        let player_npc = AbstractNpc {
            id: NpcId(1),
            age: 91,
            job: "Retired".to_string(),
            district: "Heaven".to_string(),
            household_id: 1,
            traits: Traits::default(),
            seed: 123,
            attachment_style: AttachmentStyle::Secure,
        };
        world.npcs.insert(NpcId(1), player_npc);

        let mut mem = MemoryEntry::new(
            "mem1".to_string(),
            "evt1".to_string(),
            NpcId(1),
            SimTick(100),
            0.8,
        )
        .with_tags(vec!["support", "kindness"]);
        mem.participants = vec![1]; // Mark player as participant

        let memories = vec![mem];

        let imprint = build_digital_imprint(&world, &memories);
        assert_eq!(imprint.created_at_stage, LifeStage::Digital);
        assert_eq!(imprint.created_at_age_years, 91);
        assert!(imprint.memory_tag_counts.contains_key("support"));
    }

    #[test]
    fn test_ensure_digital_imprint_for_postlife() {
        let mut world = WorldState::new(WorldSeed(42), NpcId(1));
        world.player_life_stage = LifeStage::Digital;
        world.player_age_years = 91;

        let memories = vec![];
        ensure_digital_imprint_for_postlife(&mut world, &memories);

        assert!(world.digital_legacy.primary_imprint.is_some());
    }

    #[test]
    fn test_tick_postlife_drift() {
        let mut world = WorldState::new(WorldSeed(42), NpcId(1));
        world.player_life_stage = LifeStage::Digital;

        let memories = vec![];
        ensure_digital_imprint_for_postlife(&mut world, &memories);

        // Set an extreme stability value
        if let Some(imprint) = &mut world.digital_legacy.primary_imprint {
            imprint.legacy_vector.stability_vs_chaos = 0.9;
        }

        // Drift for multiple ticks
        for _ in 0..100 {
            tick_postlife_drift(&mut world);
        }

        // Should have drifted toward 0
        if let Some(imprint) = &world.digital_legacy.primary_imprint {
            assert!(imprint.legacy_vector.stability_vs_chaos < 0.9);
        }
    }
}
