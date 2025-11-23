use serde::{Serialize, Deserialize};

/// Coarse-grained day phase, used for schedules and narrative pacing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DayPhase {
    Morning,
    Afternoon,
    Evening,
    Night,
}

impl DayPhase {
    pub fn all() -> [DayPhase; 4] {
        [DayPhase::Morning, DayPhase::Afternoon, DayPhase::Evening, DayPhase::Night]
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GameTime {
    /// Global tick counter; interpretation is up to sim layer (e.g. 1 tick = 15 minutes).
    pub tick: u64,
    /// Day index since game start.
    pub day: u64,
    /// Current phase of the day.
    pub phase: DayPhase,
}

impl Default for GameTime {
    fn default() -> Self {
        Self { tick: 0, day: 0, phase: DayPhase::Morning }
    }
}

impl GameTime {
    pub fn advance_ticks(&mut self, ticks: u64, ticks_per_day: u64) {
        self.tick = self.tick.wrapping_add(ticks);
        let day = self.tick / ticks_per_day;
        let phase_index = ((self.tick % ticks_per_day) * 4 / ticks_per_day) as u8; // 4 phases
        self.day = day;
        self.phase = match phase_index {
            0 => DayPhase::Morning,
            1 => DayPhase::Afternoon,
            2 => DayPhase::Evening,
            _ => DayPhase::Night,
        };
    }
}
