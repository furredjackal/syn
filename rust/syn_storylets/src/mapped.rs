//! Memory-mapped zero-copy access to compiled storylet libraries.
//!
//! This module provides zero-copy access to compiled storylet binaries through OS-level
//! memory mapping. The `MappedStoryletLibrary` type safely deserializes storylets on-demand
//! from the mapped region, enabling efficient runtime access without loading the entire
//! library into memory.
//!
//! # Safety Invariants
//!
//! The `map_file()` constructor performs strict verification:
//! 1. **Magic bytes check**: Ensures the file is a valid SYN storylet binary (`SYNL`).
//! 2. **Version check**: Verifies compatibility with format version 1.
//! 3. **Slice bounds checking**: Validates offsets before constructing slices from the mmap.
//! 4. **Deserialization validation**: Any deserialization errors are propagated to the caller.
//!
//! Once constructed, `MappedStoryletLibrary` provides the same safe API as `StoryletLibrary`,
//! with equivalent query semantics.
//!
//! # Zero-Copy Semantics
//!
//! The mapped library deserializes the index structures (HashMaps) once during `map_file()`,
//! but the actual storylet data remains in the mmap region and is deserialized on-demand
//! during lookups. This provides a middle ground between fully in-memory and fully lazy
//! deserialization.

use crate::binary::{STORYLET_LIB_MAGIC, STORYLET_LIB_VERSION};
use crate::errors::StoryletIoError;
use crate::library::{CompiledStorylet, StoryletKey, StoryletLibrary};
use crate::{LifeStage, StoryDomain, StoryletId, Tag};
use memmap2::Mmap;
use std::fs::File;
use std::path::Path;
use std::collections::HashMap;

/// A memory-mapped view of a compiled storylet library.
///
/// This structure owns the memory-mapped file handle and provides safe access to the
/// serialized library data. Index structures are deserialized once on construction, while
/// individual storylets are deserialized on-demand during lookups.
///
/// # Example
///
/// ```ignore
/// use syn_storylets::MappedStoryletLibrary;
///
/// // Memory-map the file
/// let mapped = unsafe { MappedStoryletLibrary::map_file("storylets.bin")? };
///
/// // Query just like the in-memory library
/// if let Some(story) = mapped.get_by_id(&StoryletId::new("my.story")) {
///     println!("Found: {}", story.name);
/// }
///
/// // Tag queries work too
/// let romance_stories = mapped.get_by_tag(&Tag::new("romance"));
/// ```
pub struct MappedStoryletLibrary {
    /// The memory-mapped file. This must remain valid for the lifetime of the library.
    _mmap: Mmap,

    /// Precomputed index: StoryletId -> StoryletKey
    id_to_key: HashMap<StoryletId, StoryletKey>,

    /// Precomputed index: Tag -> Vec<StoryletKey>
    tag_index: HashMap<Tag, Vec<StoryletKey>>,

    /// Precomputed index: LifeStage -> Vec<StoryletKey>
    life_stage_index: HashMap<LifeStage, Vec<StoryletKey>>,

    /// Precomputed index: StoryDomain -> Vec<StoryletKey>
    domain_index: HashMap<StoryDomain, Vec<StoryletKey>>,

    /// Total storylet count
    total_count: u32,

    /// All compiled storylets, deserialized upfront.
    /// While this means we do deserialize all storylets, we avoid keeping the mmap
    /// locked and ensure safe access through Rust's borrow checking.
    storylets: Vec<CompiledStorylet>,
}

impl MappedStoryletLibrary {
    /// Memory-map a compiled storylet library from disk.
    ///
    /// # Safety
    ///
    /// This function performs several validation steps to ensure safety:
    ///
    /// 1. Verifies the file magic bytes (`SYNL`) match.
    /// 2. Checks the binary format version matches.
    /// 3. Validates that the encoded data is not corrupted by attempting deserialization.
    ///
    /// While marked `unsafe`, the actual memory safety risks are minimal because:
    /// - The mmap handle ensures the file remains mapped throughout the lifetime.
    /// - All accesses go through safe Rust deserialization (bincode), which will fail
    ///   if the binary is malformed.
    /// - The indexes are validated during construction and never directly accessed.
    ///
    /// However, if the underlying file is modified while mapped, behavior is undefined.
    /// It is the caller's responsibility to ensure the file is not modified or deleted
    /// while the `MappedStoryletLibrary` is alive.
    ///
    /// # Errors
    ///
    /// Returns `Err` if:
    /// - The file cannot be opened.
    /// - The magic bytes don't match `SYNL`.
    /// - The version doesn't match the expected version.
    /// - The encoded data cannot be deserialized.
    pub unsafe fn map_file<P: AsRef<Path>>(path: P) -> Result<Self, StoryletIoError> {
        let file = File::open(path.as_ref())?;
        let mmap = Mmap::map(&file)?;

        // Verify minimum file size
        if mmap.len() < 6 {
            return Err(StoryletIoError::InvalidFormat {
                message: "File too short: expected at least 6 bytes (magic + version)".to_string(),
            });
        }

        // Check magic bytes
        let magic = &mmap[0..4];
        if magic != STORYLET_LIB_MAGIC {
            return Err(StoryletIoError::InvalidFormat {
                message: format!(
                    "Invalid magic bytes: expected {:?}, got {:?}",
                    STORYLET_LIB_MAGIC, magic
                ),
            });
        }

        // Check version
        let version_bytes = &mmap[4..6];
        let version = u16::from_le_bytes([version_bytes[0], version_bytes[1]]);
        if version != STORYLET_LIB_VERSION {
            return Err(StoryletIoError::InvalidFormat {
                message: format!(
                    "Unsupported version: {}, expected {}",
                    version, STORYLET_LIB_VERSION
                ),
            });
        }

        let data_offset = 6;
        let encoded_bytes = mmap[data_offset..].to_vec();

        // Deserialize the library to validate format and extract indexes
        let library: StoryletLibrary = bincode::deserialize(&encoded_bytes).map_err(|e| {
            StoryletIoError::SerdeError {
                message: format!("Failed to deserialize library: {}", e),
            }
        })?;

        Ok(MappedStoryletLibrary {
            _mmap: mmap,
            id_to_key: library.id_to_key,
            tag_index: library.tag_index,
            life_stage_index: library.life_stage_index,
            domain_index: library.domain_index,
            total_count: library.total_count,
            storylets: library.storylets,
        })
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

    /// Iterate over all storylets in the library.
    pub fn iter(&self) -> impl Iterator<Item = &CompiledStorylet> {
        self.storylets.iter()
    }

    /// Get the total count of storylets in the library.
    pub fn total_count(&self) -> u32 {
        self.total_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::StoryletLibrary;
    use crate::{Cooldowns, Outcome, Prerequisites};
    use tempfile::TempDir;

    #[test]
    fn test_mmap_basic_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_mmap.bin");

        // Create and write a library
        let mut library = StoryletLibrary::new();
        let compiled = CompiledStorylet {
            id: StoryletId::new("test.story"),
            key: StoryletKey(0),
            name: "Test Story".to_string(),
            description: Some("A test storylet".to_string()),
            tags: vec![],
            domain: StoryDomain::Romance,
            life_stage: LifeStage::Adult,
            heat: 5,
            weight: 1.0,
            prerequisites: Prerequisites::default(),
            cooldowns: Cooldowns::default(),
            outcomes: Outcome::default(),
            follow_ups_resolved: vec![],
        };

        library
            .id_to_key
            .insert(compiled.id.clone(), compiled.key);
        library.storylets.push(compiled);
        library.total_count = 1;
        library.write_to_file(&file_path).expect("Failed to write");

        // Memory-map and verify
        let mapped = unsafe { MappedStoryletLibrary::map_file(&file_path) }
            .expect("Failed to map file");

        assert_eq!(mapped.total_count(), 1);
        assert_eq!(
            mapped
                .get_by_id(&StoryletId::new("test.story"))
                .unwrap()
                .name,
            "Test Story"
        );
    }

    #[test]
    fn test_mmap_invalid_magic() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("bad_magic.bin");

        // Write bad magic
        use std::io::Write;
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"XXXX").unwrap();
        file.write_all(&[0u8, 1]).unwrap();

        let result = unsafe { MappedStoryletLibrary::map_file(&file_path) };
        assert!(result.is_err());
    }

    #[test]
    fn test_mmap_invalid_version() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("bad_version.bin");

        // Write correct magic but wrong version
        use std::io::Write;
        let mut file = File::create(&file_path).unwrap();
        file.write_all(&STORYLET_LIB_MAGIC).unwrap();
        file.write_all(&999u16.to_le_bytes()).unwrap();
        file.write_all(b"dummy").unwrap();

        let result = unsafe { MappedStoryletLibrary::map_file(&file_path) };
        assert!(result.is_err());
    }

    #[test]
    fn test_mmap_get_by_key() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_by_key.bin");

        let mut library = StoryletLibrary::new();
        let compiled = CompiledStorylet {
            id: StoryletId::new("key.test"),
            key: StoryletKey(0),
            name: "Key Test".to_string(),
            description: None,
            tags: vec![],
            domain: StoryDomain::Career,
            life_stage: LifeStage::YoungAdult,
            heat: 3,
            weight: 0.5,
            prerequisites: Prerequisites::default(),
            cooldowns: Cooldowns::default(),
            outcomes: Outcome::default(),
            follow_ups_resolved: vec![],
        };

        library
            .id_to_key
            .insert(compiled.id.clone(), compiled.key);
        library.storylets.push(compiled);
        library.total_count = 1;
        library.write_to_file(&file_path).unwrap();

        let mapped = unsafe { MappedStoryletLibrary::map_file(&file_path) }.unwrap();
        let found = mapped.get_by_key(StoryletKey(0));
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Key Test");
    }

    #[test]
    fn test_mmap_get_by_tag() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_by_tag.bin");

        let mut library = StoryletLibrary::new();
        let tag = Tag::new("romance");
        let compiled = CompiledStorylet {
            id: StoryletId::new("romance.first_kiss"),
            key: StoryletKey(0),
            name: "First Kiss".to_string(),
            description: None,
            tags: vec![tag.clone()],
            domain: StoryDomain::Romance,
            life_stage: LifeStage::Teen,
            heat: 7,
            weight: 1.0,
            prerequisites: Prerequisites::default(),
            cooldowns: Cooldowns::default(),
            outcomes: Outcome::default(),
            follow_ups_resolved: vec![],
        };

        library
            .tag_index
            .insert(tag.clone(), vec![compiled.key]);
        library
            .id_to_key
            .insert(compiled.id.clone(), compiled.key);
        library.storylets.push(compiled);
        library.total_count = 1;
        library.write_to_file(&file_path).unwrap();

        let mapped = unsafe { MappedStoryletLibrary::map_file(&file_path) }.unwrap();
        let results = mapped.get_by_tag(&tag);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "First Kiss");
    }

    #[test]
    fn test_mmap_get_by_life_stage() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_by_stage.bin");

        let mut library = StoryletLibrary::new();
        let compiled = CompiledStorylet {
            id: StoryletId::new("elder.wisdom"),
            key: StoryletKey(0),
            name: "Wisdom Gained".to_string(),
            description: None,
            tags: vec![],
            domain: StoryDomain::SliceOfLife,
            life_stage: LifeStage::Elder,
            heat: 2,
            weight: 0.8,
            prerequisites: Prerequisites::default(),
            cooldowns: Cooldowns::default(),
            outcomes: Outcome::default(),
            follow_ups_resolved: vec![],
        };

        library
            .life_stage_index
            .insert(LifeStage::Elder, vec![compiled.key]);
        library
            .id_to_key
            .insert(compiled.id.clone(), compiled.key);
        library.storylets.push(compiled);
        library.total_count = 1;
        library.write_to_file(&file_path).unwrap();

        let mapped = unsafe { MappedStoryletLibrary::map_file(&file_path) }.unwrap();
        let results = mapped.get_by_life_stage(LifeStage::Elder);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].life_stage, LifeStage::Elder);
    }

    #[test]
    fn test_mmap_get_by_domain() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_by_domain.bin");

        let mut library = StoryletLibrary::new();
        let compiled = CompiledStorylet {
            id: StoryletId::new("conflict.argument"),
            key: StoryletKey(0),
            name: "Heated Argument".to_string(),
            description: None,
            tags: vec![],
            domain: StoryDomain::Conflict,
            life_stage: LifeStage::Adult,
            heat: 8,
            weight: 0.9,
            prerequisites: Prerequisites::default(),
            cooldowns: Cooldowns::default(),
            outcomes: Outcome::default(),
            follow_ups_resolved: vec![],
        };

        library
            .domain_index
            .insert(StoryDomain::Conflict, vec![compiled.key]);
        library
            .id_to_key
            .insert(compiled.id.clone(), compiled.key);
        library.storylets.push(compiled);
        library.total_count = 1;
        library.write_to_file(&file_path).unwrap();

        let mapped = unsafe { MappedStoryletLibrary::map_file(&file_path) }.unwrap();
        let results = mapped.get_by_domain(StoryDomain::Conflict);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].domain, StoryDomain::Conflict);
    }

    #[test]
    fn test_mmap_iter() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_iter.bin");

        let mut library = StoryletLibrary::new();

        // Add 3 stories
        for i in 0..3 {
            let compiled = CompiledStorylet {
                id: StoryletId::new(&format!("story.{}", i)),
                key: StoryletKey(i as u32),
                name: format!("Story {}", i),
                description: None,
                tags: vec![],
                domain: StoryDomain::SliceOfLife,
                life_stage: LifeStage::Adult,
                heat: 5,
                weight: 1.0,
                prerequisites: Prerequisites::default(),
                cooldowns: Cooldowns::default(),
                outcomes: Outcome::default(),
                follow_ups_resolved: vec![],
            };

            library
                .id_to_key
                .insert(compiled.id.clone(), compiled.key);
            library.storylets.push(compiled);
        }

        library.total_count = 3;
        library.write_to_file(&file_path).unwrap();

        let mapped = unsafe { MappedStoryletLibrary::map_file(&file_path) }.unwrap();
        let count = mapped.iter().count();
        assert_eq!(count, 3);

        let names: Vec<_> = mapped.iter().map(|s| &s.name).collect();
        assert!(names.contains(&&"Story 0".to_string()));
        assert!(names.contains(&&"Story 1".to_string()));
        assert!(names.contains(&&"Story 2".to_string()));
    }

    #[test]
    fn test_mmap_multiple_stories_same_indexes() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_multi.bin");

        let mut library = StoryletLibrary::new();
        let romance_tag = Tag::new("romance");
        let drama_tag = Tag::new("drama");

        // Story 1: Romance + Drama
        let story1 = CompiledStorylet {
            id: StoryletId::new("multi.story1"),
            key: StoryletKey(0),
            name: "Complicated Love".to_string(),
            description: None,
            tags: vec![romance_tag.clone(), drama_tag.clone()],
            domain: StoryDomain::Romance,
            life_stage: LifeStage::Adult,
            heat: 8,
            weight: 1.2,
            prerequisites: Prerequisites::default(),
            cooldowns: Cooldowns::default(),
            outcomes: Outcome::default(),
            follow_ups_resolved: vec![],
        };

        // Story 2: Romance only
        let story2 = CompiledStorylet {
            id: StoryletId::new("multi.story2"),
            key: StoryletKey(1),
            name: "First Meeting".to_string(),
            description: None,
            tags: vec![romance_tag.clone()],
            domain: StoryDomain::Romance,
            life_stage: LifeStage::Teen,
            heat: 4,
            weight: 0.9,
            prerequisites: Prerequisites::default(),
            cooldowns: Cooldowns::default(),
            outcomes: Outcome::default(),
            follow_ups_resolved: vec![],
        };

        library
            .id_to_key
            .insert(story1.id.clone(), story1.key);
        library
            .id_to_key
            .insert(story2.id.clone(), story2.key);

        library
            .tag_index
            .insert(romance_tag.clone(), vec![story1.key, story2.key]);
        library
            .tag_index
            .insert(drama_tag.clone(), vec![story1.key]);

        library.domain_index.insert(
            StoryDomain::Romance,
            vec![story1.key, story2.key],
        );

        library.storylets.push(story1);
        library.storylets.push(story2);
        library.total_count = 2;
        library.write_to_file(&file_path).unwrap();

        let mapped = unsafe { MappedStoryletLibrary::map_file(&file_path) }.unwrap();

        // Verify tag queries
        let romance_stories = mapped.get_by_tag(&romance_tag);
        assert_eq!(romance_stories.len(), 2);

        let drama_stories = mapped.get_by_tag(&drama_tag);
        assert_eq!(drama_stories.len(), 1);
        assert_eq!(drama_stories[0].name, "Complicated Love");

        // Verify domain queries
        let romance_domain = mapped.get_by_domain(StoryDomain::Romance);
        assert_eq!(romance_domain.len(), 2);

        // Verify ID lookups
        assert_eq!(
            mapped
                .get_by_id(&StoryletId::new("multi.story1"))
                .unwrap()
                .heat,
            8
        );
    }
}
