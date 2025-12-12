//! Integration test for journal archival to cold storage.

#[cfg(feature = "storage")]
mod with_storage {
    use syn_core::{NpcId, SimTick};
    use syn_memory::{MemoryEntry, MemorySystem};
    use syn_storage::HybridStorage;
    use tempfile::TempDir;

    #[test]
    fn test_archive_and_load_journal() {
        // Create temporary storage
        let temp_dir = TempDir::new().unwrap();
        let hot_path = temp_dir.path().join("test.redb");
        let cold_path = temp_dir.path().join("test.duckdb");
        
        let storage = HybridStorage::new(
            hot_path.to_str().unwrap(),
            cold_path.to_str().unwrap(),
        )
        .expect("Failed to create storage");

        let mut memory_sys = MemorySystem::new();
        let npc_id = NpcId(42);

        // Create a journal with some memories
        for i in 0..5 {
            let entry = MemoryEntry::new(
                format!("mem_{}", i),
                "event_test".to_string(),
                npc_id,
                SimTick(i * 10),
                0.5,
            );
            memory_sys.record_memory(entry);
        }

        let original_journal = memory_sys.get_journal(npc_id).unwrap().clone();
        assert_eq!(original_journal.entries.len(), 5);

        // Archive the journal
        memory_sys
            .archive_journal(npc_id, &storage)
            .expect("Failed to archive journal");

        // Clear the in-memory journal
        memory_sys.journals.clear();
        assert!(memory_sys.get_journal(npc_id).is_none());

        // Load the journal back
        let loaded_journal = memory_sys
            .load_archived_journal(npc_id, &storage)
            .expect("Failed to load journal")
            .expect("Journal not found");

        // Verify the loaded journal matches the original
        assert_eq!(loaded_journal.npc_id, original_journal.npc_id);
        assert_eq!(loaded_journal.entries.len(), original_journal.entries.len());
        
        for (loaded, original) in loaded_journal.entries.iter().zip(original_journal.entries.iter()) {
            assert_eq!(loaded.id, original.id);
            assert_eq!(loaded.event_id, original.event_id);
            assert_eq!(loaded.sim_tick, original.sim_tick);
        }
    }

    #[test]
    fn test_load_nonexistent_journal() {
        let temp_dir = TempDir::new().unwrap();
        let hot_path = temp_dir.path().join("test.redb");
        let cold_path = temp_dir.path().join("test.duckdb");
        
        let storage = HybridStorage::new(
            hot_path.to_str().unwrap(),
            cold_path.to_str().unwrap(),
        )
        .expect("Failed to create storage");

        let mut memory_sys = MemorySystem::new();
        let npc_id = NpcId(999);

        // Try to load a journal that doesn't exist
        let result = memory_sys
            .load_archived_journal(npc_id, &storage)
            .expect("Should not error");

        assert!(result.is_none());
    }

    #[test]
    fn test_prune_with_archive() {
        let temp_dir = TempDir::new().unwrap();
        let hot_path = temp_dir.path().join("test.redb");
        let cold_path = temp_dir.path().join("test.duckdb");
        
        let storage = HybridStorage::new(
            hot_path.to_str().unwrap(),
            cold_path.to_str().unwrap(),
        )
        .expect("Failed to create storage");

        let mut memory_sys = MemorySystem::new();
        let npc_id = NpcId(100);

        // Create memories spanning 10 days
        for day in 0..10 {
            let entry = MemoryEntry::new(
                format!("mem_{}", day),
                "event_daily".to_string(),
                npc_id,
                SimTick(day * 24),
                0.5,
            );
            memory_sys.record_memory(entry);
        }

        assert_eq!(memory_sys.get_journal(npc_id).unwrap().entries.len(), 10);

        // Prune to keep only last 5 days (with archival)
        let pruned = memory_sys
            .prune_old_memories(
                npc_id,
                SimTick(9 * 24), // current is day 9
                5,
                Some(&storage),
            )
            .expect("Failed to prune");

        assert!(pruned > 0); // Some memories were pruned
        
        // Verify in-memory journal is pruned
        let current_entries = memory_sys.get_journal(npc_id).unwrap().entries.len();
        assert!(current_entries < 10);

        // Verify full journal was archived
        let mut new_sys = MemorySystem::new();
        let archived = new_sys
            .load_archived_journal(npc_id, &storage)
            .expect("Failed to load")
            .expect("Archive not found");
        
        assert_eq!(archived.entries.len(), 10); // Full journal before pruning
    }
}
