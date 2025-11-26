//! Runtime loader for compiled storylet libraries.
//!
//! This module provides the glue between the bootstrap code and the compiled
//! storylet binary format. It supports both in-memory and memory-mapped loading.

use std::path::Path;
use syn_director::StoryletSource;
use syn_storylets::errors::StoryletIoError;
use syn_storylets::library::StoryletLibrary;

/// Load a compiled storylet library from a binary file.
///
/// Attempts to load the library using memory-mapping if the `mmap` feature is enabled,
/// falling back to in-memory loading otherwise.
///
/// # Arguments
///
/// * `path` - Path to the compiled storylet binary file (format: magic "SYNL" + version + bincode data)
///
/// # Errors
///
/// Returns `Err` if:
/// - The file cannot be opened or read
/// - The magic bytes don't match `SYNL`
/// - The version is unsupported
/// - The binary data is corrupted
pub fn load_storylet_library_from_file<P: AsRef<Path>>(
    path: P,
) -> Result<Box<dyn StoryletSource>, StoryletIoError> {
    let path_ref = path.as_ref();

    // Try memory-mapped loading first (if feature enabled)
    #[cfg(feature = "mmap")]
    {
        match unsafe { syn_storylets::mapped::MappedStoryletLibrary::map_file(path_ref) } {
            Ok(mapped) => {
                log_library_load(path_ref, "memory-mapped");
                return Ok(Box::new(mapped));
            }
            Err(e) => {
                eprintln!(
                    "Warning: Failed to memory-map storylets from {:?}, falling back to in-memory: {}",
                    path_ref, e
                );
            }
        }
    }

    // Fall back to in-memory loading
    let library = StoryletLibrary::read_from_file(path_ref)?;
    log_library_load(path_ref, "in-memory");
    Ok(Box::new(library))
}

/// Load a compiled storylet library with a sensible default path.
///
/// Looks for `$SYN_STORYLET_BIN` environment variable first, then falls back to
/// `./assets/storylets/storylets.bin` relative to the current directory.
///
/// # Errors
///
/// Returns the same errors as `load_storylet_library_from_file`.
pub fn load_default_storylet_library() -> Result<Box<dyn StoryletSource>, StoryletIoError> {
    let default_path = std::env::var("SYN_STORYLET_BIN")
        .unwrap_or_else(|_| "./assets/storylets/storylets.bin".to_string());

    load_storylet_library_from_file(&default_path)
}

/// Log information about library loading.
#[inline]
fn log_library_load(path: &Path, method: &str) {
    if let Some(path_str) = path.to_str() {
        eprintln!("[SYN] Loaded storylet library ({}): {}", method, path_str);
    }
}

/// Try to load a storylet library from a file, with a fallback to an empty library.
///
/// This is useful for optional library paths where an empty library is acceptable.
pub fn load_storylet_library_or_empty<P: AsRef<Path>>(
    path: P,
) -> Box<dyn StoryletSource> {
    match load_storylet_library_from_file(path) {
        Ok(library) => library,
        Err(e) => {
            eprintln!("[SYN] Failed to load storylets: {}. Using empty library.", e);
            Box::new(StoryletLibrary::new())
        }
    }
}

/// Try to load the default storylet library, with a fallback to an empty library.
pub fn load_default_storylet_library_or_empty() -> Box<dyn StoryletSource> {
    match load_default_storylet_library() {
        Ok(library) => library,
        Err(e) => {
            eprintln!("[SYN] Failed to load default storylets: {}. Using empty library.", e);
            Box::new(StoryletLibrary::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_load_in_memory_library() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_lib.bin");

        // Create a simple library
        let library = StoryletLibrary::new();
        library.write_to_file(&file_path).unwrap();

        // Load it
        let loaded = load_storylet_library_from_file(&file_path).unwrap();
        assert_eq!(loaded.total_count(), 0);
    }

    #[test]
    fn test_load_nonexistent_file() {
        let result = load_storylet_library_from_file("/nonexistent/path/to/storylets.bin");
        assert!(result.is_err());
    }

    #[test]
    fn test_load_or_empty_fallback() {
        let library = load_storylet_library_or_empty("/nonexistent/path");
        assert_eq!(library.total_count(), 0);
    }

    #[test]
    fn test_load_default_or_empty_fallback() {
        let library = load_default_storylet_library_or_empty();
        // Should always return a valid library (even if empty)
        assert_eq!(library.total_count(), 0);
    }
}
