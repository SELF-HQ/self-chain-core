//! PoAI v1 Vote Type
//!
//! Spec-compliant vote structure as defined in `docs/POAI_SPECIFICATION.md`.
//!
//! ## Vote Types
//!
//! PoAI supports two voting modes:
//!
//! 1. **Tendermint-style** (Prevote/Precommit): Yes/No on a single proposal
//! 2. **PoAI Ranked**: Vote for BEST proposal by efficiency
//!
//! This module defines the wire format for both.

/// Vote step in Tendermint-style consensus
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum VoteStep {
    /// Prevote step (first voting round)
    Prevote = 1,
    /// Precommit step (second voting round)
    Precommit = 2,
}

impl VoteStep {
    /// Get domain separation prefix for this vote step
    pub fn domain_prefix(&self) -> &'static [u8] {
        match self {
            VoteStep::Prevote => b"self-chain-vote-prevote-v1",
            VoteStep::Precommit => b"self-chain-vote-precommit-v1",
        }
    }
}

/// PoAI v1 Vote (spec-compliant)
///
/// Used in Tendermint-style consensus for Prevote/Precommit phases.
///
/// ## Canonical Encoding Order
///
/// 1. `height` (u64, little-endian)
/// 2. `round` (u64, little-endian)
/// 3. `step` (u8: 1=Prevote, 2=Precommit)
/// 4. `block_hash` (32 bytes)
/// 5. `validator_id` (string, UTF-8, length-prefixed)
/// 6. `signature` (64 bytes)
///
/// ## Signature Format
///
/// ```text
/// prefix = "self-chain-vote-prevote-v1" or "self-chain-vote-precommit-v1"
/// message = prefix || bincode(vote_without_signature)
/// signature = Ed25519.sign(validator_key, message)
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vote {
    /// Block height being voted on
    pub height: u64,
    
    /// Consensus round (0-indexed, increments on timeout)
    pub round: u64,
    
    /// Vote step (Prevote or Precommit)
    pub step: VoteStep,
    
    /// Hash of block being voted on (32 bytes)
    ///
    /// All zeros indicates a "nil" vote (no valid proposal received).
    pub block_hash: [u8; 32],
    
    /// Validator ID casting the vote
    pub validator_id: String,
    
    /// Ed25519 signature (64 bytes)
    pub signature: [u8; 64],
}

impl Vote {
    /// Create a new prevote
    pub fn prevote(
        height: u64,
        round: u64,
        block_hash: [u8; 32],
        validator_id: String,
    ) -> Self {
        Self {
            height,
            round,
            step: VoteStep::Prevote,
            block_hash,
            validator_id,
            signature: [0u8; 64],
        }
    }
    
    /// Create a new precommit
    pub fn precommit(
        height: u64,
        round: u64,
        block_hash: [u8; 32],
        validator_id: String,
    ) -> Self {
        Self {
            height,
            round,
            step: VoteStep::Precommit,
            block_hash,
            validator_id,
            signature: [0u8; 64],
        }
    }
    
    /// Create a nil vote (no valid proposal)
    pub fn nil(height: u64, round: u64, step: VoteStep, validator_id: String) -> Self {
        Self {
            height,
            round,
            step,
            block_hash: [0u8; 32], // All zeros = nil
            validator_id,
            signature: [0u8; 64],
        }
    }
    
    /// Check if this is a nil vote
    pub fn is_nil(&self) -> bool {
        self.block_hash == [0u8; 32]
    }
    
    /// Get the domain prefix for signature verification
    pub fn domain_prefix(&self) -> &'static [u8] {
        self.step.domain_prefix()
    }
}

/// PoAI Ranked Vote (competition model)
///
/// In the PoAI competition model, validators vote for the BEST proposal
/// rather than yes/no on a single proposal.
///
/// ## Differences from Tendermint Vote
///
/// | Aspect | Tendermint Vote | Ranked Vote |
/// |--------|----------------|-------------|
/// | Vote content | Yes/No/Nil | Block hash of best proposal |
/// | Rounds | Prevote → Precommit | Single round |
/// | Selection | First valid block | Highest efficiency |
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RankedVote {
    /// Block height
    pub height: u64,
    
    /// Consensus round
    pub round: u64,
    
    /// Hash of the proposal being voted for (best efficiency)
    pub block_hash: [u8; 32],
    
    /// Efficiency score of chosen proposal
    pub efficiency_score: u64,
    
    /// Validator ID
    pub validator_id: String,
    
    /// Ed25519 signature (64 bytes)
    pub signature: [u8; 64],
}

impl RankedVote {
    /// Domain separation prefix for ranked votes
    pub const DOMAIN_PREFIX: &'static [u8] = b"self-chain-ranked-vote-v1";
    
    /// Create a new ranked vote
    pub fn new(
        height: u64,
        round: u64,
        block_hash: [u8; 32],
        efficiency_score: u64,
        validator_id: String,
    ) -> Self {
        Self {
            height,
            round,
            block_hash,
            efficiency_score,
            validator_id,
            signature: [0u8; 64],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vote_step_values() {
        assert_eq!(VoteStep::Prevote as u8, 1);
        assert_eq!(VoteStep::Precommit as u8, 2);
    }
    
    #[test]
    fn test_vote_domain_prefix() {
        let prevote = Vote::prevote(1, 0, [0u8; 32], "v1".to_string());
        let precommit = Vote::precommit(1, 0, [0u8; 32], "v1".to_string());
        
        assert_eq!(prevote.domain_prefix(), b"self-chain-vote-prevote-v1");
        assert_eq!(precommit.domain_prefix(), b"self-chain-vote-precommit-v1");
    }
    
    #[test]
    fn test_nil_vote() {
        let nil_vote = Vote::nil(1, 0, VoteStep::Prevote, "validator-1".to_string());
        
        assert!(nil_vote.is_nil());
        assert_eq!(nil_vote.block_hash, [0u8; 32]);
    }
    
    #[test]
    fn test_non_nil_vote() {
        let block_hash = [1u8; 32];
        let vote = Vote::prevote(1, 0, block_hash, "validator-1".to_string());
        
        assert!(!vote.is_nil());
    }
    
    #[test]
    fn test_ranked_vote() {
        let ranked = RankedVote::new(
            1,
            0,
            [0xAB; 32],
            5000, // efficiency score
            "validator-1".to_string(),
        );
        
        assert_eq!(ranked.height, 1);
        assert_eq!(ranked.efficiency_score, 5000);
        assert_eq!(ranked.block_hash, [0xAB; 32]);
    }
}
