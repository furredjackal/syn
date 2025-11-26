//! Pressure and milestone management for the director.
//!
//! This module provides:
//! - **Pressures**: Ticking constraints that push the narrative (rent due, exam tomorrow, etc.)
//! - **Milestones**: Longer-term goals/arcs that storylets can advance
//!
//! Both systems integrate with scoring to bias storylet selection toward
//! events that address pressures or advance milestones.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use syn_core::SimTick;
use syn_storylets::library::StoryletKey;
use syn_storylets::{StoryDomain, Tag};

use crate::config::{MilestoneConfig, PressureConfig};
use crate::queue::{QueueSource, QueuedEvent};
use crate::state::DirectorState;

// ============================================================================
// Pressure Types
// ============================================================================

/// Unique identifier for a pressure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PressureId(pub u64);

impl PressureId {
    pub fn new(id: u64) -> Self {
        PressureId(id)
    }
}

/// The kind/category of pressure.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PressureKind {
    /// Money troubles: rent, bills, debt
    Financial,
    /// School/study pressure: exams, assignments
    Academic,
    /// Legal issues: court dates, probation
    Legal,
    /// Social pressure: reputation, peer pressure
    Social,
    /// Health concerns: illness, injury
    Health,
    /// Relationship tension: conflict, distance
    Relationship,
    /// Work-related: deadlines, performance
    Career,
    /// Custom pressure type with identifier
    Custom(String),
}

impl PressureKind {
    /// Get a string tag for matching storylets.
    pub fn as_tag(&self) -> String {
        match self {
            PressureKind::Financial => "financial".to_string(),
            PressureKind::Academic => "academic".to_string(),
            PressureKind::Legal => "legal".to_string(),
            PressureKind::Social => "social".to_string(),
            PressureKind::Health => "health".to_string(),
            PressureKind::Relationship => "relationship".to_string(),
            PressureKind::Career => "career".to_string(),
            PressureKind::Custom(s) => s.clone(),
        }
    }

    /// Map to a StoryDomain if applicable.
    pub fn to_domain(&self) -> Option<StoryDomain> {
        match self {
            PressureKind::Financial => Some(StoryDomain::Career),
            PressureKind::Academic => Some(StoryDomain::Career),
            PressureKind::Career => Some(StoryDomain::Career),
            PressureKind::Relationship => Some(StoryDomain::Romance),
            PressureKind::Social => Some(StoryDomain::Friendship),
            PressureKind::Health => Some(StoryDomain::SliceOfLife),
            PressureKind::Legal => Some(StoryDomain::Conflict),
            PressureKind::Custom(_) => None,
        }
    }
}

/// An active pressure pushing the narrative forward.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pressure {
    /// Unique identifier.
    pub id: PressureId,
    
    /// What kind of pressure this is.
    pub kind: PressureKind,
    
    /// When this pressure was created.
    pub created_at: SimTick,
    
    /// Optional deadline tick (after which severity escalates dramatically).
    pub deadline: Option<SimTick>,
    
    /// Current severity (0.0 to 1.0).
    /// Increases as deadline approaches or time passes.
    pub severity: f32,
    
    /// Whether this pressure has been resolved.
    pub resolved: bool,
    
    /// Tags associated with this pressure for storylet matching.
    pub tags: Vec<Tag>,
    
    /// Optional storylet key that can resolve this pressure.
    pub resolution_storylet: Option<StoryletKey>,
    
    /// Description for debugging/display.
    pub description: String,
}

impl Pressure {
    /// Create a new pressure.
    pub fn new(
        id: PressureId,
        kind: PressureKind,
        created_at: SimTick,
        description: String,
    ) -> Self {
        Pressure {
            id,
            kind,
            created_at,
            deadline: None,
            severity: 0.1, // Start low
            resolved: false,
            tags: Vec::new(),
            resolution_storylet: None,
            description,
        }
    }

    /// Builder: set deadline.
    pub fn with_deadline(mut self, deadline: SimTick) -> Self {
        self.deadline = Some(deadline);
        self
    }

    /// Builder: set initial severity.
    pub fn with_severity(mut self, severity: f32) -> Self {
        self.severity = severity.clamp(0.0, 1.0);
        self
    }

    /// Builder: set associated tags.
    pub fn with_tags(mut self, tags: Vec<Tag>) -> Self {
        self.tags = tags;
        self
    }

    /// Builder: set resolution storylet.
    pub fn with_resolution(mut self, key: StoryletKey) -> Self {
        self.resolution_storylet = Some(key);
        self
    }

    /// Check if this pressure is urgent (severity above threshold).
    pub fn is_urgent(&self, threshold: f32) -> bool {
        !self.resolved && self.severity >= threshold
    }

    /// Check if this pressure is in crisis (severity above crisis threshold).
    pub fn is_crisis(&self, threshold: f32) -> bool {
        !self.resolved && self.severity >= threshold
    }

    /// Check if deadline has passed.
    pub fn is_overdue(&self, now: SimTick) -> bool {
        self.deadline.map(|d| now.0 > d.0).unwrap_or(false)
    }

    /// Get remaining ticks until deadline (or 0 if overdue/no deadline).
    pub fn ticks_until_deadline(&self, now: SimTick) -> u64 {
        self.deadline
            .map(|d| d.0.saturating_sub(now.0))
            .unwrap_or(0)
    }
}

/// State tracking all active pressures.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PressureState {
    /// Active pressures indexed by ID.
    pub active: HashMap<PressureId, Pressure>,
    
    /// Next ID to assign.
    next_id: u64,
}

impl PressureState {
    /// Create new empty pressure state.
    pub fn new() -> Self {
        PressureState {
            active: HashMap::new(),
            next_id: 1,
        }
    }

    /// Add a new pressure and return its ID.
    pub fn add_pressure(&mut self, mut pressure: Pressure) -> PressureId {
        let id = PressureId::new(self.next_id);
        self.next_id += 1;
        pressure.id = id;
        self.active.insert(id, pressure);
        id
    }

    /// Get a pressure by ID.
    pub fn get(&self, id: PressureId) -> Option<&Pressure> {
        self.active.get(&id)
    }

    /// Get a mutable pressure by ID.
    pub fn get_mut(&mut self, id: PressureId) -> Option<&mut Pressure> {
        self.active.get_mut(&id)
    }

    /// Resolve a pressure.
    pub fn resolve(&mut self, id: PressureId) {
        if let Some(p) = self.active.get_mut(&id) {
            p.resolved = true;
            p.severity = 0.0;
        }
    }

    /// Remove resolved pressures.
    pub fn cleanup_resolved(&mut self) {
        self.active.retain(|_, p| !p.resolved);
    }

    /// Check if there are any active (unresolved) pressures.
    pub fn has_active_pressures(&self) -> bool {
        self.active.values().any(|p| !p.resolved)
    }

    /// Get all active (unresolved) pressures.
    pub fn active_pressures(&self) -> impl Iterator<Item = &Pressure> {
        self.active.values().filter(|p| !p.resolved)
    }

    /// Get the highest severity among active pressures.
    pub fn max_severity(&self) -> f32 {
        self.active_pressures()
            .map(|p| p.severity)
            .fold(0.0, f32::max)
    }

    /// Get pressures matching a kind.
    pub fn by_kind(&self, kind: &PressureKind) -> impl Iterator<Item = &Pressure> {
        let kind = kind.clone();
        self.active.values().filter(move |p| !p.resolved && p.kind == kind)
    }

    /// Get count of active pressures.
    pub fn active_count(&self) -> usize {
        self.active.values().filter(|p| !p.resolved).count()
    }
}

// ============================================================================
// Milestone Types
// ============================================================================

/// Unique identifier for a milestone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MilestoneId(pub u64);

impl MilestoneId {
    pub fn new(id: u64) -> Self {
        MilestoneId(id)
    }
}

/// The kind/category of milestone (narrative arc).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MilestoneKind {
    /// Career progression arc
    CareerArc,
    /// Romance development arc
    RomanceArc,
    /// Personal healing/growth arc
    HealingArc,
    /// Downward spiral arc
    DownfallArc,
    /// Friendship building arc
    FriendshipArc,
    /// Self-discovery arc
    IdentityArc,
    /// Redemption arc
    RedemptionArc,
    /// Custom arc type
    Custom(String),
}

impl MilestoneKind {
    /// Get the primary domain for this milestone kind.
    pub fn primary_domain(&self) -> StoryDomain {
        match self {
            MilestoneKind::CareerArc => StoryDomain::Career,
            MilestoneKind::RomanceArc => StoryDomain::Romance,
            MilestoneKind::HealingArc => StoryDomain::Trauma,
            MilestoneKind::DownfallArc => StoryDomain::Addiction,
            MilestoneKind::FriendshipArc => StoryDomain::Friendship,
            MilestoneKind::IdentityArc => StoryDomain::SliceOfLife,
            MilestoneKind::RedemptionArc => StoryDomain::Trauma,
            MilestoneKind::Custom(_) => StoryDomain::SliceOfLife,
        }
    }

    /// Get a string tag for matching storylets.
    pub fn as_tag(&self) -> String {
        match self {
            MilestoneKind::CareerArc => "career_arc".to_string(),
            MilestoneKind::RomanceArc => "romance_arc".to_string(),
            MilestoneKind::HealingArc => "healing_arc".to_string(),
            MilestoneKind::DownfallArc => "downfall_arc".to_string(),
            MilestoneKind::FriendshipArc => "friendship_arc".to_string(),
            MilestoneKind::IdentityArc => "identity_arc".to_string(),
            MilestoneKind::RedemptionArc => "redemption_arc".to_string(),
            MilestoneKind::Custom(s) => s.clone(),
        }
    }
}

/// A milestone tracking progress toward a narrative goal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    /// Unique identifier.
    pub id: MilestoneId,
    
    /// What kind of arc this milestone represents.
    pub kind: MilestoneKind,
    
    /// Current progress (0.0 to 1.0).
    pub progress: f32,
    
    /// Whether this milestone is completed.
    pub completed: bool,
    
    /// When this milestone was started.
    pub started_at: SimTick,
    
    /// When this milestone was completed (if completed).
    pub completed_at: Option<SimTick>,
    
    /// Tags that advance this milestone.
    pub advancing_tags: Vec<Tag>,
    
    /// Optional climax storylet to queue when nearing completion.
    pub climax_storylet: Option<StoryletKey>,
    
    /// Progress threshold at which to queue climax (default 0.8).
    pub climax_threshold: f32,
    
    /// Whether climax has been queued.
    pub climax_queued: bool,
    
    /// Description for debugging/display.
    pub description: String,
}

impl Milestone {
    /// Create a new milestone.
    pub fn new(
        id: MilestoneId,
        kind: MilestoneKind,
        started_at: SimTick,
        description: String,
    ) -> Self {
        Milestone {
            id,
            kind,
            progress: 0.0,
            completed: false,
            started_at,
            completed_at: None,
            advancing_tags: Vec::new(),
            climax_storylet: None,
            climax_threshold: 0.8,
            climax_queued: false,
            description,
        }
    }

    /// Builder: set advancing tags.
    pub fn with_advancing_tags(mut self, tags: Vec<Tag>) -> Self {
        self.advancing_tags = tags;
        self
    }

    /// Builder: set climax storylet.
    pub fn with_climax(mut self, key: StoryletKey, threshold: f32) -> Self {
        self.climax_storylet = Some(key);
        self.climax_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Builder: set initial progress.
    pub fn with_progress(mut self, progress: f32) -> Self {
        self.progress = progress.clamp(0.0, 1.0);
        self
    }

    /// Check if this milestone is "hot" (mid-arc, not complete).
    pub fn is_hot(&self) -> bool {
        !self.completed && self.progress >= 0.2 && self.progress < 0.9
    }

    /// Check if this milestone is nearing climax.
    pub fn is_nearing_climax(&self) -> bool {
        !self.completed && self.progress >= self.climax_threshold
    }

    /// Advance progress by an amount.
    pub fn advance(&mut self, amount: f32, now: SimTick) {
        if self.completed {
            return;
        }
        self.progress = (self.progress + amount).min(1.0);
        if self.progress >= 1.0 {
            self.completed = true;
            self.completed_at = Some(now);
        }
    }
}

/// State tracking all milestones.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MilestoneState {
    /// All milestones indexed by ID.
    pub milestones: HashMap<MilestoneId, Milestone>,
    
    /// Next ID to assign.
    next_id: u64,
}

impl MilestoneState {
    /// Create new empty milestone state.
    pub fn new() -> Self {
        MilestoneState {
            milestones: HashMap::new(),
            next_id: 1,
        }
    }

    /// Add a new milestone and return its ID.
    pub fn add_milestone(&mut self, mut milestone: Milestone) -> MilestoneId {
        let id = MilestoneId::new(self.next_id);
        self.next_id += 1;
        milestone.id = id;
        self.milestones.insert(id, milestone);
        id
    }

    /// Get a milestone by ID.
    pub fn get(&self, id: MilestoneId) -> Option<&Milestone> {
        self.milestones.get(&id)
    }

    /// Get a mutable milestone by ID.
    pub fn get_mut(&mut self, id: MilestoneId) -> Option<&mut Milestone> {
        self.milestones.get_mut(&id)
    }

    /// Get all active (incomplete) milestones.
    pub fn active_milestones(&self) -> impl Iterator<Item = &Milestone> {
        self.milestones.values().filter(|m| !m.completed)
    }

    /// Get all completed milestones.
    pub fn completed_milestones(&self) -> impl Iterator<Item = &Milestone> {
        self.milestones.values().filter(|m| m.completed)
    }

    /// Get "hot" milestones (mid-arc, good for bonus).
    pub fn hot_milestones(&self) -> impl Iterator<Item = &Milestone> {
        self.milestones.values().filter(|m| m.is_hot())
    }

    /// Get milestones nearing climax.
    pub fn nearing_climax(&self) -> impl Iterator<Item = &Milestone> {
        self.milestones.values().filter(|m| m.is_nearing_climax())
    }

    /// Get count of active milestones.
    pub fn active_count(&self) -> usize {
        self.milestones.values().filter(|m| !m.completed).count()
    }
    
    /// Check if any milestones exist (completed or not).
    pub fn has_milestones(&self) -> bool {
        !self.milestones.is_empty()
    }
    
    /// Iterate over all milestones.
    pub fn iter(&self) -> impl Iterator<Item = &Milestone> {
        self.milestones.values()
    }
}

// ============================================================================
// Tick Update Functions
// ============================================================================

/// Update pressures at the start of each tick.
///
/// - Increases severity as deadlines approach or are exceeded
/// - Clamps severity to [0, 1]
/// - Removes resolved pressures older than cleanup_age ticks
pub fn tick_pressures(
    state: &mut DirectorState,
    config: &PressureConfig,
    now: SimTick,
) {
    for pressure in state.active_pressures.active.values_mut() {
        if pressure.resolved {
            continue;
        }

        // Base severity increase over time
        pressure.severity += config.base_severity_increase;

        // Extra increase as deadline approaches
        if let Some(deadline) = pressure.deadline {
            if now.0 >= deadline.0 {
                // Past deadline: rapid severity increase
                pressure.severity += config.overdue_severity_increase;
            } else {
                // Approaching deadline: gradual increase
                let total_duration = deadline.0.saturating_sub(pressure.created_at.0);
                let elapsed = now.0.saturating_sub(pressure.created_at.0);
                if total_duration > 0 {
                    let urgency_factor = elapsed as f32 / total_duration as f32;
                    pressure.severity += config.deadline_urgency_factor * urgency_factor * 0.1;
                }
            }
        }

        // Clamp severity
        pressure.severity = pressure.severity.clamp(0.0, 1.0);
    }

    // Cleanup old resolved pressures
    let cleanup_threshold = now.0.saturating_sub(config.resolved_cleanup_ticks);
    state.active_pressures.active.retain(|_, p| {
        !p.resolved || p.created_at.0 > cleanup_threshold
    });
}

/// Resolve a pressure by ID.
pub fn resolve_pressure(state: &mut DirectorState, id: PressureId) {
    state.active_pressures.resolve(id);
}

/// Check pressures for crisis conditions and schedule forced events if needed.
///
/// Returns a list of events to queue.
pub fn check_pressure_crises(
    state: &DirectorState,
    config: &PressureConfig,
    now: SimTick,
) -> Vec<QueuedEvent> {
    let mut events = Vec::new();

    for pressure in state.active_pressures.active_pressures() {
        // Skip if already past crisis - let it play out naturally
        if pressure.severity < config.crisis_threshold {
            continue;
        }

        // If we have a resolution storylet and we're in crisis, schedule it
        if let Some(resolution_key) = pressure.resolution_storylet {
            // High priority forced event
            let event = QueuedEvent::new(
                resolution_key,
                now, // Fire immediately
                100, // High priority
                true, // Forced
                QueueSource::PressureRelief,
            );
            events.push(event);
        }
    }

    events
}

/// Update milestone progress based on a fired storylet.
///
/// Checks if the storylet's tags or domain advance any milestones.
pub fn update_milestone_progress(
    state: &mut DirectorState,
    config: &MilestoneConfig,
    _storylet_key: StoryletKey,
    storylet_domain: StoryDomain,
    storylet_tags: &[Tag],
    now: SimTick,
) {
    for milestone in state.milestones.milestones.values_mut() {
        if milestone.completed {
            continue;
        }

        let mut progress_amount = 0.0;

        // Domain match gives base progress
        if milestone.kind.primary_domain() == storylet_domain {
            progress_amount += config.domain_match_progress;
        }

        // Tag matches give additional progress
        for tag in storylet_tags {
            if milestone.advancing_tags.contains(tag) {
                progress_amount += config.tag_match_progress;
            }
        }

        if progress_amount > 0.0 {
            milestone.advance(progress_amount, now);
        }
    }
}

/// Check milestones for climax conditions and schedule events if needed.
///
/// Returns a list of events to queue.
pub fn check_milestone_climaxes(
    state: &mut MilestoneState,
    now: SimTick,
) -> Vec<QueuedEvent> {
    let mut events = Vec::new();

    for milestone in state.milestones.values_mut() {
        // Skip if already queued or completed
        if milestone.climax_queued || milestone.completed {
            continue;
        }

        // Check if nearing climax
        if milestone.is_nearing_climax() {
            if let Some(climax_key) = milestone.climax_storylet {
                // Schedule climax event with moderate priority
                let event = QueuedEvent::new(
                    climax_key,
                    SimTick::new(now.0 + 1), // Fire next tick
                    50, // Moderate priority
                    false, // Not forced - can compete
                    QueueSource::Milestone,
                );
                events.push(event);
                milestone.climax_queued = true;
            }
        }
    }

    events
}

// ============================================================================
// Scoring Integration
// ============================================================================

/// Compute pressure bonus for a storylet.
///
/// Storylets that address active pressures get a bonus proportional to severity.
pub fn compute_pressure_bonus(
    pressures: &PressureState,
    config: &PressureConfig,
    storylet_domain: StoryDomain,
    storylet_tags: &[Tag],
) -> f32 {
    let mut total_bonus = 0.0;

    for pressure in pressures.active_pressures() {
        let mut match_strength = 0.0;

        // Domain match
        if let Some(pressure_domain) = pressure.kind.to_domain() {
            if pressure_domain == storylet_domain {
                match_strength += 0.5;
            }
        }

        // Tag matches
        let pressure_tag = Tag::new(pressure.kind.as_tag());
        for tag in storylet_tags {
            if tag == &pressure_tag || pressure.tags.contains(tag) {
                match_strength += 0.3;
                break; // Count once
            }
        }

        // Resolution storylet match
        if pressure.resolution_storylet.is_some() {
            // Would need storylet_key to check, but we don't have it here
            // This is handled separately
        }

        if match_strength > 0.0 {
            // Bonus scales with severity and match strength
            total_bonus += config.addressing_bonus * pressure.severity * match_strength;
        }
    }

    // Cap at 2x base bonus
    total_bonus.min(config.addressing_bonus * 2.0)
}

/// Compute milestone bonus for a storylet.
///
/// Storylets that advance "hot" milestones get a bonus.
pub fn compute_milestone_bonus(
    milestones: &MilestoneState,
    config: &MilestoneConfig,
    storylet_domain: StoryDomain,
    storylet_tags: &[Tag],
) -> f32 {
    let mut total_bonus = 0.0;

    for milestone in milestones.hot_milestones() {
        let mut match_strength = 0.0;

        // Domain match
        if milestone.kind.primary_domain() == storylet_domain {
            match_strength += 0.4;
        }

        // Tag matches
        for tag in storylet_tags {
            if milestone.advancing_tags.contains(tag) {
                match_strength += 0.3;
                break;
            }
        }

        // Milestone arc tag match
        let arc_tag = Tag::new(milestone.kind.as_tag());
        if storylet_tags.contains(&arc_tag) {
            match_strength += 0.3;
        }

        if match_strength > 0.0 {
            // Bonus scales with how "hot" the milestone is (mid-arc is hottest)
            let heat_factor = 1.0 - (milestone.progress - 0.5).abs() * 2.0; // Peak at 0.5
            total_bonus += config.hot_milestone_bonus * match_strength * heat_factor.max(0.3);
        }
    }

    // Cap at configured max
    total_bonus.min(config.max_milestone_bonus)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_pressure_config() -> PressureConfig {
        PressureConfig {
            pressure_decay_rate: 0.1,
            max_pressure: 100.0,
            base_severity_increase: 0.01,
            overdue_severity_increase: 0.1,
            deadline_urgency_factor: 0.5,
            urgency_threshold: 0.5,
            crisis_threshold: 0.8,
            resolved_cleanup_ticks: 100,
            addressing_bonus: 3.0,
        }
    }

    fn create_test_milestone_config() -> MilestoneConfig {
        MilestoneConfig {
            progress_per_event: 0.1,
            hot_milestone_bonus: 1.0,
            hot_threshold: 0.2,
            climax_threshold: 0.8,
            min_ticks_before_climax: 24,
            domain_match_progress: 0.05,
            tag_match_progress: 0.1,
            max_milestone_bonus: 2.0,
        }
    }

    // =========================================================================
    // Pressure Tests
    // =========================================================================

    #[test]
    fn test_pressure_creation() {
        let pressure = Pressure::new(
            PressureId(1),
            PressureKind::Financial,
            SimTick::new(0),
            "Rent due".to_string(),
        );

        assert_eq!(pressure.id, PressureId(1));
        assert_eq!(pressure.kind, PressureKind::Financial);
        assert!(!pressure.resolved);
        assert!(pressure.severity > 0.0);
    }

    #[test]
    fn test_pressure_with_deadline() {
        let pressure = Pressure::new(
            PressureId(1),
            PressureKind::Academic,
            SimTick::new(0),
            "Exam tomorrow".to_string(),
        )
        .with_deadline(SimTick::new(24));

        assert_eq!(pressure.deadline, Some(SimTick::new(24)));
        assert!(!pressure.is_overdue(SimTick::new(10)));
        assert!(pressure.is_overdue(SimTick::new(25)));
    }

    #[test]
    fn test_pressure_state_add_and_resolve() {
        let mut state = PressureState::new();

        let pressure = Pressure::new(
            PressureId(0), // Will be overwritten
            PressureKind::Financial,
            SimTick::new(0),
            "Test".to_string(),
        );

        let id = state.add_pressure(pressure);
        assert!(state.has_active_pressures());
        assert_eq!(state.active_count(), 1);

        state.resolve(id);
        assert!(!state.has_active_pressures());
    }

    #[test]
    fn test_pressure_severity_increases_over_time() {
        let mut state = DirectorState::new();
        let config = create_test_pressure_config();

        let pressure = Pressure::new(
            PressureId(0),
            PressureKind::Financial,
            SimTick::new(0),
            "Rent".to_string(),
        )
        .with_severity(0.1)
        .with_deadline(SimTick::new(100));

        state.active_pressures.add_pressure(pressure);
        let initial_severity = state.active_pressures.active.values().next().unwrap().severity;

        // Tick several times
        for tick in 1..=10 {
            tick_pressures(&mut state, &config, SimTick::new(tick));
        }

        let final_severity = state.active_pressures.active.values().next().unwrap().severity;
        assert!(final_severity > initial_severity,
            "Severity should increase: {} -> {}", initial_severity, final_severity);
    }

    #[test]
    fn test_pressure_severity_increases_faster_near_deadline() {
        let mut state1 = DirectorState::new();
        let mut state2 = DirectorState::new();
        let config = create_test_pressure_config();

        // Pressure near deadline
        let pressure_near = Pressure::new(
            PressureId(0),
            PressureKind::Financial,
            SimTick::new(0),
            "Near".to_string(),
        )
        .with_severity(0.1)
        .with_deadline(SimTick::new(20)); // Deadline at 20

        // Pressure far from deadline
        let pressure_far = Pressure::new(
            PressureId(0),
            PressureKind::Financial,
            SimTick::new(0),
            "Far".to_string(),
        )
        .with_severity(0.1)
        .with_deadline(SimTick::new(200)); // Deadline at 200

        state1.active_pressures.add_pressure(pressure_near);
        state2.active_pressures.add_pressure(pressure_far);

        // Tick to near the deadline
        for tick in 1..=15 {
            tick_pressures(&mut state1, &config, SimTick::new(tick));
            tick_pressures(&mut state2, &config, SimTick::new(tick));
        }

        let near_severity = state1.active_pressures.active.values().next().unwrap().severity;
        let far_severity = state2.active_pressures.active.values().next().unwrap().severity;

        assert!(near_severity > far_severity,
            "Near-deadline pressure should have higher severity: {} vs {}", 
            near_severity, far_severity);
    }

    #[test]
    fn test_pressure_crisis_schedules_event() {
        let mut state = DirectorState::new();
        let config = create_test_pressure_config();

        let pressure = Pressure::new(
            PressureId(0),
            PressureKind::Financial,
            SimTick::new(0),
            "Crisis".to_string(),
        )
        .with_severity(0.9) // Above crisis threshold
        .with_resolution(StoryletKey(42));

        state.active_pressures.add_pressure(pressure);

        let events = check_pressure_crises(&state, &config, SimTick::new(10));

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].storylet_key, StoryletKey(42));
        assert!(events[0].forced);
        assert_eq!(events[0].source, QueueSource::PressureRelief);
    }

    #[test]
    fn test_pressure_bonus_calculation() {
        let mut state = PressureState::new();
        let config = create_test_pressure_config();

        let pressure = Pressure::new(
            PressureId(0),
            PressureKind::Financial,
            SimTick::new(0),
            "Rent".to_string(),
        )
        .with_severity(0.8)
        .with_tags(vec![Tag::new("rent"), Tag::new("money")]);

        state.add_pressure(pressure);

        // Matching domain and tag
        let bonus = compute_pressure_bonus(
            &state,
            &config,
            StoryDomain::Career,
            &[Tag::new("rent"), Tag::new("job")],
        );

        assert!(bonus > 0.0, "Should have positive bonus for matching storylet");

        // Non-matching
        let no_bonus = compute_pressure_bonus(
            &state,
            &config,
            StoryDomain::Romance,
            &[Tag::new("love"), Tag::new("date")],
        );

        assert!(bonus > no_bonus, "Matching should have higher bonus: {} vs {}", bonus, no_bonus);
    }

    // =========================================================================
    // Milestone Tests
    // =========================================================================

    #[test]
    fn test_milestone_creation() {
        let milestone = Milestone::new(
            MilestoneId(1),
            MilestoneKind::RomanceArc,
            SimTick::new(0),
            "Find love".to_string(),
        );

        assert_eq!(milestone.id, MilestoneId(1));
        assert_eq!(milestone.kind, MilestoneKind::RomanceArc);
        assert!(!milestone.completed);
        assert_eq!(milestone.progress, 0.0);
    }

    #[test]
    fn test_milestone_progress_and_completion() {
        let mut milestone = Milestone::new(
            MilestoneId(1),
            MilestoneKind::RomanceArc,
            SimTick::new(0),
            "Find love".to_string(),
        );

        assert!(!milestone.is_hot()); // Progress too low

        milestone.advance(0.3, SimTick::new(10));
        assert!(milestone.is_hot()); // Now in hot range

        milestone.advance(0.7, SimTick::new(20));
        assert!(milestone.completed);
        assert!(milestone.completed_at.is_some());
    }

    #[test]
    fn test_milestone_climax_detection() {
        let milestone = Milestone::new(
            MilestoneId(1),
            MilestoneKind::RomanceArc,
            SimTick::new(0),
            "Find love".to_string(),
        )
        .with_progress(0.85)
        .with_climax(StoryletKey(99), 0.8);

        assert!(milestone.is_nearing_climax());
    }

    #[test]
    fn test_milestone_climax_schedules_event() {
        let mut state = MilestoneState::new();

        let milestone = Milestone::new(
            MilestoneId(0),
            MilestoneKind::RomanceArc,
            SimTick::new(0),
            "Find love".to_string(),
        )
        .with_progress(0.85)
        .with_climax(StoryletKey(99), 0.8);

        state.add_milestone(milestone);

        let events = check_milestone_climaxes(&mut state, SimTick::new(100));

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].storylet_key, StoryletKey(99));
        assert_eq!(events[0].source, QueueSource::Milestone);

        // Should not schedule again
        let events2 = check_milestone_climaxes(&mut state, SimTick::new(101));
        assert!(events2.is_empty());
    }

    #[test]
    fn test_milestone_progress_from_storylet() {
        let mut state = DirectorState::new();
        let config = create_test_milestone_config();

        let milestone = Milestone::new(
            MilestoneId(0),
            MilestoneKind::RomanceArc,
            SimTick::new(0),
            "Find love".to_string(),
        )
        .with_advancing_tags(vec![Tag::new("romantic"), Tag::new("confession")]);

        state.milestones.add_milestone(milestone);

        let initial_progress = state.milestones.milestones.values().next().unwrap().progress;

        // Fire a matching storylet
        update_milestone_progress(
            &mut state,
            &config,
            StoryletKey(1),
            StoryDomain::Romance, // Matches
            &[Tag::new("romantic"), Tag::new("date")], // Tag matches
            SimTick::new(10),
        );

        let final_progress = state.milestones.milestones.values().next().unwrap().progress;
        assert!(final_progress > initial_progress,
            "Progress should increase: {} -> {}", initial_progress, final_progress);
    }

    #[test]
    fn test_milestone_completes_after_enough_progress() {
        let mut state = DirectorState::new();
        let config = create_test_milestone_config();

        let milestone = Milestone::new(
            MilestoneId(0),
            MilestoneKind::RomanceArc,
            SimTick::new(0),
            "Find love".to_string(),
        )
        .with_advancing_tags(vec![Tag::new("romantic")]);

        state.milestones.add_milestone(milestone);

        // Fire many matching storylets
        for i in 0..20 {
            update_milestone_progress(
                &mut state,
                &config,
                StoryletKey(i as u32),
                StoryDomain::Romance,
                &[Tag::new("romantic")],
                SimTick::new(i as u64 * 10),
            );
        }

        let milestone = state.milestones.milestones.values().next().unwrap();
        assert!(milestone.completed, "Milestone should be completed");
        assert_eq!(milestone.progress, 1.0);
    }

    #[test]
    fn test_milestone_bonus_calculation() {
        let mut state = MilestoneState::new();
        let config = create_test_milestone_config();

        let milestone = Milestone::new(
            MilestoneId(0),
            MilestoneKind::RomanceArc,
            SimTick::new(0),
            "Find love".to_string(),
        )
        .with_progress(0.5) // Hot zone
        .with_advancing_tags(vec![Tag::new("romantic")]);

        state.add_milestone(milestone);

        // Matching domain and tag
        let bonus = compute_milestone_bonus(
            &state,
            &config,
            StoryDomain::Romance,
            &[Tag::new("romantic"), Tag::new("date")],
        );

        assert!(bonus > 0.0, "Should have positive bonus for matching storylet");

        // Non-matching
        let no_bonus = compute_milestone_bonus(
            &state,
            &config,
            StoryDomain::Career,
            &[Tag::new("work"), Tag::new("job")],
        );

        assert!(bonus > no_bonus, "Matching should have higher bonus: {} vs {}", bonus, no_bonus);
    }

    // =========================================================================
    // Integration Tests
    // =========================================================================

    #[test]
    fn test_pressure_and_milestone_interaction() {
        let mut state = DirectorState::new();
        let pressure_config = create_test_pressure_config();
        let milestone_config = create_test_milestone_config();

        // Add a pressure
        let pressure = Pressure::new(
            PressureId(0),
            PressureKind::Relationship,
            SimTick::new(0),
            "Relationship strain".to_string(),
        )
        .with_severity(0.6)
        .with_tags(vec![Tag::new("relationship_crisis")]);
        state.active_pressures.add_pressure(pressure);

        // Add a related milestone
        let milestone = Milestone::new(
            MilestoneId(0),
            MilestoneKind::RomanceArc,
            SimTick::new(0),
            "Fix relationship".to_string(),
        )
        .with_progress(0.4)
        .with_advancing_tags(vec![Tag::new("relationship_healing")]);
        state.milestones.add_milestone(milestone);

        // A storylet that addresses both
        let pressure_bonus = compute_pressure_bonus(
            &state.active_pressures,
            &pressure_config,
            StoryDomain::Romance,
            &[Tag::new("relationship_crisis"), Tag::new("relationship_healing")],
        );

        let milestone_bonus = compute_milestone_bonus(
            &state.milestones,
            &milestone_config,
            StoryDomain::Romance,
            &[Tag::new("relationship_healing")],
        );

        assert!(pressure_bonus > 0.0);
        assert!(milestone_bonus > 0.0);

        // Total bonus should be significant
        let total = pressure_bonus + milestone_bonus;
        assert!(total > 1.0, "Combined bonus should be significant: {}", total);
    }
}
