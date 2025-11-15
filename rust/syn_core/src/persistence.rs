//! SQLite persistence layer for SYN world state.
//!
//! Handles schema initialization, save/load, and world state serialization.

use rusqlite::{params, Connection, Result as SqlResult};
use serde_json;
use crate::types::*;

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

            CREATE INDEX IF NOT EXISTS idx_relationships ON relationships(world_seed, from_npc_id);
            CREATE INDEX IF NOT EXISTS idx_npcs ON npcs(world_seed, npc_id);
            CREATE INDEX IF NOT EXISTS idx_memories ON memory_entries(world_seed, npc_id);
            ",
        )?;
        Ok(())
    }

    /// Save world state to database.
    pub fn save_world(&mut self, world: &WorldState) -> SqlResult<()> {
        let player_stats_json = serde_json::to_string(&world.player_stats)
            .map_err(|_| rusqlite::Error::InvalidQuery)?;
        let player_karma_json = world.player_karma.0;

        self.conn.execute(
            "INSERT OR REPLACE INTO world_state 
             (seed, player_id, current_tick, player_stats, player_age, player_life_stage, player_karma, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)",
            params![
                world.seed.0,
                world.player_id.0,
                world.current_tick.0,
                player_stats_json,
                world.player_age,
                format!("{:?}", world.player_life_stage),
                player_karma_json,
            ],
        )?;

        // Save relationships
        for ((from_id, to_id), rel) in &world.relationships {
            let rel_json = serde_json::to_string(rel)
                .map_err(|_| rusqlite::Error::InvalidQuery)?;
            self.conn.execute(
                "INSERT OR REPLACE INTO relationships (world_seed, from_npc_id, to_npc_id, relationship_data)
                 VALUES (?, ?, ?, ?)",
                params![world.seed.0, from_id.0, to_id.0, rel_json],
            )?;
        }

        // Save NPCs
        for (npc_id, npc) in &world.npcs {
            let npc_json = serde_json::to_string(npc)
                .map_err(|_| rusqlite::Error::InvalidQuery)?;
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
            "SELECT player_id, current_tick, player_stats, player_age, player_life_stage, player_karma 
             FROM world_state WHERE seed = ?",
        )?;

        let world = stmt.query_row(params![seed.0], |row| {
            let player_id = NpcId(row.get::<_, u64>(0)?);
            let current_tick = SimTick(row.get(1)?);
            let stats_json: String = row.get(2)?;
            let player_age: u32 = row.get(3)?;
            let life_stage_str: String = row.get(4)?;
            let karma_value: f32 = row.get(5)?;

            let player_stats: Stats = serde_json::from_str(&stats_json)
                .unwrap_or_default();
            let player_life_stage = match life_stage_str.as_str() {
                "Child" => LifeStage::Child,
                "Teen" => LifeStage::Teen,
                "YoungAdult" => LifeStage::YoungAdult,
                "Adult" => LifeStage::Adult,
                "Elder" => LifeStage::Elder,
                "Digital" => LifeStage::Digital,
                _ => LifeStage::Child,
            };

            let world = WorldState {
                seed,
                current_tick,
                player_id,
                player_stats,
                player_age,
                player_life_stage,
                player_karma: Karma(karma_value),
                relationships: Default::default(),
                npcs: Default::default(),
            };

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
            let rel: Relationship = serde_json::from_str(&rel_json)
                .unwrap_or_default();
            Ok(((from_id, to_id), rel))
        })?;

        let mut world = world;
        for rel in relationships {
            let (key, rel_data) = rel?;
            world.relationships.insert(key, rel_data);
        }

        // Load NPCs
        let mut npc_stmt = self.conn.prepare(
            "SELECT npc_id, npc_data FROM npcs WHERE world_seed = ?"
        )?;
        let npcs = npc_stmt.query_map(params![seed.0], |row| {
            let npc_id = NpcId(row.get(0)?);
            let npc_json: String = row.get(1)?;
            let npc: AbstractNpc = serde_json::from_str(&npc_json)
                .unwrap_or_else(|_| AbstractNpc {
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
        self.conn.execute(
            "DELETE FROM npcs WHERE world_seed = ?",
            params![seed.0],
        )?;
        self.conn.execute(
            "DELETE FROM relationships WHERE world_seed = ?",
            params![seed.0],
        )?;
        self.conn.execute(
            "DELETE FROM world_state WHERE seed = ?",
            params![seed.0],
        )?;
        Ok(())
    }

    /// Check if a world exists.
    pub fn world_exists(&mut self, seed: WorldSeed) -> SqlResult<bool> {
        let mut stmt = self.conn.prepare("SELECT 1 FROM world_state WHERE seed = ?")?;
        let exists = stmt.exists(params![seed.0])?;
        Ok(exists)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_persistence_save_load() {
        let db_path = "test_persistence.db";
        let _ = fs::remove_file(db_path);

        let mut db = Persistence::new(db_path).expect("Failed to create persistence");
        let mut world = WorldState::new(WorldSeed(42), NpcId(1));
        world.player_age = 25;

        db.save_world(&world).expect("Failed to save world");
        let loaded = db.load_world(WorldSeed(42)).expect("Failed to load world");

        assert_eq!(loaded.player_age, 25);
        assert_eq!(loaded.seed, WorldSeed(42));

        let _ = fs::remove_file(db_path);
    }
}
