use syn_core::{NpcId, SimTick, StatDelta, StatKind};
use syn_memory::MemoryEntry;

#[test]
fn memory_entry_stores_stat_deltas() {
    let entry = MemoryEntry::new(
        "mem_1".to_string(),
        "event_test".to_string(),
        NpcId(1),
        SimTick(0),
        0.5,
    )
    .with_stat_deltas(vec![
        StatDelta {
            kind: StatKind::Mood,
            delta: -3.0,
            source: Some("unit".into()),
        },
        StatDelta {
            kind: StatKind::Reputation,
            delta: 5.0,
            source: None,
        },
    ]);

    assert_eq!(entry.stat_deltas.len(), 2);
    assert_eq!(entry.stat_deltas[0].kind, StatKind::Mood);
    assert_eq!(entry.stat_deltas[1].kind, StatKind::Reputation);
}
