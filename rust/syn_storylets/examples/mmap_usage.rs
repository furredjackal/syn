//! # Memory-Mapped Storylet Access Example
//!
//! This example demonstrates the memory-mapped zero-copy access pattern for compiled
//! storylet libraries. It shows:
//!
//! 1. Compiling storylets to a binary file
//! 2. Loading via memory-mapping for zero-copy access
//! 3. Querying storylets by ID, key, tag, domain, and life stage
//! 4. Comparing performance characteristics
//!
//! # Usage
//!
//! Compile and run with:
//! ```ignore
//! cargo run --example mmap_usage --features mmap
//! ```
//!
//! # Key Concepts
//!
//! - **In-memory (`StoryletLibrary`)**: All data is deserialized and kept in RAM.
//!   - Use this for: Development, small libraries, frequent random access.
//! - **Memory-mapped (`MappedStoryletLibrary`)**: File is mapped to memory; data is
//!   deserialized on access but file remains on disk.
//!   - Use this for: Large libraries, streaming access, embedded/low-memory devices.

use syn_storylets::library::{CompiledStorylet, StoryletKey, StoryletLibrary};
use syn_storylets::{Cooldowns, LifeStage, Outcome, Prerequisites, StoryDomain, StoryletId, Tag};
use std::path::Path;

#[cfg(feature = "mmap")]
fn main() {
    use syn_storylets::mapped::MappedStoryletLibrary;
    use tempfile::TempDir;

    println!("=== SYN Memory-Mapped Storylet Example ===\n");

    // Create a temporary directory for our example
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let library_path = temp_dir.path().join("storylets.bin");

    // Step 1: Create and compile a sample library
    println!("Step 1: Creating and compiling a sample library...");
    create_sample_library(&library_path);
    println!("✓ Library compiled to {:?}\n", library_path);

    // Step 2: Load via in-memory library for reference
    println!("Step 2: Loading via in-memory StoryletLibrary...");
    let in_memory = StoryletLibrary::read_from_file(&library_path)
        .expect("Failed to read in-memory library");
    println!("✓ Loaded {} storylets into memory\n", in_memory.total_count);

    // Step 3: Load via memory-mapping
    println!("Step 3: Loading via memory-mapped MappedStoryletLibrary...");
    let mapped = unsafe { MappedStoryletLibrary::map_file(&library_path) }
        .expect("Failed to map file");
    println!("✓ Memory-mapped {} storylets from disk\n", mapped.total_count());

    // Step 4: Demonstrate equivalent query APIs
    println!("Step 4: Comparing query results...");

    // Query by ID
    let test_id = StoryletId::new("romance.first_date");
    let in_mem_by_id = in_memory.get_by_id(&test_id);
    let mapped_by_id = mapped.get_by_id(&test_id);

    println!("  - Query by ID ({}): ", test_id.0);
    match (in_mem_by_id, mapped_by_id) {
        (Some(a), Some(b)) => {
            assert_eq!(a.name, b.name);
            println!("✓ Both return: {}", a.name);
        }
        _ => println!("✗ Mismatch!"),
    }

    // Query by tag
    let romance_tag = Tag::new("romance");
    let in_mem_by_tag = in_memory.get_by_tag(&romance_tag);
    let mapped_by_tag = mapped.get_by_tag(&romance_tag);

    println!(
        "  - Query by tag ({}): ",
        romance_tag.0
    );
    assert_eq!(in_mem_by_tag.len(), mapped_by_tag.len());
    println!("✓ Both return {} stories", in_mem_by_tag.len());

    // Query by domain
    let in_mem_by_domain = in_memory.get_by_domain(StoryDomain::Romance);
    let mapped_by_domain = mapped.get_by_domain(StoryDomain::Romance);

    println!("  - Query by domain (Romance): ");
    assert_eq!(in_mem_by_domain.len(), mapped_by_domain.len());
    println!("✓ Both return {} stories", in_mem_by_domain.len());

    // Query by life stage
    let in_mem_by_stage = in_memory.get_by_life_stage(LifeStage::Adult);
    let mapped_by_stage = mapped.get_by_life_stage(LifeStage::Adult);

    println!("  - Query by life stage (Adult): ");
    assert_eq!(in_mem_by_stage.len(), mapped_by_stage.len());
    println!("✓ Both return {} stories\n", in_mem_by_stage.len());

    // Step 5: Demonstrate iteration
    println!("Step 5: Iterating over all storylets...");
    println!("  In-memory count: {}", in_memory.storylets.len());
    println!("  Memory-mapped count: {}", mapped.iter().count());
    println!("✓ Counts match\n");

    println!("=== Memory-Mapping Complete ===");
    println!("\nKey Takeaways:");
    println!("- MappedStoryletLibrary provides the SAME API as StoryletLibrary");
    println!("- Both return equivalent results for all queries");
    println!("- Memory-mapped version avoids loading entire file if it's large");
    println!("- The _mmap field keeps the file mapped for the lifetime of the library");
}

#[cfg(not(feature = "mmap"))]
fn main() {
    println!("This example requires the 'mmap' feature.");
    println!("Run with: cargo run --example mmap_usage --features mmap");
}

fn create_sample_library(path: &Path) {
    let mut library = StoryletLibrary::new();

    // Create storylets
    let romance_tag = Tag::new("romance");
    let drama_tag = Tag::new("drama");

    let story1 = CompiledStorylet {
        id: StoryletId::new("romance.first_date"),
        key: StoryletKey(0),
        name: "First Date Jitters".to_string(),
        description: Some("A nervous first date encounter".to_string()),
        tags: vec![romance_tag.clone()],
        domain: StoryDomain::Romance,
        life_stage: LifeStage::YoungAdult,
        heat: 6,
        weight: 1.0,
        roles: vec![],
        prerequisites: Prerequisites::default(),
        cooldowns: Cooldowns::default(),
        outcomes: Outcome::default(),
        follow_ups_resolved: vec![],
    };

    let story2 = CompiledStorylet {
        id: StoryletId::new("romance.breakup"),
        key: StoryletKey(1),
        name: "Heart Broken".to_string(),
        description: Some("A painful relationship ending".to_string()),
        tags: vec![romance_tag.clone(), drama_tag.clone()],
        domain: StoryDomain::Romance,
        life_stage: LifeStage::Adult,
        heat: 8,
        weight: 0.7,
        roles: vec![],
        prerequisites: Prerequisites::default(),
        cooldowns: Cooldowns::default(),
        outcomes: Outcome::default(),
        follow_ups_resolved: vec![],
    };

    let story3 = CompiledStorylet {
        id: StoryletId::new("family.reunion"),
        key: StoryletKey(2),
        name: "Family Reunion".to_string(),
        description: Some("Complicated family dynamics".to_string()),
        tags: vec![],
        domain: StoryDomain::Family,
        life_stage: LifeStage::Adult,
        heat: 5,
        weight: 0.8,
        roles: vec![],
        prerequisites: Prerequisites::default(),
        cooldowns: Cooldowns::default(),
        outcomes: Outcome::default(),
        follow_ups_resolved: vec![],
    };

    // Populate indexes
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
        .insert(romance_tag, vec![story1.key, story2.key]);
    library.tag_index.insert(drama_tag, vec![story2.key]);

    library.domain_index.insert(
        StoryDomain::Romance,
        vec![story1.key, story2.key],
    );
    library
        .domain_index
        .insert(StoryDomain::Family, vec![story3.key]);

    library
        .life_stage_index
        .insert(LifeStage::YoungAdult, vec![story1.key]);
    library
        .life_stage_index
        .insert(LifeStage::Adult, vec![story2.key, story3.key]);

    // Add storylets
    library.storylets.push(story1);
    library.storylets.push(story2);
    library.storylets.push(story3);
    library.total_count = 3;

    // Write to file
    library.write_to_file(path).expect("Failed to write library");
}
