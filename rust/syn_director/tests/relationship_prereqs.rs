use syn_director::{EventDirector, RelationshipPrereq, Storylet, StoryletPrerequisites, StoryletRole};
use syn_core::relationship_model::RelationshipAxis;
use syn_core::{NpcId, Relationship, WorldSeed, WorldState, SimTick};
use syn_memory::MemorySystem;
use std::collections::HashMap;

#[test]
fn relationship_prereqs_pass_when_affection_in_range() {
    let mut director = EventDirector::new();
    let mut world = WorldState::new(WorldSeed(42), NpcId(1));
    let memory = MemorySystem::new();

    // Create an NPC
    world.npcs.insert(
        NpcId(2),
        syn_core::AbstractNpc {
            id: NpcId(2),
            age: 30,
            job: "Teacher".to_string(),
            district: "Downtown".to_string(),
            household_id: 1,
            traits: syn_core::Traits::default(),
            seed: 12345,
            attachment_style: syn_core::AttachmentStyle::Secure,
        },
    );

    // Set relationship with affection = 5.0 (in range 2.5-10.0)
    world.set_relationship(
        NpcId(1),
        NpcId(2),
        Relationship {
            affection: 5.0,
            trust: 3.0,
            attraction: 2.0,
            familiarity: 4.0,
            resentment: 0.0,
            state: syn_core::RelationshipState::Friend,
        },
    );

    // Create storylet that requires affection between 2.5 and 10.0
    let storylet = Storylet {
        id: "affectionate_event".to_string(),
        name: "Friendly interaction".to_string(),
        tags: vec![],
        prerequisites: StoryletPrerequisites {
            min_relationship_affection: None,
            min_relationship_resentment: None,
            stat_conditions: HashMap::new(),
            life_stages: vec![],
            tags: vec![],
            relationship_states: vec![],
            memory_tags_required: vec![],
            memory_tags_forbidden: vec![],
            memory_recency_ticks: None,
            relationship_prereqs: vec![RelationshipPrereq {
                actor_id: None, // defaults to player
                target_id: 2,
                axis: RelationshipAxis::Affection,
                min_value: Some(2.5),
                max_value: Some(10.0),
                min_band: None,
                max_band: None,
            }],
            allowed_life_stages: vec![],
        },
        heat: 50.0,
        weight: 0.5,
        cooldown_ticks: 100,
        roles: vec![StoryletRole {
            name: "target".to_string(),
            npc_id: NpcId(2),
        }],
        heat_category: None,
    };

    // Register the storylet and check eligibility via find_eligible
    director.register_storylet(storylet.clone());
    let eligible = director.find_eligible(&world, &memory, SimTick(0));
    assert!(eligible.iter().any(|s| s.id == storylet.id));
}

#[test]
fn relationship_prereqs_fail_when_affection_below_min() {
    let mut director = EventDirector::new();
    let mut world = WorldState::new(WorldSeed(42), NpcId(1));
    let memory = MemorySystem::new();

    // Create an NPC
    world.npcs.insert(
        NpcId(2),
        syn_core::AbstractNpc {
            id: NpcId(2),
            age: 30,
            job: "Teacher".to_string(),
            district: "Downtown".to_string(),
            household_id: 1,
            traits: syn_core::Traits::default(),
            seed: 12345,
            attachment_style: syn_core::AttachmentStyle::Secure,
        },
    );

    // Set relationship with affection = 1.0 (below min 2.5)
    world.set_relationship(
        NpcId(1),
        NpcId(2),
        Relationship {
            affection: 1.0,
            trust: 3.0,
            attraction: 2.0,
            familiarity: 4.0,
            resentment: 0.0,
            state: syn_core::RelationshipState::Friend,
        },
    );

    // Create storylet that requires affection >= 2.5
    let storylet = Storylet {
        id: "affectionate_event".to_string(),
        name: "Friendly interaction".to_string(),
        tags: vec![],
        prerequisites: StoryletPrerequisites {
            min_relationship_affection: None,
            min_relationship_resentment: None,
            stat_conditions: HashMap::new(),
            life_stages: vec![],
            tags: vec![],
            relationship_states: vec![],
            memory_tags_required: vec![],
            memory_tags_forbidden: vec![],
            memory_recency_ticks: None,
            relationship_prereqs: vec![RelationshipPrereq {
                actor_id: None,
                target_id: 2,
                axis: RelationshipAxis::Affection,
                min_value: Some(2.5),
                max_value: None,
                min_band: None,
                max_band: None,
            }],
            allowed_life_stages: vec![],
        },
        heat: 50.0,
        weight: 0.5,
        cooldown_ticks: 100,
        roles: vec![StoryletRole {
            name: "target".to_string(),
            npc_id: NpcId(2),
        }],
        heat_category: None,
    };

    // Register the storylet and check it's not eligible
    director.register_storylet(storylet.clone());
    let eligible = director.find_eligible(&world, &memory, SimTick(0));
    assert!(!eligible.iter().any(|s| s.id == storylet.id));
}

#[test]
fn relationship_prereqs_pass_with_band_based_criteria() {
    let mut director = EventDirector::new();
    let mut world = WorldState::new(WorldSeed(42), NpcId(1));
    let memory = MemorySystem::new();

    // Create an NPC
    world.npcs.insert(
        NpcId(2),
        syn_core::AbstractNpc {
            id: NpcId(2),
            age: 30,
            job: "Teacher".to_string(),
            district: "Downtown".to_string(),
            household_id: 1,
            traits: syn_core::Traits::default(),
            seed: 12345,
            attachment_style: syn_core::AttachmentStyle::Secure,
        },
    );

    // Set relationship with affection = 6.0 (should be "Close" band)
    world.set_relationship(
        NpcId(1),
        NpcId(2),
        Relationship {
            affection: 6.0,
            trust: 3.0,
            attraction: 2.0,
            familiarity: 4.0,
            resentment: 0.0,
            state: syn_core::RelationshipState::Friend,
        },
    );

    // Create storylet that requires at least "Friendly" band
    let storylet = Storylet {
        id: "close_event".to_string(),
        name: "Close interaction".to_string(),
        tags: vec![],
        prerequisites: StoryletPrerequisites {
            min_relationship_affection: None,
            min_relationship_resentment: None,
            stat_conditions: HashMap::new(),
            life_stages: vec![],
            tags: vec![],
            relationship_states: vec![],
            memory_tags_required: vec![],
            memory_tags_forbidden: vec![],
            memory_recency_ticks: None,
            relationship_prereqs: vec![RelationshipPrereq {
                actor_id: None,
                target_id: 2,
                axis: RelationshipAxis::Affection,
                min_value: None,
                max_value: None,
                min_band: Some("Friendly".to_string()),
                max_band: None,
            }],
            allowed_life_stages: vec![],
        },
        heat: 50.0,
        weight: 0.5,
        cooldown_ticks: 100,
        roles: vec![StoryletRole {
            name: "target".to_string(),
            npc_id: NpcId(2),
        }],
        heat_category: None,
    };

    // Register the storylet and check eligibility
    director.register_storylet(storylet.clone());
    let eligible = director.find_eligible(&world, &memory, SimTick(0));
    // Should be eligible (6.0 affection is in "Close" band, which is >= "Friendly")
    assert!(eligible.iter().any(|s| s.id == storylet.id));
}

#[test]
fn relationship_prereqs_fail_when_relationship_missing() {
    let mut director = EventDirector::new();
    let mut world = WorldState::new(WorldSeed(42), NpcId(1));
    let memory = MemorySystem::new();

    // Create an NPC but don't set a relationship
    world.npcs.insert(
        NpcId(2),
        syn_core::AbstractNpc {
            id: NpcId(2),
            age: 30,
            job: "Teacher".to_string(),
            district: "Downtown".to_string(),
            household_id: 1,
            traits: syn_core::Traits::default(),
            seed: 12345,
            attachment_style: syn_core::AttachmentStyle::Secure,
        },
    );

    // Create storylet that requires a relationship
    let storylet = Storylet {
        id: "missing_rel_event".to_string(),
        name: "Requires relationship".to_string(),
        tags: vec![],
        prerequisites: StoryletPrerequisites {
            min_relationship_affection: None,
            min_relationship_resentment: None,
            stat_conditions: HashMap::new(),
            life_stages: vec![],
            tags: vec![],
            relationship_states: vec![],
            memory_tags_required: vec![],
            memory_tags_forbidden: vec![],
            memory_recency_ticks: None,
            relationship_prereqs: vec![RelationshipPrereq {
                actor_id: None,
                target_id: 2,
                axis: RelationshipAxis::Affection,
                min_value: Some(0.0),
                max_value: None,
                min_band: None,
                max_band: None,
            }],
            allowed_life_stages: vec![],
        },
        heat: 50.0,
        weight: 0.5,
        cooldown_ticks: 100,
        roles: vec![StoryletRole {
            name: "target".to_string(),
            npc_id: NpcId(2),
        }],
        heat_category: None,
    };

    // Register the storylet and check it's not eligible
    director.register_storylet(storylet.clone());
    let eligible = director.find_eligible(&world, &memory, SimTick(0));
    // Should NOT be eligible (no relationship exists)
    assert!(!eligible.iter().any(|s| s.id == storylet.id));
}

#[test]
fn relationship_prereqs_with_explicit_actor_id() {
    let mut director = EventDirector::new();
    let mut world = WorldState::new(WorldSeed(42), NpcId(1));
    let memory = MemorySystem::new();

    // Create two NPCs
    world.npcs.insert(
        NpcId(2),
        syn_core::AbstractNpc {
            id: NpcId(2),
            age: 30,
            job: "Teacher".to_string(),
            district: "Downtown".to_string(),
            household_id: 1,
            traits: syn_core::Traits::default(),
            seed: 12345,
            attachment_style: syn_core::AttachmentStyle::Secure,
        },
    );

    world.npcs.insert(
        NpcId(3),
        syn_core::AbstractNpc {
            id: NpcId(3),
            age: 25,
            job: "Engineer".to_string(),
            district: "Uptown".to_string(),
            household_id: 2,
            traits: syn_core::Traits::default(),
            seed: 12346,
            attachment_style: syn_core::AttachmentStyle::Secure,
        },
    );

    // Set relationship between NPC 2 and NPC 3 (not player)
    world.set_relationship(
        NpcId(2),
        NpcId(3),
        Relationship {
            affection: 7.0,
            trust: 5.0,
            attraction: 2.0,
            familiarity: 6.0,
            resentment: 0.0,
            state: syn_core::RelationshipState::Friend,
        },
    );

    // Create storylet that checks relationship between NPC 2 and NPC 3
    let storylet = Storylet {
        id: "npc_to_npc_event".to_string(),
        name: "NPC relationship event".to_string(),
        tags: vec![],
        prerequisites: StoryletPrerequisites {
            min_relationship_affection: None,
            min_relationship_resentment: None,
            stat_conditions: HashMap::new(),
            life_stages: vec![],
            tags: vec![],
            relationship_states: vec![],
            memory_tags_required: vec![],
            memory_tags_forbidden: vec![],
            memory_recency_ticks: None,
            relationship_prereqs: vec![RelationshipPrereq {
                actor_id: Some(2), // Explicitly check NPC 2's relationship
                target_id: 3,
                axis: RelationshipAxis::Affection,
                min_value: Some(5.0),
                max_value: None,
                min_band: None,
                max_band: None,
            }],
            allowed_life_stages: vec![],
        },
        heat: 50.0,
        weight: 0.5,
        cooldown_ticks: 100,
        roles: vec![],
        heat_category: None,
    };

    // Register the storylet and check eligibility
    director.register_storylet(storylet.clone());
    let eligible = director.find_eligible(&world, &memory, SimTick(0));
    // Should be eligible (NPC 2 has affection 7.0 with NPC 3, which is >= 5.0)
    assert!(eligible.iter().any(|s| s.id == storylet.id));
}
