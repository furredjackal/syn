use syn_core::time::GameTime;

#[test]
fn advancing_24_ticks_advances_day_and_resets_hour() {
    let mut gt = GameTime::new();
    assert_eq!(gt.day(), 0);
    assert_eq!(gt.hour_in_day(), 0);

    // advance 24 ticks (24 in-game hours)
    gt.advance_ticks(24);
    assert_eq!(gt.day(), 1);
    assert_eq!(gt.hour_in_day(), 0);
}
