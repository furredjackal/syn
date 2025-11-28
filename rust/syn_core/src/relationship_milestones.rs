//! Relationship Milestones System
//!
//! Tracks significant relationship role transitions (e.g., Friend→Rival, Stranger→Romance).
//! These milestones are narrative-significant events that can trigger storylets and achievements.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

use crate::relationship_model::{RelationshipRole, RelationshipVector};

/// Types of significant relationship role transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationshipMilestoneKind {
    /// Friend became a Rival (betrayal, conflict).
    FriendToRival,
    /// Rival became an Ally (shared crisis, redemption).
    RivalToAlly,
    /// Stranger became a Romance (attraction sparked).
    StrangerToRomance,
    /// Romance ended (breakup, betrayal).
    RomanceCollapse,
    /// Friend became like Family (deep bond).
    FriendToFamily,
}

/// A milestone event recording a significant role transition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RelationshipMilestoneEvent {
    /// NPC whose relationship changed.
    pub actor_id: u64,
    /// NPC the relationship is with.
    pub target_id: u64,
    /// Type of milestone.
    pub kind: RelationshipMilestoneKind,
    /// Previous role label.
    pub from_role: String,
    /// New role label.
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

/// State for tracking relationship milestones.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct RelationshipMilestoneState {
    /// Last known role for each (actor, target) pair.
    #[serde(default)]
    pub last_role: HashMap<(u64, u64), RelationshipRole>,
    /// FIFO queue of recent milestone events.
    #[serde(default)]
    pub queue: VecDeque<RelationshipMilestoneEvent>,
}

impl RelationshipMilestoneState {
    /// Record the current role for a relationship pair.
    pub fn record_role_for_pair(
        &mut self,
        actor_id: u64,
        target_id: u64,
        new_role: RelationshipRole,
    ) {
        self.last_role.insert((actor_id, target_id), new_role);
    }

    /// Pop the next milestone event from the queue.
    pub fn pop_next(&mut self) -> Option<RelationshipMilestoneEvent> {
        self.queue.pop_front()
    }

    /// Peek at the next milestone event without removing it.
    pub fn peek_next(&self) -> Option<&RelationshipMilestoneEvent> {
        self.queue.front()
    }

    /// Evaluate a relationship for milestones and record any that occurred.
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

    /// Detect if a role transition constitutes a milestone based on memory tags.
    ///
    /// Returns the milestone kind if the transition is significant, None otherwise.
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

    /// Get a human-readable reason string for a milestone kind.
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
