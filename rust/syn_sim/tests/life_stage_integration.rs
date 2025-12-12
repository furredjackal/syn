use syn_core::LifeStage;
use syn_core::{NpcId, WorldSeed, WorldState};
use syn_sim::{SimulationTickConfig, WorldSimState, tick_simulation};

#[test]
fn teen_heat_weights_are_stronger_than_late() {
    let mut world = WorldState::new(WorldSeed(1), NpcId(1));
    world.player_age = 15;
    world.player_age_years = 15;
    world.player_life_stage = LifeStage::Teen;
    world.player_stats.mood = -9.0;
    world.player_stats.health = 10.0;
    world.player_stats.wealth = 10.0;

    let mut world_sim = WorldSimState::new();
    let config = SimulationTickConfig::default();
    tick_simulation(&mut world, &mut world_sim, &config);
    let teen_heat = world.narrative_heat.value();

    let mut world_late = WorldState::new(WorldSeed(1), NpcId(1));
    world_late.player_age = 70;
    world_late.player_age_years = 70;
    world_late.player_life_stage = LifeStage::Elder;
    world_late.player_stats.mood = -9.0;
    world_late.player_stats.health = 10.0;
    world_late.player_stats.wealth = 10.0;

    let mut world_sim_late = WorldSimState::new();
    tick_simulation(&mut world_late, &mut world_sim_late, &config);
    let elder_heat = world_late.narrative_heat.value();

    assert!(teen_heat > elder_heat);
}
