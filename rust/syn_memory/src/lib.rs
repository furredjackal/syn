//! syn_memory: Memory entries and journal system for SYN.
//!
//! Records player choices, event outcomes, and emotional impacts.
//! Memories are used by the Event Director to trigger echos and narrative chains.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
pub use syn_core::{NpcId, SimTick, StatDelta};

/// A single memory entry recording an event and its impact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,                              // Unique memory ID
    pub event_id: String,                        // Which storylet fired
    pub npc_id: NpcId,                           // Who holds this memory
    pub sim_tick: SimTick,                       // When it happened
    pub emotional_intensity: f32,                // -1.0 (negative) to +1.0 (positive)
    pub stat_impacts: Vec<StatDelta>,     // e.g., [{"kind": Mood, "delta": -2.0}]
    pub tags: Vec<String>,                      // e.g., ["betrayal", "trauma", "relationship"]
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
            stat_impacts: Vec::new(),
            tags: Vec::new(),
        }
    }

    /// Add stat impacts to this memory.
    pub fn with_stat_deltas(mut self, deltas: Vec<StatDelta>) -> Self {
        self.stat_impacts = deltas;
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
        self.entries.iter().filter(|e| e.tags.contains(&tag.to_string())).collect()
    }

    /// Retrieve memories within a time window (in ticks).
    pub fn memories_since(&self, since_tick: SimTick) -> Vec<&MemoryEntry> {
        self.entries.iter().filter(|e| e.sim_tick.0 >= since_tick.0).collect()
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

    /// Get or create a journal for an NPC.
    pub fn get_or_create_journal(&mut self, npc_id: NpcId) -> &mut Journal {
        self.journals.entry(npc_id).or_insert_with(|| Journal::new(npc_id))
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

impl Default for MemorySystem {
    fn default() -> Self {
        Self::new()
    }
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
}
