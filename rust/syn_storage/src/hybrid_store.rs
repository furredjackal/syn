use crate::cold::DuckDbColdStore;
use crate::hot::RedbHotStore;
use crate::models::AbstractNpc;
use crate::storage_error::StorageError;

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
    pub fn new(hot_path: &str, cold_path: &str) -> Result<Self, StorageError> {
        let hot = RedbHotStore::new(hot_path)?;
        let cold = DuckDbColdStore::new(cold_path)?;
        Ok(Self { hot, cold })
    }

    // Tier 1 (Active) API
    pub fn save_active(&self, npc: &AbstractNpc) -> Result<(), StorageError> {
        self.hot.put_active_npc(npc)
    }

    pub fn load_active(&self, id: u64) -> Result<Option<AbstractNpc>, StorageError> {
        self.hot.get_active_npc(id)
    }

    // Tier 3 (Dormant) API
    pub fn save_dormant(&self, npc: &AbstractNpc) -> Result<(), StorageError> {
        self.cold.insert_dormant(npc)
    }

    pub fn load_dormant(&self, id: u64) -> Result<Option<AbstractNpc>, StorageError> {
        self.cold.load_dormant(id)
    }

    // LOD transitions
    pub fn promote(&self, id: u64) -> Result<(), StorageError> {
        if let Some(npc) = self.load_dormant(id)? {
            self.save_active(&npc)?;
        }
        Ok(())
    }

    pub fn demote(&self, id: u64) -> Result<(), StorageError> {
        if let Some(npc) = self.load_active(id)? {
            self.save_dormant(&npc)?;
        }
        Ok(())
    }
}
