//! Offline storylet compiler: loads JSON files, validates, and builds indexes.

use crate::library::{CompiledStorylet, ResolvedFollowUp, StoryletKey, StoryletLibrary};
use crate::validation::{StoryletValidator, validate_storylets};
use crate::{StoryletDef, StoryletId};
use crate::errors::StoryletCompileError;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Configuration and execution of storylet compilation.
#[derive(Debug)]
pub struct StoryletCompiler {
    validator: StoryletValidator,
}

impl StoryletCompiler {
    /// Create a new compiler with a given validator.
    pub fn new(validator: StoryletValidator) -> Self {
        StoryletCompiler { validator }
    }

    /// Compile all `.json` storylet files from a directory into a library.
    ///
    /// Returns a fully indexed `StoryletLibrary` ready for serialization or runtime use.
    /// Reports all errors at once: duplicates, validation failures, missing follow-ups.
    pub fn compile_from_dir<P: AsRef<Path>>(
        &self,
        dir: P,
    ) -> Result<StoryletLibrary, Vec<StoryletCompileError>> {
        let dir = dir.as_ref();

        // Step 1: Load all JSON files
        let loaded_storylets = self.load_json_files(dir)?;
        if loaded_storylets.is_empty() {
            return Err(vec![StoryletCompileError::NoStorylets {
                dir: dir.to_path_buf(),
            }]);
        }

        // Step 2: Validate all storylets
        let storylets: Vec<_> = loaded_storylets.iter().map(|(_, def)| def.clone()).collect();
        if let Err(validation_failures) = validate_storylets(&self.validator, &storylets) {
            let errors = validation_failures
                .into_iter()
                .map(|(id, errs)| {
                    let path = loaded_storylets
                        .iter()
                        .find(|(_, def)| def.id == id)
                        .map(|(p, _)| p.clone())
                        .unwrap_or_else(|| PathBuf::from("unknown"));
                    StoryletCompileError::Validation {
                        id,
                        path,
                        errors: errs,
                    }
                })
                .collect();
            return Err(errors);
        }

        // Step 3: Check for duplicate IDs
        let mut id_map: HashMap<StoryletId, PathBuf> = HashMap::new();
        let mut dup_errors = Vec::new();

        for (path, def) in &loaded_storylets {
            if let Some(first_path) = id_map.get(&def.id) {
                dup_errors.push(StoryletCompileError::DuplicateId {
                    id: def.id.clone(),
                    first_path: first_path.clone(),
                    duplicate_path: path.clone(),
                });
            } else {
                id_map.insert(def.id.clone(), path.clone());
            }
        }

        if !dup_errors.is_empty() {
            return Err(dup_errors);
        }

        // Step 4: Build library with indexes
        self.build_library(&loaded_storylets)
    }

    /// Load all `.json` files from a directory.
    fn load_json_files(
        &self,
        dir: &Path,
    ) -> Result<Vec<(PathBuf, StoryletDef)>, Vec<StoryletCompileError>> {
        let mut loaded = Vec::new();
        let mut errors = Vec::new();

        if !dir.exists() {
            return Err(vec![StoryletCompileError::Io {
                path: dir.to_path_buf(),
                error: std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Directory not found",
                ),
            }]);
        }

        // Recursively find all .json files
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(err) => {
                return Err(vec![StoryletCompileError::Io {
                    path: dir.to_path_buf(),
                    error: err,
                }]);
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(err) => {
                    errors.push(StoryletCompileError::Io {
                        path: dir.to_path_buf(),
                        error: err,
                    });
                    continue;
                }
            };

            let path = entry.path();

            if path.is_dir() {
                // Recursively load from subdirectories
                match self.load_json_files(&path) {
                    Ok(subloaded) => loaded.extend(subloaded),
                    Err(suberrs) => errors.extend(suberrs),
                }
            } else if path.extension().and_then(|s| s.to_str()) == Some("json") {
                // Load JSON file
                match self.load_single_file(&path) {
                    Ok(def) => loaded.push((path, def)),
                    Err(err) => errors.push(err),
                }
            }
        }

        if errors.is_empty() {
            Ok(loaded)
        } else {
            Err(errors)
        }
    }

    /// Load a single JSON storylet file.
    fn load_single_file(&self, path: &Path) -> Result<StoryletDef, StoryletCompileError> {
        let content = std::fs::read_to_string(path).map_err(|err| {
            StoryletCompileError::Io {
                path: path.to_path_buf(),
                error: err,
            }
        })?;

        serde_json::from_str(&content).map_err(|err| StoryletCompileError::JsonParse {
            path: path.to_path_buf(),
            error: err,
        })
    }

    /// Build the compiled library with all indexes.
    fn build_library(
        &self,
        loaded_storylets: &[(PathBuf, StoryletDef)],
    ) -> Result<StoryletLibrary, Vec<StoryletCompileError>> {
        let mut library = StoryletLibrary::new();
        let mut id_to_path: HashMap<StoryletId, PathBuf> = HashMap::new();

        // Assign keys and create ID -> key mapping
        for (path, def) in loaded_storylets {
            let key = StoryletKey(library.total_count);
            id_to_path.insert(def.id.clone(), path.clone());
            library.id_to_key.insert(def.id.clone(), key);
            library.total_count += 1;
        }

        // Compile each storylet and build indexes
        let mut compile_errors = Vec::new();

        for (path, def) in loaded_storylets {
            let key = library.id_to_key[&def.id];

            // Resolve follow-up IDs to keys
            let mut follow_ups_resolved = Vec::new();
            if let Some(follow_ups) = &def.outcomes.follow_ups {
                for fu in follow_ups {
                    let target_id = StoryletId::new(&fu.storylet_id);
                    let target_key = library.id_to_key.get(&target_id).copied();

                    // We don't fail compilation if follow-up is missing at compile time,
                    // but we track it. Runtime should handle missing keys gracefully.
                    if target_key.is_none() {
                        compile_errors.push(StoryletCompileError::MissingFollowUp {
                            from_id: def.id.clone(),
                            missing_id: fu.storylet_id.clone(),
                            path: path.clone(),
                        });
                    }

                    follow_ups_resolved.push(ResolvedFollowUp {
                        target_key,
                        delay_ticks: fu.delay_ticks,
                        conditional_on_flag: fu.conditional_on_flag.clone(),
                    });
                }
            }

            let compiled = CompiledStorylet {
                id: def.id.clone(),
                key,
                name: def.name.clone(),
                description: def.description.clone(),
                tags: def.tags.clone(),
                domain: def.domain,
                life_stage: def.life_stage,
                heat: def.heat,
                weight: def.weight,
                roles: def.roles.clone(),
                prerequisites: def.prerequisites.clone(),
                cooldowns: def.cooldowns.clone(),
                outcomes: def.outcomes.clone(),
                follow_ups_resolved,
            };

            // Add to indexes
            for tag in &compiled.tags {
                library
                    .tag_index
                    .entry(tag.clone())
                    .or_insert_with(Vec::new)
                    .push(key);
            }

            library
                .life_stage_index
                .entry(compiled.life_stage)
                .or_insert_with(Vec::new)
                .push(key);

            library
                .domain_index
                .entry(compiled.domain)
                .or_insert_with(Vec::new)
                .push(key);

            library.storylets.push(compiled);
        }

        if !compile_errors.is_empty() {
            return Err(compile_errors);
        }

        Ok(library)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::default_storylet_validator;

    #[test]
    fn test_compiler_creation() {
        let validator = default_storylet_validator();
        let _compiler = StoryletCompiler::new(validator);
        // Compiler created successfully
    }

    #[test]
    fn test_load_nonexistent_dir() {
        let validator = default_storylet_validator();
        let compiler = StoryletCompiler::new(validator);
        let result = compiler.compile_from_dir("/nonexistent/path");
        assert!(result.is_err());
    }
}
