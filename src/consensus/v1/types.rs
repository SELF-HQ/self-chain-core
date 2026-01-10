//! PoAI v1 Consensus Types
//!
//! Common types used throughout the decentralized consensus implementation.

use std::time::Duration;
use thiserror::Error;

/// Protocol constants from `docs/POAI_SPECIFICATION.md`
pub mod constants {
    use std::time::Duration;

    /// Chain identifier for replay protection
    pub const CHAIN_ID: &str = "self-chain-mainnet";
    
    /// Target time between blocks (60 seconds)
    pub const BLOCK_TIME_TARGET: Duration = Duration::from_secs(60);
    
    /// Quorum threshold: 2/3 of committee must agree
    pub const QUORUM_NUMERATOR: u64 = 2;
    pub const QUORUM_DENOMINATOR: u64 = 3;
    
    /// Committee size bounds
    pub const COMMITTEE_SIZE_MIN: usize = 10;
    pub const COMMITTEE_SIZE_MAX: usize = 100;
    
    /// Maximum transactions per block
    pub const MAX_TX_PER_BLOCK: usize = 1000;
    
    /// Maximum block size (1 MB)
    pub const MAX_BLOCK_SIZE: usize = 1_000_000;
    
    /// PoAI Competition Model timeout values
    pub const TIMEOUT_PROPOSE_WINDOW: Duration = Duration::from_secs(50);
    pub const TIMEOUT_VOTING: Duration = Duration::from_secs(8);
    pub const TIMEOUT_FINALIZE: Duration = Duration::from_secs(2);
    
    /// Clock drift tolerance
    pub const CLOCK_DRIFT_TOLERANCE: Duration = Duration::from_secs(5);
    
    /// Domain separation prefixes for signatures
    pub const DOMAIN_PREFIX_BLOCK: &[u8] = b"self-chain-block-header-v1";
    pub const DOMAIN_PREFIX_TRANSACTION: &[u8] = b"self-chain-transaction-v1";
    pub const DOMAIN_PREFIX_PROPOSAL: &[u8] = b"self-chain-proposal-v1";
    pub const DOMAIN_PREFIX_PREVOTE: &[u8] = b"self-chain-vote-prevote-v1";
    pub const DOMAIN_PREFIX_PRECOMMIT: &[u8] = b"self-chain-vote-precommit-v1";
    pub const DOMAIN_PREFIX_RANKED_VOTE: &[u8] = b"self-chain-ranked-vote-v1";
}

/// Configuration for the consensus engine
#[derive(Debug, Clone)]
pub struct ConsensusConfig {
    /// Chain identifier
    pub chain_id: String,
    
    /// Target block time
    pub block_time: Duration,
    
    /// Proposal window timeout (PoAI competition model)
    pub timeout_propose_window: Duration,
    
    /// Voting timeout
    pub timeout_voting: Duration,
    
    /// Finalization timeout
    pub timeout_finalize: Duration,
    
    /// Minimum committee size
    pub committee_size_min: usize,
    
    /// Maximum committee size
    pub committee_size_max: usize,
    
    /// Maximum transactions per block
    pub max_tx_per_block: usize,
    
    /// Maximum block size in bytes
    pub max_block_size: usize,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            chain_id: constants::CHAIN_ID.to_string(),
            block_time: constants::BLOCK_TIME_TARGET,
            timeout_propose_window: constants::TIMEOUT_PROPOSE_WINDOW,
            timeout_voting: constants::TIMEOUT_VOTING,
            timeout_finalize: constants::TIMEOUT_FINALIZE,
            committee_size_min: constants::COMMITTEE_SIZE_MIN,
            committee_size_max: constants::COMMITTEE_SIZE_MAX,
            max_tx_per_block: constants::MAX_TX_PER_BLOCK,
            max_block_size: constants::MAX_BLOCK_SIZE,
        }
    }
}

impl ConsensusConfig {
    /// Total round duration
    pub fn round_duration(&self) -> Duration {
        self.timeout_propose_window + self.timeout_voting + self.timeout_finalize
    }
    
    /// Calculate quorum threshold for a committee of given size
    pub fn quorum_threshold(&self, committee_size: usize) -> usize {
        // 2/3 + 1 for BFT safety
        (committee_size * 2 / 3) + 1
    }
}

/// Current step in the PoAI consensus round
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RoundStep {
    /// Proposal window: all builders submit proposals (0-50s)
    ProposeWindow,
    /// Voting: validators vote for best proposal (50-58s)
    Voting,
    /// Finalization: process results and finalize (58-60s)
    Finalize,
    /// Committed: waiting for next height
    Committed,
}

impl std::fmt::Display for RoundStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RoundStep::ProposeWindow => write!(f, "ProposeWindow"),
            RoundStep::Voting => write!(f, "Voting"),
            RoundStep::Finalize => write!(f, "Finalize"),
            RoundStep::Committed => write!(f, "Committed"),
        }
    }
}

/// State of the current consensus round
#[derive(Debug, Clone)]
pub struct RoundState {
    /// Current block height
    pub height: u64,
    
    /// Current consensus round (0-indexed, increments on timeout)
    pub round: u64,
    
    /// Current step in the round
    pub step: RoundStep,
    
    /// Reference block efficiency for this round
    pub reference_efficiency: u64,
    
    /// Number of proposals received
    pub proposals_received: usize,
    
    /// Number of votes received
    pub votes_received: usize,
    
    /// Winning block hash (if determined)
    pub winner_hash: Option<[u8; 32]>,
}

impl RoundState {
    /// Create a new round state at the given height
    pub fn new(height: u64) -> Self {
        Self {
            height,
            round: 0,
            step: RoundStep::ProposeWindow,
            reference_efficiency: 0,
            proposals_received: 0,
            votes_received: 0,
            winner_hash: None,
        }
    }
    
    /// Advance to the next round (on timeout/failure)
    pub fn advance_round(&mut self) {
        self.round += 1;
        self.step = RoundStep::ProposeWindow;
        self.proposals_received = 0;
        self.votes_received = 0;
        self.winner_hash = None;
    }
    
    /// Start a new height after successful commit
    pub fn new_height(&mut self, height: u64) {
        self.height = height;
        self.round = 0;
        self.step = RoundStep::ProposeWindow;
        self.reference_efficiency = 0;
        self.proposals_received = 0;
        self.votes_received = 0;
        self.winner_hash = None;
    }
}

/// Information about a validator
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatorInfo {
    /// Unique validator identifier
    pub validator_id: String,
    
    /// Ed25519 public key (32 bytes)
    pub public_key: [u8; 32],
    
    /// Whether validator is currently eligible for committee selection
    pub is_eligible: bool,
    
    /// Constellation ID this validator belongs to
    pub constellation_id: String,
}

impl ValidatorInfo {
    /// Create a new validator info
    pub fn new(
        validator_id: String,
        public_key: [u8; 32],
        constellation_id: String,
    ) -> Self {
        Self {
            validator_id,
            public_key,
            is_eligible: true,
            constellation_id,
        }
    }
}

/// Messages exchanged during consensus
#[derive(Debug, Clone)]
pub enum ConsensusMessage {
    /// Block proposal from a builder
    Proposal {
        height: u64,
        round: u64,
        proposer_id: String,
        block_hash: [u8; 32],
        efficiency_score: u64,
        /// Serialized block data
        block_data: Vec<u8>,
        /// Proposer's signature
        signature: [u8; 64],
    },
    
    /// Ranked vote for best proposal
    RankedVote {
        height: u64,
        round: u64,
        /// Hash of proposal being voted for
        block_hash: [u8; 32],
        /// Efficiency score of chosen proposal
        efficiency_score: u64,
        validator_id: String,
        signature: [u8; 64],
    },
    
    /// Commit proof (2/3+ signatures)
    Commit {
        height: u64,
        round: u64,
        block_hash: [u8; 32],
        /// Signatures from 2/3+ committee members
        signatures: Vec<CommitSignatureMsg>,
    },
}

/// Signature included in commit proof
#[derive(Debug, Clone)]
pub struct CommitSignatureMsg {
    pub validator_id: String,
    pub signature: [u8; 64],
}

impl ConsensusMessage {
    /// Get the height this message is for
    pub fn height(&self) -> u64 {
        match self {
            ConsensusMessage::Proposal { height, .. } => *height,
            ConsensusMessage::RankedVote { height, .. } => *height,
            ConsensusMessage::Commit { height, .. } => *height,
        }
    }
    
    /// Get the round this message is for
    pub fn round(&self) -> u64 {
        match self {
            ConsensusMessage::Proposal { round, .. } => *round,
            ConsensusMessage::RankedVote { round, .. } => *round,
            ConsensusMessage::Commit { round, .. } => *round,
        }
    }
}

/// Errors that can occur during consensus
#[derive(Debug, Error)]
pub enum ConsensusError {
    #[error("Invalid proposal: {0}")]
    InvalidProposal(String),
    
    #[error("Invalid vote: {0}")]
    InvalidVote(String),
    
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
    
    #[error("Validator not in committee: {0}")]
    NotInCommittee(String),
    
    #[error("Quorum not reached")]
    QuorumNotReached,
    
    #[error("Wrong height: expected {expected}, got {got}")]
    WrongHeight { expected: u64, got: u64 },
    
    #[error("Wrong round: expected {expected}, got {got}")]
    WrongRound { expected: u64, got: u64 },
    
    #[error("Timeout in step {step}")]
    Timeout { step: RoundStep },
    
    #[error("Duplicate vote from validator: {0}")]
    DuplicateVote(String),
    
    #[error("Equivocation detected: validator {validator_id} signed conflicting messages")]
    Equivocation { validator_id: String },
    
    #[error("Block validation failed: {0}")]
    BlockValidation(String),
    
    #[error("Efficiency mismatch: claimed {claimed}, actual {actual}")]
    EfficiencyMismatch { claimed: u64, actual: u64 },
    
    #[error("Below reference efficiency: {proposal} < {reference}")]
    BelowReference { proposal: u64, reference: u64 },
    
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type for consensus operations
pub type ConsensusResult<T> = Result<T, ConsensusError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consensus_config_default() {
        let config = ConsensusConfig::default();
        assert_eq!(config.chain_id, "self-chain-mainnet");
        assert_eq!(config.timeout_propose_window, Duration::from_secs(50));
        assert_eq!(config.timeout_voting, Duration::from_secs(8));
        assert_eq!(config.timeout_finalize, Duration::from_secs(2));
    }

    #[test]
    fn test_round_duration() {
        let config = ConsensusConfig::default();
        assert_eq!(config.round_duration(), Duration::from_secs(60));
    }

    #[test]
    fn test_quorum_threshold() {
        let config = ConsensusConfig::default();
        
        // 10 validators: 2/3 = 6.67, +1 = 7
        assert_eq!(config.quorum_threshold(10), 7);
        
        // 100 validators: 2/3 = 66.67, +1 = 67
        assert_eq!(config.quorum_threshold(100), 67);
    }

    #[test]
    fn test_round_state_advance() {
        let mut state = RoundState::new(1);
        assert_eq!(state.height, 1);
        assert_eq!(state.round, 0);
        assert_eq!(state.step, RoundStep::ProposeWindow);
        
        state.advance_round();
        assert_eq!(state.round, 1);
        assert_eq!(state.step, RoundStep::ProposeWindow);
    }

    #[test]
    fn test_round_state_new_height() {
        let mut state = RoundState::new(1);
        state.round = 2;
        state.winner_hash = Some([1u8; 32]);
        
        state.new_height(2);
        assert_eq!(state.height, 2);
        assert_eq!(state.round, 0);
        assert_eq!(state.winner_hash, None);
    }
    
    #[test]
    fn test_validator_info() {
        let validator = ValidatorInfo::new(
            "validator-1".to_string(),
            [0xAB; 32],
            "test-constellation".to_string(),
        );
        
        assert_eq!(validator.validator_id, "validator-1");
        assert!(validator.is_eligible);
    }
}
