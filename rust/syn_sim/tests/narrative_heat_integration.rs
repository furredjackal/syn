use syn_core::{NpcId, Relationship, RelationshipState, WorldSeed, WorldState};
use syn_sim::Simulator;

#[test]
fn narrative_heat_rises_with_extremes() {
    let mut world = WorldState::new(WorldSeed(1), NpcId(1));
    world.player_stats.mood = -9.0;
    world.player_stats.health = 5.0;
    world.player_stats.wealth = 5.0;

    world.relationships.insert(
        (NpcId(1), NpcId(2)),
        Relationship {
            affection: 0.0,
            trust: 0.0,
            attraction: 0.0,
            familiarity: 0.0,
            resentment: 8.0,
            state: RelationshipState::Stranger,
        },
    );

    let mut simulator = Simulator::new(1);
    simulator.instantiate_npc(syn_core::AbstractNpc {
        id: NpcId(1),
        age: 20,
        job: "Test".into(),
        district: "Test".into(),
        household_id: 1,
        traits: syn_core::Traits::default(),
        seed: 1,
        attachment_style: syn_core::AttachmentStyle::Secure,
    });

    // Run a few ticks to accumulate heat updates.
    for _ in 0..5 {
        simulator.tick(&mut world);
    }

    assert!(world.narrative_heat.value() > 10.0);
    assert!(matches!(
        world.narrative_heat.band(),
        syn_core::narrative_heat::NarrativeHeatBand::Medium
            | syn_core::narrative_heat::NarrativeHeatBand::High
            | syn_core::narrative_heat::NarrativeHeatBand::Critical
    ));
}
