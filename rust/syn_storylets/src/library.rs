//! Compiled storylet library with precomputed indexes.
//!
//! This module defines the runtime-efficient representation of a compiled storylet library.
//! It includes dense key mapping, tag/domain/life-stage indexes for fast querying, and
//! resolved follow-up references.

use crate::{
    Cooldowns, LifeStage, Outcome, Prerequisites, RoleSlot, StoryDomain,
    StoryletId, Tag,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A dense numeric key for a storylet, assigned during compilation.
///
/// Enables fast array access and efficient reference resolution instead of string IDs.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct StoryletKey(pub u32);

/// A storylet after compilation: optimized for runtime access.
///
/// Contains all necessary data for the Event Director to query and execute the storylet,
/// with follow-up IDs resolved to `StoryletKey` references.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompiledStorylet {
    /// Original storylet ID string (useful for debugging).
    pub id: StoryletId,
    /// Assigned compilation key for fast array indexing.
    pub key: StoryletKey,
    /// Human-readable name.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// Semantic tags.
    pub tags: Vec<Tag>,
    /// Primary narrative domain.
    pub domain: StoryDomain,
    /// Target life stage.
    pub life_stage: LifeStage,
    /// Narrative intensity (0–10).
    pub heat: u8,
    /// Base selection weight (> 0.0).
    pub weight: f32,
    /// Named roles for actors in this storylet.
    pub roles: Vec<RoleSlot>,
    /// All prerequisites.
    pub prerequisites: Prerequisites,
    /// Cooldown settings.
    pub cooldowns: Cooldowns,
    /// Outcomes.
    pub outcomes: Outcome,
    /// Follow-ups with IDs resolved to keys (where available).
    pub follow_ups_resolved: Vec<ResolvedFollowUp>,
}

/// A follow-up storylet with resolved key reference.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResolvedFollowUp {
    /// The resolved storylet key, if found in the library.
    pub target_key: Option<StoryletKey>,
    /// Delay in ticks.
    pub delay_ticks: u32,
    /// Conditional flag requirement.
    pub conditional_on_flag: Option<String>,
}

/// The complete compiled storylet library with all precomputed indexes.
///
/// This structure is optimized for fast runtime queries by domain, tag, life stage, etc.
/// It's designed to be serializable to a compact binary format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryletLibrary {
    /// All compiled storylets indexed by their key.
    pub storylets: Vec<CompiledStorylet>,
    /// Maps storylet ID to its compiled key.
    pub id_to_key: HashMap<StoryletId, StoryletKey>,
    /// Maps each tag to all storylets that have it.
    pub tag_index: HashMap<Tag, Vec<StoryletKey>>,
    /// Maps each life stage to all eligible storylets.
    pub life_stage_index: HashMap<LifeStage, Vec<StoryletKey>>,
    /// Maps each domain to all storylets in it.
    pub domain_index: HashMap<StoryDomain, Vec<StoryletKey>>,
    /// Metadata: total count of compiled storylets.
    pub total_count: u32,
}

impl StoryletLibrary {
    /// Create a new empty library.
    pub fn new() -> Self {
        StoryletLibrary {
            storylets: Vec::new(),
            id_to_key: HashMap::new(),
            tag_index: HashMap::new(),
            life_stage_index: HashMap::new(),
            domain_index: HashMap::new(),
            total_count: 0,
        }
    }

    /// Look up a storylet by its string ID.
    pub fn get_by_id(&self, id: &StoryletId) -> Option<&CompiledStorylet> {
        self.id_to_key
            .get(id)
            .and_then(|key| self.get_by_key(*key))
    }

    /// Look up a storylet by its compiled key.
    pub fn get_by_key(&self, key: StoryletKey) -> Option<&CompiledStorylet> {
        if (key.0 as usize) < self.storylets.len() {
            Some(&self.storylets[key.0 as usize])
        } else {
            None
        }
    }

    /// Get all storylets with a given tag.
    pub fn get_by_tag(&self, tag: &Tag) -> Vec<&CompiledStorylet> {
        self.tag_index
            .get(tag)
            .map(|keys| {
                keys.iter()
                    .filter_map(|key| self.get_by_key(*key))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all storylets for a given life stage.
    pub fn get_by_life_stage(&self, stage: LifeStage) -> Vec<&CompiledStorylet> {
        self.life_stage_index
            .get(&stage)
            .map(|keys| {
                keys.iter()
                    .filter_map(|key| self.get_by_key(*key))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all storylets for a given domain.
    pub fn get_by_domain(&self, domain: StoryDomain) -> Vec<&CompiledStorylet> {
        self.domain_index
            .get(&domain)
            .map(|keys| {
                keys.iter()
                    .filter_map(|key| self.get_by_key(*key))
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl Default for StoryletLibrary {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_lookup_by_id() {
        let mut library = StoryletLibrary::new();

        let compiled = CompiledStorylet {
            id: StoryletId::new("test.story"),
            key: StoryletKey(0),
            name: "Test".to_string(),
            description: None,
            tags: vec![],
            domain: StoryDomain::Romance,
            life_stage: LifeStage::Adult,
            heat: 5,
            weight: 1.0,
            roles: vec![],
            prerequisites: crate::Prerequisites::default(),
            cooldowns: crate::Cooldowns::default(),
            outcomes: crate::Outcome::default(),
            follow_ups_resolved: vec![],
        };

        library.id_to_key.insert(compiled.id.clone(), compiled.key);
        library.storylets.push(compiled.clone());
        library.total_count = 1;

        let found = library.get_by_id(&StoryletId::new("test.story"));
        assert!(found.is_some());
        assert_eq!(found.unwrap().id.0, "test.story");
    }

    #[test]
    fn test_library_tag_index() {
        let mut library = StoryletLibrary::new();

        let tag = Tag::new("romance");
        let compiled = CompiledStorylet {
            id: StoryletId::new("romance.story"),
            key: StoryletKey(0),
            name: "Romance".to_string(),
            description: None,
            tags: vec![tag.clone()],
            domain: StoryDomain::Romance,
            life_stage: LifeStage::Adult,
            heat: 5,
            weight: 1.0,
            roles: vec![],
            prerequisites: crate::Prerequisites::default(),
            cooldowns: crate::Cooldowns::default(),
            outcomes: crate::Outcome::default(),
            follow_ups_resolved: vec![],
        };

        library
            .tag_index
            .insert(tag.clone(), vec![compiled.key]);
        library.storylets.push(compiled);
        library.total_count = 1;

        let results = library.get_by_tag(&tag);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id.0, "romance.story");
    }

    #[test]
    fn test_library_domain_index() {
        let mut library = StoryletLibrary::new();

        let compiled = CompiledStorylet {
            id: StoryletId::new("career.promotion"),
            key: StoryletKey(0),
            name: "Promotion".to_string(),
            description: None,
            tags: vec![],
            domain: StoryDomain::Career,
            life_stage: LifeStage::Adult,
            heat: 5,
            weight: 1.0,
            roles: vec![],
            prerequisites: crate::Prerequisites::default(),
            cooldowns: crate::Cooldowns::default(),
            outcomes: crate::Outcome::default(),
            follow_ups_resolved: vec![],
        };

        library
            .domain_index
            .insert(StoryDomain::Career, vec![compiled.key]);
        library.storylets.push(compiled);
        library.total_count = 1;

        let results = library.get_by_domain(StoryDomain::Career);
        assert_eq!(results.len(), 1);
    }
}
