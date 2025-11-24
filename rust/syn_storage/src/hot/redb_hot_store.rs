use redb::{Database, ReadableTable, TableDefinition};

use crate::models::AbstractNpc;
use crate::storage_error::StorageError;

const NPC_TABLE: TableDefinition<u64, &[u8]> = TableDefinition::new("npc_active");

pub struct RedbHotStore {
    db: Database,
}

impl RedbHotStore {
    pub fn new(path: &str) -> Result<Self, StorageError> {
        let db = Database::create(path).map_err(redb::Error::from)?;
        Ok(Self { db })
    }

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
