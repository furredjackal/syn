use syn_core::{
    apply_relationship_deltas, RelationshipAxis, RelationshipDelta, RelationshipStore,
    RelationshipVector, NpcId,
};

#[test]
fn relationship_vector_get_set_delta_clamps() {
    let mut vec = RelationshipVector::default();
    vec.set(RelationshipAxis::Affection, 12.0);
    assert_eq!(vec.get(RelationshipAxis::Affection), 10.0);

    vec.apply_delta(RelationshipAxis::Affection, -25.0);
    assert_eq!(vec.get(RelationshipAxis::Affection), -10.0);

    vec.set(RelationshipAxis::Trust, 5.0);
    assert_eq!(vec.get(RelationshipAxis::Trust), 5.0);
}

struct MapStore(std::collections::HashMap<NpcId, RelationshipVector>);

impl RelationshipStore for MapStore {
    fn apply_delta(&mut self, delta: &RelationshipDelta) {
        let entry = self
            .0
            .entry(delta.target_id)
            .or_insert_with(RelationshipVector::default);
        entry.apply_delta(delta.axis, delta.delta);
    }
}

#[test]
fn apply_relationship_deltas_applies_all() {
    let mut store = MapStore(std::collections::HashMap::new());
    let deltas = vec![
        RelationshipDelta {
            target_id: NpcId(1),
            axis: RelationshipAxis::Affection,
            delta: 3.0,
            source: None,
        },
        RelationshipDelta {
            target_id: NpcId(1),
            axis: RelationshipAxis::Trust,
            delta: -2.0,
            source: Some("test".into()),
        },
    ];

    apply_relationship_deltas(&mut store, &deltas);

    let vec = store.0.get(&NpcId(1)).unwrap();
    assert_eq!(vec.get(RelationshipAxis::Affection), 3.0);
    assert_eq!(vec.get(RelationshipAxis::Trust), -2.0);
}
