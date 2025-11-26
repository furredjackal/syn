//! Director state persistence module.
//!
//! This module provides serialization and deserialization for `DirectorState`,
//! enabling save/load across game sessions while maintaining determinism.
//!
//! # Format
//!
//! Director state is serialized using bincode for efficiency, wrapped with
//! a version header for future-proofing and migration support.
//!
//! # Determinism Guarantee
//!
//! Same world seed + same saved director state + same world state = same narrative behavior.
//! All RNG in the director uses deterministic seeds derived from world state, so
//! restoring the same director state with the same world produces identical results.
//!
//! # Example
//!
//! ```ignore
//! // Create a snapshot
//! let snapshot = director.snapshot();
//!
//! // Serialize to bytes
//! let bytes = serialize_snapshot(&snapshot)?;
//!
//! // Later, restore from bytes
//! let restored_snapshot = deserialize_snapshot(&bytes)?;
//! let director = CompiledEventDirector::restore_from_snapshot(storylets, config, restored_snapshot);
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::state::DirectorState;

/// Current version of the director persistence format.
///
/// Increment this when making breaking changes to DirectorState serialization.
/// The deserialization code can check this to handle migrations.
pub const CURRENT_FORMAT_VERSION: u32 = 1;

/// Magic bytes identifying a director snapshot file.
/// "SYND" for "SYN Director"
pub const SNAPSHOT_MAGIC: [u8; 4] = [0x53, 0x59, 0x4E, 0x44]; // "SYND"

/// A snapshot of the director state for persistence.
///
/// This struct wraps the full `DirectorState` with version information
/// to support future format migrations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectorSnapshot {
    /// The full director state.
    pub state: DirectorState,
    
    /// Format version for migration support.
    pub format_version: u32,
    
    /// Optional config version if we need to track config compatibility.
    pub config_version: Option<u32>,
}

impl DirectorSnapshot {
    /// Create a new snapshot from director state.
    pub fn new(state: DirectorState) -> Self {
        DirectorSnapshot {
            state,
            format_version: CURRENT_FORMAT_VERSION,
            config_version: None,
        }
    }

    /// Create a snapshot with explicit config version.
    pub fn with_config_version(state: DirectorState, config_version: u32) -> Self {
        DirectorSnapshot {
            state,
            format_version: CURRENT_FORMAT_VERSION,
            config_version: Some(config_version),
        }
    }
}

/// Errors that can occur during director persistence operations.
#[derive(Debug)]
pub enum DirectorPersistError {
    /// Serialization failed.
    SerializationError(String),
    
    /// Deserialization failed.
    DeserializationError(String),
    
    /// Invalid magic bytes in snapshot header.
    InvalidMagic,
    
    /// Unsupported format version.
    UnsupportedVersion {
        found: u32,
        expected: u32,
    },
    
    /// Data is too short to be a valid snapshot.
    TruncatedData,
    
    /// JSON serialization/deserialization error.
    JsonError(String),
}

impl fmt::Display for DirectorPersistError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DirectorPersistError::SerializationError(msg) => {
                write!(f, "Serialization error: {}", msg)
            }
            DirectorPersistError::DeserializationError(msg) => {
                write!(f, "Deserialization error: {}", msg)
            }
            DirectorPersistError::InvalidMagic => {
                write!(f, "Invalid snapshot magic bytes")
            }
            DirectorPersistError::UnsupportedVersion { found, expected } => {
                write!(
                    f,
                    "Unsupported format version: found {}, expected {}",
                    found, expected
                )
            }
            DirectorPersistError::TruncatedData => {
                write!(f, "Truncated snapshot data")
            }
            DirectorPersistError::JsonError(msg) => {
                write!(f, "JSON error: {}", msg)
            }
        }
    }
}

impl std::error::Error for DirectorPersistError {}

/// Serialize a director snapshot to bytes.
///
/// Format: [MAGIC:4][VERSION:4][JSON_DATA:...]
///
/// Uses JSON for human-readability and easier debugging during development.
/// Can be changed to bincode later for production if needed.
pub fn serialize_snapshot(snapshot: &DirectorSnapshot) -> Result<Vec<u8>, DirectorPersistError> {
    let json_data = serde_json::to_vec(snapshot).map_err(|e| {
        DirectorPersistError::SerializationError(e.to_string())
    })?;
    
    let mut bytes = Vec::with_capacity(8 + json_data.len());
    
    // Write magic bytes
    bytes.extend_from_slice(&SNAPSHOT_MAGIC);
    
    // Write version as little-endian u32
    bytes.extend_from_slice(&CURRENT_FORMAT_VERSION.to_le_bytes());
    
    // Write JSON data
    bytes.extend_from_slice(&json_data);
    
    Ok(bytes)
}

/// Deserialize a director snapshot from bytes.
///
/// Validates magic bytes and version before deserializing.
pub fn deserialize_snapshot(bytes: &[u8]) -> Result<DirectorSnapshot, DirectorPersistError> {
    // Minimum size: magic (4) + version (4) + at least some JSON
    if bytes.len() < 10 {
        return Err(DirectorPersistError::TruncatedData);
    }
    
    // Validate magic bytes
    if bytes[0..4] != SNAPSHOT_MAGIC {
        return Err(DirectorPersistError::InvalidMagic);
    }
    
    // Read version
    let version = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    
    // Currently only support version 1
    if version != CURRENT_FORMAT_VERSION {
        return Err(DirectorPersistError::UnsupportedVersion {
            found: version,
            expected: CURRENT_FORMAT_VERSION,
        });
    }
    
    // Deserialize JSON data
    let json_data = &bytes[8..];
    let snapshot: DirectorSnapshot = serde_json::from_slice(json_data).map_err(|e| {
        DirectorPersistError::DeserializationError(e.to_string())
    })?;
    
    Ok(snapshot)
}

/// Serialize a director snapshot to a JSON string (for debugging or alternative storage).
pub fn serialize_to_json(snapshot: &DirectorSnapshot) -> Result<String, DirectorPersistError> {
    serde_json::to_string_pretty(snapshot).map_err(|e| {
        DirectorPersistError::JsonError(e.to_string())
    })
}

/// Deserialize a director snapshot from a JSON string.
pub fn deserialize_from_json(json: &str) -> Result<DirectorSnapshot, DirectorPersistError> {
    serde_json::from_str(json).map_err(|e| {
        DirectorPersistError::JsonError(e.to_string())
    })
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pressure::{
        Milestone, MilestoneId, MilestoneKind, MilestoneState,
        Pressure, PressureId, PressureKind, PressureState,
    };
    use crate::queue::{EventQueue, QueueSource, QueuedEvent};
    use crate::state::{DirectorState, NarrativePhase};
    use syn_core::SimTick;
    use syn_storylets::library::StoryletKey;
    use syn_storylets::{StoryDomain, Tag};

    fn create_complex_state() -> DirectorState {
        let mut state = DirectorState::new();
        
        // Set non-default values
        state.tick = SimTick::new(100);
        state.narrative_heat = 45.5;
        state.narrative_phase = NarrativePhase::Rising;
        state.phase_started_at = SimTick::new(80);
        
        // Add queued events
        state.pending_queue.push_unchecked(QueuedEvent::new(
            StoryletKey(42),
            SimTick::new(105),
            10,
            false,
            QueueSource::FollowUp,
        ));
        state.pending_queue.push_unchecked(QueuedEvent::new(
            StoryletKey(123),
            SimTick::new(110),
            5,
            true,
            QueueSource::PressureRelief,
        ));
        
        // Add pressures
        let pressure = Pressure::new(
            PressureId(1),
            PressureKind::Financial,
            SimTick::new(50),
            "Rent due soon".to_string(),
        )
        .with_deadline(SimTick::new(120))
        .with_severity(0.7);
        state.active_pressures.add_pressure(pressure);
        
        // Add milestones
        let milestone = Milestone::new(
            MilestoneId(1),
            MilestoneKind::RomanceArc,
            SimTick::new(20),
            "Confess feelings".to_string(),
        )
        .with_progress(0.6)
        .with_advancing_tags(vec![Tag::new("romance"), Tag::new("confession")]);
        state.milestones.add_milestone(milestone);
        
        // Add cooldowns
        state.cooldowns.global_cooldowns.insert(StoryletKey(10), SimTick::new(150));
        state.cooldowns.actor_cooldowns.insert((StoryletKey(20), 5), SimTick::new(180));
        
        // Add last fired tracking
        state.last_fired.record_fired(
            StoryletKey(99),
            StoryDomain::Romance,
            &[Tag::new("romance"), Tag::new("drama")],
            SimTick::new(95),
        );
        
        state
    }

    #[test]
    fn test_snapshot_roundtrip() {
        let original_state = create_complex_state();
        let snapshot = DirectorSnapshot::new(original_state.clone());
        
        // Serialize
        let bytes = serialize_snapshot(&snapshot).expect("Serialization should succeed");
        
        // Check magic bytes
        assert_eq!(&bytes[0..4], &SNAPSHOT_MAGIC);
        
        // Check version
        let version = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        assert_eq!(version, CURRENT_FORMAT_VERSION);
        
        // Deserialize
        let restored = deserialize_snapshot(&bytes).expect("Deserialization should succeed");
        
        // Verify state matches
        assert_eq!(restored.state.tick, original_state.tick);
        assert_eq!(restored.state.narrative_heat, original_state.narrative_heat);
        assert_eq!(restored.state.narrative_phase, original_state.narrative_phase);
        assert_eq!(restored.state.phase_started_at, original_state.phase_started_at);
        
        // Verify queue
        assert_eq!(
            restored.state.pending_queue.len(),
            original_state.pending_queue.len()
        );
        
        // Verify pressures
        assert_eq!(
            restored.state.active_pressures.active_count(),
            original_state.active_pressures.active_count()
        );
        
        // Verify milestones
        assert_eq!(
            restored.state.milestones.active_count(),
            original_state.milestones.active_count()
        );
        
        // Verify cooldowns
        assert_eq!(
            restored.state.cooldowns.global_cooldowns.len(),
            original_state.cooldowns.global_cooldowns.len()
        );
    }

    #[test]
    fn test_json_roundtrip() {
        let original_state = create_complex_state();
        let snapshot = DirectorSnapshot::new(original_state.clone());
        
        // Serialize to JSON
        let json = serialize_to_json(&snapshot).expect("JSON serialization should succeed");
        
        // Should be valid JSON
        assert!(json.starts_with('{'));
        assert!(json.contains("\"tick\""));
        assert!(json.contains("\"narrative_heat\""));
        
        // Deserialize
        let restored = deserialize_from_json(&json).expect("JSON deserialization should succeed");
        
        // Verify
        assert_eq!(restored.state.tick, original_state.tick);
        assert_eq!(restored.state.narrative_heat, original_state.narrative_heat);
    }

    #[test]
    fn test_invalid_magic() {
        let mut bytes = vec![0x00, 0x00, 0x00, 0x00]; // Wrong magic
        bytes.extend_from_slice(&1u32.to_le_bytes());
        bytes.extend_from_slice(b"{}");
        
        let result = deserialize_snapshot(&bytes);
        assert!(matches!(result, Err(DirectorPersistError::InvalidMagic)));
    }

    #[test]
    fn test_unsupported_version() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&SNAPSHOT_MAGIC);
        bytes.extend_from_slice(&999u32.to_le_bytes()); // Future version
        bytes.extend_from_slice(b"{}");
        
        let result = deserialize_snapshot(&bytes);
        assert!(matches!(
            result,
            Err(DirectorPersistError::UnsupportedVersion { found: 999, .. })
        ));
    }

    #[test]
    fn test_truncated_data() {
        let bytes = vec![0x53, 0x59]; // Only 2 bytes
        
        let result = deserialize_snapshot(&bytes);
        assert!(matches!(result, Err(DirectorPersistError::TruncatedData)));
    }

    #[test]
    fn test_snapshot_with_config_version() {
        let state = DirectorState::new();
        let snapshot = DirectorSnapshot::with_config_version(state, 42);
        
        assert_eq!(snapshot.config_version, Some(42));
        assert_eq!(snapshot.format_version, CURRENT_FORMAT_VERSION);
        
        // Roundtrip
        let bytes = serialize_snapshot(&snapshot).unwrap();
        let restored = deserialize_snapshot(&bytes).unwrap();
        
        assert_eq!(restored.config_version, Some(42));
    }

    #[test]
    fn test_empty_state_roundtrip() {
        let state = DirectorState::new();
        let snapshot = DirectorSnapshot::new(state);
        
        let bytes = serialize_snapshot(&snapshot).unwrap();
        let restored = deserialize_snapshot(&bytes).unwrap();
        
        assert_eq!(restored.state.tick.0, 0);
        assert_eq!(restored.state.narrative_heat, 0.0);
        assert_eq!(restored.state.narrative_phase, NarrativePhase::LowKey);
        assert_eq!(restored.state.pending_queue.len(), 0);
    }

    #[test]
    fn test_state_equality_after_roundtrip() {
        let original = create_complex_state();
        let snapshot = DirectorSnapshot::new(original.clone());
        
        let bytes = serialize_snapshot(&snapshot).unwrap();
        let restored = deserialize_snapshot(&bytes).unwrap();
        
        // Deep equality checks
        assert_eq!(restored.state.tick, original.tick);
        assert_eq!(restored.state.narrative_heat, original.narrative_heat);
        assert_eq!(restored.state.narrative_phase, original.narrative_phase);
        assert_eq!(restored.state.phase_started_at, original.phase_started_at);
        
        // Check queue events individually
        let original_events = original.pending_queue.all_events();
        let restored_events = restored.state.pending_queue.all_events();
        assert_eq!(original_events.len(), restored_events.len());
        for (o, r) in original_events.iter().zip(restored_events.iter()) {
            assert_eq!(o.storylet_key, r.storylet_key);
            assert_eq!(o.scheduled_tick, r.scheduled_tick);
            assert_eq!(o.priority, r.priority);
            assert_eq!(o.forced, r.forced);
            assert_eq!(o.source, r.source);
        }
        
        // Check cooldowns
        assert_eq!(
            restored.state.cooldowns.global_cooldowns,
            original.cooldowns.global_cooldowns
        );
        
        // Check last fired domains
        assert_eq!(
            restored.state.last_fired.last_by_domain.len(),
            original.last_fired.last_by_domain.len()
        );
    }
}
