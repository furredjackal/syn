//! Director persistence integration tests.
//!
//! These tests verify that director state can be saved and restored,
//! and that restored directors produce identical behavior (determinism).

use syn_core::{NpcId, WorldSeed, WorldState, SimTick};
use syn_director::{
    CompiledEventDirector, DirectorConfig,
    serialize_snapshot, deserialize_snapshot,
};
use syn_director::pressure::{
    Pressure, PressureId, PressureKind,
    Milestone, MilestoneId, MilestoneKind,
};
use syn_director::queue::{QueuedEvent, QueueSource};
use syn_storylets::library::{StoryletKey, StoryletLibrary};

/// Create an empty test storylet library.
fn create_test_library() -> StoryletLibrary {
    StoryletLibrary::new()
}

#[test]
fn test_director_snapshot_and_restore() {
    // Create initial director with complex state
    let library = create_test_library();
    let config = DirectorConfig::for_testing();
    let mut director = CompiledEventDirector::new(library.clone(), config.clone());
    
    // Set up non-trivial state
    director.state_mut().tick = SimTick::new(50);
    director.state_mut().narrative_heat = 35.0;
    
    // Add a queued event
    director.state_mut().pending_queue.push_unchecked(QueuedEvent::new(
        StoryletKey(2),
        SimTick::new(60),
        5,
        false,
        QueueSource::FollowUp,
    ));
    
    // Add a pressure
    let pressure = Pressure::new(
        PressureId(1),
        PressureKind::Financial,
        SimTick::new(30),
        "Rent due".into(),
    )
    .with_deadline(SimTick::new(100))
    .with_severity(0.5);
    director.state_mut().active_pressures.add_pressure(pressure);
    
    // Add a milestone
    let milestone = Milestone::new(
        MilestoneId(1),
        MilestoneKind::RomanceArc,
        SimTick::new(10),
        "Find love".into(),
    )
    .with_progress(0.3);
    director.state_mut().milestones.add_milestone(milestone);
    
    // Take a snapshot
    let snapshot = director.snapshot();
    
    // Serialize and deserialize
    let bytes = serialize_snapshot(&snapshot).expect("Serialization should succeed");
    let restored_snapshot = deserialize_snapshot(&bytes).expect("Deserialization should succeed");
    
    // Restore a new director
    let restored = CompiledEventDirector::restore_from_snapshot(
        library,
        config,
        restored_snapshot,
    );
    
    // Verify state matches
    assert_eq!(restored.state().tick, director.state().tick);
    assert_eq!(restored.state().narrative_heat, director.state().narrative_heat);
    assert_eq!(restored.state().pending_queue.len(), director.state().pending_queue.len());
    assert_eq!(
        restored.state().active_pressures.active_count(),
        director.state().active_pressures.active_count()
    );
    assert_eq!(
        restored.state().milestones.active_count(),
        director.state().milestones.active_count()
    );
}

#[test]
fn test_deterministic_state_after_restore() {
    // Create two identical worlds
    let _world1 = WorldState::new(WorldSeed(12345), NpcId(1));
    let _world2 = WorldState::new(WorldSeed(12345), NpcId(1));
    
    // Create director and set up state
    let library = create_test_library();
    let config = DirectorConfig::for_testing();
    let mut director = CompiledEventDirector::new(library.clone(), config.clone());
    
    // Advance state
    director.state_mut().tick = SimTick::new(100);
    director.state_mut().narrative_heat = 25.0;
    
    // Take snapshot
    let snapshot = director.snapshot();
    let bytes = serialize_snapshot(&snapshot).expect("Serialization should succeed");
    
    // Restore from snapshot
    let restored_snapshot = deserialize_snapshot(&bytes).expect("Deserialization should succeed");
    let restored = CompiledEventDirector::restore_from_snapshot(
        library,
        config,
        restored_snapshot,
    );
    
    // Both should have identical state
    assert_eq!(restored.state().tick, director.state().tick);
    assert_eq!(restored.state().narrative_heat, director.state().narrative_heat);
    assert_eq!(restored.state().narrative_phase, director.state().narrative_phase);
    assert_eq!(restored.state().pending_queue.len(), director.state().pending_queue.len());
}

#[test]
fn test_snapshot_format_version() {
    let library = create_test_library();
    let director = CompiledEventDirector::with_defaults(library);
    
    // Snapshot without config version
    let snapshot1 = director.snapshot();
    assert_eq!(snapshot1.format_version, syn_director::CURRENT_FORMAT_VERSION);
    assert_eq!(snapshot1.config_version, None);
    
    // Snapshot with config version
    let snapshot2 = director.snapshot_with_config_version(42);
    assert_eq!(snapshot2.format_version, syn_director::CURRENT_FORMAT_VERSION);
    assert_eq!(snapshot2.config_version, Some(42));
    
    // Both should roundtrip correctly
    let bytes1 = serialize_snapshot(&snapshot1).unwrap();
    let bytes2 = serialize_snapshot(&snapshot2).unwrap();
    
    let restored1 = deserialize_snapshot(&bytes1).unwrap();
    let restored2 = deserialize_snapshot(&bytes2).unwrap();
    
    assert_eq!(restored1.config_version, None);
    assert_eq!(restored2.config_version, Some(42));
}

#[test]
fn test_empty_state_persistence() {
    let library = create_test_library();
    let director = CompiledEventDirector::with_defaults(library.clone());
    
    // Fresh director should have default state
    assert_eq!(director.state().tick.0, 0);
    assert_eq!(director.state().narrative_heat, 0.0);
    assert!(director.state().pending_queue.is_empty());
    
    // Snapshot and restore
    let snapshot = director.snapshot();
    let bytes = serialize_snapshot(&snapshot).unwrap();
    let restored_snapshot = deserialize_snapshot(&bytes).unwrap();
    let restored = CompiledEventDirector::restore_from_snapshot(
        library,
        DirectorConfig::default(),
        restored_snapshot,
    );
    
    // Should still have default state
    assert_eq!(restored.state().tick.0, 0);
    assert_eq!(restored.state().narrative_heat, 0.0);
    assert!(restored.state().pending_queue.is_empty());
}

#[test]
fn test_pressure_persistence() {
    let library = create_test_library();
    let config = DirectorConfig::for_testing();
    let mut director = CompiledEventDirector::new(library.clone(), config.clone());
    
    // Add multiple pressures with various states
    let pressure1 = Pressure::new(
        PressureId(1),
        PressureKind::Financial,
        SimTick::new(10),
        "Rent".into(),
    )
    .with_deadline(SimTick::new(100))
    .with_severity(0.8);
    
    let pressure2 = Pressure::new(
        PressureId(2),
        PressureKind::Health,
        SimTick::new(20),
        "Doctor visit".into(),
    )
    .with_severity(0.5);
    
    director.state_mut().active_pressures.add_pressure(pressure1);
    director.state_mut().active_pressures.add_pressure(pressure2);
    
    // Snapshot and restore
    let snapshot = director.snapshot();
    let bytes = serialize_snapshot(&snapshot).unwrap();
    let restored_snapshot = deserialize_snapshot(&bytes).unwrap();
    let restored = CompiledEventDirector::restore_from_snapshot(
        library,
        config,
        restored_snapshot,
    );
    
    // Verify pressures persisted
    assert_eq!(restored.state().active_pressures.active_count(), 2);
    
    let p1 = restored.state().active_pressures.get(PressureId(1));
    assert!(p1.is_some());
    let p1 = p1.unwrap();
    assert_eq!(p1.kind, PressureKind::Financial);
    assert_eq!(p1.severity, 0.8);
    assert_eq!(p1.deadline, Some(SimTick::new(100)));
}

#[test]
fn test_milestone_persistence() {
    let library = create_test_library();
    let config = DirectorConfig::for_testing();
    let mut director = CompiledEventDirector::new(library.clone(), config.clone());
    
    // Add milestones
    let milestone1 = Milestone::new(
        MilestoneId(1),
        MilestoneKind::CareerArc,
        SimTick::new(5),
        "Get promoted".into(),
    )
    .with_progress(0.7);
    
    let milestone2 = Milestone::new(
        MilestoneId(2),
        MilestoneKind::RomanceArc,
        SimTick::new(15),
        "Find love".into(),
    )
    .with_progress(0.2);
    
    director.state_mut().milestones.add_milestone(milestone1);
    director.state_mut().milestones.add_milestone(milestone2);
    
    // Snapshot and restore
    let snapshot = director.snapshot();
    let bytes = serialize_snapshot(&snapshot).unwrap();
    let restored_snapshot = deserialize_snapshot(&bytes).unwrap();
    let restored = CompiledEventDirector::restore_from_snapshot(
        library,
        config,
        restored_snapshot,
    );
    
    // Verify milestones persisted
    assert_eq!(restored.state().milestones.active_count(), 2);
    
    let m1 = restored.state().milestones.get(MilestoneId(1));
    assert!(m1.is_some());
    let m1 = m1.unwrap();
    assert_eq!(m1.kind, MilestoneKind::CareerArc);
    assert!((m1.progress - 0.7).abs() < 0.001);
}

#[test]
fn test_queue_persistence() {
    let library = create_test_library();
    let config = DirectorConfig::for_testing();
    let mut director = CompiledEventDirector::new(library.clone(), config.clone());
    
    // Add queued events
    director.state_mut().pending_queue.push_unchecked(QueuedEvent::new(
        StoryletKey(1),
        SimTick::new(50),
        3,
        false,
        QueueSource::FollowUp,
    ));
    
    director.state_mut().pending_queue.push_unchecked(QueuedEvent::new(
        StoryletKey(2),
        SimTick::new(60),
        5,
        true,  // mandatory
        QueueSource::PressureRelief,
    ));
    
    // Snapshot and restore
    let snapshot = director.snapshot();
    let bytes = serialize_snapshot(&snapshot).unwrap();
    let restored_snapshot = deserialize_snapshot(&bytes).unwrap();
    let restored = CompiledEventDirector::restore_from_snapshot(
        library,
        config,
        restored_snapshot,
    );
    
    // Verify queue persisted
    assert_eq!(restored.state().pending_queue.len(), 2);
}

