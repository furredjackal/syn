//! Error types for SYN.

use std::fmt;

/// SYN error types.
#[derive(Debug)]
pub enum SynError {
    /// Database or save/load error.
    PersistenceError(String),
    /// Simulation logic error.
    SimulationError(String),
    /// Invalid game state detected.
    InvalidState(String),
    /// Entity not found.
    NotFound(String),
}

impl fmt::Display for SynError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SynError::PersistenceError(msg) => write!(f, "Persistence error: {}", msg),
            SynError::SimulationError(msg) => write!(f, "Simulation error: {}", msg),
            SynError::InvalidState(msg) => write!(f, "Invalid state: {}", msg),
            SynError::NotFound(msg) => write!(f, "Not found: {}", msg),
        }
    }
}

impl std::error::Error for SynError {}

/// Result type alias using SynError.
pub type Result<T> = std::result::Result<T, SynError>;
