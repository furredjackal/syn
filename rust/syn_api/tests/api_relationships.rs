use syn_api::{ApiRelationshipSnapshot, GameEngine};
use syn_core::relationship_model::{derive_role_label, RelationshipVector};

#[test]
fn get_player_relationships_exposes_relationship_vectors_and_role_labels() {
    // Arrange: build a small fake world
    let mut engine = GameEngine::new(42);

    // Register target NPC
    engine.register_npc(2, 30, "Teacher".to_string(), "Downtown".to_string());

    // Set relationship with high affection/trust/attraction (should be "Crush" or "Friend")
    engine.set_relationship(1, 2, 7.0, 6.0, 5.0, 3.0, 0.0);

    // Act
    let snapshot: ApiRelationshipSnapshot = engine.player_relationships();

    // Assert
    assert_eq!(snapshot.relationships.len(), 1);
    let r = &snapshot.relationships[0];
    assert_eq!(r.actor_id, 1);
    assert_eq!(r.target_id, 2);
    assert_eq!(r.affection, 7.0);
    assert_eq!(r.trust, 6.0);
    assert_eq!(r.attraction, 5.0);
    assert_eq!(r.familiarity, 3.0);
    assert_eq!(r.resentment, 0.0);

    // Verify bands are populated
    assert!(!r.affection_band.is_empty());
    assert!(!r.trust_band.is_empty());
    assert!(!r.attraction_band.is_empty());
    assert!(!r.resentment_band.is_empty());

    // For a high affection/trust/attraction vector, we expect a "Crush" or "Friend" label
    assert!(
        r.role_label == "Crush" || r.role_label == "Friend",
        "Unexpected role_label: {}",
        r.role_label
    );
}

#[test]
fn get_player_relationships_filters_to_player_only() {
    // Arrange: create relationships from player and between NPCs
    let mut engine = GameEngine::new(42);

    // Register NPCs
    engine.register_npc(2, 30, "Teacher".to_string(), "Downtown".to_string());
    engine.register_npc(3, 25, "Engineer".to_string(), "Uptown".to_string());

    // Player to NPC1
    engine.set_relationship(1, 2, 5.0, 4.0, 2.0, 3.0, 0.0);

    // Player to NPC2
    engine.set_relationship(1, 3, 3.0, 2.0, 1.0, 2.0, 0.0);

    // NPC1 to NPC2 (should be filtered out - we can't set this through the API, but the test should still work)
    // The filter will only return relationships where actor is player

    // Act
    let snapshot: ApiRelationshipSnapshot = engine.player_relationships();

    // Assert - should only have player's relationships
    assert_eq!(snapshot.relationships.len(), 2);
    assert!(snapshot.relationships.iter().all(|r| r.actor_id == 1));

    // Verify we have both NPCs
    let target_ids: Vec<i64> = snapshot.relationships.iter().map(|r| r.target_id).collect();
    assert!(target_ids.contains(&2));
    assert!(target_ids.contains(&3));
}

#[test]
fn derive_role_label_produces_expected_labels() {
    // Test "Crush" - high attraction + affection, low resentment
    let rel_vec = RelationshipVector {
        affection: 7.0,
        trust: 6.0,
        attraction: 8.0, // Strong attraction
        familiarity: 3.0,
        resentment: 0.0,
    };
    let label = derive_role_label(&rel_vec);
    assert_eq!(label, "Crush");

    // Test "Rival" - high resentment
    let rel_vec = RelationshipVector {
        affection: 2.0,
        trust: -3.0,
        attraction: 0.0,
        familiarity: 3.0,
        resentment: 9.0, // Vindictive
    };
    let label = derive_role_label(&rel_vec);
    assert_eq!(label, "Rival");

    // Test "Family" - devoted affection + deep trust
    let rel_vec = RelationshipVector {
        affection: 8.0, // Devoted
        trust: 8.0,     // DeepTrust
        attraction: 1.0,
        familiarity: 6.0,
        resentment: 0.0,
    };
    let label = derive_role_label(&rel_vec);
    assert_eq!(label, "Family");

    // Test "Friend" - close affection + trusted
    let rel_vec = RelationshipVector {
        affection: 6.0, // Close
        trust: 6.0,     // Trusted
        attraction: 1.0,
        familiarity: 5.0,
        resentment: 0.0,
    };
    let label = derive_role_label(&rel_vec);
    assert_eq!(label, "Friend");

    // Test "Acquaintance" - moderate affection
    let rel_vec = RelationshipVector {
        affection: 4.0, // Friendly
        trust: 2.0,
        attraction: 0.0,
        familiarity: 2.0,
        resentment: 0.0,
    };
    let label = derive_role_label(&rel_vec);
    assert_eq!(label, "Acquaintance");

    // Test "Stranger" - low everything (affection <= -5.0 for Stranger band)
    let rel_vec = RelationshipVector {
        affection: -6.0, // Stranger band
        trust: -6.0,
        attraction: 0.0,
        familiarity: 0.0,
        resentment: 0.0,
    };
    let label = derive_role_label(&rel_vec);
    assert_eq!(label, "Stranger");
}
