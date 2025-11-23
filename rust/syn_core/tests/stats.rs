use syn_core::{apply_stat_deltas, Karma, KarmaBand, MoodBand, StatDelta, StatKind, Stats};

#[test]
fn clamps_correctly() {
    let mut s = Stats::default();
    s.health = 150.0;
    s.mood = 15.0;
    s.reputation = -150.0;
    s.curiosity = Some(120.0);
    s.energy = Some(-10.0);
    s.libido = Some(200.0);
    s.clamp();
    assert_eq!(s.health, 100.0);
    assert_eq!(s.mood, 10.0);
    assert_eq!(s.reputation, -100.0);
    assert_eq!(s.curiosity.unwrap(), 100.0);
    assert_eq!(s.energy.unwrap(), 0.0);
    assert_eq!(s.libido.unwrap(), 100.0);
}

#[test]
fn get_set_delta_roundtrip() {
    let mut s = Stats::default();
    s.set(StatKind::Health, 80.0);
    assert_eq!(s.get(StatKind::Health), 80.0);
    s.apply_delta(StatKind::Health, 30.0);
    assert_eq!(s.get(StatKind::Health), 100.0); // clamped
}

#[test]
fn mood_band_transitions() {
    let mut s = Stats::default();
    s.mood = -8.0;
    assert_eq!(s.mood_band(), MoodBand::Despair);
    s.mood = -3.0;
    assert_eq!(s.mood_band(), MoodBand::Low);
    s.mood = 0.5;
    assert_eq!(s.mood_band(), MoodBand::Neutral);
    s.mood = 3.0;
    assert_eq!(s.mood_band(), MoodBand::High);
    s.mood = 8.0;
    assert_eq!(s.mood_band(), MoodBand::Euphoric);
}

#[test]
fn karma_band_transitions() {
    let mut k = Karma(-80.0);
    assert_eq!(k.band(), KarmaBand::Damned);
    k.0 = -30.0;
    assert_eq!(k.band(), KarmaBand::Tainted);
    k.0 = 0.0;
    assert_eq!(k.band(), KarmaBand::Balanced);
    k.0 = 30.0;
    assert_eq!(k.band(), KarmaBand::Blessed);
    k.0 = 80.0;
    assert_eq!(k.band(), KarmaBand::Ascendant);
}

#[test]
fn apply_multiple_deltas_deterministically() {
    let mut s = Stats::default();
    let deltas = vec![
        StatDelta {
            kind: StatKind::Health,
            delta: -20.0,
            source: Some("test1".into()),
        },
        StatDelta {
            kind: StatKind::Health,
            delta: 10.0,
            source: Some("test2".into()),
        },
        StatDelta {
            kind: StatKind::Mood,
            delta: 15.0,
            source: None,
        },
    ];
    apply_stat_deltas(&mut s, &deltas);
    assert_eq!(s.get(StatKind::Health), 40.0);
    assert_eq!(s.get(StatKind::Mood), 10.0); // clamped
}
