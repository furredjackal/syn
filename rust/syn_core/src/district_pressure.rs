//! District Pressure System
//!
//! Tracks significant changes in district conditions (crime spikes, economic
//! crashes, etc.) to trigger narrative events. When a district metric crosses
//! a critical threshold, a pressure event is generated that storylets can react to.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

use crate::district::District;

/// Types of district pressure events.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DistrictEventKind {
    /// Crime crossed a critical threshold (high crime spike).
    CrimeSpike,
    /// Crime dropped significantly (crackdown success).
    CrimeDrop,
    /// Economy crashed below critical threshold.
    EconomicCrash,
    /// Economy boomed above prosperity threshold.
    EconomicBoom,
    /// Unemployment spiked above critical level.
    UnemploymentCrisis,
    /// Gang activity reached dangerous levels.
    GangTakeover,
    /// Community cohesion collapsed (riots, unrest).
    SocialUnrest,
    /// Gentrification pressure reached displacement levels.
    GentrificationDisplacement,
    /// Pollution reached hazardous levels.
    EnvironmentalCrisis,
}

impl DistrictEventKind {
    /// Get narrative tags associated with this event kind.
    pub fn tags(&self) -> &'static [&'static str] {
        match self {
            Self::CrimeSpike => &["crime", "danger", "violence"],
            Self::CrimeDrop => &["safety", "police", "peace"],
            Self::EconomicCrash => &["poverty", "unemployment", "crisis"],
            Self::EconomicBoom => &["prosperity", "opportunity", "wealth"],
            Self::UnemploymentCrisis => &["jobless", "poverty", "desperation"],
            Self::GangTakeover => &["gangs", "territory", "violence"],
            Self::SocialUnrest => &["riots", "protests", "community"],
            Self::GentrificationDisplacement => &["gentrification", "displacement", "housing"],
            Self::EnvironmentalCrisis => &["pollution", "health", "environment"],
        }
    }
}

/// Thresholds for district pressure events.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DistrictThresholds {
    /// Crime level that triggers CrimeSpike (default: 75.0).
    pub crime_spike: f32,
    /// Crime level below which CrimeDrop fires (default: 25.0).
    pub crime_drop: f32,
    /// Economy level below which EconomicCrash fires (default: 20.0).
    pub economic_crash: f32,
    /// Economy level above which EconomicBoom fires (default: 80.0).
    pub economic_boom: f32,
    /// Unemployment rate that triggers crisis (default: 0.25).
    pub unemployment_crisis: f32,
    /// Gang activity level for takeover (default: 70.0).
    pub gang_takeover: f32,
    /// Community cohesion below which unrest fires (default: 20.0).
    pub social_unrest: f32,
    /// Gentrification level for displacement (default: 0.8).
    pub gentrification_displacement: f32,
    /// Pollution level for environmental crisis (default: 80.0).
    pub environmental_crisis: f32,
}

impl Default for DistrictThresholds {
    fn default() -> Self {
        Self {
            crime_spike: 75.0,
            crime_drop: 25.0,
            economic_crash: 20.0,
            economic_boom: 80.0,
            unemployment_crisis: 0.25,
            gang_takeover: 70.0,
            social_unrest: 20.0,
            gentrification_displacement: 0.8,
            environmental_crisis: 80.0,
        }
    }
}

/// A district pressure event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DistrictPressureEvent {
    /// Which district generated the event.
    pub district_id: u32,
    /// Human-readable district name.
    pub district_name: String,
    /// Type of pressure event.
    pub kind: DistrictEventKind,
    /// The metric value that triggered the event.
    pub value: f32,
    /// Tick when the event occurred.
    pub tick: u64,
}

/// Snapshot of district metrics for change detection.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
struct DistrictSnapshot {
    /// Whether each threshold was previously crossed.
    crossed: HashMap<DistrictEventKind, bool>,
}

/// State for tracking district pressure events.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct DistrictPressureState {
    /// Configuration thresholds.
    #[serde(default)]
    pub thresholds: DistrictThresholds,

    /// Last known state for each district.
    #[serde(default)]
    last_state: HashMap<u32, DistrictSnapshot>,

    /// FIFO queue of pressure events.
    #[serde(default)]
    pub queue: VecDeque<DistrictPressureEvent>,
}

impl DistrictPressureState {
    /// Create with default thresholds.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with custom thresholds.
    pub fn with_thresholds(thresholds: DistrictThresholds) -> Self {
        Self {
            thresholds,
            ..Default::default()
        }
    }

    /// Update tracking for a district, generating events if thresholds crossed.
    pub fn update_for_district(&mut self, district: &District, current_tick: u64) {
        let id = district.id.0;
        let thresholds = self.thresholds.clone();
        
        // Get or create the snapshot for this district
        let snapshot = self.last_state.entry(id).or_default();
        
        // Collect events to generate (avoiding borrow issues)
        let mut events = Vec::new();

        // Check each threshold condition
        Self::check_threshold_static(
            &mut events,
            district,
            snapshot,
            DistrictEventKind::CrimeSpike,
            district.crime,
            thresholds.crime_spike,
            true, // Fire when above threshold
            current_tick,
        );

        Self::check_threshold_static(
            &mut events,
            district,
            snapshot,
            DistrictEventKind::CrimeDrop,
            district.crime,
            thresholds.crime_drop,
            false, // Fire when below threshold
            current_tick,
        );

        Self::check_threshold_static(
            &mut events,
            district,
            snapshot,
            DistrictEventKind::EconomicCrash,
            district.economy,
            thresholds.economic_crash,
            false,
            current_tick,
        );

        Self::check_threshold_static(
            &mut events,
            district,
            snapshot,
            DistrictEventKind::EconomicBoom,
            district.economy,
            thresholds.economic_boom,
            true,
            current_tick,
        );

        Self::check_threshold_static(
            &mut events,
            district,
            snapshot,
            DistrictEventKind::UnemploymentCrisis,
            district.unemployment,
            thresholds.unemployment_crisis,
            true,
            current_tick,
        );

        Self::check_threshold_static(
            &mut events,
            district,
            snapshot,
            DistrictEventKind::GangTakeover,
            district.gang_activity,
            thresholds.gang_takeover,
            true,
            current_tick,
        );

        Self::check_threshold_static(
            &mut events,
            district,
            snapshot,
            DistrictEventKind::SocialUnrest,
            district.community_cohesion,
            thresholds.social_unrest,
            false,
            current_tick,
        );

        Self::check_threshold_static(
            &mut events,
            district,
            snapshot,
            DistrictEventKind::GentrificationDisplacement,
            district.gentrification,
            thresholds.gentrification_displacement,
            true,
            current_tick,
        );

        Self::check_threshold_static(
            &mut events,
            district,
            snapshot,
            DistrictEventKind::EnvironmentalCrisis,
            district.pollution,
            thresholds.environmental_crisis,
            true,
            current_tick,
        );

        // Add collected events to queue
        for event in events {
            self.queue.push_back(event);
        }
    }

    /// Check a single threshold and generate event on transition (static version).
    fn check_threshold_static(
        events: &mut Vec<DistrictPressureEvent>,
        district: &District,
        snapshot: &mut DistrictSnapshot,
        kind: DistrictEventKind,
        value: f32,
        threshold: f32,
        fire_when_above: bool,
        tick: u64,
    ) {
        let is_crossed = if fire_when_above {
            value >= threshold
        } else {
            value <= threshold
        };

        let was_crossed = *snapshot.crossed.get(&kind).unwrap_or(&false);

        // Fire event only on transition (edge detection)
        if is_crossed && !was_crossed {
            events.push(DistrictPressureEvent {
                district_id: district.id.0,
                district_name: district.name.clone(),
                kind,
                value,
                tick,
            });
        }

        snapshot.crossed.insert(kind, is_crossed);
    }

    /// Pop the next pressure event from the queue.
    pub fn pop_next_event(&mut self) -> Option<DistrictPressureEvent> {
        self.queue.pop_front()
    }

    /// Peek at the next pressure event without removing it.
    pub fn peek_next_event(&self) -> Option<&DistrictPressureEvent> {
        self.queue.front()
    }

    /// Check for events matching a specific district.
    pub fn events_for_district(&self, district_id: u32) -> impl Iterator<Item = &DistrictPressureEvent> {
        self.queue.iter().filter(move |e| e.district_id == district_id)
    }

    /// Check for events matching a specific kind.
    pub fn events_of_kind(&self, kind: DistrictEventKind) -> impl Iterator<Item = &DistrictPressureEvent> {
        self.queue.iter().filter(move |e| e.kind == kind)
    }

    /// Decay the queue by removing old events and enforcing size limits.
    pub fn decay_queue(&mut self, current_tick: u64, max_age_ticks: u64, max_queue_size: usize) {
        // Remove events older than max_age_ticks
        self.queue.retain(|event| current_tick.saturating_sub(event.tick) <= max_age_ticks);

        // Enforce max queue size (drop oldest events)
        while self.queue.len() > max_queue_size {
            self.queue.pop_front();
        }
    }

    /// Check if there are any pending pressure events.
    pub fn has_pending_events(&self) -> bool {
        !self.queue.is_empty()
    }

    /// Get the number of pending pressure events.
    pub fn pending_count(&self) -> usize {
        self.queue.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::district::{DistrictId, DistrictType};

    fn test_district(id: u32, name: &str) -> District {
        District::new(DistrictId(id), name.to_string(), DistrictType::Downtown)
    }

    #[test]
    fn crime_spike_triggers_on_threshold_cross() {
        let mut state = DistrictPressureState::new();
        let mut district = test_district(1, "Downtown");

        // Start below threshold
        district.crime = 50.0;
        state.update_for_district(&district, 100);
        assert_eq!(state.pending_count(), 0);

        // Cross above threshold
        district.crime = 80.0;
        state.update_for_district(&district, 101);
        assert_eq!(state.pending_count(), 1);

        let event = state.pop_next_event().unwrap();
        assert_eq!(event.kind, DistrictEventKind::CrimeSpike);
        assert_eq!(event.district_name, "Downtown");
    }

    #[test]
    fn no_repeat_events_while_above_threshold() {
        let mut state = DistrictPressureState::new();
        let mut district = test_district(1, "Downtown");

        // Cross threshold
        district.crime = 80.0;
        state.update_for_district(&district, 100);
        assert_eq!(state.pending_count(), 1);
        state.pop_next_event();

        // Stay above threshold - no new event
        district.crime = 85.0;
        state.update_for_district(&district, 101);
        assert_eq!(state.pending_count(), 0);

        // Drop below and rise again - new event
        district.crime = 50.0;
        state.update_for_district(&district, 102);
        district.crime = 80.0;
        state.update_for_district(&district, 103);
        assert_eq!(state.pending_count(), 1);
    }

    #[test]
    fn economic_crash_triggers_on_low_economy() {
        let mut state = DistrictPressureState::new();
        let mut district = test_district(2, "Industrial");

        district.economy = 50.0;
        state.update_for_district(&district, 100);

        district.economy = 15.0;
        state.update_for_district(&district, 101);

        let event = state.pop_next_event().unwrap();
        assert_eq!(event.kind, DistrictEventKind::EconomicCrash);
    }

    #[test]
    fn decay_removes_old_events() {
        let mut state = DistrictPressureState::new();
        let mut district = test_district(1, "Test");

        district.crime = 80.0;
        state.update_for_district(&district, 100);
        assert_eq!(state.pending_count(), 1);

        // Decay with current_tick far in future
        state.decay_queue(300, 168, 10);
        assert_eq!(state.pending_count(), 0);
    }

    #[test]
    fn multiple_districts_tracked_independently() {
        let mut state = DistrictPressureState::new();

        let mut d1 = test_district(1, "Downtown");
        let mut d2 = test_district(2, "Suburbs");

        d1.crime = 80.0;
        d2.economy = 15.0;

        state.update_for_district(&d1, 100);
        state.update_for_district(&d2, 100);

        assert_eq!(state.pending_count(), 2);

        let events: Vec<_> = state.queue.iter().map(|e| e.kind).collect();
        assert!(events.contains(&DistrictEventKind::CrimeSpike));
        assert!(events.contains(&DistrictEventKind::EconomicCrash));
    }
}
