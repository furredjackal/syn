use syn_core::relationship_model::RelationshipVector;
use syn_core::{NpcId, WorldSeed, WorldState};
use syn_sim::relationship_drift::{
    conflict_action_utility_modifier, RelationshipDriftConfig, RelationshipDriftSystem,
    social_action_utility_modifier,
};

#[test]
fn relationship_drift_moves_values_toward_zero_and_increases_familiarity() {
    let mut world = WorldState::new(WorldSeed(1), NpcId(1));
    let key = (NpcId(1), NpcId(2));
    world.relationships.insert(
        key,
        syn_core::Relationship {
            affection: 5.0,
            trust: -4.0,
            attraction: 0.0,
            familiarity: 0.0,
            resentment: 3.0,
            state: syn_core::RelationshipState::Stranger,
        },
    );

    let system = RelationshipDriftSystem::new(RelationshipDriftConfig {
        affection_decay_per_tick: 1.0,
        trust_decay_per_tick: 1.0,
        resentment_decay_per_tick: 0.5,
        familiarity_growth_per_tick: 0.2,
    });

    system.tick(&mut world);

    let rel = world.relationships.get(&key).unwrap();

    assert!(rel.affection < 5.0);
    assert!(rel.affection >= 0.0);
    assert!(rel.trust > -4.0);
    assert!(rel.trust <= 0.0);
    assert!(rel.resentment < 3.0);
    assert!(rel.familiarity > 0.0);
}

#[test]
fn social_utility_increases_with_affection_and_trust() {
    let low = RelationshipVector {
        affection: 0.0,
        trust: 0.0,
        attraction: 0.0,
        familiarity: 0.0,
        resentment: 0.0,
    };
    let high = RelationshipVector {
        affection: 8.0,
        trust: 5.0,
        attraction: 0.0,
        familiarity: 0.0,
        resentment: 0.0,
    };

    let low_mult = social_action_utility_modifier(&low);
    let high_mult = social_action_utility_modifier(&high);

    assert!(high_mult > low_mult);
}

#[test]
fn conflict_utility_increases_with_resentment() {
    let calm = RelationshipVector {
        affection: 0.0,
        trust: 0.0,
        attraction: 0.0,
        familiarity: 0.0,
        resentment: 0.0,
    };
    let angry = RelationshipVector {
        affection: 0.0,
        trust: 0.0,
        attraction: 0.0,
        familiarity: 0.0,
        resentment: 8.0,
    };

    let calm_mult = conflict_action_utility_modifier(&calm);
    let angry_mult = conflict_action_utility_modifier(&angry);

    assert!(angry_mult > calm_mult);
}
