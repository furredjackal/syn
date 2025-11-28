use syn_storylets::compiler::StoryletCompiler;
use syn_storylets::library::{StoryletKey, StoryletLibrary};
use syn_storylets::validation::default_storylet_validator;
use syn_storylets::{
    Cooldowns, Outcome, Prerequisites, StoryDomain, StoryletDef, StoryletId, LifeStage, Tag,
};
use std::fs;
use tempfile::TempDir;

/// Helper to create a test storylet definition
fn create_test_storylet(id: &str, name: &str, domain: StoryDomain) -> StoryletDef {
    StoryletDef {
        id: StoryletId::new(id),
        name: name.to_string(),
        description: Some(format!("Test storylet: {}", name)),
        tags: vec![Tag::new("test")],
        domain,
        life_stage: LifeStage::Adult,
        heat: 5,
        weight: 1.0,
        roles: vec![],
        prerequisites: Prerequisites::default(),
        triggers: vec![],
        cooldowns: Cooldowns::default(),
        outcomes: Outcome::default(),
    }
}

#[test]
fn test_compiler_compile_from_dir() {
    // Create temp directory with test storylets
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path();

    // Create storylet 1
    let s1 = create_test_storylet("test.romance.first_date", "First Date", StoryDomain::Romance);
    let json1 = serde_json::to_string_pretty(&s1).unwrap();
    fs::write(dir_path.join("romance.json"), json1).unwrap();

    // Create storylet 2
    let s2 = create_test_storylet(
        "test.career.promotion",
        "Promotion",
        StoryDomain::Career,
    );
    let json2 = serde_json::to_string_pretty(&s2).unwrap();
    fs::write(dir_path.join("career.json"), json2).unwrap();

    // Create storylet 3
    let s3 = create_test_storylet("test.family.reunion", "Family Reunion", StoryDomain::Family);
    let json3 = serde_json::to_string_pretty(&s3).unwrap();
    fs::write(dir_path.join("family.json"), json3).unwrap();

    // Compile
    let validator = default_storylet_validator();
    let compiler = StoryletCompiler::new(validator);
    let library = compiler.compile_from_dir(dir_path).expect("Compilation failed");

    // Verify
    assert_eq!(library.total_count, 3, "Should have 3 storylets");
    assert_eq!(library.id_to_key.len(), 3, "Should have 3 ID mappings");
    assert_eq!(
        library.domain_index.len(),
        3,
        "Should have 3 different domains"
    );

    // Verify specific lookups
    let romance_key = library
        .id_to_key
        .get(&StoryletId::new("test.romance.first_date"))
        .copied();
    assert!(romance_key.is_some(), "Should find romance storylet");

    let romance_storylet = library
        .get_by_id(&StoryletId::new("test.romance.first_date"))
        .unwrap();
    assert_eq!(romance_storylet.name, "First Date");

    // Verify domain indexing
    let romance_stories = library.get_by_domain(StoryDomain::Romance);
    assert_eq!(romance_stories.len(), 1);
    assert_eq!(romance_stories[0].id.0, "test.romance.first_date");
}

#[test]
fn test_compiler_binary_roundtrip() {
    // Create temp directory with test storylets
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path();

    // Create test storylets
    let s1 = create_test_storylet("story.one", "Story One", StoryDomain::Romance);
    let json1 = serde_json::to_string_pretty(&s1).unwrap();
    fs::write(dir_path.join("story1.json"), json1).unwrap();

    let s2 = create_test_storylet("story.two", "Story Two", StoryDomain::Career);
    let json2 = serde_json::to_string_pretty(&s2).unwrap();
    fs::write(dir_path.join("story2.json"), json2).unwrap();

    // Compile
    let validator = default_storylet_validator();
    let compiler = StoryletCompiler::new(validator);
    let library = compiler
        .compile_from_dir(dir_path)
        .expect("Compilation failed");

    // Write to binary
    let bin_path = temp_dir.path().join("library.bin");
    library.write_to_file(&bin_path).expect("Failed to write binary");

    // Verify file exists
    assert!(bin_path.exists(), "Binary file should exist");

    // Read back
    let loaded = StoryletLibrary::read_from_file(&bin_path).expect("Failed to read binary");

    // Verify
    assert_eq!(
        loaded.total_count, library.total_count,
        "Total count should match"
    );
    assert_eq!(
        loaded.id_to_key.len(),
        library.id_to_key.len(),
        "ID mappings should match"
    );

    // Verify specific lookups work after roundtrip
    let loaded_story1 = loaded
        .get_by_id(&StoryletId::new("story.one"))
        .expect("Should find story one");
    assert_eq!(loaded_story1.name, "Story One");

    let loaded_story2 = loaded
        .get_by_id(&StoryletId::new("story.two"))
        .expect("Should find story two");
    assert_eq!(loaded_story2.name, "Story Two");

    // Verify domain index works
    let romance_stories = loaded.get_by_domain(StoryDomain::Romance);
    assert_eq!(romance_stories.len(), 1);

    let career_stories = loaded.get_by_domain(StoryDomain::Career);
    assert_eq!(career_stories.len(), 1);
}

#[test]
fn test_compiler_tag_indexing() {
    // Create temp directory with test storylets
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path();

    // Create storylet with custom tags
    let mut s1 = create_test_storylet("story.tagged", "Tagged Story", StoryDomain::Romance);
    s1.tags = vec![Tag::new("romance"), Tag::new("milestone"), Tag::new("adult")];
    let json1 = serde_json::to_string_pretty(&s1).unwrap();
    fs::write(dir_path.join("tagged.json"), json1).unwrap();

    // Compile
    let validator = default_storylet_validator();
    let compiler = StoryletCompiler::new(validator);
    let library = compiler.compile_from_dir(dir_path).expect("Compilation failed");

    // Verify tag index
    assert_eq!(library.tag_index.len(), 3, "Should have 3 tags indexed");

    let romance_tag = Tag::new("romance");
    let romance_stories = library.get_by_tag(&romance_tag);
    assert_eq!(romance_stories.len(), 1);
    assert_eq!(romance_stories[0].name, "Tagged Story");
}

#[test]
fn test_compiler_no_files() {
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path();

    let validator = default_storylet_validator();
    let compiler = StoryletCompiler::new(validator);
    let result = compiler.compile_from_dir(dir_path);

    assert!(result.is_err(), "Should fail with no JSON files");
}

#[test]
fn test_compiler_invalid_json() {
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path();

    // Write invalid JSON
    fs::write(dir_path.join("invalid.json"), "{invalid json}").unwrap();

    let validator = default_storylet_validator();
    let compiler = StoryletCompiler::new(validator);
    let result = compiler.compile_from_dir(dir_path);

    assert!(result.is_err(), "Should fail with invalid JSON");
}

#[test]
fn test_compiler_duplicate_ids() {
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path();

    let s1 = create_test_storylet("duplicate.id", "First", StoryDomain::Romance);
    let json1 = serde_json::to_string_pretty(&s1).unwrap();
    fs::write(dir_path.join("story1.json"), json1).unwrap();

    let s2 = create_test_storylet("duplicate.id", "Second", StoryDomain::Career);
    let json2 = serde_json::to_string_pretty(&s2).unwrap();
    fs::write(dir_path.join("story2.json"), json2).unwrap();

    let validator = default_storylet_validator();
    let compiler = StoryletCompiler::new(validator);
    let result = compiler.compile_from_dir(dir_path);

    assert!(
        result.is_err(),
        "Should fail when duplicate IDs are detected"
    );
}

#[test]
fn test_library_lookup_methods() {
    let mut library = StoryletLibrary::new();

    // Create and add a compiled storylet
    let compiled = syn_storylets::library::CompiledStorylet {
        id: StoryletId::new("test.story"),
        key: StoryletKey(0),
        name: "Test".to_string(),
        description: None,
        tags: vec![Tag::new("test"), Tag::new("adult")],
        domain: StoryDomain::Romance,
        life_stage: LifeStage::Adult,
        heat: 5,
        weight: 1.0,
        roles: vec![],
        prerequisites: Prerequisites::default(),
        cooldowns: Cooldowns::default(),
        outcomes: Outcome::default(),
        follow_ups_resolved: vec![],
    };

    library
        .id_to_key
        .insert(compiled.id.clone(), compiled.key);
    library
        .tag_index
        .insert(Tag::new("test"), vec![compiled.key]);
    library
        .tag_index
        .insert(Tag::new("adult"), vec![compiled.key]);
    library
        .domain_index
        .insert(StoryDomain::Romance, vec![compiled.key]);
    library
        .life_stage_index
        .insert(LifeStage::Adult, vec![compiled.key]);

    library.storylets.push(compiled);
    library.total_count = 1;

    // Test lookups
    assert!(
        library
            .get_by_id(&StoryletId::new("test.story"))
            .is_some(),
        "Should find by ID"
    );
    assert!(
        library.get_by_key(StoryletKey(0)).is_some(),
        "Should find by key"
    );
    assert_eq!(
        library.get_by_tag(&Tag::new("test")).len(),
        1,
        "Should find by tag"
    );
    assert_eq!(
        library.get_by_domain(StoryDomain::Romance).len(),
        1,
        "Should find by domain"
    );
    assert_eq!(
        library.get_by_life_stage(LifeStage::Adult).len(),
        1,
        "Should find by life stage"
    );
}
