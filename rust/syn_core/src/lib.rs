//! syn_core: Foundation types, RNG, and persistence for SYN simulation engine.
//!
//! This crate provides:
//! - Seeded RNG for deterministic simulation
//! - Core types (Stats, Traits, Relationships, NPCs, World)
//! - SQLite persistence layer
//! - Utility types for serialization and querying
//! - Character generation from seeds
//! - District system with crime/economy simulation
//! - Gossip/social spread mechanics
//! - Population simulation with job markets and demographics
//! - Failure/recovery systems with trauma spirals
//! - High-performance collection types (FxHashMap, SmallVec)
//! - Bitflag-based world flags for O(1) flag checks
//! - String interning for identifiers (memory reduction + O(1) comparisons)
//! - Optional mimalloc global allocator (enable `mimalloc-allocator` feature)

// Bleeding-edge stable: deny unsafe, warn on common issues
#![deny(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::all)]

// Global allocator module (activated via feature flag)
#[cfg(feature = "mimalloc-allocator")]
pub mod allocator;

pub mod character_gen;
pub mod collections;
pub mod digital_legacy;
pub mod district;
pub mod errors;
pub mod failure_recovery;
pub mod gossip;
pub mod gossip_pressure;
pub mod intern;
pub mod life_stage;
pub mod narrative_heat;
pub mod npc;
pub mod npc_actions;
pub mod npc_behavior;
pub mod district_pressure;
pub mod persistence;
pub mod population;
pub mod relationship_milestones;
pub mod relationship_model;
pub mod relationship_pressure;
pub mod relationships;
pub mod rng;
pub mod skills;
pub mod snapshot;
pub mod stats;
pub mod time;
pub mod types;
pub mod world_flags;

pub use character_gen::*;
pub use collections::*;
pub use district::*;
pub use errors::*;
pub use failure_recovery::*;
pub use gossip::*;
pub use intern::*;
pub use persistence::*;
pub use population::*;
pub use relationships::*;
pub use rng::*;
pub use skills::*;
pub use stats::*;
pub use types::*;
pub use world_flags::*;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Helper to build a serializable snapshot of a world (for tests/integration checks).
pub fn world_snapshot(world: &WorldState) -> WorldStateSnapshot {
    WorldStateSnapshot::from_world(world)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::Persistence;
    use crate::relationship_milestones::{RelationshipMilestoneEvent, RelationshipMilestoneKind};
    use crate::relationship_pressure::{RelationshipEventKind, RelationshipPressureEvent};
    use crate::time::TickContext;
    use crate::types::{MemoryEntryRecord, WorldStateSnapshot};
    use std::collections::VecDeque;
    use std::time::SystemTime;

    #[test]
    fn world_persistence_roundtrip_basic() {
        // Unique temp path
        let tmp_path = std::env::temp_dir().join(format!(
            "syn_core_world_rt_{}.db",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_file(&tmp_path);

        let mut db = Persistence::new(tmp_path.to_string_lossy().as_ref())
            .expect("Failed to init persistence");

        let mut world = WorldState::new(WorldSeed(123), NpcId(1));
        let mut ctx = TickContext::default();
        // Advance 10 in-game days.
        for _ in 0..(24 * 10) {
            world.tick(&mut ctx);
        }

        // Populate relationship-driven systems and storylet usage.
        world.storylet_usage.times_fired.insert("story_a".into(), 1);
        world.relationship_pressure.changed_pairs.push((1, 2));
        world.relationship_pressure.queue = VecDeque::from([RelationshipPressureEvent {
            actor_id: 1,
            target_id: 2,
            kind: RelationshipEventKind::AffectionBandChanged,
            old_band: "Stranger".into(),
            new_band: "Acquaintance".into(),
            source: Some("rt_test".into()),
            tick: Some(world.current_tick.0),
        }]);
        world.relationship_milestones.queue = VecDeque::from([RelationshipMilestoneEvent {
            actor_id: 1,
            target_id: 2,
            kind: RelationshipMilestoneKind::FriendToRival,
            from_role: "Friend".into(),
            to_role: "Rival".into(),
            reason: "rt_test".into(),
            source: Some("rt_test".into()),
            tick: Some(world.current_tick.0),
        }]);
        world.memory_entries.push(MemoryEntryRecord {
            id: "mem_rt".into(),
            event_id: "evt_rt".into(),
            npc_id: NpcId(1),
            sim_tick: world.current_tick,
            emotional_intensity: 0.5,
            stat_deltas: Vec::new(),
            relationship_deltas: Vec::new(),
            tags: vec!["rt".into()],
            participants: vec![1, 2],
        });
        world.district_state.insert("Downtown".into(), "ok".into());
        world.world_flags.set_any("rt_flag");
        world.known_npcs.push(NpcId(2));

        let before = WorldStateSnapshot::from_world(&world);

        db.save_world(&world).expect("Failed to save world");
        let loaded = db
            .load_world(WorldSeed(123))
            .expect("Failed to load world");

        // Ensure tick consistency after load
        assert_eq!(loaded.current_tick.0, loaded.game_time.tick_index);

        let after = WorldStateSnapshot::from_world(&loaded);
        assert_eq!(
            before, after,
            "WorldState did not round-trip correctly through persistence"
        );

        let _ = std::fs::remove_file(&tmp_path);
    }

    #[test]
    fn narrative_pressure_and_milestones_persist() {
        use crate::relationship_model::RelationshipRole;
        use crate::relationship_pressure::RelationshipEventKind;
        use tempfile::NamedTempFile;

        let tmp = NamedTempFile::new().expect("failed to create temp db");
        let mut db =
            Persistence::new(tmp.path().to_string_lossy().as_ref()).expect("Failed to init DB");

        let mut world = WorldState::new(WorldSeed(321), NpcId(1));
        let mut ctx = TickContext::default();
        for _ in 0..24 {
            world.tick(&mut ctx);
        }

        // Seed relationship pressure queue and milestones
        world.relationship_pressure.queue.push_back(RelationshipPressureEvent {
            actor_id: 1,
            target_id: 2,
            kind: RelationshipEventKind::TrustBandChanged,
            old_band: "Unknown".into(),
            new_band: "Trusted".into(),
            source: Some("persist_test".into()),
            tick: Some(world.current_tick.0),
        });
        world
            .relationship_milestones
            .last_role
            .insert((1, 2), RelationshipRole::Friend);
        world.relationship_milestones.queue.push_back(RelationshipMilestoneEvent {
            actor_id: 1,
            target_id: 2,
            kind: RelationshipMilestoneKind::RivalToAlly,
            from_role: "Rival".into(),
            to_role: "Ally".into(),
            reason: "persist_test".into(),
            source: Some("persist_test".into()),
            tick: Some(world.current_tick.0),
        });

        // Memory entry
        world.memory_entries.push(MemoryEntryRecord {
            id: "persist_mem".into(),
            event_id: "evt_persist".into(),
            npc_id: NpcId(1),
            sim_tick: world.current_tick,
            emotional_intensity: -0.2,
            stat_deltas: Vec::new(),
            relationship_deltas: Vec::new(),
            tags: vec!["persist".into()],
            participants: vec![1, 2],
        });

        let before = WorldStateSnapshot::from_world(&world);

        db.save_world(&world).expect("save should succeed");
        let loaded = db
            .load_world(WorldSeed(321))
            .expect("load should succeed");
        let after = WorldStateSnapshot::from_world(&loaded);

        assert_eq!(
            before.relationship_pressure.queue,
            after.relationship_pressure.queue,
            "Relationship pressure queue did not persist correctly"
        );
        assert_eq!(
            before.relationship_milestones.queue,
            after.relationship_milestones.queue,
            "Relationship milestones queue did not persist correctly"
        );
        assert_eq!(
            before.memory_entries,
            after.memory_entries,
            "Memory entries did not persist correctly"
        );

        assert_eq!(before, after, "WorldState snapshot mismatch after persistence");
    }

    #[test]
    fn gametime_and_current_tick_stay_in_sync_across_persistence() {
        use tempfile::NamedTempFile;

        let mut world = WorldState::new(WorldSeed(555), NpcId(1));
        let mut ctx = TickContext::default();

        // Advance 17 in-game days.
        for _ in 0..(24 * 17) {
            world.tick(&mut ctx);
        }

        let snapshot_before = WorldStateSnapshot::from_world(&world);
        let tick_before = snapshot_before.current_tick.0;
        let gametime_tick_before = snapshot_before.game_time_tick;

        assert_eq!(tick_before, gametime_tick_before, "Pre-save tick mismatch");

        let tmp = NamedTempFile::new().expect("failed to create temp db");
        let mut db =
            Persistence::new(tmp.path().to_string_lossy().as_ref()).expect("Failed to init DB");

        db.save_world(&world).expect("Failed to save world");
        let loaded = db
            .load_world(WorldSeed(555))
            .expect("Failed to load world");
        let snapshot_after = WorldStateSnapshot::from_world(&loaded);

        let tick_after = snapshot_after.current_tick.0;
        let gametime_tick_after = snapshot_after.game_time_tick;

        assert_eq!(
            tick_before, tick_after,
            "Tick index changed across persistence"
        );
        assert_eq!(
            gametime_tick_before, gametime_tick_after,
            "GameTime tick index changed across persistence"
        );
        assert_eq!(
            tick_after, gametime_tick_after,
            "Post-load tick mismatch between current_tick and GameTime"
        );
    }
}
