//! Core NPC identity, personality, and prototype definitions.

use crate::time::DayPhase;
use serde::{Deserialize, Serialize};

use crate::types::NpcId;
use crate::{LifeStage, Stats};

/// High-level role tags for NPCs used by Director/systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NpcRoleTag {
    /// Family member (parent, sibling, etc.).
    Family,
    /// Same-age peer (classmate, friend).
    Peer,
    /// Work colleague.
    Coworker,
    /// Authority figure (teacher, boss, police).
    Authority,
    /// Potential or current romantic interest.
    RomanticInterest,
    /// Conflict source (bully, rival).
    Antagonist,
    /// Guidance provider (coach, mentor).
    Mentor,
    /// Low-impact background NPC.
    Background,
}

/// Personality vector (GDD-aligned axes).
/// Keep it small and deterministic.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct PersonalityVector {
    /// Cold (-1) to warm (+1).
    pub warmth: f32,
    /// Submissive (-1) to dominant (+1).
    pub dominance: f32,
    /// Stable (-1) to explosive (+1).
    pub volatility: f32,
    /// Careless (0) to diligent (1).
    pub conscientiousness: f32,
    /// Closed (0) to open (1).
    pub openness: f32,
}

impl PersonalityVector {
    /// Clamp all axes to valid ranges.
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

/// "Definition" of an NPC type: how they should be initialized.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NpcPrototype {
    /// Unique NPC identifier.
    pub id: NpcId,
    /// Display name shown in UI.
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
    /// At home.
    Home,
    /// At work.
    Work,
    /// At school.
    School,
    /// Out at night (bar, club, party).
    Nightlife,
    /// Running errands.
    Errands,
    /// Only available online.
    OnlineOnly,
    /// Not available for interaction.
    Offscreen,
}

/// One day-phase schedule slot.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NpcScheduleSlot {
    /// Which time of day.
    pub phase: DayPhase,
    /// What they're doing.
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
    /// Set a default work schedule (work morning/afternoon, home evening/night).
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

    /// Set a default school schedule (school morning/afternoon, home evening/night).
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

    /// Set a default nightlife schedule (home morning, errands afternoon, nightlife evening/night).
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
