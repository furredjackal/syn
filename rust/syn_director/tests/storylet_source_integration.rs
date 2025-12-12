/// Integration test for StoryletSource and library loading in syn_director.
///
/// This test demonstrates:
/// 1. Compiling real storylet JSON files into a library
/// 2. Loading the compiled binary library
/// 3. Querying storylets by various indexes
/// 4. Serializing and deserializing the library to/from disk

use syn_director::StoryletSource;
use syn_storylets::library::{CompiledStorylet, StoryletLibrary, StoryletKey};
use syn_storylets::{Cooldowns, Outcome, Prerequisites, StoryDomain, LifeStage, StoryletId, Tag, RoleSlot};
use syn_storylets::compiler::StoryletCompiler;
use syn_storylets::validation::default_storylet_validator;
use tempfile::TempDir;
use std::path::PathBuf;

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
        roles: vec![],  // No roles for these simple test stories
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
fn test_compile_real_storylets() {
    // Locate the real storylets directory
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let project_root = manifest_dir.parent().unwrap().parent().unwrap();
    let storylets_dir = project_root.join("storylets");
    
    if !storylets_dir.exists() {
        eprintln!("Warning: storylets directory not found at {:?}, skipping test", storylets_dir);
        return;
    }
    
    // Compile storylets from the real JSON files
    let validator = default_storylet_validator();
    let compiler = StoryletCompiler::new(validator);
    
    let library = compiler.compile_from_dir(&storylets_dir)
        .expect("Failed to compile real storylets");
    
    // Verify the library has content
    assert!(library.total_count > 0, "Expected at least one storylet in the library");
    
    println!("Compiled {} storylets from {:?}", library.total_count, storylets_dir);
    println!("  - {} unique tags", library.tag_index.len());
    println!("  - {} life stages", library.life_stage_index.len());
    println!("  - {} domains", library.domain_index.len());
    
    // Test querying through StoryletSource trait
    let source: &dyn StoryletSource = &library;
    
    // Test iteration
    let all_storylets: Vec<_> = source.iter_all_storylets().collect();
    assert_eq!(all_storylets.len() as u32, library.total_count);
    
    // Test tag queries for common tags
    for storylet in &all_storylets {
        for tag in &storylet.tags {
            let candidates = source.candidates_for_tag(tag);
            assert!(!candidates.is_empty(), "Tag {:?} should have candidates", tag);
            assert!(candidates.contains(&storylet.key), 
                "Storylet {:?} should be in candidates for its own tag {:?}", storylet.id, tag);
        }
    }
    
    // Test domain queries
    for domain in [StoryDomain::Romance, StoryDomain::Career, StoryDomain::Family, 
                   StoryDomain::Conflict, StoryDomain::Trauma, StoryDomain::SliceOfLife] {
        let candidates = source.candidates_for_domain(domain);
        println!("  Domain {:?}: {} storylets", domain, candidates.len());
    }
    
    // Test life stage queries
    for stage in [LifeStage::Teen, LifeStage::Adult, LifeStage::Elder] {
        let candidates = source.candidates_for_life_stage(stage);
        println!("  Life stage {:?}: {} storylets", stage, candidates.len());
    }
}

#[test]
fn test_binary_persistence_with_real_storylets() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let project_root = manifest_dir.parent().unwrap().parent().unwrap();
    let storylets_dir = project_root.join("storylets");
    
    if !storylets_dir.exists() {
        eprintln!("Warning: storylets directory not found, skipping test");
        return;
    }
    
    // Compile from source
    let validator = default_storylet_validator();
    let compiler = StoryletCompiler::new(validator);
    let library = compiler.compile_from_dir(&storylets_dir)
        .expect("Failed to compile storylets");
    
    let original_count = library.total_count;
    
    // Write to binary
    let temp_dir = TempDir::new().unwrap();
    let binary_path = temp_dir.path().join("storylets.bin");
    
    library.write_to_file(&binary_path)
        .expect("Failed to write binary library");
    
    // Read from binary
    let loaded = StoryletLibrary::read_from_file(&binary_path)
        .expect("Failed to read binary library");
    
    // Verify it matches
    assert_eq!(loaded.total_count, original_count);
    assert_eq!(loaded.tag_index.len(), library.tag_index.len());
    assert_eq!(loaded.domain_index.len(), library.domain_index.len());
    assert_eq!(loaded.life_stage_index.len(), library.life_stage_index.len());
    
    // Verify query results match
    let source_original: &dyn StoryletSource = &library;
    let source_loaded: &dyn StoryletSource = &loaded;
    
    let tag = Tag::new("romance");
    let orig_candidates = source_original.candidates_for_tag(&tag);
    let loaded_candidates = source_loaded.candidates_for_tag(&tag);
    assert_eq!(orig_candidates.len(), loaded_candidates.len());
}
