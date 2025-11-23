use syn_core::LifeStage;

#[test]
fn stage_for_age_maps_expected_ranges() {
    assert_eq!(LifeStage::stage_for_age(8), LifeStage::Child);
    assert_eq!(LifeStage::stage_for_age(15), LifeStage::Teen);
    assert_eq!(LifeStage::stage_for_age(25), LifeStage::YoungAdult);
    assert_eq!(LifeStage::stage_for_age(45), LifeStage::Adult);
    assert_eq!(LifeStage::stage_for_age(70), LifeStage::Elder);
    assert_eq!(LifeStage::stage_for_age(95), LifeStage::Digital);
}

#[test]
fn stage_config_visibility_matches_expectations() {
    let teen_cfg = LifeStage::Teen.config();
    assert_eq!(teen_cfg.visibility.show_reputation, true);
    assert_eq!(teen_cfg.visibility.show_wealth, false);

    let adult_cfg = LifeStage::Adult.config();
    assert!(adult_cfg.visibility.show_wealth);
    assert!(adult_cfg.visibility.show_wisdom);
}
