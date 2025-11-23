use syn_core::narrative_heat::{
    compute_heat_delta, NarrativeHeat, NarrativeHeatBand, NarrativeHeatConfig, NarrativeHeatInputs,
};
use syn_core::relationship_model::RelationshipVector;
use syn_core::Stats;

#[test]
fn clamp_and_band_mapping() {
    let mut heat = NarrativeHeat::new(150.0);
    assert_eq!(heat.value(), 100.0);
    assert_eq!(heat.band(), NarrativeHeatBand::Critical);

    heat.set(10.0);
    assert_eq!(heat.band(), NarrativeHeatBand::Low);
    heat.set(40.0);
    assert_eq!(heat.band(), NarrativeHeatBand::Medium);
    heat.set(60.0);
    assert_eq!(heat.band(), NarrativeHeatBand::High);
}

#[test]
fn decay_toward_baseline() {
    let mut heat = NarrativeHeat::new(80.0);
    heat.decay_toward(10.0, 5.0);
    assert_eq!(heat.value(), 75.0);
    heat.decay_toward(10.0, 100.0);
    assert_eq!(heat.value(), 10.0);
}

#[test]
fn compute_heat_delta_responds_to_inputs() {
    let stats = Stats {
        mood: -9.0,
        health: 5.0,
        wealth: 5.0,
        ..Stats::default()
    };
    let relationships = vec![(
        &(1_u64, 2_u64),
        &RelationshipVector {
            affection: 0.0,
            trust: 0.0,
            attraction: 0.0,
            familiarity: 0.0,
            resentment: 9.0,
        },
    )];

    let inputs = NarrativeHeatInputs {
        player_stats: &stats,
        relationships: &relationships,
        has_recent_trauma: true,
        has_recent_betrayal: false,
        has_recent_major_win: false,
        stat_profile: None,
    };

    let delta = compute_heat_delta(&inputs, &NarrativeHeatConfig::default());
    assert!(delta > 0.0);
}
