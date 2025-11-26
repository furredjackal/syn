//! Error types for storylet compilation.

use crate::validation::StoryletValidationError;
use crate::StoryletId;
use std::io;
use std::path::PathBuf;

/// Errors that can occur during storylet compilation.
#[derive(Debug)]
pub enum StoryletCompileError {
    /// IO error while reading files.
    Io {
        path: PathBuf,
        error: io::Error,
    },
    /// JSON parse error.
    JsonParse {
        path: PathBuf,
        error: serde_json::Error,
    },
    /// Storylet validation failed.
    Validation {
        id: StoryletId,
        path: PathBuf,
        errors: Vec<StoryletValidationError>,
    },
    /// Duplicate storylet ID found in multiple files.
    DuplicateId {
        id: StoryletId,
        first_path: PathBuf,
        duplicate_path: PathBuf,
    },
    /// Follow-up references a storylet ID that doesn't exist in the library.
    MissingFollowUp {
        from_id: StoryletId,
        missing_id: String,
        path: PathBuf,
    },
    /// No JSON files found in the specified directory.
    NoStorylets {
        dir: PathBuf,
    },
    /// Error reading/writing binary format.
    BinaryFormat {
        message: String,
    },
}

impl std::fmt::Display for StoryletCompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io { path, error } => {
                write!(f, "IO error reading {}: {}", path.display(), error)
            }
            Self::JsonParse { path, error } => {
                write!(f, "JSON parse error in {}: {}", path.display(), error)
            }
            Self::Validation {
                id,
                path,
                errors,
            } => {
                write!(
                    f,
                    "Validation error for '{}' in {}:\n",
                    id.0,
                    path.display()
                )?;
                for err in errors {
                    write!(f, "  - {}\n", err)?;
                }
                Ok(())
            }
            Self::DuplicateId {
                id,
                first_path,
                duplicate_path,
            } => {
                write!(
                    f,
                    "Duplicate storylet ID '{}': first in {}, duplicate in {}",
                    id.0,
                    first_path.display(),
                    duplicate_path.display()
                )
            }
            Self::MissingFollowUp {
                from_id,
                missing_id,
                path,
            } => {
                write!(
                    f,
                    "Storylet '{}' in {} references missing follow-up '{}' that doesn't exist in compiled library",
                    from_id.0, path.display(), missing_id
                )
            }
            Self::NoStorylets { dir } => {
                write!(f, "No JSON storylets found in {}", dir.display())
            }
            Self::BinaryFormat { message } => {
                write!(f, "Binary format error: {}", message)
            }
        }
    }
}

impl std::error::Error for StoryletCompileError {}

/// Errors that can occur during binary I/O.
#[derive(Debug)]
pub enum StoryletIoError {
    /// IO error.
    Io(io::Error),
    /// Invalid binary format (bad magic or version).
    InvalidFormat {
        message: String,
    },
    /// Serialization/deserialization error.
    SerdeError {
        message: String,
    },
}

impl std::fmt::Display for StoryletIoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "IO error: {}", err),
            Self::InvalidFormat { message } => write!(f, "Invalid binary format: {}", message),
            Self::SerdeError { message } => write!(f, "Serialization error: {}", message),
        }
    }
}

impl std::error::Error for StoryletIoError {}

impl From<io::Error> for StoryletIoError {
    fn from(err: io::Error) -> Self {
        StoryletIoError::Io(err)
    }
}

impl From<serde_json::Error> for StoryletIoError {
    fn from(err: serde_json::Error) -> Self {
        StoryletIoError::SerdeError {
            message: err.to_string(),
        }
    }
}

impl From<bincode::Error> for StoryletIoError {
    fn from(err: bincode::Error) -> Self {
        StoryletIoError::SerdeError {
            message: err.to_string(),
        }
    }
}
