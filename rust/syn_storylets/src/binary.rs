//! Binary format for compiled storylet libraries.
//!
//! Format:
//! - Magic: 4 bytes "SYNL"
//! - Version: 2 bytes (u16, little-endian)
//! - Serialized StoryletLibrary (bincode format for simplicity)
//!
//! This format is designed to be stable and versionable.

use crate::library::StoryletLibrary;
use crate::errors::StoryletIoError;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

/// Magic bytes identifying a SYN storylet library.
pub const STORYLET_LIB_MAGIC: [u8; 4] = *b"SYNL";

/// Current version of the binary format.
pub const STORYLET_LIB_VERSION: u16 = 1;

impl StoryletLibrary {
    /// Serialize the library to a binary file.
    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), StoryletIoError> {
        let mut file = File::create(path.as_ref())?;

        // Write magic bytes
        file.write_all(&STORYLET_LIB_MAGIC)?;

        // Write version (little-endian u16)
        file.write_all(&STORYLET_LIB_VERSION.to_le_bytes())?;

        // Serialize library using bincode
        let encoded = bincode::serialize(self).map_err(|e| StoryletIoError::SerdeError {
            message: format!("Failed to serialize library: {}", e),
        })?;

        file.write_all(&encoded)?;
        file.sync_all()?;

        Ok(())
    }

    /// Deserialize a library from a binary file.
    pub fn read_from_file<P: AsRef<Path>>(path: P) -> Result<Self, StoryletIoError> {
        let mut file = File::open(path.as_ref())?;

        // Read and verify magic
        let mut magic = [0u8; 4];
        file.read_exact(&mut magic)?;

        if magic != STORYLET_LIB_MAGIC {
            return Err(StoryletIoError::InvalidFormat {
                message: format!(
                    "Invalid magic bytes: expected {:?}, got {:?}",
                    STORYLET_LIB_MAGIC, magic
                ),
            });
        }

        // Read and verify version
        let mut version_bytes = [0u8; 2];
        file.read_exact(&mut version_bytes)?;
        let version = u16::from_le_bytes(version_bytes);

        if version != STORYLET_LIB_VERSION {
            return Err(StoryletIoError::InvalidFormat {
                message: format!(
                    "Unsupported version: {}, expected {}",
                    version, STORYLET_LIB_VERSION
                ),
            });
        }

        // Read remaining bytes and deserialize
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;

        let library = bincode::deserialize(&buf).map_err(|e| StoryletIoError::SerdeError {
            message: format!("Failed to deserialize library: {}", e),
        })?;

        Ok(library)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::{CompiledStorylet, StoryletKey};
    use crate::{StoryDomain, LifeStage, StoryletId};
    use tempfile::TempDir;

    #[test]
    fn test_binary_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.bin");

        // Create a simple library
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
            roles: vec![],
            prerequisites: crate::Prerequisites::default(),
            cooldowns: crate::Cooldowns::default(),
            outcomes: crate::Outcome::default(),
            follow_ups_resolved: vec![],
        };

        library
            .id_to_key
            .insert(compiled.id.clone(), compiled.key);
        library.storylets.push(compiled);
        library.total_count = 1;

        // Write to file
        library.write_to_file(&file_path).expect("Failed to write");

        // Read from file
        let loaded = StoryletLibrary::read_from_file(&file_path).expect("Failed to read");

        // Verify
        assert_eq!(loaded.total_count, 1);
        assert_eq!(loaded.get_by_key(StoryletKey(0)).unwrap().name, "Test Story");
    }

    #[test]
    fn test_invalid_magic() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("bad.bin");

        // Write bad magic
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"XXXX").unwrap();
        file.write_all(&[0u8, 1]).unwrap();

        let result = StoryletLibrary::read_from_file(&file_path);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), StoryletIoError::InvalidFormat { .. }));
    }

    #[test]
    fn test_invalid_version() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("bad_version.bin");

        // Write correct magic but wrong version
        let mut file = File::create(&file_path).unwrap();
        file.write_all(&STORYLET_LIB_MAGIC).unwrap();
        file.write_all(&999u16.to_le_bytes()).unwrap();

        let result = StoryletLibrary::read_from_file(&file_path);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), StoryletIoError::InvalidFormat { .. }));
    }
}
