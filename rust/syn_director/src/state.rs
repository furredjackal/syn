//! Consolidated director state module.
//!
//! This module contains all runtime state owned by the Event Director.
//! Having a single `DirectorState` struct ensures:
//! - Clear ownership of all mutable director-internal state
//! - Easy serialization/persistence of director state
//! - Clean separation from read-only storylet library and immutable config

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use syn_core::SimTick;
use syn_storylets::library::StoryletKey;
use syn_storylets::{StoryDomain, Tag};

// Re-export queue types for convenience
pub use crate::queue::{EventQueue, QueuedEvent, QueueSource};

/// The consolidated runtime state of the Event Director.
///
/// All evolving director-internal state lives here. This enables:
/// - Single point of persistence for director state
/// - Clear separation from read-only storylet library
/// - Easy state inspection and debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectorState {
    /// Current simulation tick.
    pub tick: SimTick,
    
    /// Current narrative intensity (0.0 = calm, 100.0 = crisis).
    /// Used for pacing decisions.
    pub narrative_heat: f32,
    
    /// Current phase of the narrative arc.
    pub narrative_phase: NarrativePhase,
    
    /// Queue of scheduled and pending storylets.
    pub pending_queue: EventQueue,
    
    /// Active pressure points from relationships/world state.
    pub active_pressures: PressureState,
    
    /// Milestone tracking for major narrative events.
    pub milestones: MilestoneState,
    
    /// Cooldown tracking for storylets, actors, and districts.
    pub cooldowns: CooldownState,
    
    /// Tracks when storylets/domains/tags last fired for variety.
    pub last_fired: LastFiredState,
    
    /// Tick when the current phase started (for min_phase_duration checks).
    pub phase_started_at: SimTick,
}

impl DirectorState {
    /// Create a new DirectorState with sensible defaults.
    pub fn new() -> Self {
        DirectorState {
            tick: SimTick::new(0),
            narrative_heat: 0.0,
            narrative_phase: NarrativePhase::LowKey,
            pending_queue: EventQueue::new(),
            active_pressures: PressureState::new(),
            milestones: MilestoneState::new(),
            cooldowns: CooldownState::new(),
            last_fired: LastFiredState::new(),
            phase_started_at: SimTick::new(0),
        }
    }

    /// Create a DirectorState starting at a specific tick.
    pub fn with_tick(tick: SimTick) -> Self {
        DirectorState {
            tick,
            phase_started_at: tick,
            ..Self::new()
        }
    }
    
    /// Clamp narrative heat to configured bounds.
    pub fn clamp_heat(&mut self, min_heat: f32, max_heat: f32) {
        self.narrative_heat = self.narrative_heat.clamp(min_heat, max_heat);
    }
    
    /// Update narrative phase based on current heat and thresholds.
    ///
    /// Phase transitions follow a state machine with hysteresis:
    /// - LowKey → Rising: when heat rises above lowkey_to_rising
    /// - Rising → Peak: when heat rises above rising_to_peak
    /// - Peak → Fallout: when heat drops below peak_to_fallout
    /// - Fallout → Recovery: when heat drops below fallout_to_recovery
    /// - Recovery → LowKey: when heat drops below recovery_to_lowkey
    ///
    /// Returns true if a phase transition occurred.
    pub fn update_phase(&mut self, thresholds: &crate::config::PhaseThresholds, min_phase_duration: u64) -> bool {
        let ticks_in_phase = self.tick.0.saturating_sub(self.phase_started_at.0);
        
        // Don't transition if we haven't been in current phase long enough
        if ticks_in_phase < min_phase_duration {
            return false;
        }
        
        let new_phase = match self.narrative_phase {
            NarrativePhase::LowKey => {
                if self.narrative_heat >= thresholds.lowkey_to_rising {
                    Some(NarrativePhase::Rising)
                } else {
                    None
                }
            }
            NarrativePhase::Rising => {
                if self.narrative_heat >= thresholds.rising_to_peak {
                    Some(NarrativePhase::Peak)
                } else if self.narrative_heat < thresholds.recovery_to_lowkey {
                    // Allow direct drop to LowKey if heat drops dramatically
                    Some(NarrativePhase::LowKey)
                } else {
                    None
                }
            }
            NarrativePhase::Peak => {
                if self.narrative_heat < thresholds.peak_to_fallout {
                    Some(NarrativePhase::Fallout)
                } else {
                    None
                }
            }
            NarrativePhase::Fallout => {
                if self.narrative_heat < thresholds.fallout_to_recovery {
                    Some(NarrativePhase::Recovery)
                } else if self.narrative_heat >= thresholds.rising_to_peak {
                    // Allow re-escalation to Peak if heat rises
                    Some(NarrativePhase::Peak)
                } else {
                    None
                }
            }
            NarrativePhase::Recovery => {
                if self.narrative_heat < thresholds.recovery_to_lowkey {
                    Some(NarrativePhase::LowKey)
                } else if self.narrative_heat >= thresholds.lowkey_to_rising {
                    // Allow escalation if heat rises during recovery
                    Some(NarrativePhase::Rising)
                } else {
                    None
                }
            }
        };
        
        if let Some(phase) = new_phase {
            self.narrative_phase = phase;
            self.phase_started_at = self.tick;
            true
        } else {
            false
        }
    }
    
    /// Get how many ticks we've been in the current phase.
    pub fn ticks_in_current_phase(&self) -> u64 {
        self.tick.0.saturating_sub(self.phase_started_at.0)
    }
}

/// Phase of the narrative arc for pacing decisions.
///
/// The director uses this to modulate event selection:
/// - LowKey: slice-of-life, building blocks, quiet moments
/// - Rising: tension building, foreshadowing, setup
/// - Peak: high-drama, crisis moments, confrontations
/// - Fallout: aftermath, consequences, emotional processing
/// - Recovery: healing, reconciliation, new equilibrium
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum NarrativePhase {
    /// Calm periods with slice-of-life events.
    #[default]
    LowKey,
    /// Tension is building toward a climax.
    Rising,
    /// High-drama peak moments.
    Peak,
    /// Aftermath and consequences.
    Fallout,
    /// Healing and establishing new equilibrium.
    Recovery,
}

/// Tracks when storylets, domains, and tags last fired.
///
/// Used for:
/// - Enforcing minimum intervals between similar content
/// - Ensuring narrative variety (avoid same domain back-to-back)
/// - Repetition detection and prevention
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LastFiredState {
    /// Last tick each storylet was fired.
    pub last_storylet_tick: HashMap<StoryletKey, SimTick>,
    
    /// Last tick each domain had a storylet fire.
    pub last_by_domain: HashMap<StoryDomain, SimTick>,
    
    /// Last tick each tag had a storylet fire.
    pub last_by_tag: HashMap<Tag, SimTick>,
}

impl LastFiredState {
    /// Create a new empty LastFiredState.
    pub fn new() -> Self {
        LastFiredState {
            last_storylet_tick: HashMap::new(),
            last_by_domain: HashMap::new(),
            last_by_tag: HashMap::new(),
        }
    }

    /// Record that a storylet was fired at the given tick.
    pub fn record_fired(
        &mut self,
        key: StoryletKey,
        domain: StoryDomain,
        tags: &[Tag],
        tick: SimTick,
    ) {
        self.last_storylet_tick.insert(key, tick);
        self.last_by_domain.insert(domain, tick);
        for tag in tags {
            self.last_by_tag.insert(tag.clone(), tick);
        }
    }

    /// Check if a storylet has been fired since a given tick.
    pub fn storylet_fired_since(&self, key: StoryletKey, since: SimTick) -> bool {
        self.last_storylet_tick
            .get(&key)
            .map(|&t| t.0 >= since.0)
            .unwrap_or(false)
    }

    /// Check if a domain has had a storylet fire since a given tick.
    pub fn domain_fired_since(&self, domain: StoryDomain, since: SimTick) -> bool {
        self.last_by_domain
            .get(&domain)
            .map(|&t| t.0 >= since.0)
            .unwrap_or(false)
    }

    /// Check if a tag has had a storylet fire since a given tick.
    pub fn tag_fired_since(&self, tag: &Tag, since: SimTick) -> bool {
        self.last_by_tag
            .get(tag)
            .map(|&t| t.0 >= since.0)
            .unwrap_or(false)
    }

    /// Get the last tick a storylet was fired, if ever.
    pub fn last_tick_for_storylet(&self, key: StoryletKey) -> Option<SimTick> {
        self.last_storylet_tick.get(&key).copied()
    }

    /// Get the last tick a domain had a storylet fire, if ever.
    pub fn last_tick_for_domain(&self, domain: StoryDomain) -> Option<SimTick> {
        self.last_by_domain.get(&domain).copied()
    }
}

// Re-export pressure and milestone types from the pressure module
pub use crate::pressure::{
    Pressure, PressureId, PressureKind, PressureState,
    Milestone, MilestoneId, MilestoneKind, MilestoneState,
};

// Serializable wrapper types for HashMap with tuple keys
mod cooldown_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::collections::HashMap;
    use syn_core::SimTick;
    use syn_storylets::library::StoryletKey;

    // Actor cooldown: (storylet_key, actor_id) as "key-actor"
    pub fn serialize_actor_cooldowns<S>(
        map: &HashMap<(StoryletKey, u64), SimTick>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let string_map: HashMap<String, u64> = map
            .iter()
            .map(|((k, a), t)| (format!("{}-{}", k.0, a), t.0))
            .collect();
        string_map.serialize(serializer)
    }

    pub fn deserialize_actor_cooldowns<'de, D>(
        deserializer: D,
    ) -> Result<HashMap<(StoryletKey, u64), SimTick>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string_map: HashMap<String, u64> = HashMap::deserialize(deserializer)?;
        let mut result = HashMap::new();
        for (key, tick) in string_map {
            let parts: Vec<&str> = key.split('-').collect();
            if parts.len() == 2 {
                if let (Ok(storylet), Ok(actor)) = (parts[0].parse::<u32>(), parts[1].parse::<u64>()) {
                    result.insert((StoryletKey(storylet), actor), SimTick::new(tick));
                }
            }
        }
        Ok(result)
    }

    // District cooldown: (storylet_key, district_id) as "key-district"
    pub fn serialize_district_cooldowns<S>(
        map: &HashMap<(StoryletKey, u64), SimTick>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let string_map: HashMap<String, u64> = map
            .iter()
            .map(|((k, d), t)| (format!("{}-{}", k.0, d), t.0))
            .collect();
        string_map.serialize(serializer)
    }

    pub fn deserialize_district_cooldowns<'de, D>(
        deserializer: D,
    ) -> Result<HashMap<(StoryletKey, u64), SimTick>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string_map: HashMap<String, u64> = HashMap::deserialize(deserializer)?;
        let mut result = HashMap::new();
        for (key, tick) in string_map {
            let parts: Vec<&str> = key.split('-').collect();
            if parts.len() == 2 {
                if let (Ok(storylet), Ok(district)) = (parts[0].parse::<u32>(), parts[1].parse::<u64>()) {
                    result.insert((StoryletKey(storylet), district), SimTick::new(tick));
                }
            }
        }
        Ok(result)
    }

    // Relationship cooldown: (storylet_key, actor_id, target_id) as "key-actor-target"
    pub fn serialize_relationship_cooldowns<S>(
        map: &HashMap<(StoryletKey, u64, u64), SimTick>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let string_map: HashMap<String, u64> = map
            .iter()
            .map(|((k, a, t), tick)| (format!("{}-{}-{}", k.0, a, t), tick.0))
            .collect();
        string_map.serialize(serializer)
    }

    pub fn deserialize_relationship_cooldowns<'de, D>(
        deserializer: D,
    ) -> Result<HashMap<(StoryletKey, u64, u64), SimTick>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string_map: HashMap<String, u64> = HashMap::deserialize(deserializer)?;
        let mut result = HashMap::new();
        for (key, tick) in string_map {
            let parts: Vec<&str> = key.split('-').collect();
            if parts.len() == 3 {
                if let (Ok(storylet), Ok(actor), Ok(target)) = 
                    (parts[0].parse::<u32>(), parts[1].parse::<u64>(), parts[2].parse::<u64>()) {
                    result.insert((StoryletKey(storylet), actor, target), SimTick::new(tick));
                }
            }
        }
        Ok(result)
    }
}

/// Cooldown tracking for storylets, actors, and districts.
///
/// Replaces the old `CooldownTracker` with a more comprehensive system:
/// - Global cooldowns (per-storylet)
/// - Per-actor cooldowns (same storylet with same NPC)
/// - Per-district cooldowns (same storylet in same location)
/// - Per-relationship cooldowns (same storylet with same pair)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CooldownState {
    /// Global cooldowns: storylet_key -> until_tick
    pub global_cooldowns: HashMap<StoryletKey, SimTick>,
    
    /// Per-actor cooldowns: (storylet_key, actor_id) -> until_tick
    #[serde(
        serialize_with = "cooldown_serde::serialize_actor_cooldowns",
        deserialize_with = "cooldown_serde::deserialize_actor_cooldowns"
    )]
    pub actor_cooldowns: HashMap<(StoryletKey, u64), SimTick>,
    
    /// Per-district cooldowns: (storylet_key, district_id) -> until_tick
    #[serde(
        serialize_with = "cooldown_serde::serialize_district_cooldowns",
        deserialize_with = "cooldown_serde::deserialize_district_cooldowns"
    )]
    pub district_cooldowns: HashMap<(StoryletKey, u64), SimTick>,
    
    /// Per-relationship cooldowns: (storylet_key, actor_id, target_id) -> until_tick
    #[serde(
        serialize_with = "cooldown_serde::serialize_relationship_cooldowns",
        deserialize_with = "cooldown_serde::deserialize_relationship_cooldowns"
    )]
    pub relationship_cooldowns: HashMap<(StoryletKey, u64, u64), SimTick>,
}

impl CooldownState {
    /// Create a new empty CooldownState.
    pub fn new() -> Self {
        CooldownState {
            global_cooldowns: HashMap::new(),
            actor_cooldowns: HashMap::new(),
            district_cooldowns: HashMap::new(),
            relationship_cooldowns: HashMap::new(),
        }
    }

    /// Check if a storylet is globally ready (not on cooldown).
    pub fn is_globally_ready(&self, key: StoryletKey, current_tick: SimTick) -> bool {
        self.global_cooldowns
            .get(&key)
            .map(|&until| current_tick.0 >= until.0)
            .unwrap_or(true)
    }

    /// Check if a storylet is ready for a specific actor.
    pub fn is_actor_ready(&self, key: StoryletKey, actor_id: u64, current_tick: SimTick) -> bool {
        self.actor_cooldowns
            .get(&(key, actor_id))
            .map(|&until| current_tick.0 >= until.0)
            .unwrap_or(true)
    }

    /// Check if a storylet is ready for a specific district.
    pub fn is_district_ready(&self, key: StoryletKey, district_id: u64, current_tick: SimTick) -> bool {
        self.district_cooldowns
            .get(&(key, district_id))
            .map(|&until| current_tick.0 >= until.0)
            .unwrap_or(true)
    }

    /// Check if a storylet is ready for a specific relationship pair.
    pub fn is_relationship_ready(
        &self,
        key: StoryletKey,
        actor_id: u64,
        target_id: u64,
        current_tick: SimTick,
    ) -> bool {
        // Check both orderings for symmetry
        let forward = self
            .relationship_cooldowns
            .get(&(key, actor_id, target_id))
            .map(|&until| current_tick.0 >= until.0)
            .unwrap_or(true);
        let reverse = self
            .relationship_cooldowns
            .get(&(key, target_id, actor_id))
            .map(|&until| current_tick.0 >= until.0)
            .unwrap_or(true);
        forward && reverse
    }

    /// Mark a global cooldown for a storylet.
    pub fn mark_global_cooldown(&mut self, key: StoryletKey, cooldown_ticks: u32, current_tick: SimTick) {
        let until = SimTick::new(current_tick.0 + cooldown_ticks as u64);
        self.global_cooldowns.insert(key, until);
    }

    /// Mark an actor-specific cooldown for a storylet.
    pub fn mark_actor_cooldown(
        &mut self,
        key: StoryletKey,
        actor_id: u64,
        cooldown_ticks: u32,
        current_tick: SimTick,
    ) {
        let until = SimTick::new(current_tick.0 + cooldown_ticks as u64);
        self.actor_cooldowns.insert((key, actor_id), until);
    }

    /// Mark a district-specific cooldown for a storylet.
    pub fn mark_district_cooldown(
        &mut self,
        key: StoryletKey,
        district_id: u64,
        cooldown_ticks: u32,
        current_tick: SimTick,
    ) {
        let until = SimTick::new(current_tick.0 + cooldown_ticks as u64);
        self.district_cooldowns.insert((key, district_id), until);
    }

    /// Mark a relationship-specific cooldown for a storylet.
    pub fn mark_relationship_cooldown(
        &mut self,
        key: StoryletKey,
        actor_id: u64,
        target_id: u64,
        cooldown_ticks: u32,
        current_tick: SimTick,
    ) {
        let until = SimTick::new(current_tick.0 + cooldown_ticks as u64);
        self.relationship_cooldowns.insert((key, actor_id, target_id), until);
    }

    /// Clear expired cooldowns to save memory.
    pub fn cleanup_expired(&mut self, current_tick: SimTick) {
        self.global_cooldowns.retain(|_, &mut until| until.0 > current_tick.0);
        self.actor_cooldowns.retain(|_, &mut until| until.0 > current_tick.0);
        self.district_cooldowns.retain(|_, &mut until| until.0 > current_tick.0);
        self.relationship_cooldowns.retain(|_, &mut until| until.0 > current_tick.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_director_state_default() {
        let state = DirectorState::new();
        assert_eq!(state.tick.0, 0);
        assert_eq!(state.narrative_heat, 0.0);
        assert_eq!(state.narrative_phase, NarrativePhase::LowKey);
        assert!(state.pending_queue.is_empty());
        assert!(!state.active_pressures.has_active_pressures());
        assert!(!state.milestones.has_milestones());
    }

    #[test]
    fn test_director_state_with_tick() {
        let state = DirectorState::with_tick(SimTick::new(100));
        assert_eq!(state.tick.0, 100);
        assert_eq!(state.narrative_phase, NarrativePhase::LowKey);
    }

    #[test]
    fn test_narrative_phase_default() {
        let phase = NarrativePhase::default();
        assert_eq!(phase, NarrativePhase::LowKey);
    }

    #[test]
    fn test_cooldown_state_global() {
        let mut cooldowns = CooldownState::new();
        let key = StoryletKey(1);
        let tick = SimTick::new(100);

        // Initially ready
        assert!(cooldowns.is_globally_ready(key, tick));

        // Mark cooldown for 10 ticks
        cooldowns.mark_global_cooldown(key, 10, tick);

        // Not ready at tick 105
        assert!(!cooldowns.is_globally_ready(key, SimTick::new(105)));

        // Ready at tick 110
        assert!(cooldowns.is_globally_ready(key, SimTick::new(110)));
    }

    #[test]
    fn test_cooldown_state_actor() {
        let mut cooldowns = CooldownState::new();
        let key = StoryletKey(1);
        let actor_id = 42;
        let tick = SimTick::new(100);

        cooldowns.mark_actor_cooldown(key, actor_id, 5, tick);

        // Not ready at tick 103
        assert!(!cooldowns.is_actor_ready(key, actor_id, SimTick::new(103)));

        // Ready at tick 105
        assert!(cooldowns.is_actor_ready(key, actor_id, SimTick::new(105)));

        // Different actor is ready
        assert!(cooldowns.is_actor_ready(key, 99, tick));
    }

    #[test]
    fn test_last_fired_state() {
        let mut last_fired = LastFiredState::new();
        let key = StoryletKey(1);
        let domain = StoryDomain::Romance;
        let tags = vec![Tag::new("flirt"), Tag::new("date")];
        let tick = SimTick::new(100);

        last_fired.record_fired(key, domain, &tags, tick);

        assert!(last_fired.storylet_fired_since(key, SimTick::new(50)));
        assert!(!last_fired.storylet_fired_since(key, SimTick::new(150)));
        assert!(last_fired.domain_fired_since(domain, SimTick::new(50)));
        assert_eq!(last_fired.last_tick_for_storylet(key), Some(tick));
    }

    #[test]
    fn test_cooldown_cleanup() {
        let mut cooldowns = CooldownState::new();
        let key1 = StoryletKey(1);
        let key2 = StoryletKey(2);

        cooldowns.mark_global_cooldown(key1, 10, SimTick::new(100)); // expires at 110
        cooldowns.mark_global_cooldown(key2, 50, SimTick::new(100)); // expires at 150

        assert_eq!(cooldowns.global_cooldowns.len(), 2);

        cooldowns.cleanup_expired(SimTick::new(120));

        // key1 should be removed, key2 should remain
        assert_eq!(cooldowns.global_cooldowns.len(), 1);
        assert!(!cooldowns.global_cooldowns.contains_key(&key1));
        assert!(cooldowns.global_cooldowns.contains_key(&key2));
    }
}
