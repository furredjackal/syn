use syn_core::time::{GameTime, DayPhase};
use syn_core::npc::{NpcSchedule, NpcScheduleSlot, NpcActivityKind};

#[test]
fn game_time_advances_phases_every_6_ticks() {
    let mut gt = GameTime::default();
    // 24 ticks per day configured in WorldState::tick, but here we test primitive
    let tpd = 24u64;
    assert_eq!(gt.phase, DayPhase::Morning);
    gt.advance_ticks(6, tpd);
    assert_eq!(gt.phase, DayPhase::Afternoon);
    gt.advance_ticks(6, tpd);
    assert_eq!(gt.phase, DayPhase::Evening);
    gt.advance_ticks(6, tpd);
    assert_eq!(gt.phase, DayPhase::Night);
    gt.advance_ticks(6, tpd);
    assert_eq!(gt.phase, DayPhase::Morning);
    assert_eq!(gt.day, 1);
}

#[test]
fn schedule_activity_lookup_defaults_and_overrides() {
    let mut sched = NpcSchedule { daily_slots: vec![] };
    assert_eq!(sched.activity_for_phase(DayPhase::Morning), NpcActivityKind::Home);
    // Add specific slot
    sched.daily_slots.push(NpcScheduleSlot{ phase: DayPhase::Morning, activity: NpcActivityKind::Work });
    assert_eq!(sched.activity_for_phase(DayPhase::Morning), NpcActivityKind::Work);
    // Other phases fall back to Home
    assert_eq!(sched.activity_for_phase(DayPhase::Evening), NpcActivityKind::Home);
}
