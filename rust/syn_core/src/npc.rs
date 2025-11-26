//! Core NPC identity, personality, and prototype definitions.

use crate::time::DayPhase;
use serde::{Deserialize, Serialize};

use crate::types::NpcId;
use crate::{LifeStage, Stats};

/// High-level role tags for NPCs used by Director/systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NpcRoleTag {
    Family,
    Peer,
    Coworker,
    Authority,
    RomanticInterest,
    Antagonist,
    Mentor,
    Background, // low-impact filler
}

/// Personality vector (GDD-aligned axes).
/// Keep it small and deterministic.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct PersonalityVector {
    pub warmth: f32,            // -1.0 (cold) .. 1.0 (warm)
    pub dominance: f32,         // -1.0 (submissive) .. 1.0 (dominant)
    pub volatility: f32,        // -1.0 (stable) .. 1.0 (explosive)
    pub conscientiousness: f32, // 0.0 .. 1.0
    pub openness: f32,          // 0.0 .. 1.0
}

impl PersonalityVector {
    pub fn clamp(&mut self) {
        let clamp01 = |v: &mut f32| {
            if *v < 0.0 {
                *v = 0.0;
            } else if *v > 1.0 {
                *v = 1.0;
            }
        };
        // warmth/dominance/volatility are -1..1
        self.warmth = self.warmth.clamp(-1.0, 1.0);
        self.dominance = self.dominance.clamp(-1.0, 1.0);
        self.volatility = self.volatility.clamp(-1.0, 1.0);
        clamp01(&mut self.conscientiousness);
        clamp01(&mut self.openness);
    }
}

/// “Definition” of an NPC type: how they should be initialized.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NpcPrototype {
    pub id: NpcId,
    pub display_name: String,

    /// Optional narrative label: "Your Childhood Friend", etc.
    #[serde(default)]
    pub role_label: Option<String>,

    /// High-level role tags for selection & storylets.
    #[serde(default)]
    pub role_tags: Vec<NpcRoleTag>,

    /// Personality baseline.
    pub personality: PersonalityVector,

    /// Baseline stats; actual instance stats can drift from this.
    pub base_stats: Stats,

    /// Preferred life stage(s) where this NPC is active/relevant.
    #[serde(default)]
    pub active_stages: Vec<LifeStage>,

    /// Default daily schedule template for this NPC.
    #[serde(default)]
    pub schedule: NpcSchedule,
}

/// High-level activity type for schedule and presence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NpcActivityKind {
    Home,
    Work,
    School,
    Nightlife,
    Errands,
    OnlineOnly,
    Offscreen,
}

/// One day-phase schedule slot.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NpcScheduleSlot {
    pub phase: DayPhase,
    pub activity: NpcActivityKind,
}

/// Daily schedule by day phase (same each day for now).
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct NpcSchedule {
    /// Daily schedule by day phase. If empty, defaults to Home.
    #[serde(default)]
    pub daily_slots: Vec<NpcScheduleSlot>,
}

impl NpcSchedule {
    /// Get scheduled activity for a given day phase; defaults to Home.
    pub fn activity_for_phase(&self, phase: DayPhase) -> NpcActivityKind {
        for slot in &self.daily_slots {
            if slot.phase == phase {
                return slot.activity;
            }
        }
        NpcActivityKind::Home
    }
}

impl NpcPrototype {
    pub fn with_default_work_schedule(mut self) -> Self {
        self.schedule = NpcSchedule {
            daily_slots: vec![
                NpcScheduleSlot {
                    phase: DayPhase::Morning,
                    activity: NpcActivityKind::Work,
                },
                NpcScheduleSlot {
                    phase: DayPhase::Afternoon,
                    activity: NpcActivityKind::Work,
                },
                NpcScheduleSlot {
                    phase: DayPhase::Evening,
                    activity: NpcActivityKind::Home,
                },
                NpcScheduleSlot {
                    phase: DayPhase::Night,
                    activity: NpcActivityKind::Home,
                },
            ],
        };
        self
    }

    pub fn with_default_school_schedule(mut self) -> Self {
        self.schedule = NpcSchedule {
            daily_slots: vec![
                NpcScheduleSlot {
                    phase: DayPhase::Morning,
                    activity: NpcActivityKind::School,
                },
                NpcScheduleSlot {
                    phase: DayPhase::Afternoon,
                    activity: NpcActivityKind::School,
                },
                NpcScheduleSlot {
                    phase: DayPhase::Evening,
                    activity: NpcActivityKind::Home,
                },
                NpcScheduleSlot {
                    phase: DayPhase::Night,
                    activity: NpcActivityKind::Home,
                },
            ],
        };
        self
    }

    pub fn with_default_nightlife_schedule(mut self) -> Self {
        self.schedule = NpcSchedule {
            daily_slots: vec![
                NpcScheduleSlot {
                    phase: DayPhase::Morning,
                    activity: NpcActivityKind::Home,
                },
                NpcScheduleSlot {
                    phase: DayPhase::Afternoon,
                    activity: NpcActivityKind::Errands,
                },
                NpcScheduleSlot {
                    phase: DayPhase::Evening,
                    activity: NpcActivityKind::Nightlife,
                },
                NpcScheduleSlot {
                    phase: DayPhase::Night,
                    activity: NpcActivityKind::Nightlife,
                },
            ],
        };
        self
    }
}
