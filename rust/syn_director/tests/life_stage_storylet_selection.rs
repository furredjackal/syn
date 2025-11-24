use syn_core::{LifeStage, NpcId, SimTick, WorldSeed, WorldState};
use syn_director::{
    EventDirector, Storylet, StoryletCooldown, StoryletOutcomeSet, StoryletPrerequisites,
    StoryletRoles, TagBitset,
};
use syn_memory::MemorySystem;

fn storylet_with_stage(id: &str, allowed: Vec<LifeStage>) -> Storylet {
    Storylet {
        id: id.to_string(),
        name: id.to_string(),
        tags: TagBitset::default(),
        prerequisites: StoryletPrerequisites {
            allowed_life_stages: allowed,
            ..Default::default()
        },
        roles: StoryletRoles::default(),
        heat: 10,
        triggers: Default::default(),
        outcomes: StoryletOutcomeSet::default(),
        cooldown: StoryletCooldown { ticks: 0 },
        weight: 1.0,
    }
}

#[test]
fn life_stage_prereqs_gate_storylets() {
    let mut director = EventDirector::new();
    let mut world = WorldState::new(WorldSeed(1), NpcId(1));
    world.player_life_stage = LifeStage::Teen;
    let memory = MemorySystem::new();

    let teen_storylet = storylet_with_stage("teen", vec![LifeStage::Teen]);
    let mid_storylet = storylet_with_stage("mid", vec![LifeStage::Adult]);

    director.register_storylet(teen_storylet.clone());
    director.register_storylet(mid_storylet.clone());

    let eligible = director.find_eligible(&world, &memory, SimTick(0));
    assert!(eligible.iter().any(|s| s.id == teen_storylet.id));
    assert!(!eligible.iter().any(|s| s.id == mid_storylet.id));
}
