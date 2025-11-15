//! Error types for SYN.

use std::fmt;

/// SYN error types.
#[derive(Debug)]
pub enum SynError {
    PersistenceError(String),
    SimulationError(String),
    InvalidState(String),
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

pub type Result<T> = std::result::Result<T, SynError>;
