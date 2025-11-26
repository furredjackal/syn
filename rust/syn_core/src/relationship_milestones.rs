use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

use crate::relationship_model::{RelationshipRole, RelationshipVector};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationshipMilestoneKind {
    FriendToRival,
    RivalToAlly,
    StrangerToRomance,
    RomanceCollapse,
    FriendToFamily,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RelationshipMilestoneEvent {
    pub actor_id: u64,
    pub target_id: u64,
    pub kind: RelationshipMilestoneKind,
    pub from_role: String,
    pub to_role: String,
    /// Optional reason string based on GDD cues ("betrayal memory", "shared trauma", etc.).
    #[serde(default)]
    pub reason: String,
    /// Optional source tag (e.g. "storylet:<id>", "drift").
    #[serde(default)]
    pub source: Option<String>,
    /// Optional tick/time index for ordering/debugging.
    #[serde(default)]
    pub tick: Option<u64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct RelationshipMilestoneState {
    #[serde(default)]
    pub last_role: HashMap<(u64, u64), RelationshipRole>,
    #[serde(default)]
    pub queue: VecDeque<RelationshipMilestoneEvent>,
}

impl RelationshipMilestoneState {
    pub fn record_role_for_pair(
        &mut self,
        actor_id: u64,
        target_id: u64,
        new_role: RelationshipRole,
    ) {
        self.last_role.insert((actor_id, target_id), new_role);
    }

    pub fn pop_next(&mut self) -> Option<RelationshipMilestoneEvent> {
        self.queue.pop_front()
    }

    pub fn peek_next(&self) -> Option<&RelationshipMilestoneEvent> {
        self.queue.front()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn evaluate_and_record_milestones_for_pair(
        &mut self,
        actor_id: u64,
        target_id: u64,
        current_rel: &RelationshipVector,
        memory_tags: &[String],
        source: Option<String>,
        tick: Option<u64>,
    ) {
        let new_role = current_rel.role();
        let prev_role = self
            .last_role
            .get(&(actor_id, target_id))
            .copied()
            .unwrap_or(RelationshipRole::Stranger);

        if let Some(kind) = Self::detect_milestone(prev_role, new_role, memory_tags) {
            self.queue.push_back(RelationshipMilestoneEvent {
                actor_id,
                target_id,
                kind,
                from_role: prev_role.to_string(),
                to_role: new_role.to_string(),
                reason: Self::reason_for_milestone(kind, memory_tags),
                source,
                tick,
            });
        }

        self.record_role_for_pair(actor_id, target_id, new_role);
    }

    pub fn detect_milestone(
        prev: RelationshipRole,
        new: RelationshipRole,
        memory_tags: &[String],
    ) -> Option<RelationshipMilestoneKind> {
        let has_tag = |needle: &str| memory_tags.iter().any(|t| t.eq_ignore_ascii_case(needle));

        if matches!(
            prev,
            RelationshipRole::Friend | RelationshipRole::Acquaintance
        ) && matches!(new, RelationshipRole::Rival)
            && (has_tag("betrayal") || has_tag("backstab"))
        {
            return Some(RelationshipMilestoneKind::FriendToRival);
        }

        if matches!(prev, RelationshipRole::Rival)
            && matches!(new, RelationshipRole::Ally | RelationshipRole::Friend)
            && (has_tag("shared_trauma") || has_tag("trauma") || has_tag("crisis_shared"))
        {
            return Some(RelationshipMilestoneKind::RivalToAlly);
        }

        if matches!(
            prev,
            RelationshipRole::Stranger | RelationshipRole::Acquaintance
        ) && matches!(new, RelationshipRole::Romance)
            && (has_tag("chemistry") || has_tag("flirt") || has_tag("romantic_event"))
        {
            return Some(RelationshipMilestoneKind::StrangerToRomance);
        }

        if matches!(prev, RelationshipRole::Romance)
            && matches!(
                new,
                RelationshipRole::Stranger
                    | RelationshipRole::Rival
                    | RelationshipRole::Acquaintance
            )
            && (has_tag("betrayal") || has_tag("trust_break") || has_tag("spiral"))
        {
            return Some(RelationshipMilestoneKind::RomanceCollapse);
        }

        if matches!(prev, RelationshipRole::Friend | RelationshipRole::Ally)
            && matches!(new, RelationshipRole::Family)
            && (has_tag("shared_home") || has_tag("long_term") || has_tag("life_event"))
        {
            return Some(RelationshipMilestoneKind::FriendToFamily);
        }

        None
    }

    fn reason_for_milestone(kind: RelationshipMilestoneKind, _memory_tags: &[String]) -> String {
        match kind {
            RelationshipMilestoneKind::FriendToRival => "high resentment + betrayal memory".into(),
            RelationshipMilestoneKind::RivalToAlly => "shared trauma drew them together".into(),
            RelationshipMilestoneKind::StrangerToRomance => {
                "attraction spike + shared event".into()
            }
            RelationshipMilestoneKind::RomanceCollapse => "trust failure spiral".into(),
            RelationshipMilestoneKind::FriendToFamily => {
                "long-term stability + shared life events".into()
            }
        }
    }
}
