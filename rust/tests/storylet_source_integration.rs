/// Integration test for StoryletSource and library loading.
///
/// This test demonstrates:
/// 1. Creating a compiled storylet library in memory
/// 2. Loading it via the StoryletSource trait
/// 3. Querying storylets by various indexes
/// 4. Serializing and deserializing the library to/from disk

#[cfg(test)]
mod storylet_source_integration {
    use syn_director::StoryletSource;
    use syn_storylets::library::{CompiledStorylet, StoryletLibrary, StoryletKey};
    use syn_storylets::{Cooldowns, Outcome, Prerequisites, StoryDomain, LifeStage, StoryletId, Tag};
    use tempfile::TempDir;

    /// Helper to create a test storylet with various metadata.
    fn create_test_storylet(
        id: &str,
        key: u32,
        tags: Vec<&str>,
        domain: StoryDomain,
        life_stage: LifeStage,
        heat: u8,
    ) -> CompiledStorylet {
        CompiledStorylet {
            id: StoryletId::new(id),
            key: StoryletKey(key),
            name: format!("Story: {}", id),
            description: Some(format!("A story about {}", id)),
            tags: tags.iter().map(|t| Tag::new(t.to_string())).collect(),
            domain,
            life_stage,
            heat,
            weight: 1.0,
            prerequisites: Prerequisites::default(),
            cooldowns: Cooldowns::default(),
            outcomes: Outcome::default(),
            follow_ups_resolved: vec![],
        }
    }

    #[test]
    fn test_storylet_source_basic_queries() {
        let mut library = StoryletLibrary::new();

        // Create a diverse set of storylets
        let stories = vec![
            create_test_storylet("romance_first_kiss", 0, vec!["romance", "adult"], 
                                StoryDomain::Romance, LifeStage::Adult, 7),
            create_test_storylet("romance_breakup", 1, vec!["romance", "trauma"], 
                                StoryDomain::Romance, LifeStage::Adult, 8),
            create_test_storylet("teen_crush", 2, vec!["romance"], 
                                StoryDomain::Romance, LifeStage::Teen, 4),
            create_test_storylet("career_promotion", 3, vec!["career"], 
                                StoryDomain::Career, LifeStage::Adult, 5),
            create_test_storylet("family_reunion", 4, vec!["family"], 
                                StoryDomain::Family, LifeStage::Adult, 3),
            create_test_storylet("conflict_argument", 5, vec!["conflict", "trauma"], 
                                StoryDomain::Conflict, LifeStage::Adult, 6),
        ];

        // Populate the library indexes
        for story in &stories {
            library.id_to_key.insert(story.id.clone(), story.key);
            
            for tag in &story.tags {
                library.tag_index.entry(tag.clone())
                    .or_insert_with(Vec::new)
                    .push(story.key);
            }
            
            library.life_stage_index.entry(story.life_stage)
                .or_insert_with(Vec::new)
                .push(story.key);
            
            library.domain_index.entry(story.domain)
                .or_insert_with(Vec::new)
                .push(story.key);
        }

        library.storylets = stories;
        library.total_count = 6;

        let source: &dyn StoryletSource = &library;

        // Test: total count
        assert_eq!(source.total_count(), 6);

        // Test: query by ID
        assert_eq!(
            source.get_storylet_by_id(&StoryletId::new("romance_first_kiss")).unwrap().name,
            "Story: romance_first_kiss"
        );

        // Test: query by key
        assert_eq!(
            source.get_storylet_by_key(StoryletKey(3)).unwrap().name,
            "Story: career_promotion"
        );

        // Test: candidates for tag (romance)
        let romance_candidates = source.candidates_for_tag(&Tag::new("romance"));
        assert_eq!(romance_candidates.len(), 3); // first_kiss, breakup, teen_crush
        assert!(romance_candidates.contains(&StoryletKey(0)));
        assert!(romance_candidates.contains(&StoryletKey(1)));
        assert!(romance_candidates.contains(&StoryletKey(2)));

        // Test: candidates for tag (trauma)
        let trauma_candidates = source.candidates_for_tag(&Tag::new("trauma"));
        assert_eq!(trauma_candidates.len(), 2); // breakup, conflict_argument
        assert!(trauma_candidates.contains(&StoryletKey(1)));
        assert!(trauma_candidates.contains(&StoryletKey(5)));

        // Test: candidates for life stage
        let adult_candidates = source.candidates_for_life_stage(LifeStage::Adult);
        assert_eq!(adult_candidates.len(), 5); // all except teen_crush

        let teen_candidates = source.candidates_for_life_stage(LifeStage::Teen);
        assert_eq!(teen_candidates.len(), 1);
        assert_eq!(teen_candidates[0], StoryletKey(2));

        // Test: candidates for domain
        let romance_domain = source.candidates_for_domain(StoryDomain::Romance);
        assert_eq!(romance_domain.len(), 3);

        let career_domain = source.candidates_for_domain(StoryDomain::Career);
        assert_eq!(career_domain.len(), 1);

        // Test: iteration
        let all_stories: Vec<_> = source.iter_all_storylets().collect();
        assert_eq!(all_stories.len(), 6);
    }

    #[test]
    fn test_storylet_library_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_library.bin");

        // Create and populate library
        let mut library = StoryletLibrary::new();
        
        let stories = vec![
            create_test_storylet("story1", 0, vec!["tag1"], StoryDomain::Romance, LifeStage::Adult, 5),
            create_test_storylet("story2", 1, vec!["tag2"], StoryDomain::Career, LifeStage::Adult, 6),
            create_test_storylet("story3", 2, vec!["tag1", "tag2"], StoryDomain::Family, LifeStage::Teen, 3),
        ];

        for story in &stories {
            library.id_to_key.insert(story.id.clone(), story.key);
            
            for tag in &story.tags {
                library.tag_index.entry(tag.clone())
                    .or_insert_with(Vec::new)
                    .push(story.key);
            }
            
            library.life_stage_index.entry(story.life_stage)
                .or_insert_with(Vec::new)
                .push(story.key);
            
            library.domain_index.entry(story.domain)
                .or_insert_with(Vec::new)
                .push(story.key);
        }

        library.storylets = stories;
        library.total_count = 3;

        // Serialize to disk
        library.write_to_file(&file_path).expect("Failed to write library");
        assert!(file_path.exists(), "Library file was not created");

        // Deserialize from disk
        let loaded_library = StoryletLibrary::read_from_file(&file_path)
            .expect("Failed to read library");

        let source: &dyn StoryletSource = &loaded_library;

        // Verify the loaded library has the same content
        assert_eq!(source.total_count(), 3);
        assert!(source.get_storylet_by_id(&StoryletId::new("story1")).is_some());
        assert!(source.get_storylet_by_id(&StoryletId::new("story2")).is_some());
        assert!(source.get_storylet_by_id(&StoryletId::new("story3")).is_some());
        
        let tag1_stories = source.candidates_for_tag(&Tag::new("tag1"));
        assert_eq!(tag1_stories.len(), 2);

        let tag2_stories = source.candidates_for_tag(&Tag::new("tag2"));
        assert_eq!(tag2_stories.len(), 2);
    }

    #[test]
    fn test_empty_library_through_source() {
        let library = StoryletLibrary::new();
        let source: &dyn StoryletSource = &library;

        assert_eq!(source.total_count(), 0);
        assert!(source.get_storylet_by_id(&StoryletId::new("nonexistent")).is_none());
        assert_eq!(source.candidates_for_tag(&Tag::new("any")).len(), 0);
        assert_eq!(source.candidates_for_domain(StoryDomain::Romance).len(), 0);
        assert_eq!(source.iter_all_storylets().count(), 0);
    }

    #[test]
    fn test_storylet_filtering_by_multiple_criteria() {
        let mut library = StoryletLibrary::new();

        // Create storylets with overlapping tags and domains
        let stories = vec![
            create_test_storylet("romance_adult", 0, vec!["romance", "adult"], 
                                StoryDomain::Romance, LifeStage::Adult, 7),
            create_test_storylet("romance_teen", 1, vec!["romance", "teen_appropriate"], 
                                StoryDomain::Romance, LifeStage::Teen, 4),
            create_test_storylet("conflict_adult", 2, vec!["conflict", "adult"], 
                                StoryDomain::Conflict, LifeStage::Adult, 8),
            create_test_storylet("adventure_adult", 3, vec!["adventure", "adult"], 
                                StoryDomain::Adventure, LifeStage::Adult, 6),
        ];

        for story in &stories {
            library.id_to_key.insert(story.id.clone(), story.key);
            
            for tag in &story.tags {
                library.tag_index.entry(tag.clone())
                    .or_insert_with(Vec::new)
                    .push(story.key);
            }
            
            library.life_stage_index.entry(story.life_stage)
                .or_insert_with(Vec::new)
                .push(story.key);
            
            library.domain_index.entry(story.domain)
                .or_insert_with(Vec::new)
                .push(story.key);
        }

        library.storylets = stories;
        library.total_count = 4;

        let source: &dyn StoryletSource = &library;

        // Test complex filtering scenarios
        
        // All adult-tagged stories
        let adult_tagged = source.candidates_for_tag(&Tag::new("adult"));
        assert_eq!(adult_tagged.len(), 3);

        // All romance stories
        let romance_stories = source.candidates_for_domain(StoryDomain::Romance);
        assert_eq!(romance_stories.len(), 2);

        // All adult life stage stories
        let adult_life_stage = source.candidates_for_life_stage(LifeStage::Adult);
        assert_eq!(adult_life_stage.len(), 3);

        // Teen life stage stories
        let teen_life_stage = source.candidates_for_life_stage(LifeStage::Teen);
        assert_eq!(teen_life_stage.len(), 1);
        assert_eq!(teen_life_stage[0], StoryletKey(1));

        // Verify iteration includes all
        let all_count = source.iter_all_storylets().count();
        assert_eq!(all_count, 4);
    }
}
