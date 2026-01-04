use thiserror::Error;
use std::fmt;

#[derive(Error, Debug)]
pub enum ConsensusError {
    #[error("Block validation failed: {0}")]
    BlockValidationFailed(String),

    #[error("Voting error: {0}")]
    VotingError(String),

    #[error("AI validation error: {0}")]
    AIValidationError(String),

    #[error("Transaction validation failed: {0}")]
    TransactionValidationFailed(String),

    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),

    #[error("Validation timeout")]
    ValidationTimeout,

    #[error("Insufficient participation in voting ({0}% required)")]
    InsufficientParticipation(f64),

    #[error("Block efficiency too low ({0} < {1})")]
    LowBlockEfficiency(f64, f64),

    #[error("Invalid transaction color transition")]
    InvalidColorTransition,

    #[error("Validator not eligible to vote")]
    ValidatorNotEligible,

    #[error("No voting result available")]
    NoVotingResult,

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<anyhow::Error> for ConsensusError {
    fn from(e: anyhow::Error) -> Self {
        ConsensusError::AIValidationError(e.to_string())
    }
}

impl From<serde_json::Error> for ConsensusError {
    fn from(e: serde_json::Error) -> Self {
        ConsensusError::SerializationError(e.to_string())
    }
}

impl From<std::io::Error> for ConsensusError {
    fn from(e: std::io::Error) -> Self {
        ConsensusError::StorageError(e.to_string())
    }
}

impl From<std::time::SystemTimeError> for ConsensusError {
    fn from(e: std::time::SystemTimeError) -> Self {
        ConsensusError::InternalError(e.to_string())
    }
}

impl From<String> for ConsensusError {
    fn from(e: String) -> Self {
        ConsensusError::InternalError(e)
    }
}

#[derive(Debug)]
pub struct VotingErrorDetails {
    pub block_hash: String,
    pub error: ConsensusError,
    pub timestamp: u64,
}

impl fmt::Display for VotingErrorDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Voting error for block {}: {} at {}",
            self.block_hash,
            self.error,
            self.timestamp
        )
    }
}
