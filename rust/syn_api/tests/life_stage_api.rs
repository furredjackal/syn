use syn_api::{ApiLifeStageInfo, GameEngine};
use syn_core::LifeStage;

#[test]
fn life_stage_info_matches_world() {
    let mut engine = GameEngine::new(42);
    engine.set_player_life_stage(LifeStage::Teen, 15);
    let info: ApiLifeStageInfo = engine.life_stage_info();
    assert!(info.life_stage.contains("Teen"));
    assert_eq!(info.player_age_years, 15);
}
