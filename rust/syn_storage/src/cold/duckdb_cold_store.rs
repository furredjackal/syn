//! DuckDB-based cold storage for dormant NPCs.

use duckdb::Connection;

use crate::models::AbstractNpc;
use crate::storage_error::StorageError;

/// Cold storage using DuckDB for dormant NPC data.
///
/// Stores dormant (Tier 3) NPCs in a columnar database optimized
/// for batch queries and long-term storage.
pub struct DuckDbColdStore {
    conn: Connection,
}

impl DuckDbColdStore {
    /// Create or open a DuckDB database at the given path.
    pub fn new(path: &str) -> Result<Self, StorageError> {
        let conn = Connection::open(path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS npc_dormant (
                id BIGINT PRIMARY KEY,
                age INTEGER,
                district INTEGER,
                wealth INTEGER,
                health DOUBLE,
                seed BIGINT
            )",
            [],
        )?;
        Ok(Self { conn })
    }

    /// Insert or update a dormant NPC.
    pub fn insert_dormant(&self, npc: &AbstractNpc) -> Result<(), StorageError> {
        self.conn.execute(
            "INSERT OR REPLACE INTO npc_dormant (id, age, district, wealth, health, seed)
             VALUES (?, ?, ?, ?, ?, ?)",
            duckdb::params![
                npc.id as i64,
                npc.age as i32,
                npc.district as i32,
                npc.wealth,
                npc.health as f64,
                npc.seed as i64
            ],
        )?;
        Ok(())
    }

    /// Load a dormant NPC by ID.
    pub fn load_dormant(&self, id: u64) -> Result<Option<AbstractNpc>, StorageError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, age, district, wealth, health, seed FROM npc_dormant WHERE id = ?",
        )?;
        let mut rows = stmt.query([id])?;
        if let Some(row) = rows.next()? {
            let npc = AbstractNpc {
                id: row.get(0)?,
                age: row.get(1)?,
                district: row.get(2)?,
                wealth: row.get(3)?,
                health: row.get(4)?,
                seed: row.get(5)?,
            };
            Ok(Some(npc))
        } else {
            Ok(None)
        }
    }
}
