use syn_core::relationships::{RelationshipAxis, RelationshipDelta};
use syn_memory::MemoryEntry;

#[test]
fn memory_entry_records_relationship_deltas() {
    let entry = MemoryEntry::new(
        "mem_test".into(),
        "event_test".into(),
        syn_core::NpcId(1),
        syn_core::SimTick(0),
        0.0,
    )
    .with_relationship_deltas(vec![RelationshipDelta {
        target_id: syn_core::NpcId(7),
        axis: RelationshipAxis::Trust,
        delta: -3.0,
        source: Some("test".into()),
    }]);

    assert_eq!(entry.relationship_deltas.len(), 1);
    assert_eq!(entry.relationship_deltas[0].target_id.0, 7);
    assert_eq!(entry.relationship_deltas[0].axis, RelationshipAxis::Trust);
}
