// Vote module for the consensus system
// Contains types used for voting on blocks in the PoAI consensus mechanism

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Represents a vote cast by a validator for a block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    /// Hash of the block being voted on
    pub block_hash: String,
    
    /// ID of the validator casting the vote
    pub validator_id: String,
    
    /// Validation score (higher is better)
    pub score: u64,
    
    /// Timestamp when the vote was cast (seconds since epoch)
    pub timestamp: u64,
    
    /// Digital signature of the vote data
    pub signature: Option<String>,
}

impl Vote {
    /// Create a new vote for a block
    pub fn new(block_hash: String, validator_id: String, score: u64) -> Self {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)
            .expect("Time went backwards").as_secs();
            
        Self {
            block_hash,
            validator_id,
            score,
            timestamp: now,
            signature: None,
        }
    }
    
    /// Sign the vote with the validator's key
    pub fn sign(&mut self, signature: String) {
        self.signature = Some(signature);
    }
    
    /// Verify the vote signature
    pub fn verify_signature(&self) -> bool {
        // In a real implementation, this would verify the digital signature
        // For now, just check that a signature exists
        self.signature.is_some()
    }
}

/// The result of a voting round
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotingResult {
    /// Hash of the block that was voted on
    pub block_hash: String,
    
    /// Total number of votes cast
    pub total_votes: u64,
    
    /// Number of validators who participated
    pub participants: u64,
    
    /// Average score across all votes
    pub average_score: f64,
    
    /// Whether the block was approved
    pub approved: bool,
    
    /// Collection of all votes
    pub votes: HashMap<String, Vote>,
    
    /// Timestamp when voting completed
    pub timestamp: u64,
}

impl VotingResult {
    /// Create a new voting result
    pub fn new(block_hash: String, votes: HashMap<String, Vote>, approved: bool) -> Self {
        let total_votes = votes.len() as u64;
        let participants = votes.len() as u64;
        
        let sum: u64 = votes.values().map(|v| v.score).sum();
        let average_score = if total_votes > 0 {
            sum as f64 / total_votes as f64
        } else {
            0.0
        };
        
        let now = SystemTime::now().duration_since(UNIX_EPOCH)
            .expect("Time went backwards").as_secs();
            
        Self {
            block_hash,
            total_votes,
            participants,
            average_score,
            approved,
            votes,
            timestamp: now,
        }
    }
}
