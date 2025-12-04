//! Gossip-to-Pressure bridge.
//!
//! Converts gossip events (rumor spread, reputation damage) into pressure events
//! that the director can use for storylet eligibility and scoring.

use crate::gossip::SpreadResult;
use crate::types::NpcId;
use serde::{Deserialize, Serialize};

/// Types of gossip-related pressure events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GossipEventKind {
    /// A negative rumor about the player is spreading
    NegativeRumorSpreading,
    /// Player's reputation has dropped significantly
    ReputationDamaged,
    /// Someone the player trusts is spreading rumors about them
    BetrayalGossip,
    /// A scandalous rumor about the player is circulating
    ScandalCirculating,
    /// The player is being excluded from social circles
    SocialExclusion,
    /// Positive buzz: a good rumor is spreading
    PositiveBuzz,
}

impl GossipEventKind {
    /// Get tags for storylet matching.
    pub fn tags(&self) -> &'static [&'static str] {
        match self {
            Self::NegativeRumorSpreading => &["gossip", "rumor", "reputation", "social"],
            Self::ReputationDamaged => &["reputation", "social", "conflict"],
            Self::BetrayalGossip => &["betrayal", "trust", "gossip", "relationship"],
            Self::ScandalCirculating => &["scandal", "gossip", "reputation", "drama"],
            Self::SocialExclusion => &["isolation", "social", "exclusion", "loneliness"],
            Self::PositiveBuzz => &["gossip", "positive", "reputation", "social"],
        }
    }

    /// Heat bonus for this event type.
    pub fn heat_bonus(&self) -> f32 {
        match self {
            Self::NegativeRumorSpreading => 3.0,
            Self::ReputationDamaged => 5.0,
            Self::BetrayalGossip => 8.0,
            Self::ScandalCirculating => 10.0,
            Self::SocialExclusion => 4.0,
            Self::PositiveBuzz => 1.0,
        }
    }
}

/// A gossip-related pressure event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipPressureEvent {
    /// Type of gossip event.
    pub kind: GossipEventKind,
    /// The NPC who is the subject of the gossip (usually player).
    pub subject_id: NpcId,
    /// The NPC who spread the rumor (if applicable).
    pub spreader_id: Option<NpcId>,
    /// Tick when this event was detected.
    pub detected_tick: u64,
    /// Severity (0.0..1.0) based on rumor valence and belief.
    pub severity: f32,
}

/// Tracks gossip pressure events and thresholds.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GossipPressureState {
    /// Active gossip pressure events.
    pub events: Vec<GossipPressureEvent>,
    /// Cumulative negative gossip about player (for threshold detection).
    cumulative_negative: f32,
    /// Cumulative positive gossip about player.
    cumulative_positive: f32,
    /// Last tick we checked for threshold crossing.
    last_check_tick: u64,
    /// Whether we've already fired reputation_damaged this cycle.
    reputation_damaged_fired: bool,
    /// Whether we've already fired scandal_circulating this cycle.
    scandal_fired: bool,
}

impl GossipPressureState {
    /// Create a new gossip pressure state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Process spread results and generate pressure events.
    /// Returns any new events that were generated.
    pub fn process_spread_results(
        &mut self,
        results: &[SpreadResult],
        player_id: NpcId,
        rumor_valences: &std::collections::HashMap<String, f32>,
        rumor_scandalous: &std::collections::HashMap<String, bool>,
        current_tick: u64,
    ) -> Vec<GossipPressureEvent> {
        let mut new_events = Vec::new();

        for result in results {
            if !result.accepted {
                continue;
            }

            // Look up rumor info
            let valence = rumor_valences.get(&result.rumor_id).copied().unwrap_or(0.0);
            let is_scandalous = rumor_scandalous.get(&result.rumor_id).copied().unwrap_or(false);
            let subject_id = result.subject_id;

            // Only track gossip about the player for pressure events
            if subject_id != player_id {
                continue;
            }

            let severity = (result.belief * valence.abs()).clamp(0.0, 1.0);

            if valence < -0.3 {
                // Negative rumor spreading
                self.cumulative_negative += severity;

                let event = GossipPressureEvent {
                    kind: GossipEventKind::NegativeRumorSpreading,
                    subject_id,
                    spreader_id: Some(result.spreader_id),
                    detected_tick: current_tick,
                    severity,
                };
                new_events.push(event.clone());
                self.events.push(event);

                // Check for scandal
                if is_scandalous && !self.scandal_fired && severity > 0.5 {
                    let scandal_event = GossipPressureEvent {
                        kind: GossipEventKind::ScandalCirculating,
                        subject_id,
                        spreader_id: Some(result.spreader_id),
                        detected_tick: current_tick,
                        severity: severity * 1.5,
                    };
                    new_events.push(scandal_event.clone());
                    self.events.push(scandal_event);
                    self.scandal_fired = true;
                }
            } else if valence > 0.3 {
                // Positive buzz
                self.cumulative_positive += severity;

                if severity > 0.4 {
                    let event = GossipPressureEvent {
                        kind: GossipEventKind::PositiveBuzz,
                        subject_id,
                        spreader_id: Some(result.spreader_id),
                        detected_tick: current_tick,
                        severity,
                    };
                    new_events.push(event.clone());
                    self.events.push(event);
                }
            }
        }

        // Check cumulative thresholds
        if self.cumulative_negative > 3.0 && !self.reputation_damaged_fired {
            let event = GossipPressureEvent {
                kind: GossipEventKind::ReputationDamaged,
                subject_id: player_id,
                spreader_id: None,
                detected_tick: current_tick,
                severity: (self.cumulative_negative / 5.0).clamp(0.5, 1.0),
            };
            new_events.push(event.clone());
            self.events.push(event);
            self.reputation_damaged_fired = true;
        }

        self.last_check_tick = current_tick;
        new_events
    }

    /// Decay old events and reset thresholds periodically.
    pub fn decay(&mut self, current_tick: u64, ttl_ticks: u64, max_events: usize) {
        // Remove expired events
        self.events.retain(|e| current_tick.saturating_sub(e.detected_tick) < ttl_ticks);

        // Cap event count
        if self.events.len() > max_events {
            let excess = self.events.len() - max_events;
            self.events.drain(0..excess);
        }

        // Decay cumulative values over time
        self.cumulative_negative *= 0.95;
        self.cumulative_positive *= 0.95;

        // Reset threshold flags if cumulative drops low enough
        if self.cumulative_negative < 1.0 {
            self.reputation_damaged_fired = false;
        }
        if self.cumulative_negative < 2.0 {
            self.scandal_fired = false;
        }
    }

    /// Get all active events of a specific kind.
    pub fn events_of_kind(&self, kind: GossipEventKind) -> Vec<&GossipPressureEvent> {
        self.events.iter().filter(|e| e.kind == kind).collect()
    }

    /// Check if any event matches the given tags.
    pub fn has_event_with_tags(&self, tags: &[&str]) -> bool {
        self.events.iter().any(|e| {
            let event_tags = e.kind.tags();
            tags.iter().any(|t| event_tags.contains(t))
        })
    }

    /// Get the highest severity event.
    pub fn highest_severity(&self) -> Option<&GossipPressureEvent> {
        self.events.iter().max_by(|a, b| a.severity.partial_cmp(&b.severity).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_spread_result(
        rumor_id: &str,
        subject_id: u64,
        spreader_id: u64,
        belief: f32,
    ) -> SpreadResult {
        SpreadResult {
            rumor_id: rumor_id.to_string(),
            subject_id: NpcId(subject_id),
            recipient_id: NpcId(100), // Recipient doesn't matter for pressure
            spreader_id: NpcId(spreader_id),
            accepted: true,
            belief,
            distortion: 0.1,
            will_spread: false,
        }
    }

    #[test]
    fn negative_rumor_generates_pressure() {
        let mut state = GossipPressureState::new();
        let player_id = NpcId(1);
        
        let results = vec![make_spread_result("rumor1", 1, 2, 0.8)];
        let mut valences = std::collections::HashMap::new();
        valences.insert("rumor1".to_string(), -0.7); // Negative valence
        let scandalous = std::collections::HashMap::new();

        let events = state.process_spread_results(&results, player_id, &valences, &scandalous, 100);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, GossipEventKind::NegativeRumorSpreading);
        assert!(events[0].severity > 0.0);
    }

    #[test]
    fn scandal_triggers_on_high_severity_scandalous_rumor() {
        let mut state = GossipPressureState::new();
        let player_id = NpcId(1);
        
        let results = vec![make_spread_result("scandal1", 1, 2, 0.9)];
        let mut valences = std::collections::HashMap::new();
        valences.insert("scandal1".to_string(), -0.8);
        let mut scandalous = std::collections::HashMap::new();
        scandalous.insert("scandal1".to_string(), true);

        let events = state.process_spread_results(&results, player_id, &valences, &scandalous, 100);

        // Should have both NegativeRumorSpreading and ScandalCirculating
        assert!(events.iter().any(|e| e.kind == GossipEventKind::NegativeRumorSpreading));
        assert!(events.iter().any(|e| e.kind == GossipEventKind::ScandalCirculating));
    }

    #[test]
    fn cumulative_damage_triggers_reputation_event() {
        let mut state = GossipPressureState::new();
        let player_id = NpcId(1);
        
        let mut valences = std::collections::HashMap::new();
        valences.insert("r1".to_string(), -0.6);
        valences.insert("r2".to_string(), -0.7);
        valences.insert("r3".to_string(), -0.8);
        let scandalous = std::collections::HashMap::new();

        // Process multiple negative rumors
        for (i, rumor_id) in ["r1", "r2", "r3"].iter().enumerate() {
            let results = vec![make_spread_result(rumor_id, 1, 2, 0.9)];
            state.process_spread_results(&results, player_id, &valences, &scandalous, 100 + i as u64);
        }

        // After enough cumulative damage, should have ReputationDamaged
        assert!(state.events.iter().any(|e| e.kind == GossipEventKind::ReputationDamaged));
    }

    #[test]
    fn positive_buzz_generates_event() {
        let mut state = GossipPressureState::new();
        let player_id = NpcId(1);
        
        let results = vec![make_spread_result("good_news", 1, 2, 0.8)];
        let mut valences = std::collections::HashMap::new();
        valences.insert("good_news".to_string(), 0.7); // Positive valence
        let scandalous = std::collections::HashMap::new();

        let events = state.process_spread_results(&results, player_id, &valences, &scandalous, 100);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, GossipEventKind::PositiveBuzz);
    }

    #[test]
    fn decay_removes_old_events() {
        let mut state = GossipPressureState::new();
        state.events.push(GossipPressureEvent {
            kind: GossipEventKind::NegativeRumorSpreading,
            subject_id: NpcId(1),
            spreader_id: Some(NpcId(2)),
            detected_tick: 100,
            severity: 0.5,
        });

        // Decay with TTL of 50 ticks, current tick 200
        state.decay(200, 50, 10);

        assert!(state.events.is_empty());
    }
}
