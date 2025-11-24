use syn_core::narrative_heat::NarrativeHeatBand;
use syn_core::{NpcId, SimTick, WorldSeed, WorldState};
use syn_director::{
    EventDirector, Storylet, StoryletCooldown, StoryletHeatCategory, StoryletOutcome,
    StoryletOutcomeSet, StoryletPrerequisites, StoryletRoles, TagBitset,
};
use syn_memory::MemorySystem;

fn basic_storylet(id: &str, category: StoryletHeatCategory) -> Storylet {
    let mut outcomes = StoryletOutcomeSet::default();
    outcomes.heat_category = Some(category);
    Storylet {
        id: id.to_string(),
        name: id.to_string(),
        tags: TagBitset::default(),
        prerequisites: StoryletPrerequisites::default(),
        roles: StoryletRoles::default(),
        heat: 10,
        triggers: Default::default(),
        outcomes,
        cooldown: StoryletCooldown { ticks: 0 },
        weight: 1.0,
    }
}

#[test]
fn heat_band_influences_selection() {
    let mut director = EventDirector::new();
    let mut world = WorldState::new(WorldSeed(1), NpcId(1));
    let memory = MemorySystem::new();

    let slice = basic_storylet("slice", StoryletHeatCategory::SliceOfLife);
    let critical = basic_storylet("critical", StoryletHeatCategory::CriticalArc);
    director.register_storylet(slice.clone());
    director.register_storylet(critical.clone());

    world.narrative_heat.set(10.0);
    let selected = director
        .select_next_event(&world, &memory, SimTick(0))
        .expect("expected selection");
    assert_eq!(selected.id, "slice");

    world.narrative_heat.set(85.0);
    let selected = director
        .select_next_event(&world, &memory, SimTick(0))
        .expect("expected selection");
    assert_eq!(world.narrative_heat.band(), NarrativeHeatBand::Critical);
    assert_eq!(selected.id, "critical");
}

#[test]
fn critical_arc_cools_down_heat() {
    let mut director = EventDirector::new();
    let mut world = WorldState::new(WorldSeed(1), NpcId(1));
    let mut memory = MemorySystem::new();

    let critical = basic_storylet("critical", StoryletHeatCategory::CriticalArc);
    director.register_storylet(critical.clone());

    world.narrative_heat.set(90.0);

    director.fire_storylet(
        &critical,
        &mut world,
        &mut memory,
        StoryletOutcome::default(),
        SimTick(0),
    );

    assert!(world.narrative_heat.value() < 90.0);
}
