use syn_core::{LifeStage, NpcId, SimTick, WorldSeed, WorldState};
use syn_director::{EventDirector, Storylet, StoryletPrerequisites};
use syn_memory::MemorySystem;

fn storylet_with_stage(id: &str, allowed: Vec<LifeStage>) -> Storylet {
    Storylet {
        id: id.to_string(),
        name: id.to_string(),
        tags: vec![],
        prerequisites: StoryletPrerequisites {
            min_relationship_affection: None,
            min_relationship_resentment: None,
            stat_conditions: Default::default(),
            life_stages: vec![],
            tags: vec![],
            relationship_states: vec![],
            memory_tags_required: vec![],
            memory_tags_forbidden: vec![],
            memory_recency_ticks: None,
            relationship_prereqs: vec![],
            allowed_life_stages: allowed,
        },
        heat: 10.0,
        weight: 1.0,
        cooldown_ticks: 0,
        roles: vec![],
        max_uses: None,
        choices: vec![],
        heat_category: None,
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
