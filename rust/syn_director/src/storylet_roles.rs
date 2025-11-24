use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use crate::{EventContext, NpcId, StoryletRole};

pub type RoleSlot = StoryletRole;
pub type RoleAssignment = HashMap<String, NpcId>;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RoleScoring {
    #[serde(default)]
    pub weights: HashMap<String, f32>,
}

/// Role set for a storylet (GDD 3.16.3).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StoryletRoles {
    #[serde(default)]
    pub required: Vec<RoleSlot>,
    #[serde(default)]
    pub scoring: RoleScoring,
}

impl StoryletRoles {
    pub fn new(required: Vec<RoleSlot>) -> Self {
        StoryletRoles {
            required,
            scoring: RoleScoring::default(),
        }
    }

    /// Assign roles based on context (placeholder).
    pub fn assign(&self, _ctx: &EventContext) -> Option<RoleAssignment> {
        // Stub: integrate role selection logic when available.
        Some(RoleAssignment::new())
    }
}

impl From<Vec<RoleSlot>> for StoryletRoles {
    fn from(required: Vec<RoleSlot>) -> Self {
        StoryletRoles::new(required)
    }
}

impl FromIterator<RoleSlot> for StoryletRoles {
    fn from_iter<T: IntoIterator<Item = RoleSlot>>(iter: T) -> Self {
        let required: Vec<RoleSlot> = iter.into_iter().collect();
        StoryletRoles::new(required)
    }
}

impl Deref for StoryletRoles {
    type Target = Vec<RoleSlot>;
    fn deref(&self) -> &Self::Target {
        &self.required
    }
}

impl DerefMut for StoryletRoles {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.required
    }
}

impl<'a> IntoIterator for &'a StoryletRoles {
    type Item = &'a RoleSlot;
    type IntoIter = std::slice::Iter<'a, RoleSlot>;
    fn into_iter(self) -> Self::IntoIter {
        self.required.iter()
    }
}

impl<'a> IntoIterator for &'a mut StoryletRoles {
    type Item = &'a mut RoleSlot;
    type IntoIter = std::slice::IterMut<'a, RoleSlot>;
    fn into_iter(self) -> Self::IntoIter {
        self.required.iter_mut()
    }
}
