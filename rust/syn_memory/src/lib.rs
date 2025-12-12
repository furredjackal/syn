//! syn_memory: Memory entries and journal system for SYN.
//!
//! Records player choices, event outcomes, and emotional impacts.
//! Memories are used by the Event Director to trigger echos and narrative chains.
//!
//! ## Cold Storage Integration
//!
//! When the `storage` feature is enabled (default), journals can be archived
//! to cold storage via HybridStorage, enabling long-term memory persistence
//! for dormant NPCs.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use syn_core::npc_behavior::BehaviorKind;
pub use syn_core::relationships::RelationshipDelta;
pub use syn_core::{NpcId, SimTick, StatDelta};

#[cfg(feature = "storage")]
use syn_storage::HybridStorage;

#[cfg(feature = "storage")]
use syn_storage::storage_error::StorageError;

/// A single memory entry recording an event and its impact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,               // Unique memory ID
    pub event_id: String,         // Which storylet fired
    pub npc_id: NpcId,            // Who holds this memory
    pub sim_tick: SimTick,        // When it happened
    pub emotional_intensity: f32, // -1.0 (negative) to +1.0 (positive)
    #[serde(default)]
    pub stat_deltas: Vec<StatDelta>, // e.g., [{"kind": Mood, "delta": -2.0}]
    #[serde(default)]
    pub relationship_deltas: Vec<RelationshipDelta>,
    #[serde(default)]
    pub tags: Vec<String>, // e.g., ["betrayal", "trauma", "relationship"]
    /// Optional list of participant IDs involved in this memory.
    #[serde(default)]
    pub participants: Vec<u64>,
}

impl MemoryEntry {
    pub fn new(
        id: String,
        event_id: String,
        npc_id: NpcId,
        sim_tick: SimTick,
        emotional_intensity: f32,
    ) -> Self {
        MemoryEntry {
            id,
            event_id,
            npc_id,
            sim_tick,
            emotional_intensity: emotional_intensity.clamp(-1.0, 1.0),
            stat_deltas: Vec::new(),
            relationship_deltas: Vec::new(),
            tags: Vec::new(),
            participants: Vec::new(),
        }
    }

    /// Add stat deltas to this memory.
    pub fn with_stat_deltas(mut self, deltas: Vec<StatDelta>) -> Self {
        self.stat_deltas = deltas;
        self
    }

    pub fn with_relationship_deltas(mut self, deltas: Vec<RelationshipDelta>) -> Self {
        self.relationship_deltas = deltas;
        self
    }

    /// Add tags to categorize the memory.
    pub fn with_tags<T>(mut self, tags: Vec<T>) -> Self
    where
        T: Into<String>,
    {
        self.tags = tags.into_iter().map(|t| t.into()).collect();
        self
    }
}

/// A journal stores memories for an NPC, supporting queries for narrative triggers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Journal {
    pub npc_id: NpcId,
    pub entries: Vec<MemoryEntry>,
}

impl Journal {
    pub fn new(npc_id: NpcId) -> Self {
        Journal {
            npc_id,
            entries: Vec::new(),
        }
    }

    /// Add a memory entry to the journal.
    pub fn record(&mut self, entry: MemoryEntry) {
        self.entries.push(entry);
    }

    /// Retrieve memories with a specific tag.
    pub fn memories_with_tag(&self, tag: &str) -> Vec<&MemoryEntry> {
        self.entries
            .iter()
            .filter(|e| e.tags.contains(&tag.to_string()))
            .collect()
    }

    /// Retrieve memories within a time window (in ticks).
    pub fn memories_since(&self, since_tick: SimTick) -> Vec<&MemoryEntry> {
        self.entries
            .iter()
            .filter(|e| e.sim_tick.0 >= since_tick.0)
            .collect()
    }

    /// Find the most recent memory with a given tag.
    pub fn recent_memory_with_tag(&self, tag: &str) -> Option<&MemoryEntry> {
        self.entries
            .iter()
            .rev()
            .find(|e| e.tags.contains(&tag.to_string()))
    }

    /// Calculate aggregate emotional impact from recent memories (default: last 7 days).
    pub fn recent_emotional_aggregate(&self, current_tick: SimTick, days: u32) -> f32 {
        let tick_window = days as u64 * 24; // 24 ticks per day
        let since_tick = SimTick::new(current_tick.0.saturating_sub(tick_window));
        let recent = self.memories_since(since_tick);

        if recent.is_empty() {
            0.0
        } else {
            recent.iter().map(|m| m.emotional_intensity).sum::<f32>() / recent.len() as f32
        }
    }

    /// Count traumatic memories (highly negative, tagged "trauma").
    pub fn trauma_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| e.emotional_intensity < -0.7 && e.tags.contains(&"trauma".to_string()))
            .count()
    }

    /// Get all memories sorted by recency.
    pub fn timeline(&self) -> Vec<&MemoryEntry> {
        let mut sorted = self.entries.iter().collect::<Vec<_>>();
        sorted.sort_by(|a, b| b.sim_tick.0.cmp(&a.sim_tick.0));
        sorted
    }
}

/// Global memory store for all NPCs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySystem {
    pub journals: HashMap<NpcId, Journal>,
}

impl MemorySystem {
    pub fn new() -> Self {
        MemorySystem {
            journals: HashMap::new(),
        }
    }

    /// Archive a journal to cold storage (requires `storage` feature).
    ///
    /// This serializes the journal as JSON and stores it in the cold tier
    /// using the NPC's ID as the key. Useful for archiving memories of
    /// dormant NPCs to free up RAM.
    #[cfg(feature = "storage")]
    pub fn archive_journal(
        &self,
        npc_id: NpcId,
        storage: &HybridStorage,
    ) -> Result<(), StorageError> {
        if let Some(journal) = self.journals.get(&npc_id) {
            let json = serde_json::to_string(journal)
                .map_err(|e| StorageError::Unknown(format!("JSON serialization failed: {}", e)))?;
            
            // Store in cold tier using DuckDB journal archive
            storage.archive_journal(npc_id.0, &json)?;
        }
        Ok(())
    }

    /// Load an archived journal from cold storage (requires `storage` feature).
    ///
    /// Retrieves a serialized journal from the cold tier and deserializes it
    /// into memory. Returns None if no archived journal exists for this NPC.
    #[cfg(feature = "storage")]
    pub fn load_archived_journal(
        &mut self,
        npc_id: NpcId,
        storage: &HybridStorage,
    ) -> Result<Option<Journal>, StorageError> {
        if let Some(json_str) = storage.load_archived_journal(npc_id.0)? {
            let journal: Journal = serde_json::from_str(&json_str)
                .map_err(|e| StorageError::Unknown(format!("JSON deserialization failed: {}", e)))?;
            self.journals.insert(npc_id, journal.clone());
            Ok(Some(journal))
        } else {
            Ok(None)
        }
    }

    /// Prune old memories from a journal, keeping only recent ones.
    ///
    /// Archives the full journal before pruning if storage is provided.
    /// Keeps memories from the last `days_to_keep` days.
    #[cfg(feature = "storage")]
    pub fn prune_old_memories(
        &mut self,
        npc_id: NpcId,
        current_tick: SimTick,
        days_to_keep: u32,
        storage: Option<&HybridStorage>,
    ) -> Result<usize, StorageError> {
        // Archive before pruning if storage provided
        if let Some(store) = storage {
            self.archive_journal(npc_id, store)?;
        }

        let cutoff_tick = SimTick::new(
            current_tick.0.saturating_sub(days_to_keep as u64 * 24)
        );

        if let Some(journal) = self.journals.get_mut(&npc_id) {
            let original_count = journal.entries.len();
            journal.entries.retain(|e| e.sim_tick.0 >= cutoff_tick.0);
            let pruned = original_count - journal.entries.len();
            return Ok(pruned);
        }
        Ok(0)
    }

    /// Prune old memories (non-storage variant).
    pub fn prune_old_memories_no_archive(
        &mut self,
        npc_id: NpcId,
        current_tick: SimTick,
        days_to_keep: u32,
    ) -> usize {
        let cutoff_tick = SimTick::new(
            current_tick.0.saturating_sub(days_to_keep as u64 * 24)
        );

        if let Some(journal) = self.journals.get_mut(&npc_id) {
            let original_count = journal.entries.len();
            journal.entries.retain(|e| e.sim_tick.0 >= cutoff_tick.0);
            return original_count - journal.entries.len();
        }
        0
    }

    /// Get or create a journal for an NPC.
    pub fn get_or_create_journal(&mut self, npc_id: NpcId) -> &mut Journal {
        self.journals
            .entry(npc_id)
            .or_insert_with(|| Journal::new(npc_id))
    }

    /// Record a memory for an NPC.
    pub fn record_memory(&mut self, entry: MemoryEntry) {
        let journal = self.get_or_create_journal(entry.npc_id);
        journal.record(entry);
    }

    /// Query memories across all NPCs by event_id.
    pub fn memories_by_event(&self, event_id: &str) -> Vec<(&NpcId, &MemoryEntry)> {
        self.journals
            .iter()
            .flat_map(|(npc_id, journal)| {
                journal
                    .entries
                    .iter()
                    .filter(|e| e.event_id == event_id)
                    .map(move |e| (npc_id, e))
            })
            .collect()
    }

    /// Get a journal for an NPC.
    pub fn get_journal(&self, npc_id: NpcId) -> Option<&Journal> {
        self.journals.get(&npc_id)
    }

    /// Clear all memories (for new world generation).
    pub fn clear(&mut self) {
        self.journals.clear();
    }
}

pub fn memories_for_pair_with_tags<'a>(
    entries: &'a [MemoryEntry],
    actor_id: u64,
    target_id: u64,
    required_any_tags: &[&str],
) -> Vec<&'a MemoryEntry> {
    entries
        .iter()
        .filter(|m| {
            let has_actor = m.participants.contains(&actor_id);
            let has_target = m.participants.contains(&target_id);
            if !(has_actor && has_target) {
                return false;
            }

            if required_any_tags.is_empty() {
                return true;
            }

            let lower_tags: Vec<String> = m.tags.iter().map(|t| t.to_lowercase()).collect();
            required_any_tags
                .iter()
                .any(|t| lower_tags.contains(&t.to_lowercase()))
        })
        .collect()
}

impl Default for MemorySystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Build a frequency map of normalized tags for a given actor id.
pub fn tag_counts_for_actor(memories: &[MemoryEntry], actor_id: u64) -> HashMap<String, u32> {
    let mut counts = HashMap::new();

    for m in memories {
        if !m.participants.contains(&actor_id) {
            continue;
        }

        for tag in &m.tags {
            let norm = tag.to_lowercase();
            *counts.entry(norm).or_insert(0) += 1;
        }
    }

    counts
}

/// Helper: record a lightweight memory echo when an NPC exhibits a notable behavior toward the player.
pub fn add_npc_behavior_memory(
    memory: &mut MemorySystem,
    npc_id: u64,
    player_id: u64,
    behavior_kind: &BehaviorKind,
    tick: SimTick,
) {
    let mut tags = Vec::new();
    tags.push("npc_behavior".to_string());

    match behavior_kind {
        BehaviorKind::SeekSocial => tags.push("support".to_string()),
        BehaviorKind::SeekRecognition => tags.push("attention".to_string()),
        BehaviorKind::SeekAutonomy => tags.push("conflict".to_string()),
        BehaviorKind::SeekComfort => tags.push("withdrawal".to_string()),
        BehaviorKind::SeekSecurity => tags.push("stability".to_string()),
        BehaviorKind::Idle => {}
    }

    let id = format!("behav:{}:{}:{}", npc_id, player_id, tick.0);
    let mut entry = MemoryEntry::new(id, "npc_behavior".to_string(), NpcId(npc_id), tick, 0.0);
    entry.tags = tags;
    entry.participants = vec![npc_id, player_id];
    memory.record_memory(entry);
}

/// Expanded helper: record a behavior memory with explicit tags.
/// Non-breaking additive API; useful for NPC action execution layer.
pub fn add_npc_behavior_memory_with_tags(
    memory: &mut MemorySystem,
    npc_id: u64,
    player_id: u64,
    tags: Vec<String>,
    tick: SimTick,
) {
    let id = format!("behav_tags:{}:{}:{}", npc_id, player_id, tick.0);
    let mut entry = MemoryEntry::new(
        id,
        "npc_behavior_action".to_string(),
        NpcId(npc_id),
        tick,
        0.0,
    );
    entry.tags = tags;
    entry.participants = vec![npc_id, player_id];
    memory.record_memory(entry);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_entry_creation() {
        let entry = MemoryEntry::new(
            "mem_001".to_string(),
            "event_betrayal".to_string(),
            NpcId(1),
            SimTick(100),
            -0.8,
        )
        .with_tags(vec!["betrayal", "trust"]);

        assert_eq!(entry.emotional_intensity, -0.8);
        assert!(entry.tags.contains(&"betrayal".to_string()));
    }

    #[test]
    fn test_journal_record_and_query() {
        let mut journal = Journal::new(NpcId(1));
        let entry = MemoryEntry::new(
            "mem_001".to_string(),
            "event_1".to_string(),
            NpcId(1),
            SimTick(50),
            0.5,
        )
        .with_tags(vec!["positive"]);

        journal.record(entry);
        assert_eq!(journal.entries.len(), 1);
        assert_eq!(journal.memories_with_tag("positive").len(), 1);
    }

    #[test]
    fn test_memories_since() {
        let mut journal = Journal::new(NpcId(1));
        journal.record(MemoryEntry::new(
            "mem_001".to_string(),
            "event_1".to_string(),
            NpcId(1),
            SimTick(100),
            0.5,
        ));
        journal.record(MemoryEntry::new(
            "mem_002".to_string(),
            "event_2".to_string(),
            NpcId(1),
            SimTick(200),
            -0.3,
        ));

        let recent = journal.memories_since(SimTick(150));
        assert_eq!(recent.len(), 1);
    }

    #[test]
    fn test_emotional_aggregate() {
        let mut journal = Journal::new(NpcId(1));
        journal.record(MemoryEntry::new(
            "mem_001".to_string(),
            "event_1".to_string(),
            NpcId(1),
            SimTick(100),
            0.5,
        ));
        journal.record(MemoryEntry::new(
            "mem_002".to_string(),
            "event_2".to_string(),
            NpcId(1),
            SimTick(150),
            -0.3,
        ));

        let agg = journal.recent_emotional_aggregate(SimTick(200), 7);
        assert!(agg > -0.3 && agg < 0.5);
    }

    #[test]
    fn test_memory_system_record() {
        let mut memory_sys = MemorySystem::new();
        let entry = MemoryEntry::new(
            "mem_001".to_string(),
            "event_1".to_string(),
            NpcId(1),
            SimTick(100),
            0.8,
        );

        memory_sys.record_memory(entry);
        assert!(memory_sys.get_journal(NpcId(1)).is_some());
    }

    #[test]
    fn test_prune_old_memories_no_archive() {
        let mut memory_sys = MemorySystem::new();
        let npc_id = NpcId(1);

        // Add memories spanning 14 days
        for day in 0..14 {
            let entry = MemoryEntry::new(
                format!("mem_{}", day),
                "event_daily".to_string(),
                npc_id,
                SimTick(day * 24),
                0.5,
            );
            memory_sys.record_memory(entry);
        }

        let journal = memory_sys.get_journal(npc_id).unwrap();
        assert_eq!(journal.entries.len(), 14);

        // Prune to keep only last 7 days
        // Current is day 13 (tick 312), cutoff is 312 - 168 = 144
        // Days 0-5 (ticks 0, 24, 48, 72, 96, 120) are removed (< 144)
        // Days 6-13 (ticks 144, 168, ..., 312) are kept (>= 144)
        let pruned = memory_sys.prune_old_memories_no_archive(
            npc_id,
            SimTick(13 * 24), // current is day 13 at tick 312
            7,
        );

        assert_eq!(pruned, 6); // Removes days 0-5 (6 entries)
        let journal = memory_sys.get_journal(npc_id).unwrap();
        assert_eq!(journal.entries.len(), 8); // Keeps days 6-13 (8 entries)
    }
}
