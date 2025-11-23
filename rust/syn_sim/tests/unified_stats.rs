use syn_core::{AbstractNpc, AttachmentStyle, NpcId, StatKind, Traits};
use syn_sim::SimulatedNpc;

fn make_test_npc() -> SimulatedNpc {
    let abstract_npc = AbstractNpc {
        id: NpcId(1),
        age: 25,
        job: "Tester".to_string(),
        district: "Nowhere".to_string(),
        household_id: 1,
        traits: Traits::default(),
        seed: 1,
        attachment_style: AttachmentStyle::Secure,
    };
    SimulatedNpc::new(abstract_npc)
}

#[test]
fn npc_tick_updates_use_unified_stats_api() {
    let mut npc = make_test_npc();
    npc.stats.set(StatKind::Mood, 0.0);

    // Apply a decay through the unified API
    let decay = 12.5; // intentionally large to test clamping
    npc.stats.apply_delta(StatKind::Mood, -decay);

    let result = npc.stats.get(StatKind::Mood);
    assert!(result <= 10.0 && result >= -10.0);
}
