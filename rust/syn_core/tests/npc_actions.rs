use syn_core::npc_actions::{behavior_to_candidate_actions, base_effect_for_action, NpcActionKind};
use syn_core::npc_behavior::BehaviorKind;

#[test]
fn mapping_from_behavior_has_expected_candidates() {
    let social = behavior_to_candidate_actions(BehaviorKind::SeekSocial);
    assert!(social.contains(&NpcActionKind::SocialVisitPlayer));

    let security = behavior_to_candidate_actions(BehaviorKind::SeekSecurity);
    assert!(security.contains(&NpcActionKind::WorkShift));

    let recog = behavior_to_candidate_actions(BehaviorKind::SeekRecognition);
    assert!(recog.contains(&NpcActionKind::SocialVisitPlayer));

    let comfort = behavior_to_candidate_actions(BehaviorKind::SeekComfort);
    assert!(comfort.contains(&NpcActionKind::WithdrawAlone));

    let auto = behavior_to_candidate_actions(BehaviorKind::SeekAutonomy);
    assert!(auto.contains(&NpcActionKind::ProvokePlayer) || auto.contains(&NpcActionKind::SelfImprovement));
}

#[test]
fn base_effects_present_where_expected() {
    // SocialVisitPlayer should affect npc stats, player stats, and have relationship/memory tags
    let eff = base_effect_for_action(NpcActionKind::SocialVisitPlayer);
    assert!(!eff.npc_stat_deltas.is_empty());
    assert!(!eff.player_stat_deltas.is_empty());
    assert!(!eff.relationship_deltas.is_empty());
    assert!(!eff.memory_tags_for_player.is_empty());

    // WorkShift should provide wealth and busy time
    let eff2 = base_effect_for_action(NpcActionKind::WorkShift);
    assert!(eff2.npc_stat_deltas.iter().any(|d| matches!(d.kind, syn_core::StatKind::Wealth)));
    assert!(eff2.busy_for_ticks > 0);
}
