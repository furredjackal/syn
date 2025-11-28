//! Redb-based hot storage for active NPCs.

use redb::{Database, ReadableTable, TableDefinition};

use crate::models::AbstractNpc;
use crate::storage_error::StorageError;

const NPC_TABLE: TableDefinition<u64, &[u8]> = TableDefinition::new("npc_active");

/// Hot storage using redb for fast NPC access.
///
/// Stores active (Tier 1) NPCs in a key-value database for
/// low-latency reads and writes during simulation.
pub struct RedbHotStore {
    db: Database,
}

impl RedbHotStore {
    /// Create or open a redb database at the given path.
    pub fn new(path: &str) -> Result<Self, StorageError> {
        let db = Database::create(path).map_err(redb::Error::from)?;
        Ok(Self { db })
    }

    /// Store an active NPC.
    pub fn put_active_npc(&self, npc: &AbstractNpc) -> Result<(), StorageError> {
        let npc_bytes = bincode::serialize(npc)?;
        let txn = self.db.begin_write().map_err(redb::Error::from)?;
        {
            let mut table = txn.open_table(NPC_TABLE).map_err(redb::Error::from)?;
            table
                .insert(npc.id, npc_bytes.as_slice())
                .map_err(redb::Error::from)?;
        }
        txn.commit().map_err(redb::Error::from)?;
        Ok(())
    }

    /// Retrieve an active NPC by ID.
    pub fn get_active_npc(&self, id: u64) -> Result<Option<AbstractNpc>, StorageError> {
        let txn = self.db.begin_read().map_err(redb::Error::from)?;
        let table = txn.open_table(NPC_TABLE).map_err(redb::Error::from)?;
        let npc_opt = if let Some(value) = table.get(id).map_err(redb::Error::from)? {
            let bytes = value.value();
            let npc: AbstractNpc = bincode::deserialize(bytes)?;
            Some(npc)
        } else {
            None
        };
        Ok(npc_opt)
    }
}
