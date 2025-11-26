//! Abstraction layer for accessing compiled storylet libraries.
//!
//! This module provides a trait-based interface for the Event Director to query storylets
//! without being coupled to a specific storage backend (in-memory or memory-mapped).
//! This allows `syn_director` to work with both `StoryletLibrary` (owned) and
//! `MappedStoryletLibrary` (memory-mapped via `syn_storylets`).

use syn_storylets::library::{CompiledStorylet, StoryletKey};
use syn_storylets::{LifeStage, StoryDomain, StoryletId, Tag};

/// A reference to a compiled storylet for querying purposes.
///
/// This is a lightweight reference type that can be either a borrowed pointer or a copy,
/// depending on the underlying storage implementation.
pub type CompiledStoryletRef<'a> = &'a CompiledStorylet;

/// Trait for accessing compiled storylets from any storage backend.
///
/// This abstracts over `StoryletLibrary` (in-memory) and `MappedStoryletLibrary` (mapped),
/// allowing the Event Director to work with either seamlessly.
///
/// # Example
///
/// ```ignore
/// use syn_director::storylet_source::StoryletSource;
/// use syn_storylets::library::StoryletLibrary;
/// use syn_storylets::StoryletId;
///
/// let library = StoryletLibrary::new();
/// // ...populate library...
///
/// // Can query via the trait
/// if let Some(story) = library.get_storylet_by_id(&StoryletId::new("my.story")) {
///     println!("Found: {}", story.name);
/// }
/// ```
pub trait StoryletSource {
    /// Get a storylet by its string ID.
    fn get_storylet_by_id(&self, id: &StoryletId) -> Option<CompiledStoryletRef<'_>>;

    /// Get a storylet by its compiled key.
    fn get_storylet_by_key(&self, key: StoryletKey) -> Option<CompiledStoryletRef<'_>>;

    /// Get all storylet keys with a given tag.
    fn candidates_for_tag(&self, tag: &Tag) -> Vec<StoryletKey>;

    /// Get all storylet keys for a given life stage.
    fn candidates_for_life_stage(&self, stage: LifeStage) -> Vec<StoryletKey>;

    /// Get all storylet keys for a given domain.
    fn candidates_for_domain(&self, domain: StoryDomain) -> Vec<StoryletKey>;

    /// Iterate over all storylets in the library.
    fn iter_all_storylets(&self) -> Box<dyn Iterator<Item = CompiledStoryletRef<'_>> + '_>;

    /// Get the total count of storylets in the library.
    fn total_count(&self) -> u32;
}

/// Implementation of `StoryletSource` for in-memory `StoryletLibrary`.
impl StoryletSource for syn_storylets::library::StoryletLibrary {
    fn get_storylet_by_id(&self, id: &StoryletId) -> Option<CompiledStoryletRef<'_>> {
        self.get_by_id(id)
    }

    fn get_storylet_by_key(&self, key: StoryletKey) -> Option<CompiledStoryletRef<'_>> {
        self.get_by_key(key)
    }

    fn candidates_for_tag(&self, tag: &Tag) -> Vec<StoryletKey> {
        self.tag_index
            .get(tag)
            .map(|keys| keys.clone())
            .unwrap_or_default()
    }

    fn candidates_for_life_stage(&self, stage: LifeStage) -> Vec<StoryletKey> {
        self.life_stage_index
            .get(&stage)
            .map(|keys| keys.clone())
            .unwrap_or_default()
    }

    fn candidates_for_domain(&self, domain: StoryDomain) -> Vec<StoryletKey> {
        self.domain_index
            .get(&domain)
            .map(|keys| keys.clone())
            .unwrap_or_default()
    }

    fn iter_all_storylets(&self) -> Box<dyn Iterator<Item = CompiledStoryletRef<'_>> + '_> {
        Box::new(self.storylets.iter())
    }

    fn total_count(&self) -> u32 {
        self.total_count
    }
}

/// Implementation of `StoryletSource` for memory-mapped `MappedStoryletLibrary`.
#[cfg(feature = "mmap")]
pub mod mapped_impl {
    use super::*;

    impl StoryletSource for syn_storylets::mapped::MappedStoryletLibrary {
        fn get_storylet_by_id(&self, id: &StoryletId) -> Option<CompiledStoryletRef<'_>> {
            self.get_by_id(id)
        }

        fn get_storylet_by_key(&self, key: StoryletKey) -> Option<CompiledStoryletRef<'_>> {
            self.get_by_key(key)
        }

        fn candidates_for_tag(&self, tag: &Tag) -> Vec<StoryletKey> {
            self.tag_index
                .get(tag)
                .map(|keys| keys.clone())
                .unwrap_or_default()
        }

        fn candidates_for_life_stage(&self, stage: LifeStage) -> Vec<StoryletKey> {
            self.life_stage_index
                .get(&stage)
                .map(|keys| keys.clone())
                .unwrap_or_default()
        }

        fn candidates_for_domain(&self, domain: StoryDomain) -> Vec<StoryletKey> {
            self.domain_index
                .get(&domain)
                .map(|keys| keys.clone())
                .unwrap_or_default()
        }

        fn iter_all_storylets(&self) -> Box<dyn Iterator<Item = CompiledStoryletRef<'_>> + '_> {
            Box::new(self.iter())
        }

        fn total_count(&self) -> u32 {
            self.total_count()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn_storylets::library::{CompiledStorylet, StoryletLibrary};
    use syn_storylets::{Cooldowns, Outcome, Prerequisites};

    /// Helper to create a test storylet.
    fn create_test_storylet(
        id: &str,
        key: u32,
        tags: Vec<&str>,
        domain: StoryDomain,
        life_stage: LifeStage,
    ) -> CompiledStorylet {
        CompiledStorylet {
            id: StoryletId::new(id),
            key: StoryletKey(key),
            name: format!("Test Story {}", id),
            description: Some(format!("Description for {}", id)),
            tags: tags.iter().map(|t| Tag::new(t.to_string())).collect(),
            domain,
            life_stage,
            heat: 5,
            weight: 1.0,
            roles: vec![],
            prerequisites: Prerequisites::default(),
            cooldowns: Cooldowns::default(),
            outcomes: Outcome::default(),
            follow_ups_resolved: vec![],
        }
    }

    #[test]
    fn test_storylet_source_in_memory() {
        let mut library = StoryletLibrary::new();

        let tag = Tag::new("romance");
        let compiled = create_test_storylet(
            "test.story",
            0,
            vec!["romance"],
            StoryDomain::Romance,
            LifeStage::Adult,
        );

        library
            .id_to_key
            .insert(compiled.id.clone(), compiled.key);
        library.tag_index.insert(tag.clone(), vec![compiled.key]);
        library
            .life_stage_index
            .insert(LifeStage::Adult, vec![compiled.key]);
        library
            .domain_index
            .insert(StoryDomain::Romance, vec![compiled.key]);
        library.storylets.push(compiled);
        library.total_count = 1;

        // Test through trait
        let source: &dyn StoryletSource = &library;

        assert_eq!(source.total_count(), 1);
        assert!(source.get_storylet_by_id(&StoryletId::new("test.story")).is_some());
        assert!(source.get_storylet_by_key(StoryletKey(0)).is_some());
        assert_eq!(source.candidates_for_tag(&tag).len(), 1);
        assert_eq!(source.candidates_for_domain(StoryDomain::Romance).len(), 1);
        assert_eq!(source.candidates_for_life_stage(LifeStage::Adult).len(), 1);
        assert_eq!(source.iter_all_storylets().count(), 1);
    }

    #[test]
    fn test_multiple_storylets_and_tags() {
        let mut library = StoryletLibrary::new();

        // Create three storylets with different tags
        let romance_tag = Tag::new("romance");
        let conflict_tag = Tag::new("conflict");

        let story1 = create_test_storylet(
            "romance1",
            0,
            vec!["romance"],
            StoryDomain::Romance,
            LifeStage::Adult,
        );
        let story2 = create_test_storylet(
            "romance2",
            1,
            vec!["romance"],
            StoryDomain::Romance,
            LifeStage::Teen,
        );
        let story3 = create_test_storylet(
            "conflict1",
            2,
            vec!["conflict"],
            StoryDomain::Conflict,
            LifeStage::Adult,
        );

        // Populate library indexes
        library
            .id_to_key
            .insert(story1.id.clone(), story1.key);
        library
            .id_to_key
            .insert(story2.id.clone(), story2.key);
        library
            .id_to_key
            .insert(story3.id.clone(), story3.key);

        library
            .tag_index
            .insert(romance_tag.clone(), vec![story1.key, story2.key]);
        library
            .tag_index
            .insert(conflict_tag.clone(), vec![story3.key]);

        library
            .life_stage_index
            .insert(LifeStage::Adult, vec![story1.key, story3.key]);
        library
            .life_stage_index
            .insert(LifeStage::Teen, vec![story2.key]);

        library
            .domain_index
            .insert(StoryDomain::Romance, vec![story1.key, story2.key]);
        library
            .domain_index
            .insert(StoryDomain::Conflict, vec![story3.key]);

        library.storylets.push(story1);
        library.storylets.push(story2);
        library.storylets.push(story3);
        library.total_count = 3;

        let source: &dyn StoryletSource = &library;

        // Test total count
        assert_eq!(source.total_count(), 3);

        // Test get by ID
        assert!(source
            .get_storylet_by_id(&StoryletId::new("romance1"))
            .is_some());
        assert!(source
            .get_storylet_by_id(&StoryletId::new("conflict1"))
            .is_some());

        // Test get by key
        assert!(source.get_storylet_by_key(StoryletKey(0)).is_some());
        assert!(source.get_storylet_by_key(StoryletKey(2)).is_some());
        assert!(source.get_storylet_by_key(StoryletKey(999)).is_none());

        // Test candidates by tag
        let romance_candidates = source.candidates_for_tag(&romance_tag);
        assert_eq!(romance_candidates.len(), 2);
        assert!(romance_candidates.contains(&StoryletKey(0)));
        assert!(romance_candidates.contains(&StoryletKey(1)));

        let conflict_candidates = source.candidates_for_tag(&conflict_tag);
        assert_eq!(conflict_candidates.len(), 1);
        assert!(conflict_candidates.contains(&StoryletKey(2)));

        // Test candidates by life stage
        let adult_candidates = source.candidates_for_life_stage(LifeStage::Adult);
        assert_eq!(adult_candidates.len(), 2);

        let teen_candidates = source.candidates_for_life_stage(LifeStage::Teen);
        assert_eq!(teen_candidates.len(), 1);
        assert!(teen_candidates.contains(&StoryletKey(1)));

        // Test candidates by domain
        let romance_domain = source.candidates_for_domain(StoryDomain::Romance);
        assert_eq!(romance_domain.len(), 2);

        let conflict_domain = source.candidates_for_domain(StoryDomain::Conflict);
        assert_eq!(conflict_domain.len(), 1);

        // Test iteration
        let all_count = source.iter_all_storylets().count();
        assert_eq!(all_count, 3);
    }

    #[test]
    fn test_empty_library() {
        let library = StoryletLibrary::new();
        let source: &dyn StoryletSource = &library;

        assert_eq!(source.total_count(), 0);
        assert!(source
            .get_storylet_by_id(&StoryletId::new("nonexistent"))
            .is_none());
        assert!(source.get_storylet_by_key(StoryletKey(0)).is_none());
        assert_eq!(source.candidates_for_tag(&Tag::new("any")).len(), 0);
        assert_eq!(source.candidates_for_domain(StoryDomain::Romance).len(), 0);
        assert_eq!(source.candidates_for_life_stage(LifeStage::Adult).len(), 0);
        assert_eq!(source.iter_all_storylets().count(), 0);
    }

    #[test]
    fn test_nonexistent_lookups() {
        let mut library = StoryletLibrary::new();
        let story = create_test_storylet(
            "test",
            0,
            vec!["tag1"],
            StoryDomain::Romance,
            LifeStage::Adult,
        );

        library
            .id_to_key
            .insert(story.id.clone(), story.key);
        library
            .domain_index
            .insert(StoryDomain::Romance, vec![story.key]);
        library.storylets.push(story);
        library.total_count = 1;

        let source: &dyn StoryletSource = &library;

        // Lookups that should fail
        assert!(source
            .get_storylet_by_id(&StoryletId::new("nonexistent"))
            .is_none());
        assert!(source.get_storylet_by_key(StoryletKey(999)).is_none());
        assert_eq!(source.candidates_for_tag(&Tag::new("nonexistent")).len(), 0);
        assert_eq!(source.candidates_for_life_stage(LifeStage::Child).len(), 0);
    }
}
