//! Stats system: player and NPC stats, clamp helpers, and bands.
//!
//! Stats are clamped to valid ranges:
//! - Most stats: 0-100
//! - Mood: -10 to +10
//! - Reputation: -100 to +100
//! - Karma: -100 to +100

use serde::{Deserialize, Serialize};

/// Authoritative stat kinds for all systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StatKind {
    /// Physical health (0-100).
    Health,
    /// Cognitive ability (0-100).
    Intelligence,
    /// Social appeal (0-100).
    Charisma,
    /// Financial resources (0-100).
    Wealth,
    /// Emotional state (-10 to +10).
    Mood,
    /// Physical attractiveness (0-100).
    Appearance,
    /// Social standing (-100 to +100).
    Reputation,
    /// Life experience (0-100).
    Wisdom,
    /// Child stat: inquisitiveness (0-100).
    Curiosity,
    /// Child stat: vitality (0-100).
    Energy,
    /// Adult stat: libido (0-100, NSFW mode).
    Libido,
}

/// Ordered list of all stat kinds.
pub const ALL_STAT_KINDS: [StatKind; 11] = [
    StatKind::Health,
    StatKind::Intelligence,
    StatKind::Charisma,
    StatKind::Wealth,
    StatKind::Mood,
    StatKind::Appearance,
    StatKind::Reputation,
    StatKind::Wisdom,
    StatKind::Curiosity,
    StatKind::Energy,
    StatKind::Libido,
];

/// Mood bands for UI/logic thresholds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MoodBand {
    /// Severely depressed (-10 to -6).
    Despair,
    /// Low mood (-6 to -2).
    Low,
    /// Normal (-2 to +2).
    Neutral,
    /// Happy (+2 to +6).
    High,
    /// Ecstatic (+6 to +10).
    Euphoric,
}

/// Karma bands for UI/logic thresholds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KarmaBand {
    /// Deeply evil (-100 to -60).
    Damned,
    /// Morally compromised (-60 to -10).
    Tainted,
    /// Neutral (-10 to +10).
    Balanced,
    /// Good (+10 to +60).
    Blessed,
    /// Saintly (+60 to +100).
    Ascendant,
}

/// Clamp helper per stat kind.
pub fn clamp_for(kind: StatKind, value: f32) -> f32 {
    match kind {
        StatKind::Mood => value.clamp(-10.0, 10.0),
        StatKind::Reputation => value.clamp(-100.0, 100.0),
        StatKind::Curiosity
        | StatKind::Energy
        | StatKind::Libido
        | StatKind::Health
        | StatKind::Intelligence
        | StatKind::Charisma
        | StatKind::Wealth
        | StatKind::Appearance
        | StatKind::Wisdom => value.clamp(0.0, 100.0),
    }
}

/// Clamp helper for karma (-100..100).
pub fn clamp_karma(value: f32) -> f32 {
    value.clamp(-100.0, 100.0)
}

/// Stat delta for storylets and sim updates.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StatDelta {
    /// Which stat to modify.
    pub kind: StatKind,
    /// Amount to change (+/-).
    pub delta: f32,
    /// Optional source event/storylet.
    pub source: Option<String>,
}

/// Apply a list of stat deltas through the unified API.
pub fn apply_stat_deltas(stats: &mut crate::Stats, deltas: &[StatDelta]) {
    for d in deltas {
        stats.apply_delta(d.kind, d.delta);
    }
}
