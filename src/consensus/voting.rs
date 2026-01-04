//! # Voting System
//!
//! The VotingSystem implements the decentralized voting mechanism for PoAI consensus.
//!
//! ## Voting Process
//!
//! 1. Block proposer initiates voting round
//! 2. Eligible validators cast votes
//! 3. Votes are validated and counted
//! 4. Block is selected based on majority vote
//!
//! ## Configuration
//!
//! - `voting_window`: Duration of voting round in seconds
//! - `min_voters`: Minimum number of voters required
//! - `min_participation`: Minimum participation rate (0.0 - 1.0)

use crate::blockchain::Block;
use crate::consensus::error::ConsensusError;
use crate::consensus::metrics::ConsensusMetrics;
use crate::consensus::vote::{Vote, VotingResult};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

/// Configuration for the voting system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotingConfig {
    /// Duration of voting window in seconds
    pub voting_window: u64,
    /// Minimum number of voters required
    pub min_voters: u64,
    /// Minimum participation rate (0.0 - 1.0)
    pub min_participation: f64,
}

impl Default for VotingConfig {
    fn default() -> Self {
        Self {
            voting_window: 60,        // 60 seconds (1 minute rounds)
            min_voters: 3,
            min_participation: 0.5,   // 50% participation required
        }
    }
}

/// VotingSystem manages the decentralized voting process
#[derive(Debug)]
pub struct VotingSystem {
    config: VotingConfig,
    votes: Arc<RwLock<HashMap<String, Vote>>>,
    current_round: Arc<RwLock<Option<VotingRound>>>,
    metrics: Arc<ConsensusMetrics>,
}

/// Active voting round
#[derive(Debug, Clone)]
pub struct VotingRound {
    pub block_hash: String,
    pub started_at: u64,
    pub ends_at: u64,
}

impl VotingSystem {
    /// Create a new voting system with default configuration
    pub fn new(metrics: Arc<ConsensusMetrics>) -> Self {
        Self {
            config: VotingConfig::default(),
            votes: Arc::new(RwLock::new(HashMap::new())),
            current_round: Arc::new(RwLock::new(None)),
            metrics,
        }
    }

    /// Create a new voting system with custom configuration
    pub fn with_config(config: VotingConfig, metrics: Arc<ConsensusMetrics>) -> Self {
        Self {
            config,
            votes: Arc::new(RwLock::new(HashMap::new())),
            current_round: Arc::new(RwLock::new(None)),
            metrics,
        }
    }

    /// Start a new voting round for a block
    pub async fn start_voting_round(&self, block: &Block) -> Result<(), ConsensusError> {
        let current = self.current_round.read().await;
        if current.is_some() {
            return Err(ConsensusError::VotingError(
                "Voting round already in progress".to_string(),
            ));
        }
        drop(current);

        // Clear previous votes
        self.votes.write().await.clear();

        // Create new round
        let now = Self::current_timestamp();
        let round = VotingRound {
            block_hash: block.hash.clone(),
            started_at: now,
            ends_at: now + self.config.voting_window,
        };

        *self.current_round.write().await = Some(round);
        self.metrics.increment_voting_rounds_started();

        Ok(())
    }

    /// Cast a vote for the current round
    pub async fn cast_vote(
        &self,
        validator_id: &str,
        block_hash: &str,
        score: u64,
    ) -> Result<(), ConsensusError> {
        // Verify voting round is active
        let round = self.current_round.read().await;
        let round = round.as_ref().ok_or_else(|| {
            ConsensusError::VotingError("No active voting round".to_string())
        })?;

        if round.block_hash != block_hash {
            return Err(ConsensusError::VotingError(
                "Vote for wrong block hash".to_string(),
            ));
        }

        // Check if voting window is still open
        let now = Self::current_timestamp();
        if now > round.ends_at {
            return Err(ConsensusError::VotingError(
                "Voting window has closed".to_string(),
            ));
        }
        drop(round);

        // Create and store vote
        let vote = Vote::new(block_hash.to_string(), validator_id.to_string(), score);
        self.votes.write().await.insert(validator_id.to_string(), vote);
        self.metrics.increment_votes_cast();

        Ok(())
    }

    /// End the current voting round and calculate results
    pub async fn end_voting_round(&self) -> Result<VotingResult, ConsensusError> {
        let round = self.current_round.write().await.take().ok_or_else(|| {
            ConsensusError::VotingError("No active voting round".to_string())
        })?;

        let votes = self.votes.read().await;
        let vote_count = votes.len();

        // Check minimum participation
        if (vote_count as u64) < self.config.min_voters {
            return Err(ConsensusError::InsufficientParticipation(
                self.config.min_participation * 100.0,
            ));
        }

        // Calculate average score
        let total_score: u64 = votes.values().map(|v| v.score).sum();
        let avg_score = if vote_count > 0 {
            total_score as f64 / vote_count as f64
        } else {
            0.0
        };

        // Block is approved if average score > 50
        let approved = avg_score > 50.0;

        // Build result
        let vote_map: HashMap<String, Vote> = votes
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        self.metrics.observe_voting_participation_rate(vote_count as f64 / 10.0); // Assuming 10 validators

        Ok(VotingResult::new(round.block_hash, vote_map, approved))
    }

    /// Get the current voting round status
    pub async fn get_current_round(&self) -> Option<VotingRound> {
        self.current_round.read().await.clone()
    }

    /// Check if a validator has already voted in the current round
    pub async fn has_voted(&self, validator_id: &str) -> bool {
        self.votes.read().await.contains_key(validator_id)
    }

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blockchain::{Block, BlockHeader, BlockMeta};

    fn create_test_block() -> Block {
        Block {
            header: BlockHeader {
                index: 1,
                timestamp: 1704067200,
                previous_hash: "0000000000".to_string(),
                ai_threshold: 5,
            },
            transactions: vec![],
            meta: BlockMeta::default(),
            hash: "test_block_hash".to_string(),
        }
    }

    #[tokio::test]
    async fn test_start_voting_round() {
        let registry = prometheus::Registry::new();
        let metrics = Arc::new(ConsensusMetrics::new(&registry).unwrap());
        let voting = VotingSystem::new(metrics);

        let block = create_test_block();
        voting.start_voting_round(&block).await.unwrap();

        let round = voting.get_current_round().await;
        assert!(round.is_some());
        assert_eq!(round.unwrap().block_hash, "test_block_hash");
    }

    #[tokio::test]
    async fn test_cast_vote() {
        let registry = prometheus::Registry::new();
        let metrics = Arc::new(ConsensusMetrics::new(&registry).unwrap());
        let voting = VotingSystem::new(metrics);

        let block = create_test_block();
        voting.start_voting_round(&block).await.unwrap();

        voting.cast_vote("validator-001", "test_block_hash", 75).await.unwrap();

        assert!(voting.has_voted("validator-001").await);
        assert!(!voting.has_voted("validator-002").await);
    }
}
