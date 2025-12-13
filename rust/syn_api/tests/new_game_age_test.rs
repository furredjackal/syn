//! Test that engine_new_game returns correct player_age

use syn_api::{engine_new_game, ApiPlayerConfig};

#[test]
fn test_engine_new_game_returns_correct_player_age() {
    let config = ApiPlayerConfig {
        name: "TestPlayer".to_string(),
        pronouns: None,
        archetype: "STORYTELLER".to_string(),
        difficulty: "BALANCED".to_string(),
        sfw_mode: true,
    };
    
    let result = engine_new_game(42, config);
    assert!(result.is_some(), "engine_new_game should return Some");
    
    let state = result.unwrap();
    println!("DEBUG: player_age = {}, current_day = {}, life_stage = {}", 
             state.player_age, state.current_day, state.life_stage);
    
    assert_eq!(state.player_age, 6, "Player age should be 6, got {}", state.player_age);
    assert_eq!(state.current_day, 0, "Current day should be 0, got {}", state.current_day);
    assert_eq!(state.life_stage, "Child", "Life stage should be Child, got {}", state.life_stage);
}
