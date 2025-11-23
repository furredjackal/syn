use syn_memory::{tag_counts_for_actor, MemoryEntry, NpcId, SimTick};

#[test]
fn counts_tags_for_participating_actor() {
    let mut mem_support = MemoryEntry::new(
        "mem_support".to_string(),
        "evt1".to_string(),
        NpcId(1),
        SimTick(10),
        0.5,
    )
    .with_tags(vec!["Support", "support"]);
    mem_support.participants = vec![1, 2];

    let mut mem_betrayal = MemoryEntry::new(
        "mem_betrayal".to_string(),
        "evt2".to_string(),
        NpcId(2),
        SimTick(20),
        -0.8,
    )
    .with_tags(vec!["betrayal"]);
    // Actor 1 not present, should be ignored.
    mem_betrayal.participants = vec![2];

    let mut mem_ambition = MemoryEntry::new(
        "mem_ambition".to_string(),
        "evt3".to_string(),
        NpcId(1),
        SimTick(30),
        0.3,
    )
    .with_tags(vec!["Ambition"]);
    mem_ambition.participants = vec![1];

    let memories = vec![mem_support, mem_betrayal, mem_ambition];
    let counts = tag_counts_for_actor(&memories, 1);

    assert_eq!(counts.get("support"), Some(&2));
    assert_eq!(counts.get("ambition"), Some(&1));
    assert!(!counts.contains_key("betrayal"));
}
