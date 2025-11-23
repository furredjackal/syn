use syn_memory::{MemorySystem, add_npc_behavior_memory_with_tags};
use syn_core::SimTick;

#[test]
fn add_behavior_memory_with_tags_pushes_entry() {
    let mut mem = MemorySystem::new();
    let npc_id = 10u64;
    let player_id = 1u64;
    let tags = vec!["npc_behavior".to_string(), "support".to_string()];
    let tick = SimTick::new(100);

    add_npc_behavior_memory_with_tags(&mut mem, npc_id, player_id, tags.clone(), tick);

    let journal = mem.get_journal(syn_core::NpcId(npc_id)).expect("journal exists");
    assert_eq!(journal.entries.len(), 1);
    let entry = &journal.entries[0];
    assert_eq!(entry.tags, tags);
    assert_eq!(entry.participants, vec![npc_id, player_id]);
}
