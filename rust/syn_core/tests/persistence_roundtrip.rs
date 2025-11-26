use syn_core::relationship_milestones::{RelationshipMilestoneEvent, RelationshipMilestoneKind};
use syn_core::relationship_pressure::RelationshipPressureEvent;
use syn_core::{
    time::TickContext, world_snapshot, MemoryEntryRecord, NpcId, Persistence, Relationship,
    RelationshipState, WorldSeed, WorldState, WorldStateSnapshot,
};

fn init_world_with_basic_state() -> WorldState {
    let mut world = WorldState::new(WorldSeed(777), NpcId(1));
    let mut ctx = TickContext::default();
    for _ in 0..24 {
        world.tick(&mut ctx);
    }

    // Minimal relationship to ensure HashMap fields populated
    let mut rel = Relationship::default();
    rel.state = RelationshipState::Friend;
    world.set_relationship(NpcId(1), NpcId(2), rel);

    // Relationship pressure queue
    world.relationship_pressure.queue.push_back(RelationshipPressureEvent {
        actor_id: 1,
        target_id: 2,
        kind: syn_core::relationship_pressure::RelationshipEventKind::AffectionBandChanged,
        old_band: "Stranger".into(),
        new_band: "Friend".into(),
        source: Some("integration".into()),
        tick: Some(world.current_tick.0),
    });

    // Milestone queue
    world.relationship_milestones.queue.push_back(RelationshipMilestoneEvent {
        actor_id: 1,
        target_id: 2,
        kind: RelationshipMilestoneKind::FriendToRival,
        from_role: "Friend".into(),
        to_role: "Rival".into(),
        reason: "integration".into(),
        source: Some("integration".into()),
        tick: Some(world.current_tick.0),
    });

    // Memory entry
    world.memory_entries.push(MemoryEntryRecord {
        id: "int_mem".into(),
        event_id: "evt_int".into(),
        npc_id: NpcId(1),
        sim_tick: world.current_tick,
        emotional_intensity: 0.1,
        stat_deltas: Vec::new(),
        relationship_deltas: Vec::new(),
        tags: vec!["integration".into()],
        participants: vec![1, 2],
    });

    world.district_state.insert("Downtown".into(), "ok".into());
    world.world_flags.insert("int_flag".into(), true);
    world.known_npcs.push(NpcId(2));

    world
}

#[test]
fn persistence_roundtrip_snapshot_matches() {
    let tmp = std::env::temp_dir().join("syn_core_int_persist.db");
    let _ = std::fs::remove_file(&tmp);
    let mut db = Persistence::new(tmp.to_string_lossy().as_ref()).expect("init db");

    let world = init_world_with_basic_state();
    let snapshot_before: WorldStateSnapshot = world_snapshot(&world);

    db.save_world(&world).expect("save world");
    let loaded = db
        .load_world(WorldSeed(777))
        .expect("load world back");
    let snapshot_after = world_snapshot(&loaded);

    assert_eq!(
        snapshot_before, snapshot_after,
        "WorldState did not round-trip correctly through persistence (integration test)"
    );

    let _ = std::fs::remove_file(&tmp);
}
