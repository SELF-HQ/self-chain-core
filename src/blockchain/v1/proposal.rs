//! PoAI v1 Block Proposal Type
//!
//! Spec-compliant block proposal structure as defined in `docs/POAI_SPECIFICATION.md`.
//!
//! ## PoAI Competition Model
//!
//! In PoAI, ALL eligible builders can submit proposals (not just a single VRF-selected
//! proposer). Validators vote for the BEST proposal by efficiency score.
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │              PoAI Proposal Flow                              │
//! ├─────────────────────────────────────────────────────────────┤
//! │  0-50s   │ All builders submit proposals                    │
//! │  50-58s  │ Validators vote for best (by efficiency)         │
//! │  58-60s  │ Winner with 2/3+ votes is finalized              │
//! └─────────────────────────────────────────────────────────────┘
//! ```

use crate::blockchain::v1::Block;

/// PoAI v1 Block Proposal (spec-compliant)
///
/// Submitted by builders during the proposal window.
///
/// ## Wire Format
///
/// ```text
/// signature = Ed25519.sign(
///     proposer_key,
///     "self-chain-proposal-v1" || bincode(proposal_without_signature)
/// )
/// ```
///
/// ## Efficiency Score
///
/// The `efficiency_score` in the block header is calculated deterministically:
///
/// ```text
/// efficiency = (total_points / block_size) * fill_ratio * price_stability
/// ```
///
/// Validators independently verify this calculation matches the claimed score.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockProposal {
    /// Block height
    pub height: u64,
    
    /// Consensus round
    pub round: u64,
    
    /// Validator ID of the proposer
    pub proposer_id: String,
    
    /// Full block being proposed
    pub block: Block,
    
    /// Ed25519 signature of proposer (64 bytes)
    pub signature: [u8; 64],
}

impl BlockProposal {
    /// Domain separation prefix for proposal signatures
    pub const DOMAIN_PREFIX: &'static [u8] = b"self-chain-proposal-v1";
    
    /// Create a new unsigned proposal
    pub fn new(height: u64, round: u64, proposer_id: String, block: Block) -> Self {
        Self {
            height,
            round,
            proposer_id,
            block,
            signature: [0u8; 64],
        }
    }
    
    /// Get the efficiency score from the block header
    pub fn efficiency_score(&self) -> u64 {
        self.block.header.efficiency_score
    }
    
    /// Get transaction count
    pub fn tx_count(&self) -> usize {
        self.block.transactions.len()
    }
}

/// Validated proposal ready for voting
///
/// After a proposal is received and validated, it becomes a `ValidatedProposal`
/// with additional metadata for the voting phase.
#[derive(Debug, Clone)]
pub struct ValidatedProposal {
    /// Original proposal
    pub proposal: BlockProposal,
    
    /// Verified efficiency score (recalculated by validator)
    pub verified_efficiency: u64,
    
    /// Whether this proposal beats the reference block
    pub beats_reference: bool,
    
    /// Efficiency delta vs reference (positive = better)
    pub efficiency_delta: i64,
}

impl ValidatedProposal {
    /// Create a validated proposal
    pub fn new(
        proposal: BlockProposal,
        verified_efficiency: u64,
        reference_efficiency: u64,
    ) -> Self {
        Self {
            verified_efficiency,
            beats_reference: verified_efficiency >= reference_efficiency,
            efficiency_delta: verified_efficiency as i64 - reference_efficiency as i64,
            proposal,
        }
    }
    
    /// Get the proposer ID
    pub fn proposer_id(&self) -> &str {
        &self.proposal.proposer_id
    }
    
    /// Check if efficiency claim was accurate
    pub fn efficiency_matches_claim(&self) -> bool {
        self.verified_efficiency == self.proposal.efficiency_score()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blockchain::v1::BlockHeader;
    
    fn create_test_block(efficiency_score: u64) -> Block {
        let header = BlockHeader {
            height: 1,
            previous_hash: [0u8; 32],
            timestamp: 1704067200,
            state_root: [1u8; 32],
            transactions_root: [2u8; 32],
            proposer_id: "validator-123".to_string(),
            round: 0,
            chain_id: "test-chain".to_string(),
            efficiency_score,
            point_price: 100,
            commit_signatures: vec![],
        };
        
        Block::new(header, vec![])
    }
    
    #[test]
    fn test_block_proposal_creation() {
        let block = create_test_block(5000);
        let proposal = BlockProposal::new(
            1,
            0,
            "validator-123".to_string(),
            block,
        );
        
        assert_eq!(proposal.height, 1);
        assert_eq!(proposal.round, 0);
        assert_eq!(proposal.efficiency_score(), 5000);
        assert_eq!(proposal.tx_count(), 0);
    }
    
    #[test]
    fn test_validated_proposal_beats_reference() {
        let block = create_test_block(6000);
        let proposal = BlockProposal::new(1, 0, "v1".to_string(), block);
        
        // Reference efficiency is 5000, proposal is 6000
        let validated = ValidatedProposal::new(proposal, 6000, 5000);
        
        assert!(validated.beats_reference);
        assert_eq!(validated.efficiency_delta, 1000);
    }
    
    #[test]
    fn test_validated_proposal_below_reference() {
        let block = create_test_block(4000);
        let proposal = BlockProposal::new(1, 0, "v1".to_string(), block);
        
        // Reference efficiency is 5000, proposal is 4000
        let validated = ValidatedProposal::new(proposal, 4000, 5000);
        
        assert!(!validated.beats_reference);
        assert_eq!(validated.efficiency_delta, -1000);
    }
    
    #[test]
    fn test_efficiency_matches_claim() {
        let block = create_test_block(5000);
        let proposal = BlockProposal::new(1, 0, "v1".to_string(), block);
        
        // Verified efficiency matches claim
        let validated = ValidatedProposal::new(proposal.clone(), 5000, 4000);
        assert!(validated.efficiency_matches_claim());
        
        // Verified efficiency doesn't match claim
        let validated_mismatch = ValidatedProposal::new(proposal, 4999, 4000);
        assert!(!validated_mismatch.efficiency_matches_claim());
    }
}
