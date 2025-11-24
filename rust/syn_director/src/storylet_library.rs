use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::Path,
};

use crate::{storylet_loader, Storylet, TagBitset};

/// Stable identifier for a storylet within the library.
pub type StoryletId = String;

/// Minimal event context used for storylet eligibility checks.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MoodContext {
    pub multiplier: f32,
}

impl MoodContext {
    pub fn multiplier(&self) -> f32 {
        if self.multiplier == 0.0 { 1.0 } else { self.multiplier }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PersonalityContext {
    pub bias: f32,
}

impl PersonalityContext {
    pub fn bias(&self) -> f32 {
        if self.bias == 0.0 { 1.0 } else { self.bias }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RelationshipVectorContext {
    pub factor: f32,
}

impl RelationshipVectorContext {
    pub fn factor(&self) -> f32 {
        if self.factor == 0.0 { 1.0 } else { self.factor }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NeedsContext {
    #[serde(default)]
    pub level: f32,
}

impl NeedsContext {
    pub fn factor(&self) -> f32 {
        if self.level == 0.0 { 1.0 } else { self.level }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventContext {
    /// Precomputed required tags encoded as a bitset.
    pub required_tags: TagBitset,
    #[serde(default)]
    pub mood: MoodContext,
    #[serde(default)]
    pub personality: PersonalityContext,
    #[serde(default)]
    pub relationship_vector: RelationshipVectorContext,
    #[serde(default)]
    pub needs: NeedsContext,
    #[serde(default)]
    pub memory_echoes: Vec<String>,
    #[serde(default)]
    pub life_stage: Option<syn_core::LifeStage>,
    #[serde(default)]
    pub district_state: Option<String>,
    #[serde(default)]
    pub global_state: std::collections::HashMap<String, bool>,
    #[serde(default)]
    pub tick_index: u64,
    #[serde(default)]
    pub lod_tier: u8,
    #[serde(default)]
    pub storylet_cooldowns: std::collections::HashMap<String, u64>,
    #[serde(default)]
    pub seed: u64,
}

/// Container for all compiled storylets plus a tag index for fast lookup.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StoryletLibrary {
    pub storylets: Vec<Storylet>,
    #[serde(default)]
    pub tag_index: HashMap<TagBitset, Vec<StoryletId>>,
}

impl StoryletLibrary {
    /// Create an empty library.
    pub fn new() -> Self {
        Self {
            storylets: Vec::new(),
            tag_index: HashMap::new(),
        }
    }

    /// Create a library from an existing set of storylets, rebuilding the tag index.
    pub fn from_storylets(storylets: Vec<Storylet>) -> Self {
        let mut lib = Self::new();
        lib.ingest_storylets(storylets);
        lib
    }

    /// Load all storylet JSON files in a folder and compile them into a library.
    pub fn load_from_json_folder(path: &str) -> Self {
        let mut parsed = Vec::new();
        if let Ok(entries) = fs::read_dir(Path::new(path)) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) != Some("json") {
                    continue;
                }
                if let Ok(raw) = fs::read_to_string(&path) {
                    if let Ok(storylet) = storylet_loader::parse_storylet_str(&raw) {
                        parsed.push(storylet);
                    }
                }
            }
        }
        Self::from_storylets(parsed)
    }

    /// Placeholder default loader; integrates with content pipeline when available.
    pub fn load_default() -> Result<Self, ()> {
        Ok(Self::new())
    }

    /// Return storylets eligible for the provided context (bitset-filtered).
    pub fn eligible_for<'a>(&'a self, context: &EventContext) -> Vec<&'a Storylet> {
        if context.required_tags.is_empty() {
            return self.storylets.iter().collect();
        }

        self.tag_index
            .get(&context.required_tags)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.storylets.iter().find(|s| &s.id == id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Rebuild the tag index based on current storylets.
    pub fn rebuild_index(&mut self) {
        let storylets = std::mem::take(&mut self.storylets);
        self.ingest_storylets(storylets);
    }

    fn ingest_storylets(&mut self, storylets: Vec<Storylet>) {
        for storylet in storylets.into_iter() {
            let bitset = storylet.tags;
            self.tag_index
                .entry(bitset)
                .or_default()
                .push(storylet.id.clone());
            self.storylets.push(storylet);
        }
    }
}

pub fn tags_to_bitset(tags: &[String]) -> TagBitset {
    TagBitset::from_tags_slice(tags)
}
