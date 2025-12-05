//! SQLite persistence layer for SYN world state.
//!
//! Handles schema initialization, save/load, and world state serialization.

use crate::npc::NpcPrototype;
use crate::types::*;
use rusqlite::{params, Connection, Result as SqlResult};
use serde_json;
use std::collections::{HashMap, VecDeque};
use serde::{Deserialize, Serialize};

// Serializable wrapper for RelationshipMilestoneState with string keys instead of tuple keys
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RelationshipMilestoneSerializable {
    last_role: HashMap<String, crate::relationship_model::RelationshipRole>,
    queue: VecDeque<crate::relationship_milestones::RelationshipMilestoneEvent>,
}

fn map_invalid_query(err: rusqlite::Error, context: &str) -> rusqlite::Error {
    match err {
        rusqlite::Error::InvalidQuery => {
            panic!(
                "SQLite InvalidQuery in {}: statement or bindings are invalid",
                context
            );
        }
        other => other,
    }
}

#[derive(Debug)]
struct WorldRow {
    seed: i64,
    player_id: i64,
    current_tick: i64,
    player_stats: String,
    player_age: i64,
    player_age_years: i64,
    player_days_since_birth: i64,
    player_life_stage: String,
    player_karma: f64,
    narrative_heat: f64,
    heat_momentum: f64,
    relationships: String,
    npcs: String,
    npc_prototypes: String,
    known_npcs: String,
    game_time_tick: i64,
    relationship_pressure: String,
    relationship_milestones: String,
    digital_legacy: String,
    storylet_usage: String,
    memory_entries: String,
    district_state: String,
    world_flags: String,
}

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
    ///
    /// World table schema (canonical):
    /// - seed: INTEGER
    /// - current_tick: INTEGER
    /// - player_id: INTEGER
    /// - player_stats: TEXT (JSON)
    /// - player_age: INTEGER (legacy, kept in sync with player_age_years)
    /// - player_age_years: INTEGER
    /// - player_days_since_birth: INTEGER
    /// - player_life_stage: TEXT
    /// - player_karma: REAL
    /// - narrative_heat: REAL
    /// - heat_momentum: REAL
    /// - relationships: TEXT (JSON)
    /// - npcs: TEXT (JSON)
    /// - npc_prototypes: TEXT (JSON)
    /// - known_npcs: TEXT (JSON)
    /// - game_time_tick: INTEGER
    /// - storylet_usage: TEXT (JSON)
    /// - memory_entries: TEXT (JSON)
    /// - relationship_pressure: TEXT (JSON)
    /// - relationship_milestones: TEXT (JSON)
    /// - digital_legacy: TEXT (JSON)
    /// - district_state: TEXT (JSON)
    /// - world_flags: TEXT (JSON)
    fn init_schema(&mut self) -> SqlResult<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS world_state (
                id INTEGER PRIMARY KEY,
                seed INTEGER NOT NULL UNIQUE,
                player_id INTEGER NOT NULL,
                current_tick INTEGER NOT NULL,
                player_stats TEXT NOT NULL,
                player_age INTEGER NOT NULL DEFAULT 0,
                player_age_years INTEGER NOT NULL DEFAULT 0,
                player_days_since_birth INTEGER NOT NULL DEFAULT 0,
                player_life_stage TEXT NOT NULL,
                player_karma REAL NOT NULL,
                narrative_heat REAL NOT NULL DEFAULT 0.0,
                heat_momentum REAL NOT NULL DEFAULT 0.0,
                relationships TEXT NOT NULL DEFAULT '[]',
                npcs TEXT NOT NULL DEFAULT '{}',
                npc_prototypes TEXT NOT NULL DEFAULT '{}',
                known_npcs TEXT NOT NULL DEFAULT '[]',
                game_time_tick INTEGER NOT NULL DEFAULT 0,
                relationship_pressure TEXT NOT NULL DEFAULT '{}',
                relationship_milestones TEXT NOT NULL DEFAULT '{}',
                digital_legacy TEXT NOT NULL DEFAULT '{}',
                storylet_usage TEXT NOT NULL DEFAULT '{}',
                memory_entries TEXT NOT NULL DEFAULT '[]',
                district_state TEXT NOT NULL DEFAULT '{}',
                world_flags TEXT NOT NULL DEFAULT '{}',
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
        // Backfill columns if schema existed before; SQLite errors are ignored if columns already exist.
        let _ = self.conn.execute(
            "ALTER TABLE world_state ADD COLUMN narrative_heat REAL NOT NULL DEFAULT 0.0",
            params![],
        );
        let _ = self.conn.execute(
            "ALTER TABLE world_state ADD COLUMN player_age_years INTEGER NOT NULL DEFAULT 0",
            params![],
        );
        let _ = self.conn.execute(
            "ALTER TABLE world_state ADD COLUMN player_days_since_birth INTEGER NOT NULL DEFAULT 0",
            params![],
        );
        let _ = self.conn.execute(
            "UPDATE world_state SET player_days_since_birth = player_age * 365 WHERE player_days_since_birth = 0",
            params![],
        );
        let _ = self.conn.execute(
            "UPDATE world_state SET player_age_years = player_age WHERE player_age_years = 0",
            params![],
        );
        let _ = self.conn.execute(
            "ALTER TABLE world_state ADD COLUMN relationships TEXT NOT NULL DEFAULT '[]'",
            params![],
        );
        let _ = self.conn.execute(
            "ALTER TABLE world_state ADD COLUMN npcs TEXT NOT NULL DEFAULT '{}'",
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
        Ok(())
    }

    /// Save world state to database.
    pub fn save_world(&mut self, world: &WorldState) -> SqlResult<()> {
        let row = self.world_to_row(world)?;

        self.conn.execute(
            "INSERT OR REPLACE INTO world_state (seed, player_id, current_tick, player_stats, player_age, player_age_years, player_days_since_birth, player_life_stage, player_karma, narrative_heat, heat_momentum, relationships, npcs, npc_prototypes, known_npcs, game_time_tick, relationship_pressure, relationship_milestones, digital_legacy, storylet_usage, memory_entries, district_state, world_flags) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                row.seed,
                row.player_id,
                row.current_tick,
                row.player_stats,
                row.player_age,
                row.player_age_years,
                row.player_days_since_birth,
                row.player_life_stage,
                row.player_karma,
                row.narrative_heat,
                row.heat_momentum,
                row.relationships,
                row.npcs,
                row.npc_prototypes,
                row.known_npcs,
                row.game_time_tick,
                row.relationship_pressure,
                row.relationship_milestones,
                row.digital_legacy,
                row.storylet_usage,
                row.memory_entries,
                row.district_state,
                row.world_flags,
            ],
        )
        .map_err(|e| map_invalid_query(e, "save_world INSERT"))?;

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
            "SELECT seed, player_id, current_tick, player_stats, player_age, player_age_years, player_days_since_birth, player_life_stage, player_karma, narrative_heat, heat_momentum, relationships, npcs, npc_prototypes, known_npcs, game_time_tick, relationship_pressure, relationship_milestones, digital_legacy, storylet_usage, memory_entries, district_state, world_flags
             FROM world_state WHERE seed = ?",
        )?;

        let world = stmt.query_row(params![seed.0], |row| {
            Ok(WorldRow {
                seed: row.get::<_, i64>(0)?,
                player_id: row.get::<_, i64>(1)?,
                current_tick: row.get::<_, i64>(2)?,
                player_stats: row.get::<_, String>(3)?,
                player_age: row.get::<_, i64>(4)?,
                player_age_years: row.get::<_, i64>(5)?,
                player_days_since_birth: row.get::<_, i64>(6)?,
                player_life_stage: row.get::<_, String>(7)?,
                player_karma: row.get::<_, f64>(8)?,
                narrative_heat: row.get::<_, f64>(9)?,
                heat_momentum: row.get::<_, f64>(10)?,
                relationships: row.get::<_, String>(11)?,
                npcs: row.get::<_, String>(12)?,
                npc_prototypes: row.get::<_, String>(13)?,
                known_npcs: row.get::<_, String>(14)?,
                game_time_tick: row.get::<_, i64>(15)?,
                relationship_pressure: row.get::<_, String>(16)?,
                relationship_milestones: row.get::<_, String>(17)?,
                digital_legacy: row.get::<_, String>(18)?,
                storylet_usage: row.get::<_, String>(19)?,
                memory_entries: row.get::<_, String>(20)?,
                district_state: row.get::<_, String>(21)?,
                world_flags: row.get::<_, String>(22)?,
            })
        })?;

        let world = self.world_from_row(seed, world)?;

        Ok(world)
    }

    fn world_to_row(&self, world: &WorldState) -> SqlResult<WorldRow> {
        let relationships_serializable: Vec<((u64, u64), Relationship)> = world
            .relationships
            .iter()
            .map(|((a, b), rel)| ((a.0, b.0), rel.clone()))
            .collect();
        let npcs_serializable: HashMap<u64, AbstractNpc> = world
            .npcs
            .iter()
            .map(|(id, npc)| (id.0, npc.clone()))
            .collect();
        let npc_prototypes_serializable: HashMap<u64, NpcPrototype> = world
            .npc_prototypes
            .iter()
            .map(|(id, proto)| (id.0, proto.clone()))
            .collect();
        
        // Convert relationship_milestones last_role HashMap with tuple keys to string keys for JSON
        let relationship_milestones_serializable = RelationshipMilestoneSerializable {
            last_role: world
                .relationship_milestones
                .last_role
                .iter()
                .map(|((a, b), role)| (format!("{}-{}", a, b), *role))
                .collect(),
            queue: world.relationship_milestones.queue.clone(),
        };

        Ok(WorldRow {
            seed: world.seed.0 as i64,
            current_tick: world.current_tick.0 as i64,
            player_id: world.player_id.0 as i64,
            player_stats: serde_json::to_string(&world.player_stats)
                .map_err(|_| rusqlite::Error::InvalidQuery)?,
            player_age: world.player_age as i64,
            player_age_years: world.player_age_years as i64,
            player_days_since_birth: world.player_days_since_birth as i64,
            player_life_stage: format!("{:?}", world.player_life_stage),
            player_karma: world.player_karma.0 as f64,
            narrative_heat: world.narrative_heat.value() as f64,
            heat_momentum: world.heat_momentum as f64,
            relationships: serde_json::to_string(&relationships_serializable)
                .map_err(|_| rusqlite::Error::InvalidQuery)?,
            npcs: serde_json::to_string(&npcs_serializable)
                .map_err(|_| rusqlite::Error::InvalidQuery)?,
            npc_prototypes: serde_json::to_string(&npc_prototypes_serializable)
                .map_err(|_| rusqlite::Error::InvalidQuery)?,
            known_npcs: serde_json::to_string(&world.known_npcs)
                .map_err(|_| rusqlite::Error::InvalidQuery)?,
            game_time_tick: world.game_time.tick_index as i64,
            relationship_pressure: serde_json::to_string(&world.relationship_pressure)
                .map_err(|_| rusqlite::Error::InvalidQuery)?,
            relationship_milestones: serde_json::to_string(&relationship_milestones_serializable)
                .map_err(|_| rusqlite::Error::InvalidQuery)?,
            digital_legacy: serde_json::to_string(&world.digital_legacy)
                .map_err(|_| rusqlite::Error::InvalidQuery)?,
            storylet_usage: serde_json::to_string(&world.storylet_usage)
                .map_err(|_| rusqlite::Error::InvalidQuery)?,
            memory_entries: serde_json::to_string(&world.memory_entries)
                .map_err(|_| rusqlite::Error::InvalidQuery)?,
            district_state: serde_json::to_string(&world.district_state)
                .map_err(|_| rusqlite::Error::InvalidQuery)?,
            world_flags: serde_json::to_string(&world.world_flags)
                .map_err(|_| rusqlite::Error::InvalidQuery)?,
        })
    }

    fn world_from_row(&self, seed: WorldSeed, row: WorldRow) -> SqlResult<WorldState> {
        let player_stats: Stats =
            serde_json::from_str(&row.player_stats).map_err(|_| rusqlite::Error::InvalidQuery)?;
        let known_npcs: Vec<NpcId> =
            serde_json::from_str(&row.known_npcs).map_err(|_| rusqlite::Error::InvalidQuery)?;
        let npc_prototypes_raw: HashMap<u64, NpcPrototype> =
            serde_json::from_str(&row.npc_prototypes).map_err(|_| rusqlite::Error::InvalidQuery)?;
        let npc_prototypes = npc_prototypes_raw
            .into_iter()
            .map(|(id, proto)| (NpcId(id), proto))
            .collect();
        let relationship_pressure: crate::relationship_pressure::RelationshipPressureState =
            serde_json::from_str(&row.relationship_pressure)
                .map_err(|_| rusqlite::Error::InvalidQuery)?;
        let relationship_milestones_serializable: RelationshipMilestoneSerializable =
            serde_json::from_str(&row.relationship_milestones)
                .map_err(|_| rusqlite::Error::InvalidQuery)?;
        // Convert string keys back to tuple keys
        let relationship_milestones = crate::relationship_milestones::RelationshipMilestoneState {
            last_role: relationship_milestones_serializable
                .last_role
                .into_iter()
                .filter_map(|(key, role)| {
                    let parts: Vec<&str> = key.split('-').collect();
                    if parts.len() == 2 {
                        let a = parts[0].parse::<u64>().ok()?;
                        let b = parts[1].parse::<u64>().ok()?;
                        Some(((a, b), role))
                    } else {
                        None
                    }
                })
                .collect(),
            queue: relationship_milestones_serializable.queue,
        };
        let digital_legacy: crate::digital_legacy::DigitalLegacyState =
            serde_json::from_str(&row.digital_legacy).map_err(|_| rusqlite::Error::InvalidQuery)?;
        let storylet_usage: crate::types::StoryletUsageState =
            serde_json::from_str(&row.storylet_usage).map_err(|_| rusqlite::Error::InvalidQuery)?;
        let memory_entries: Vec<crate::types::MemoryEntryRecord> =
            serde_json::from_str(&row.memory_entries).map_err(|_| rusqlite::Error::InvalidQuery)?;
        let district_state: HashMap<String, String> =
            serde_json::from_str(&row.district_state).map_err(|_| rusqlite::Error::InvalidQuery)?;
        let world_flags: crate::world_flags::WorldFlags =
            serde_json::from_str(&row.world_flags).map_err(|_| rusqlite::Error::InvalidQuery)?;
        let relationships_pairs: Vec<((u64, u64), Relationship)> =
            serde_json::from_str(&row.relationships).map_err(|_| rusqlite::Error::InvalidQuery)?;
        let mut relationships: HashMap<(NpcId, NpcId), Relationship> = HashMap::new();
        for ((a, b), rel) in relationships_pairs {
            relationships.insert((NpcId(a), NpcId(b)), rel);
        }
        let npcs_raw: HashMap<u64, AbstractNpc> =
            serde_json::from_str(&row.npcs).map_err(|_| rusqlite::Error::InvalidQuery)?;
        let mut npcs: HashMap<NpcId, AbstractNpc> = HashMap::new();
        for (id, npc) in npcs_raw {
            npcs.insert(NpcId(id), npc);
        }

        let player_days_since_birth = row.player_days_since_birth.max(0) as u32;
        let age_years = if row.player_age_years > 0 {
            row.player_age_years as u32
        } else {
            player_days_since_birth / 365
        };
        let player_age = if row.player_age > 0 {
            row.player_age as u32
        } else {
            age_years
        };
        let life_stage = match row.player_life_stage.as_str() {
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
            current_tick: SimTick(row.current_tick.max(0) as u64),
            player_id: NpcId(row.player_id as u64),
            player_stats,
            player_age,
            player_age_years: age_years,
            player_days_since_birth,
            player_life_stage: life_stage,
            player_karma: Karma(row.player_karma as f32),
            narrative_heat: crate::narrative_heat::NarrativeHeat::new(row.narrative_heat as f32),
            heat_momentum: row.heat_momentum as f32,
            relationships,
            npcs,
            relationship_pressure,
            relationship_milestones,
            digital_legacy,
            npc_prototypes,
            known_npcs,
            game_time: crate::time::GameTime::from_tick(row.game_time_tick.max(0) as u64),
            storylet_usage,
            memory_entries,
            district_state,
            districts: crate::district::DistrictRegistry::generate_default_city(seed.0),
            district_pressure: crate::district_pressure::DistrictPressureState::default(),
            player_skills: crate::skills::PlayerSkills::default(),
            gossip: crate::gossip::GossipSystem::default(),
            gossip_pressure: crate::gossip_pressure::GossipPressureState::default(),
            population: crate::population::PopulationSimulation::default(),
            failure_recovery: crate::failure_recovery::FailureRecoverySystem::default(),
            world_flags,
        };

        // Normalize any legacy skew: if game_time_tick wasn't stored (defaulted to 0), sync it with current_tick
        if row.game_time_tick == 0 && world.current_tick.0 > 0 {
            world.game_time = crate::time::GameTime::from_tick(world.current_tick.0);
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
    /// Unique storylet identifier.
    pub id: String,
    /// Display name for the storylet.
    pub name: String,
    /// Full JSON data for the storylet.
    pub json_data: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::digital_legacy::DigitalLegacyState;
    use crate::digital_legacy::{DigitalImprint, LegacyVector};
    use crate::npc::{NpcPrototype, NpcSchedule, PersonalityVector};
    use crate::relationship_milestones::RelationshipMilestoneKind;
    use crate::relationship_milestones::{RelationshipMilestoneEvent, RelationshipMilestoneState};
    use crate::relationship_model::RelationshipRole;
    use crate::relationship_pressure::RelationshipBandSnapshot;
    use crate::relationship_pressure::RelationshipEventKind;
    use crate::relationship_pressure::{RelationshipPressureEvent, RelationshipPressureState};
    use crate::time::TickContext;
    use serde::Serialize;
    use serde_json::Value;
    use std::collections::VecDeque;
    use std::fs;

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
        world
            .relationship_pressure
            .queue
            .push_back(RelationshipPressureEvent {
                actor_id: 1,
                target_id: 2,
                kind: RelationshipEventKind::AffectionBandChanged,
                old_band: "Stranger".into(),
                new_band: "Acquaintance".into(),
                source: Some("test".into()),
                tick: Some(1),
            });
        world
            .relationship_milestones
            .last_role
            .insert((1, 2), crate::relationship_model::RelationshipRole::Friend);
        world
            .relationship_milestones
            .queue
            .push_back(RelationshipMilestoneEvent {
                actor_id: 1,
                target_id: 2,
                kind: RelationshipMilestoneKind::FriendToRival,
                from_role: "Friend".into(),
                to_role: "Rival".into(),
                reason: "test_reason".into(),
                source: Some("test_source".into()),
                tick: Some(2),
            });
        let mut rel = Relationship::default();
        rel.affection = 1.5;
        world.set_relationship(NpcId(1), NpcId(2), rel);
        let npc = AbstractNpc::new_basic(
            NpcId(2),
            30,
            "Tester".into(),
            "Downtown".into(),
            10,
            Traits::default(),
            777,
            AttachmentStyle::Secure,
        );
        world.npcs.insert(npc.id, npc);
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
        world.world_flags.set_any("met_childhood_friend");
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
            loaded
                .npc_prototypes
                .get(&NpcId(2))
                .map(|p| p.display_name.clone()),
            Some("Tester".to_string())
        );
        assert_eq!(
            loaded
                .relationships
                .get(&(NpcId(1), NpcId(2)))
                .map(|r| r.affection),
            Some(1.5)
        );
        assert_eq!(
            loaded.npcs.get(&NpcId(2)).map(|n| n.job.as_str()),
            Some("Tester")
        );
        assert!(loaded.digital_legacy.primary_imprint.is_some());
        assert_eq!(loaded.memory_entries.len(), 1);
        assert_eq!(
            loaded.district_state.get("Downtown"),
            Some(&"ok".to_string())
        );
        assert!(loaded.world_flags.has_any("met_childhood_friend"));
        let _ = snapshot_json(&loaded);

        let _ = fs::remove_file(db_path);
    }

    #[derive(Debug, Serialize)]
    struct RelationshipPressureJsonSnapshot {
        last_bands: HashMap<String, RelationshipBandSnapshot>,
        queue: VecDeque<RelationshipPressureEvent>,
        changed_pairs: Vec<(u64, u64)>,
    }

    #[derive(Debug, Serialize)]
    struct RelationshipMilestoneJsonSnapshot {
        last_role: HashMap<String, RelationshipRole>,
        queue: VecDeque<RelationshipMilestoneEvent>,
    }

    #[derive(Debug, Serialize)]
    struct DigitalImprintJsonSnapshot {
        id: u64,
        created_at_stage: LifeStage,
        created_at_age_years: u32,
        final_stats: Stats,
        final_karma: Karma,
        legacy_vector: LegacyVector,
        relationship_roles: HashMap<String, RelationshipRole>,
        relationship_milestones: Vec<RelationshipMilestoneEvent>,
        memory_tag_counts: HashMap<String, u32>,
    }

    #[derive(Debug, Serialize)]
    struct DigitalLegacyJsonSnapshot {
        primary_imprint: Option<DigitalImprintJsonSnapshot>,
        archived_imprints: Vec<DigitalImprintJsonSnapshot>,
    }

    #[derive(Debug, Serialize)]
    struct WorldStateJsonSnapshot {
        seed: WorldSeed,
        current_tick: SimTick,
        player_id: NpcId,
        player_stats: Stats,
        player_age_years: u32,
        player_days_since_birth: u32,
        player_life_stage: LifeStage,
        player_karma: Karma,
        narrative_heat: crate::narrative_heat::NarrativeHeat,
        heat_momentum: f32,
        relationships: HashMap<String, Relationship>,
        npcs: HashMap<String, AbstractNpc>,
        relationship_pressure: RelationshipPressureJsonSnapshot,
        relationship_milestones: RelationshipMilestoneJsonSnapshot,
        digital_legacy: DigitalLegacyJsonSnapshot,
        npc_prototypes: HashMap<String, NpcPrototype>,
        known_npcs: Vec<NpcId>,
        game_time_tick: u64,
        storylet_usage: StoryletUsageState,
        memory_entries: Vec<crate::types::MemoryEntryRecord>,
        district_state: HashMap<String, String>,
        world_flags: crate::world_flags::WorldFlags,
    }

    impl WorldStateJsonSnapshot {
        fn from_snapshot(snapshot: &WorldStateSnapshot) -> Self {
            let relationships = snapshot
                .relationships
                .iter()
                .map(|((a, b), v)| (format!("{}-{}", a.0, b.0), v.clone()))
                .collect();
            let npcs = snapshot
                .npcs
                .iter()
                .map(|(id, npc)| (format!("{}", id.0), npc.clone()))
                .collect();
            let npc_prototypes = snapshot
                .npc_prototypes
                .iter()
                .map(|(id, proto)| (format!("{}", id.0), proto.clone()))
                .collect();
            let relationship_pressure =
                Self::relationship_pressure_json(&snapshot.relationship_pressure);
            let relationship_milestones =
                Self::relationship_milestones_json(&snapshot.relationship_milestones);
            let digital_legacy = Self::digital_legacy_json(&snapshot.digital_legacy);

            WorldStateJsonSnapshot {
                seed: snapshot.seed,
                current_tick: snapshot.current_tick,
                player_id: snapshot.player_id,
                player_stats: snapshot.player_stats,
                player_age_years: snapshot.player_age_years,
                player_days_since_birth: snapshot.player_days_since_birth,
                player_life_stage: snapshot.player_life_stage,
                player_karma: snapshot.player_karma,
                narrative_heat: snapshot.narrative_heat,
                heat_momentum: snapshot.heat_momentum,
                relationships,
                npcs,
                relationship_pressure,
                relationship_milestones,
                digital_legacy,
                npc_prototypes,
                known_npcs: snapshot.known_npcs.clone(),
                game_time_tick: snapshot.game_time_tick,
                storylet_usage: snapshot.storylet_usage.clone(),
                memory_entries: snapshot.memory_entries.clone(),
                district_state: snapshot.district_state.clone(),
                world_flags: snapshot.world_flags.clone(),
            }
        }

        fn relationship_pressure_json(
            state: &RelationshipPressureState,
        ) -> RelationshipPressureJsonSnapshot {
            let last_bands = state
                .last_bands
                .iter()
                .map(|((actor_id, target_id), bands)| {
                    (format!("{}-{}", actor_id, target_id), bands.clone())
                })
                .collect();

            RelationshipPressureJsonSnapshot {
                last_bands,
                queue: state.queue.clone(),
                changed_pairs: state.changed_pairs.clone(),
            }
        }

        fn relationship_milestones_json(
            state: &RelationshipMilestoneState,
        ) -> RelationshipMilestoneJsonSnapshot {
            let last_role = state
                .last_role
                .iter()
                .map(|((actor_id, target_id), role)| (format!("{}-{}", actor_id, target_id), *role))
                .collect();

            RelationshipMilestoneJsonSnapshot {
                last_role,
                queue: state.queue.clone(),
            }
        }

        fn digital_legacy_json(state: &DigitalLegacyState) -> DigitalLegacyJsonSnapshot {
            let imprint_to_json = |imprint: &DigitalImprint| Self::digital_imprint_json(imprint);

            DigitalLegacyJsonSnapshot {
                primary_imprint: state.primary_imprint.as_ref().map(&imprint_to_json),
                archived_imprints: state
                    .archived_imprints
                    .iter()
                    .map(&imprint_to_json)
                    .collect(),
            }
        }

        fn digital_imprint_json(imprint: &DigitalImprint) -> DigitalImprintJsonSnapshot {
            let relationship_roles = imprint
                .relationship_roles
                .iter()
                .map(|(id, role)| (format!("{}", id.0), *role))
                .collect();

            DigitalImprintJsonSnapshot {
                id: imprint.id,
                created_at_stage: imprint.created_at_stage,
                created_at_age_years: imprint.created_at_age_years,
                final_stats: imprint.final_stats.clone(),
                final_karma: imprint.final_karma,
                legacy_vector: imprint.legacy_vector.clone(),
                relationship_roles,
                relationship_milestones: imprint.relationship_milestones.clone(),
                memory_tag_counts: imprint.memory_tag_counts.clone(),
            }
        }
    }

    fn snapshot_json(world: &WorldState) -> Value {
        let snap = WorldStateSnapshot::from_world(world);
        let json_safe = WorldStateJsonSnapshot::from_snapshot(&snap);
        serde_json::to_value(json_safe).expect("world snapshot should serialize to json value")
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
        let npc = AbstractNpc::new_basic(
            NpcId(2),
            22,
            "Engineer".into(),
            "Downtown".into(),
            10,
            Traits::default(),
            7,
            AttachmentStyle::Secure,
        );
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
        world
            .relationship_pressure
            .queue
            .push_back(RelationshipPressureEvent {
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
        world
            .relationship_milestones
            .queue
            .push_back(RelationshipMilestoneEvent {
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
        world.world_flags.set_any("flag_test");
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
