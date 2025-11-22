use serde::{Deserialize, Serialize};

/// Authoritative stat kinds for all systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StatKind {
    Health,
    Intelligence,
    Charisma,
    Wealth,
    Mood,
    Appearance,
    Reputation,
    Wisdom,
    Curiosity,
    Energy,
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
    Despair,
    Low,
    Neutral,
    High,
    Euphoric,
}

/// Karma bands for UI/logic thresholds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KarmaBand {
    Damned,
    Tainted,
    Balanced,
    Blessed,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatDelta {
    pub kind: StatKind,
    pub delta: f32,
    pub source: Option<String>,
}

/// Apply a list of stat deltas through the unified API.
pub fn apply_stat_deltas(stats: &mut crate::Stats, deltas: &[StatDelta]) {
    for d in deltas {
        stats.apply_delta(d.kind, d.delta);
    }
}
