use syn_core::{NpcId, SimTick};
use syn_memory::{memories_for_pair_with_tags, MemoryEntry};

#[test]
fn filters_memories_by_participants_and_tags() {
    let mut entries = Vec::new();

    let mut m1 = MemoryEntry::new("mem1".into(), "event1".into(), NpcId(1), SimTick(1), -0.5);
    m1.tags = vec!["betrayal".into()];
    m1.participants = vec![1, 2];
    entries.push(m1);

    let mut m2 = MemoryEntry::new("mem2".into(), "event2".into(), NpcId(2), SimTick(2), 0.3);
    m2.tags = vec!["joy".into()];
    m2.participants = vec![2, 3];
    entries.push(m2);

    let results = memories_for_pair_with_tags(&entries, 1, 2, &["betrayal"]);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "mem1");

    let none = memories_for_pair_with_tags(&entries, 1, 3, &["betrayal"]);
    assert!(none.is_empty());
}
