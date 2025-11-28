//! NPC action layer: maps high-level BehaviorKind into concrete action kinds
//! and provides effect bundles (stat/relationship deltas, simple busy flags).

use serde::{Deserialize, Serialize};

use crate::npc_behavior::{BehaviorKind, BehaviorSnapshot};
use crate::relationships::{RelationshipAxis, RelationshipDelta};
use crate::stats::{StatDelta, StatKind};
use crate::NpcId;

/// Discrete action an NPC can perform at the simulation level.
/// These are still abstract, but a level more concrete than BehaviorKind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NpcActionKind {
    /// Tries to connect with the player (chat, hang, comfort).
    SocialVisitPlayer,
    /// Spends time with another NPC (off-screen social).
    SocializeWithNpc,
    /// Works or takes on extra shift to stabilize security.
    WorkShift,
    /// Withdraws to rest/doomscroll; self-soothing.
    WithdrawAlone,
    /// Actively asserts autonomy or starts conflict with the player.
    ProvokePlayer,
    /// Invests time in long-term project (study, training).
    SelfImprovement,
    /// Does nothing meaningful; idle or background life.
    Idle,
}

/// Effects of a single NPC action on the simulation.
/// This is the “delta bundle” we’ll apply to world & relationships.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NpcActionEffect {
    /// Stat deltas applied to the NPC's own stats.
    #[serde(default)]
    pub npc_stat_deltas: Vec<StatDelta>,
    /// Optional stat deltas applied to the player.
    #[serde(default)]
    pub player_stat_deltas: Vec<StatDelta>,
    /// Relationship deltas (placeholder target id; 0 -> to be resolved at apply-time).
    #[serde(default)]
    pub relationship_deltas: Vec<RelationshipDelta>,
    /// Memory tags to write if the action involves the player.
    #[serde(default)]
    pub memory_tags_for_player: Vec<String>,
    /// Optional: marks NPC as "busy" for simple scheduling (in ticks; 0 = no busy state).
    #[serde(default)]
    pub busy_for_ticks: u64,
    /// Optional: narrative label / debugging note.
    #[serde(default)]
    pub label: Option<String>,
}

/// A fully chosen action instance for a specific NPC this tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcActionInstance {
    /// ID of the NPC performing this action.
    pub npc_id: NpcId,
    /// Type of action being performed.
    pub kind: NpcActionKind,
    /// True if this action is directed toward the player as target.
    pub targets_player: bool,
    /// Optional target NPC id.
    #[serde(default)]
    pub target_npc_id: Option<NpcId>,
    /// Precomputed effect bundle.
    pub effect: NpcActionEffect,
}

/// Pure mapping: BehaviorKind → one or more plausible NpcActionKind options.
pub fn behavior_to_candidate_actions(kind: BehaviorKind) -> Vec<NpcActionKind> {
    match kind {
        BehaviorKind::SeekSocial => vec![
            NpcActionKind::SocialVisitPlayer,
            NpcActionKind::SocializeWithNpc,
        ],
        BehaviorKind::SeekSecurity => vec![NpcActionKind::WorkShift],
        BehaviorKind::SeekRecognition => vec![NpcActionKind::SocialVisitPlayer],
        BehaviorKind::SeekComfort => vec![NpcActionKind::WithdrawAlone],
        BehaviorKind::SeekAutonomy => {
            vec![NpcActionKind::ProvokePlayer, NpcActionKind::SelfImprovement]
        }
        BehaviorKind::Idle => vec![NpcActionKind::Idle],
    }
}

/// Baseline effect bundles for each action kind (can be scaled later).
pub fn base_effect_for_action(kind: NpcActionKind) -> NpcActionEffect {
    let mut effect = NpcActionEffect::default();

    match kind {
        NpcActionKind::SocialVisitPlayer => {
            effect.npc_stat_deltas.push(StatDelta {
                kind: StatKind::Mood,
                delta: 1.0,
                source: Some("npc_action".into()),
            });
            effect.player_stat_deltas.push(StatDelta {
                kind: StatKind::Mood,
                delta: 0.5,
                source: Some("npc_action".into()),
            });
            // Relationship: mild affection increase, mild trust bump (target resolved at apply-time)
            effect.relationship_deltas.push(RelationshipDelta {
                target_id: NpcId(0),
                axis: RelationshipAxis::Affection,
                delta: 0.5,
                source: Some("npc_action".into()),
            });
            effect.relationship_deltas.push(RelationshipDelta {
                target_id: NpcId(0),
                axis: RelationshipAxis::Trust,
                delta: 0.3,
                source: Some("npc_action".into()),
            });
            effect.memory_tags_for_player = vec![
                "npc_behavior".into(),
                "npc_social_visit".into(),
                "support".into(),
            ];
            effect.label = Some("NPC social visit with player".into());
        }
        NpcActionKind::SocializeWithNpc => {
            effect.npc_stat_deltas.push(StatDelta {
                kind: StatKind::Mood,
                delta: 0.7,
                source: Some("npc_action".into()),
            });
            effect.npc_stat_deltas.push(StatDelta {
                kind: StatKind::Energy,
                delta: -0.3,
                source: Some("npc_action".into()),
            });
            effect.busy_for_ticks = 5;
            effect.label = Some("NPC socializes with another NPC".into());
        }
        NpcActionKind::WorkShift => {
            effect.npc_stat_deltas.push(StatDelta {
                kind: StatKind::Wealth,
                delta: 1.0,
                source: Some("npc_action".into()),
            });
            effect.npc_stat_deltas.push(StatDelta {
                kind: StatKind::Mood,
                delta: -0.3,
                source: Some("npc_action".into()),
            });
            effect.npc_stat_deltas.push(StatDelta {
                kind: StatKind::Energy,
                delta: -0.5,
                source: Some("npc_action".into()),
            });
            effect.busy_for_ticks = 8;
            effect.label = Some("NPC works extra shift".into());
        }
        NpcActionKind::WithdrawAlone => {
            effect.npc_stat_deltas.push(StatDelta {
                kind: StatKind::Mood,
                delta: 0.2,
                source: Some("npc_action".into()),
            });
            effect.npc_stat_deltas.push(StatDelta {
                kind: StatKind::Energy,
                delta: 0.5,
                source: Some("npc_action".into()),
            });
            effect.memory_tags_for_player = vec!["npc_behavior".into(), "withdrawal".into()];
            effect.busy_for_ticks = 4;
            effect.label = Some("NPC withdraws to be alone".into());
        }
        NpcActionKind::ProvokePlayer => {
            effect.npc_stat_deltas.push(StatDelta {
                kind: StatKind::Mood,
                delta: 0.2,
                source: Some("npc_action".into()),
            });
            effect.player_stat_deltas.push(StatDelta {
                kind: StatKind::Mood,
                delta: -0.8,
                source: Some("npc_action".into()),
            });
            effect.relationship_deltas.push(RelationshipDelta {
                target_id: NpcId(0),
                axis: RelationshipAxis::Affection,
                delta: -0.7,
                source: Some("npc_action".into()),
            });
            effect.relationship_deltas.push(RelationshipDelta {
                target_id: NpcId(0),
                axis: RelationshipAxis::Resentment,
                delta: 0.8,
                source: Some("npc_action".into()),
            });
            effect.memory_tags_for_player = vec![
                "npc_behavior".into(),
                "conflict".into(),
                "npc_provoked".into(),
            ];
            effect.label = Some("NPC provokes player".into());
        }
        NpcActionKind::SelfImprovement => {
            effect.npc_stat_deltas.push(StatDelta {
                kind: StatKind::Wisdom,
                delta: 0.3,
                source: Some("npc_action".into()),
            });
            effect.npc_stat_deltas.push(StatDelta {
                kind: StatKind::Mood,
                delta: -0.1,
                source: Some("npc_action".into()),
            });
            effect.npc_stat_deltas.push(StatDelta {
                kind: StatKind::Energy,
                delta: -0.4,
                source: Some("npc_action".into()),
            });
            effect.busy_for_ticks = 6;
            effect.label = Some("NPC self-improvement effort".into());
        }
        NpcActionKind::Idle => {
            effect.npc_stat_deltas.push(StatDelta {
                kind: StatKind::Mood,
                delta: -0.1,
                source: Some("npc_action".into()),
            });
            effect.label = Some("NPC idle / background life".into());
        }
    }

    effect
}

/// Build a deterministic action instance from a behavior snapshot.
pub fn build_action_instance_from_behavior(
    npc_id: NpcId,
    behavior: &BehaviorSnapshot,
) -> NpcActionInstance {
    let candidates = behavior_to_candidate_actions(behavior.chosen_intent.kind);
    let kind = candidates.first().copied().unwrap_or(NpcActionKind::Idle);
    let effect = base_effect_for_action(kind);
    NpcActionInstance {
        npc_id,
        kind,
        targets_player: behavior.target_player,
        target_npc_id: behavior.target_npc_id,
        effect,
    }
}
