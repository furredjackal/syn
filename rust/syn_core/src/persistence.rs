//! SQLite persistence layer for SYN world state.
//!
//! Handles schema initialization, save/load, and world state serialization.

use crate::relationship_milestones::RelationshipMilestoneEvent;
use crate::relationship_pressure::RelationshipPressureEvent;
use crate::types::*;
use rusqlite::{params, Connection, Result as SqlResult};
use serde_json;
use std::collections::{HashMap, VecDeque};

/// Persistence layer for SYN world state.
pub struct Persistence {
    conn: Connection,
}

impl Persistence {
    /// Open or create a new database at the given path.
    pub fn new(db_path: &str) -> SqlResult<Self> {
        let conn = Connection::open(db_path)?;
        let mut db = Persistence { conn };
        db.init_schema()?;
        Ok(db)
    }

    /// Initialize database schema.
    fn init_schema(&mut self) -> SqlResult<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS world_state (
                id INTEGER PRIMARY KEY,
                seed INTEGER NOT NULL UNIQUE,
                player_id INTEGER NOT NULL,
                current_tick INTEGER NOT NULL,
                player_stats TEXT NOT NULL,
                player_age INTEGER NOT NULL,
                player_life_stage TEXT NOT NULL,
                player_karma REAL NOT NULL,
                narrative_heat REAL NOT NULL DEFAULT 0.0,
                heat_momentum REAL NOT NULL DEFAULT 0.0,
                game_time_tick INTEGER NOT NULL DEFAULT 0,
                known_npcs TEXT NOT NULL DEFAULT '[]',
                npc_prototypes TEXT NOT NULL DEFAULT '{}',
                relationship_pressure TEXT NOT NULL DEFAULT '{}',
                relationship_milestones TEXT NOT NULL DEFAULT '{}',
                digital_legacy TEXT NOT NULL DEFAULT '{}',
                storylet_usage TEXT NOT NULL DEFAULT '{}',
                memory_entries TEXT NOT NULL DEFAULT '[]',
                district_state TEXT NOT NULL DEFAULT '{}',
                world_flags TEXT NOT NULL DEFAULT '{}',
                relationship_pressure_queue TEXT NOT NULL DEFAULT '[]',
                relationship_milestone_queue TEXT NOT NULL DEFAULT '[]',
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS relationships (
                id INTEGER PRIMARY KEY,
                world_seed INTEGER NOT NULL,
                from_npc_id INTEGER NOT NULL,
                to_npc_id INTEGER NOT NULL,
                relationship_data TEXT NOT NULL,
                FOREIGN KEY(world_seed) REFERENCES world_state(seed),
                UNIQUE(world_seed, from_npc_id, to_npc_id)
            );

            CREATE TABLE IF NOT EXISTS npcs (
                id INTEGER PRIMARY KEY,
                world_seed INTEGER NOT NULL,
                npc_id INTEGER NOT NULL,
                npc_data TEXT NOT NULL,
                FOREIGN KEY(world_seed) REFERENCES world_state(seed),
                UNIQUE(world_seed, npc_id)
            );

            CREATE TABLE IF NOT EXISTS memory_entries (
                id INTEGER PRIMARY KEY,
                world_seed INTEGER NOT NULL,
                npc_id INTEGER NOT NULL,
                event_id TEXT NOT NULL,
                impact_vector TEXT NOT NULL,
                sim_tick INTEGER NOT NULL,
                FOREIGN KEY(world_seed) REFERENCES world_state(seed)
            );

            CREATE TABLE IF NOT EXISTS storylets (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                json_data TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE INDEX IF NOT EXISTS idx_relationships ON relationships(world_seed, from_npc_id);
            CREATE INDEX IF NOT EXISTS idx_npcs ON npcs(world_seed, npc_id);
            CREATE INDEX IF NOT EXISTS idx_memories ON memory_entries(world_seed, npc_id);
            ",
        )?;
        // Backfill columns if schema existed before heat support. SQLite errors are ignored if columns already exist.
        let _ = self.conn.execute(
            "ALTER TABLE world_state ADD COLUMN narrative_heat REAL NOT NULL DEFAULT 0.0",
            params![],
        );
        let _ = self.conn.execute(
            "ALTER TABLE world_state ADD COLUMN heat_momentum REAL NOT NULL DEFAULT 0.0",
            params![],
        );
        let _ = self.conn.execute(
            "ALTER TABLE world_state ADD COLUMN game_time_tick INTEGER NOT NULL DEFAULT 0",
            params![],
        );
        let _ = self.conn.execute(
            "ALTER TABLE world_state ADD COLUMN known_npcs TEXT NOT NULL DEFAULT '[]'",
            params![],
        );
        let _ = self.conn.execute(
            "ALTER TABLE world_state ADD COLUMN npc_prototypes TEXT NOT NULL DEFAULT '{}'",
            params![],
        );
        let _ = self.conn.execute(
            "ALTER TABLE world_state ADD COLUMN relationship_pressure TEXT NOT NULL DEFAULT '{}'",
            params![],
        );
        let _ = self.conn.execute(
            "ALTER TABLE world_state ADD COLUMN relationship_milestones TEXT NOT NULL DEFAULT '{}'",
            params![],
        );
        let _ = self.conn.execute(
            "ALTER TABLE world_state ADD COLUMN digital_legacy TEXT NOT NULL DEFAULT '{}'",
            params![],
        );
        let _ = self.conn.execute(
            "ALTER TABLE world_state ADD COLUMN storylet_usage TEXT NOT NULL DEFAULT '{}'",
            params![],
        );
        let _ = self.conn.execute(
            "ALTER TABLE world_state ADD COLUMN memory_entries TEXT NOT NULL DEFAULT '[]'",
            params![],
        );
        let _ = self.conn.execute(
            "ALTER TABLE world_state ADD COLUMN district_state TEXT NOT NULL DEFAULT '{}'",
            params![],
        );
        let _ = self.conn.execute(
            "ALTER TABLE world_state ADD COLUMN world_flags TEXT NOT NULL DEFAULT '{}'",
            params![],
        );
        let _ = self.conn.execute(
            "ALTER TABLE world_state ADD COLUMN relationship_pressure_queue TEXT NOT NULL DEFAULT '[]'",
            params![],
        );
        let _ = self.conn.execute(
            "ALTER TABLE world_state ADD COLUMN relationship_milestone_queue TEXT NOT NULL DEFAULT '[]'",
            params![],
        );
        Ok(())
    }

    /// Save world state to database.
    pub fn save_world(&mut self, world: &WorldState) -> SqlResult<()> {
        let player_stats_json = serde_json::to_string(&world.player_stats)
            .map_err(|_| rusqlite::Error::InvalidQuery)?;
        let player_karma_json = world.player_karma.0;

        let known_npcs_json = serde_json::to_string(&world.known_npcs)
            .map_err(|_| rusqlite::Error::InvalidQuery)?;
        let npc_prototypes_json = serde_json::to_string(&world.npc_prototypes)
            .map_err(|_| rusqlite::Error::InvalidQuery)?;
        let relationship_pressure_json = serde_json::to_string(&world.relationship_pressure)
            .map_err(|_| rusqlite::Error::InvalidQuery)?;
        let relationship_milestones_json = serde_json::to_string(&world.relationship_milestones)
            .map_err(|_| rusqlite::Error::InvalidQuery)?;
        let digital_legacy_json = serde_json::to_string(&world.digital_legacy)
            .map_err(|_| rusqlite::Error::InvalidQuery)?;
        let storylet_usage_json = serde_json::to_string(&world.storylet_usage)
            .map_err(|_| rusqlite::Error::InvalidQuery)?;
        let memory_entries_json = serde_json::to_string(&world.memory_entries)
            .map_err(|_| rusqlite::Error::InvalidQuery)?;
        let district_state_json = serde_json::to_string(&world.district_state)
            .map_err(|_| rusqlite::Error::InvalidQuery)?;
        let world_flags_json = serde_json::to_string(&world.world_flags)
            .map_err(|_| rusqlite::Error::InvalidQuery)?;
        let relationship_pressure_queue_json =
            serde_json::to_string(&world.relationship_pressure.queue)
                .map_err(|_| rusqlite::Error::InvalidQuery)?;
        let relationship_milestone_queue_json =
            serde_json::to_string(&world.relationship_milestones.queue)
                .map_err(|_| rusqlite::Error::InvalidQuery)?;

        self.conn.execute(
            "INSERT OR REPLACE INTO world_state 
             (seed, player_id, current_tick, player_stats, player_age, player_life_stage, player_karma, narrative_heat, heat_momentum, game_time_tick, known_npcs, npc_prototypes, relationship_pressure, relationship_milestones, digital_legacy, storylet_usage, memory_entries, district_state, world_flags, relationship_pressure_queue, relationship_milestone_queue, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)",
            params![
                world.seed.0,
                world.player_id.0,
                world.current_tick.0,
                player_stats_json,
                world.player_age,
                format!("{:?}", world.player_life_stage),
                player_karma_json,
                world.narrative_heat.value(),
                world.heat_momentum,
                world.game_time.tick_index as i64,
                known_npcs_json,
                npc_prototypes_json,
                relationship_pressure_json,
                relationship_milestones_json,
                digital_legacy_json,
                storylet_usage_json,
                memory_entries_json,
                district_state_json,
                world_flags_json,
                relationship_pressure_queue_json,
                relationship_milestone_queue_json,
            ],
        )?;

        // Save relationships
        for ((from_id, to_id), rel) in &world.relationships {
            let rel_json = serde_json::to_string(rel).map_err(|_| rusqlite::Error::InvalidQuery)?;
            self.conn.execute(
                "INSERT OR REPLACE INTO relationships (world_seed, from_npc_id, to_npc_id, relationship_data)
                 VALUES (?, ?, ?, ?)",
                params![world.seed.0, from_id.0, to_id.0, rel_json],
            )?;
        }

        // Save NPCs
        for (npc_id, npc) in &world.npcs {
            let npc_json = serde_json::to_string(npc).map_err(|_| rusqlite::Error::InvalidQuery)?;
            self.conn.execute(
                "INSERT OR REPLACE INTO npcs (world_seed, npc_id, npc_data) VALUES (?, ?, ?)",
                params![world.seed.0, npc_id.0, npc_json],
            )?;
        }

        Ok(())
    }

    /// Load world state from database.
    pub fn load_world(&mut self, seed: WorldSeed) -> SqlResult<WorldState> {
        let mut stmt = self.conn.prepare(
            "SELECT player_id, current_tick, player_stats, player_age, player_life_stage, player_karma, narrative_heat, heat_momentum, game_time_tick, known_npcs, npc_prototypes, relationship_pressure, relationship_milestones, digital_legacy, storylet_usage, memory_entries, district_state, world_flags, relationship_pressure_queue, relationship_milestone_queue
             FROM world_state WHERE seed = ?",
        )?;

        let world = stmt.query_row(params![seed.0], |row| {
            let player_id = NpcId(row.get::<_, u64>(0)?);
            let current_tick_raw: u64 = row.get(1)?;
            let stats_json: String = row.get(2)?;
            let player_age: u32 = row.get(3)?;
            let life_stage_str: String = row.get(4)?;
            let karma_value: f32 = row.get(5)?;
            let heat_value: f32 = row.get(6).unwrap_or(0.0);
            let heat_momentum: f32 = row.get(7).unwrap_or(0.0);
            let game_time_tick_raw: i64 = row.get(8).unwrap_or(0);
            let known_npcs_json: String = row.get(9).unwrap_or_else(|_| "[]".to_string());
            let npc_prototypes_json: String = row.get(10).unwrap_or_else(|_| "{}".to_string());
            let relationship_pressure_json: String =
                row.get(11).unwrap_or_else(|_| "{}".to_string());
            let relationship_milestones_json: String =
                row.get(12).unwrap_or_else(|_| "{}".to_string());
            let digital_legacy_json: String = row.get(13).unwrap_or_else(|_| "{}".to_string());
            let storylet_usage_json: String = row.get(14).unwrap_or_else(|_| "{}".to_string());
            let memory_entries_json: String = row.get(15).unwrap_or_else(|_| "[]".to_string());
            let district_state_json: String = row.get(16).unwrap_or_else(|_| "{}".to_string());
            let world_flags_json: String = row.get(17).unwrap_or_else(|_| "{}".to_string());
            let relationship_pressure_queue_json: String =
                row.get(18).unwrap_or_else(|_| "[]".to_string());
            let relationship_milestone_queue_json: String =
                row.get(19).unwrap_or_else(|_| "[]".to_string());

            let player_stats: Stats = serde_json::from_str(&stats_json).unwrap_or_default();
            let known_npcs: Vec<NpcId> =
                serde_json::from_str(&known_npcs_json).unwrap_or_default();
            let npc_prototypes: HashMap<NpcId, crate::NpcPrototype> =
                serde_json::from_str(&npc_prototypes_json).unwrap_or_default();
            let relationship_pressure: crate::relationship_pressure::RelationshipPressureState =
                serde_json::from_str(&relationship_pressure_json).unwrap_or_default();
            let relationship_milestones: crate::relationship_milestones::RelationshipMilestoneState =
                serde_json::from_str(&relationship_milestones_json).unwrap_or_default();
            let digital_legacy: crate::digital_legacy::DigitalLegacyState =
                serde_json::from_str(&digital_legacy_json).unwrap_or_default();
            let storylet_usage: crate::types::StoryletUsageState =
                serde_json::from_str(&storylet_usage_json).unwrap_or_default();
            let memory_entries: Vec<crate::types::MemoryEntryRecord> =
                serde_json::from_str(&memory_entries_json).unwrap_or_default();
            let district_state: HashMap<String, String> =
                serde_json::from_str(&district_state_json).unwrap_or_default();
            let world_flags: HashMap<String, bool> =
                serde_json::from_str(&world_flags_json).unwrap_or_default();
            let relationship_pressure_queue: VecDeque<RelationshipPressureEvent> =
                serde_json::from_str(&relationship_pressure_queue_json).unwrap_or_default();
            let relationship_milestone_queue: VecDeque<RelationshipMilestoneEvent> =
                serde_json::from_str(&relationship_milestone_queue_json).unwrap_or_default();
            let player_life_stage = match life_stage_str.as_str() {
                "Child" => LifeStage::Child,
                "Teen" => LifeStage::Teen,
                "YoungAdult" => LifeStage::YoungAdult,
                "Adult" => LifeStage::Adult,
                "Elder" => LifeStage::Elder,
                "Digital" => LifeStage::Digital,
                _ => LifeStage::Child,
            };

            let mut world = WorldState {
                seed,
                current_tick: SimTick(current_tick_raw),
                player_id,
                player_stats,
                player_age,
                player_age_years: player_age,
                player_days_since_birth: player_age.saturating_mul(365),
                player_life_stage,
                player_karma: Karma(karma_value),
                narrative_heat: crate::narrative_heat::NarrativeHeat::new(heat_value),
                heat_momentum,
                relationships: Default::default(),
                npcs: Default::default(),
                relationship_pressure,
                relationship_milestones,
                digital_legacy,
                npc_prototypes,
                known_npcs,
                game_time: crate::time::GameTime::from_tick(current_tick_raw),
                storylet_usage,
                memory_entries,
                district_state,
                world_flags,
            };

            // Normalize any legacy skew between stored current_tick and game_time_tick.
            let stored_game_time_tick = game_time_tick_raw.max(0) as u64;
            if stored_game_time_tick != current_tick_raw {
                #[cfg(debug_assertions)]
                {
                    eprintln!(
                        "Warning: current_tick ({}) != game_time_tick ({}); normalizing to current_tick",
                        current_tick_raw, stored_game_time_tick
                    );
                }
                world.game_time = crate::time::GameTime::from_tick(current_tick_raw);
            }

            Ok(world)
        })?;

        // Load relationships
        let mut rel_stmt = self.conn.prepare(
            "SELECT from_npc_id, to_npc_id, relationship_data FROM relationships WHERE world_seed = ?"
        )?;
        let relationships = rel_stmt.query_map(params![seed.0], |row| {
            let from_id = NpcId(row.get(0)?);
            let to_id = NpcId(row.get(1)?);
            let rel_json: String = row.get(2)?;
            let rel: Relationship = serde_json::from_str(&rel_json).unwrap_or_default();
            Ok(((from_id, to_id), rel))
        })?;

        let mut world = world;
        for rel in relationships {
            let (key, rel_data) = rel?;
            world.relationships.insert(key, rel_data);
        }

        // Restore queued relationship pressure and milestone events from dedicated columns (fallback to defaults already handled).
        world.relationship_pressure.queue = relationship_pressure_queue;
        world.relationship_milestones.queue = relationship_milestone_queue;

        // Load NPCs
        let mut npc_stmt = self
            .conn
            .prepare("SELECT npc_id, npc_data FROM npcs WHERE world_seed = ?")?;
        let npcs = npc_stmt.query_map(params![seed.0], |row| {
            let npc_id = NpcId(row.get(0)?);
            let npc_json: String = row.get(1)?;
            let npc: AbstractNpc =
                serde_json::from_str(&npc_json).unwrap_or_else(|_| AbstractNpc {
                    id: npc_id,
                    age: 0,
                    job: "Unknown".to_string(),
                    district: "Unknown".to_string(),
                    household_id: 0,
                    traits: Traits::default(),
                    seed: 0,
                    attachment_style: AttachmentStyle::Secure,
                });
            Ok((npc_id, npc))
        })?;

        for npc in npcs {
            let (npc_id, npc_data) = npc?;
            world.npcs.insert(npc_id, npc_data);
        }

        Ok(world)
    }

    /// Delete a world from database.
    pub fn delete_world(&mut self, seed: WorldSeed) -> SqlResult<()> {
        self.conn.execute(
            "DELETE FROM memory_entries WHERE world_seed = ?",
            params![seed.0],
        )?;
        self.conn
            .execute("DELETE FROM npcs WHERE world_seed = ?", params![seed.0])?;
        self.conn.execute(
            "DELETE FROM relationships WHERE world_seed = ?",
            params![seed.0],
        )?;
        self.conn
            .execute("DELETE FROM world_state WHERE seed = ?", params![seed.0])?;
        Ok(())
    }

    /// Check if a world exists.
    pub fn world_exists(&mut self, seed: WorldSeed) -> SqlResult<bool> {
        let mut stmt = self
            .conn
            .prepare("SELECT 1 FROM world_state WHERE seed = ?")?;
        let exists = stmt.exists(params![seed.0])?;
        Ok(exists)
    }

    /// Insert or update a storylet record stored as JSON.
    pub fn upsert_storylet_record(&mut self, record: &StoryletRecord) -> SqlResult<()> {
        self.conn.execute(
            "INSERT INTO storylets (id, name, json_data, created_at, updated_at)
             VALUES (?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
             ON CONFLICT(id) DO UPDATE SET
                 name = excluded.name,
                 json_data = excluded.json_data,
                 updated_at = CURRENT_TIMESTAMP",
            params![record.id, record.name, record.json_data],
        )?;
        Ok(())
    }

    /// Load every storylet JSON blob from SQLite.
    pub fn load_storylet_records(&mut self) -> SqlResult<Vec<StoryletRecord>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, json_data FROM storylets")?;
        let rows = stmt.query_map([], |row| {
            Ok(StoryletRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                json_data: row.get(2)?,
            })
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Remove all stored storylets (used by tooling before re-importing).
    pub fn clear_storylets(&mut self) -> SqlResult<()> {
        self.conn.execute("DELETE FROM storylets", [])?;
        Ok(())
    }
}

/// Serialized storylet entry stored in SQLite.
#[derive(Debug, Clone)]
pub struct StoryletRecord {
    pub id: String,
    pub name: String,
    pub json_data: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::digital_legacy::{DigitalImprint, LegacyVector};
    use crate::npc::{NpcPrototype, NpcSchedule, PersonalityVector};
    use crate::relationship_milestones::RelationshipMilestoneKind;
    use crate::relationship_pressure::RelationshipEventKind;
    use crate::time::TickContext;
    use std::fs;
    use serde_json::Value;

    #[test]
    fn test_persistence_save_load() {
        let db_path = "test_persistence.db";
        let _ = fs::remove_file(db_path);

        let mut db = Persistence::new(db_path).expect("Failed to create persistence");
        let mut world = WorldState::new(WorldSeed(42), NpcId(1));
        world.player_age = 25;
        world.player_age_years = 25;
        world.player_days_since_birth = 25 * 365;
        world.narrative_heat = crate::narrative_heat::NarrativeHeat::new(40.0);
        world.heat_momentum = 5.0;
        world.known_npcs.push(NpcId(99));
        world.storylet_usage.times_fired.insert("s1".into(), 3);
        world.relationship_pressure.changed_pairs.push((1, 2));
        world.relationship_pressure.queue.push_back(RelationshipPressureEvent {
            actor_id: 1,
            target_id: 2,
            kind: crate::relationship_pressure::RelationshipEventKind::AffectionBandChanged,
            old_band: "Stranger".into(),
            new_band: "Acquaintance".into(),
            source: Some("test".into()),
            tick: Some(1),
        });
        world
            .relationship_milestones
            .last_role
            .insert((1, 2), crate::relationship_model::RelationshipRole::Friend);
        world.relationship_milestones.queue.push_back(RelationshipMilestoneEvent {
            actor_id: 1,
            target_id: 2,
            kind: crate::relationship_milestones::RelationshipMilestoneKind::FriendToRival,
            from_role: "Friend".into(),
            to_role: "Rival".into(),
            reason: "test_reason".into(),
            source: Some("test_source".into()),
            tick: Some(2),
        });
        world.game_time.advance_ticks(12);
        world.memory_entries.push(crate::types::MemoryEntryRecord {
            id: "m1".into(),
            event_id: "evt".into(),
            npc_id: NpcId(1),
            sim_tick: SimTick(5),
            emotional_intensity: -0.5,
            stat_deltas: Vec::new(),
            relationship_deltas: Vec::new(),
            tags: vec!["test".into()],
            participants: vec![1, 2],
        });
        world.district_state.insert("Downtown".into(), "ok".into());
        world.world_flags.insert("met_childhood_friend".into(), true);
        let proto = NpcPrototype {
            id: NpcId(2),
            display_name: "Tester".to_string(),
            role_label: None,
            role_tags: Vec::new(),
            personality: PersonalityVector {
                warmth: 0.2,
                dominance: 0.1,
                volatility: 0.0,
                conscientiousness: 0.5,
                openness: 0.7,
            },
            base_stats: Stats::default(),
            active_stages: vec![LifeStage::YoungAdult],
            schedule: NpcSchedule::default(),
        };
        world.npc_prototypes.insert(proto.id, proto.clone());
        world.digital_legacy.primary_imprint = Some(DigitalImprint {
            id: 7,
            created_at_stage: LifeStage::Adult,
            created_at_age_years: 40,
            final_stats: Stats::default(),
            final_karma: Karma(5.0),
            legacy_vector: LegacyVector::default(),
            relationship_roles: HashMap::new(),
            relationship_milestones: Vec::new(),
            memory_tag_counts: HashMap::new(),
        });

        db.save_world(&world).expect("Failed to save world");
        let loaded = db.load_world(WorldSeed(42)).expect("Failed to load world");

        assert_eq!(loaded.player_age, 25);
        assert_eq!(loaded.seed, WorldSeed(42));
        assert_eq!(loaded.narrative_heat.value(), 40.0);
        assert_eq!(loaded.heat_momentum, 5.0);
        assert_eq!(loaded.known_npcs, world.known_npcs);
        assert_eq!(loaded.storylet_usage.times_fired.get("s1"), Some(&3));
        assert_eq!(
            loaded
                .relationship_milestones
                .last_role
                .get(&(1, 2))
                .copied(),
            Some(crate::relationship_model::RelationshipRole::Friend)
        );
        assert_eq!(loaded.relationship_pressure.queue.len(), 1);
        assert_eq!(loaded.relationship_milestones.queue.len(), 1);
        assert_eq!(loaded.game_time.tick_index, world.game_time.tick_index);
        assert_eq!(
            loaded.npc_prototypes.get(&NpcId(2)).map(|p| p.display_name.clone()),
            Some("Tester".to_string())
        );
        assert!(loaded.digital_legacy.primary_imprint.is_some());
        assert_eq!(loaded.memory_entries.len(), 1);
        assert_eq!(loaded.district_state.get("Downtown"), Some(&"ok".to_string()));
        assert_eq!(loaded.world_flags.get("met_childhood_friend"), Some(&true));

        let _ = fs::remove_file(db_path);
    }

    fn snapshot_json(world: &WorldState) -> Value {
        serde_json::to_value(world).expect("world should serialize to json value")
    }

    #[test]
    fn test_persistence_round_trip_snapshot() {
        let db_path = "test_persistence_round_trip.db";
        let _ = fs::remove_file(db_path);

        let mut db = Persistence::new(db_path).expect("Failed to create persistence");
        let mut world = WorldState::new(WorldSeed(99), NpcId(1));

        // Populate core fields
        world.player_age = 20;
        world.player_age_years = 20;
        world.player_days_since_birth = 20 * 365;
        world.player_stats.mood = 5.0;
        world.narrative_heat = crate::narrative_heat::NarrativeHeat::new(30.0);
        world.heat_momentum = 3.0;

        // Advance time to verify tick consistency
        let mut ctx = TickContext::default();
        for _ in 0..48 {
            world.tick(&mut ctx);
        }

        // Relationships & NPCs
        let mut rel = Relationship::default();
        rel.affection = 2.0;
        world.set_relationship(NpcId(1), NpcId(2), rel);
        let npc = AbstractNpc {
            id: NpcId(2),
            age: 22,
            job: "Engineer".into(),
            district: "Downtown".into(),
            household_id: 10,
            traits: Traits::default(),
            seed: 7,
            attachment_style: AttachmentStyle::Secure,
        };
        world.npcs.insert(npc.id, npc.clone());

        // Prototypes
        let proto = NpcPrototype {
            id: NpcId(2),
            display_name: "Tester Proto".into(),
            role_label: None,
            role_tags: Vec::new(),
            personality: PersonalityVector {
                warmth: 0.3,
                dominance: 0.2,
                volatility: 0.1,
                conscientiousness: 0.6,
                openness: 0.5,
            },
            base_stats: Stats::default(),
            active_stages: vec![LifeStage::YoungAdult],
            schedule: NpcSchedule::default(),
        };
        world.npc_prototypes.insert(proto.id, proto);

        // Relationship pressure & milestones (including queues)
        world.relationship_pressure.changed_pairs.push((1, 2));
        world.relationship_pressure.queue.push_back(RelationshipPressureEvent {
            actor_id: 1,
            target_id: 2,
            kind: RelationshipEventKind::AffectionBandChanged,
            old_band: "Stranger".into(),
            new_band: "Acquaintance".into(),
            source: Some("test_pressure".into()),
            tick: Some(5),
        });
        world
            .relationship_milestones
            .last_role
            .insert((1, 2), crate::relationship_model::RelationshipRole::Friend);
        world.relationship_milestones.queue.push_back(RelationshipMilestoneEvent {
            actor_id: 1,
            target_id: 2,
            kind: RelationshipMilestoneKind::FriendToRival,
            from_role: "Friend".into(),
            to_role: "Rival".into(),
            reason: "reason".into(),
            source: Some("test_milestone".into()),
            tick: Some(6),
        });

        // Storylet usage
        world.storylet_usage.times_fired.insert("story_1".into(), 2);

        // Memory entries
        world.memory_entries.push(crate::types::MemoryEntryRecord {
            id: "mem_1".into(),
            event_id: "evt".into(),
            npc_id: NpcId(1),
            sim_tick: SimTick(10),
            emotional_intensity: 0.7,
            stat_deltas: Vec::new(),
            relationship_deltas: Vec::new(),
            tags: vec!["support".into()],
            participants: vec![1, 2],
        });

        // District/world state
        world.district_state.insert("Downtown".into(), "ok".into());
        world.world_flags.insert("flag_test".into(), true);
        world.known_npcs.push(NpcId(2));

        // Digital legacy sample
        world.digital_legacy.primary_imprint = Some(DigitalImprint {
            id: 5,
            created_at_stage: LifeStage::Adult,
            created_at_age_years: 35,
            final_stats: Stats::default(),
            final_karma: Karma(10.0),
            legacy_vector: LegacyVector::default(),
            relationship_roles: HashMap::new(),
            relationship_milestones: Vec::new(),
            memory_tag_counts: HashMap::new(),
        });

        let snapshot_before = snapshot_json(&world);

        db.save_world(&world).expect("Failed to save world");
        let loaded = db.load_world(WorldSeed(99)).expect("Failed to load world");

        // Tick consistency
        assert_eq!(loaded.current_tick.0, loaded.game_time.tick_index);

        let snapshot_after = snapshot_json(&loaded);
        assert_eq!(snapshot_before, snapshot_after);

        let _ = fs::remove_file(db_path);
    }
}
