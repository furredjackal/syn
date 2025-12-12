//! Hybrid storage combining hot and cold tiers for NPC data.

use crate::cold::DuckDbColdStore;
use crate::hot::RedbHotStore;
use crate::models::AbstractNpc;
use crate::storage_error::StorageError;

/// Unified storage interface for hot (active) and cold (dormant) NPCs.
///
/// Provides a single API for storing and retrieving NPCs across both tiers,
/// with promote/demote operations for LOD transitions.
pub struct HybridStorage {
    hot: RedbHotStore,
    cold: DuckDbColdStore,
}

impl std::fmt::Debug for HybridStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HybridStorage").finish()
    }
}

impl HybridStorage {
    /// Create a new hybrid storage with the given paths.
    ///
    /// # Arguments
    /// * `hot_path` - Path to the redb database file for hot storage
    /// * `cold_path` - Path to the DuckDB database file for cold storage
    pub fn new(hot_path: &str, cold_path: &str) -> Result<Self, StorageError> {
        let hot = RedbHotStore::new(hot_path)?;
        let cold = DuckDbColdStore::new(cold_path)?;
        Ok(Self { hot, cold })
    }

    /// Save an NPC to hot (active) storage.
    pub fn save_active(&self, npc: &AbstractNpc) -> Result<(), StorageError> {
        self.hot.put_active_npc(npc)
    }

    /// Load an NPC from hot (active) storage.
    pub fn load_active(&self, id: u64) -> Result<Option<AbstractNpc>, StorageError> {
        self.hot.get_active_npc(id)
    }

    /// Save an NPC to cold (dormant) storage.
    pub fn save_dormant(&self, npc: &AbstractNpc) -> Result<(), StorageError> {
        self.cold.insert_dormant(npc)
    }

    /// Load an NPC from cold (dormant) storage.
    pub fn load_dormant(&self, id: u64) -> Result<Option<AbstractNpc>, StorageError> {
        self.cold.load_dormant(id)
    }

    /// Promote an NPC from cold to hot storage (dormant → active).
    pub fn promote(&self, id: u64) -> Result<(), StorageError> {
        if let Some(npc) = self.load_dormant(id)? {
            self.save_active(&npc)?;
        }
        Ok(())
    }

    /// Demote an NPC from hot to cold storage (active → dormant).
    pub fn demote(&self, id: u64) -> Result<(), StorageError> {
        if let Some(npc) = self.load_active(id)? {
            self.save_dormant(&npc)?;
        }
        Ok(())
    }

    /// Archive a journal (JSON string) to cold storage.
    pub fn archive_journal(&self, npc_id: u64, journal_json: &str) -> Result<(), StorageError> {
        self.cold.archive_journal(npc_id, journal_json)
    }

    /// Load an archived journal from cold storage.
    pub fn load_archived_journal(&self, npc_id: u64) -> Result<Option<String>, StorageError> {
        self.cold.load_archived_journal(npc_id)
    }
}
