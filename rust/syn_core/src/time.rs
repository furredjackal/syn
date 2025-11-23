use serde::{Deserialize, Serialize};

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
        [
            DayPhase::Morning,
            DayPhase::Afternoon,
            DayPhase::Evening,
            DayPhase::Night,
        ]
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GameTime {
    /// Legacy/global tick counter; maintained for backward compatibility.
    pub tick: u64,
    /// Total tick index since game start (1 tick = 1 in-game hour for core scheduling).
    /// This mirrors `tick` to satisfy newer APIs while keeping old code working.
    #[serde(default)]
    pub tick_index: u64,
    /// Day index since game start.
    pub day: u64,
    /// Current phase of the day.
    pub phase: DayPhase,
}

impl Default for GameTime {
    fn default() -> Self {
        Self {
            tick: 0,
            tick_index: 0,
            day: 0,
            phase: DayPhase::Morning,
        }
    }
}

impl GameTime {
    /// Construct a new GameTime starting at tick 0.
    pub fn new() -> Self {
        Self::default()
    }

    /// New API: advance by a number of ticks assuming 24 ticks per day (1 tick = 1 in-game hour).
    /// Keeps legacy fields in sync for backward compatibility.
    pub fn advance_ticks(&mut self, ticks: u64) {
        const TPD: u64 = 24;
        self.advance_ticks_with_tpd(ticks, TPD);
    }

    /// Back-compat API retained: allows custom ticks-per-day.
    pub fn advance_ticks_with_tpd(&mut self, ticks: u64, ticks_per_day: u64) {
        // Update both counters to remain in sync
        self.tick = self.tick.wrapping_add(ticks);
        self.tick_index = self.tick_index.wrapping_add(ticks);
        let day = self.tick_index / ticks_per_day;
        let phase_index = ((self.tick_index % ticks_per_day) * 4 / ticks_per_day) as u8; // 4 phases
        self.day = day;
        self.phase = match phase_index {
            0 => DayPhase::Morning,
            1 => DayPhase::Afternoon,
            2 => DayPhase::Evening,
            _ => DayPhase::Night,
        };
    }

    /// Number of full days elapsed since start.
    pub fn day(&self) -> u64 {
        self.tick_index / 24
    }

    /// Current hour within the day (0..=23).
    pub fn hour_in_day(&self) -> u8 {
        (self.tick_index % 24) as u8
    }
}
