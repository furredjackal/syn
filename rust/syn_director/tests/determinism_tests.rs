//! Tests for RNG determinism in storylet selection.
//!
//! Verifies that the same seed + state always produces the same storylet selection.

use syn_core::{NpcId, SimTick, WorldSeed, WorldState};
use syn_director::{CompiledEventDirector, DirectorConfig, EligibilityContext};
use syn_memory::MemorySystem;
use syn_storylets::library::{CompiledStorylet, StoryletKey, StoryletLibrary};
use syn_storylets::{Cooldowns, LifeStage, Outcome, Prerequisites, StoryDomain, StoryletId, Tag};

/// Create a test storylet with given parameters.
fn make_storylet(
    key: u32,
    id: &str,
    domain: StoryDomain,
    heat: u8,
    weight: f32,
    tags: Vec<&str>,
) -> CompiledStorylet {
    CompiledStorylet {
        key: StoryletKey(key),
        id: StoryletId::new(id),
        name: id.to_string(),
        description: Some(format!("{} description", id)),
        domain,
        life_stage: LifeStage::Adult,
        heat,
        weight,
        prerequisites: Prerequisites::default(),
        cooldowns: Cooldowns::default(),
        tags: tags.into_iter().map(Tag::new).collect(),
        outcomes: Outcome::default(),
        roles: vec![],
        follow_ups_resolved: vec![],
    }
}

/// Insert a storylet into the library.
fn insert_storylet(library: &mut StoryletLibrary, storylet: CompiledStorylet) {
    let key = storylet.key;
    let id = storylet.id.clone();
    let domain = storylet.domain.clone();
    let life_stage = storylet.life_stage.clone();
    let tags = storylet.tags.clone();

    library.id_to_key.insert(id, key);
    library.domain_index.entry(domain).or_default().push(key);
    library.life_stage_index.entry(life_stage).or_default().push(key);
    for tag in tags {
        library.tag_index.entry(tag).or_default().push(key);
    }
    library.storylets.push(storylet);
    library.total_count += 1;
}

/// Create a library with multiple eligible storylets of similar weight.
fn create_multi_storylet_library() -> StoryletLibrary {
    let mut library = StoryletLibrary::new();

    // Multiple storylets with equal weight to test tie-breaking determinism
    insert_storylet(
        &mut library,
        make_storylet(0, "storylet_a", StoryDomain::SliceOfLife, 3, 50.0, vec!["daily"]),
    );
    insert_storylet(
        &mut library,
        make_storylet(1, "storylet_b", StoryDomain::SliceOfLife, 3, 50.0, vec!["daily"]),
    );
    insert_storylet(
        &mut library,
        make_storylet(2, "storylet_c", StoryDomain::SliceOfLife, 3, 50.0, vec!["daily"]),
    );

    library
}

/// Create a world state with Adult life stage.
fn create_adult_world(seed: u64) -> WorldState {
    let mut world = WorldState::new(WorldSeed(seed), NpcId(1));
    world.player_life_stage = syn_core::LifeStage::Adult;
    world.player_age = 30;
    world.player_age_years = 30;
    world
}

/// Create an eligibility context.
fn create_context<'a>(
    world: &'a WorldState,
    memory: &'a MemorySystem,
    tick: SimTick,
) -> EligibilityContext<'a> {
    EligibilityContext {
        world,
        memory,
        current_tick: tick,
    }
}

#[test]
fn test_same_seed_produces_same_storylet_selection() {
    // Create two identical worlds with the same seed
    let world1 = create_adult_world(42);
    let world2 = create_adult_world(42);

    let memory1 = MemorySystem::new();
    let memory2 = MemorySystem::new();

    // Create two identical directors
    let library1 = create_multi_storylet_library();
    let library2 = create_multi_storylet_library();

    let config = DirectorConfig::for_testing();
    let mut director1 = CompiledEventDirector::new(library1, config.clone());
    let mut director2 = CompiledEventDirector::new(library2, config);

    // Step both at the same tick
    let tick = SimTick::new(5);
    let ctx1 = create_context(&world1, &memory1, tick);
    let ctx2 = create_context(&world2, &memory2, tick);

    let result1 = director1.step(tick, &ctx1);
    let result2 = director2.step(tick, &ctx2);

    // Both should fire something
    assert!(result1.fired.is_some(), "Director 1 should fire a storylet");
    assert!(result2.fired.is_some(), "Director 2 should fire a storylet");

    // Both should fire the SAME storylet
    let key1 = result1.fired.as_ref().unwrap().key;
    let key2 = result2.fired.as_ref().unwrap().key;
    assert_eq!(
        key1, key2,
        "Same seed should produce identical selection: {:?} vs {:?}",
        key1, key2
    );
}

#[test]
fn test_different_seeds_may_produce_different_selections() {
    // Create worlds with different seeds
    let world1 = create_adult_world(42);
    let world2 = create_adult_world(999);

    let memory = MemorySystem::new();

    let library1 = create_multi_storylet_library();
    let library2 = create_multi_storylet_library();

    let mut director1 = CompiledEventDirector::for_testing(library1);
    let mut director2 = CompiledEventDirector::for_testing(library2);

    let tick = SimTick::new(1);
    let ctx1 = create_context(&world1, &memory, tick);
    let ctx2 = create_context(&world2, &memory, tick);

    let result1 = director1.step(tick, &ctx1);
    let result2 = director2.step(tick, &ctx2);

    // Both should fire something (just verify no crash)
    assert!(result1.fired.is_some(), "Director 1 should fire");
    assert!(result2.fired.is_some(), "Director 2 should fire");

    // Note: Different seeds may or may not produce different results
    // depending on weight distribution and jitter. We just verify both work.
}

#[test]
fn test_sequence_of_selections_is_deterministic() {
    let world = create_adult_world(12345);
    let memory = MemorySystem::new();

    // Run the same sequence twice with fresh directors
    let mut results1 = Vec::new();
    let mut results2 = Vec::new();

    // First run
    let library1 = create_multi_storylet_library();
    let config1 = DirectorConfig::for_testing();
    let mut director1 = CompiledEventDirector::new(library1, config1);

    for tick_val in 1..=10 {
        let tick = SimTick::new(tick_val);
        let ctx = create_context(&world, &memory, tick);

        let r1 = director1.step(tick, &ctx);
        results1.push(r1.fired.map(|f| f.key));
    }

    // Second run with fresh director (same world seed)
    let library2 = create_multi_storylet_library();
    let config2 = DirectorConfig::for_testing();
    let mut director2 = CompiledEventDirector::new(library2, config2);

    for tick_val in 1..=10 {
        let tick = SimTick::new(tick_val);
        let ctx = create_context(&world, &memory, tick);

        let r2 = director2.step(tick, &ctx);
        results2.push(r2.fired.map(|f| f.key));
    }

    // Sequences should be identical
    assert_eq!(
        results1, results2,
        "Same seed should produce identical sequence of selections"
    );
}
