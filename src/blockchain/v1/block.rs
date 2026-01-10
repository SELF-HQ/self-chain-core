//! PoAI v1 Block Header and Block Types
//!
//! Spec-compliant block structures as defined in `docs/POAI_SPECIFICATION.md`.
//!
//! ## Wire Format
//!
//! Block hash calculation uses SHA-256 with domain separation:
//! ```text
//! Hash = SHA256("self-chain-block-header-v1" || bincode(header))
//! ```

use crate::blockchain::v1::transaction::Transaction;

/// PoAI v1 Block Header (spec-compliant)
///
/// This is the canonical block header format for the v1 protocol.
///
/// ## Canonical Encoding Order
///
/// 1. `height` (u64, little-endian)
/// 2. `previous_hash` (32 bytes)
/// 3. `timestamp` (u64, little-endian)
/// 4. `state_root` (32 bytes)
/// 5. `transactions_root` (32 bytes)
/// 6. `proposer_id` (string, UTF-8, length-prefixed)
/// 7. `round` (u64, little-endian)
/// 8. `chain_id` (string, UTF-8, length-prefixed)
/// 9. `efficiency_score` (u64, little-endian)
/// 10. `point_price` (u64, little-endian)
/// 11. `commit_signatures` (length-prefixed array)
///
/// ## Key Differences from Production
///
/// | Field | Production | v1 Spec |
/// |-------|-----------|---------|
/// | `previous_hash` | `String` | `[u8; 32]` |
/// | `state_root` | Not present | `[u8; 32]` |
/// | `efficiency_score` | Not present | `u64` |
/// | `point_price` | Not present | `u64` |
/// | `chain_id` | Not present | `String` |
///
/// ## Serialization
///
/// Production uses `bincode` with `#[serde(with = "serde_bytes")]` for byte arrays.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockHeader {
    /// Block height (0 = genesis)
    pub height: u64,
    
    /// SHA-256 hash of previous block header (32 bytes)
    pub previous_hash: [u8; 32],
    
    /// Unix timestamp (seconds since epoch)
    pub timestamp: u64,
    
    /// Sparse Merkle Tree root of account state
    pub state_root: [u8; 32],
    
    /// Merkle root of transactions in block
    pub transactions_root: [u8; 32],
    
    /// Validator ID of the block proposer
    pub proposer_id: String,
    
    /// Consensus round number
    pub round: u64,
    
    /// Chain identifier (replay protection)
    pub chain_id: String,
    
    /// Deterministic efficiency score (PoAI competition metric)
    pub efficiency_score: u64,
    
    /// PointPrice for this block
    pub point_price: u64,
    
    /// 2/3+ committee signatures for finality
    pub commit_signatures: Vec<CommitSignature>,
}

/// Commit signature from a committee member
///
/// Included in finalized blocks to prove 2/3+ consensus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitSignature {
    /// Validator ID that signed
    pub validator_id: String,
    
    /// Ed25519 signature (64 bytes)
    pub signature: [u8; 64],
}

impl BlockHeader {
    /// Domain separation prefix for block header signatures
    pub const DOMAIN_PREFIX: &'static [u8] = b"self-chain-block-header-v1";
    
    /// Create a genesis block header
    pub fn genesis(chain_id: &str) -> Self {
        Self {
            height: 0,
            previous_hash: [0u8; 32],
            timestamp: 0,
            state_root: [0u8; 32],
            transactions_root: [0u8; 32],
            proposer_id: String::new(),
            round: 0,
            chain_id: chain_id.to_string(),
            efficiency_score: 0,
            point_price: 0,
            commit_signatures: vec![],
        }
    }
}

/// PoAI v1 Block (spec-compliant)
///
/// Contains a header and list of transactions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    /// Block header with consensus metadata
    pub header: BlockHeader,
    
    /// Transactions in this block (ordered)
    pub transactions: Vec<Transaction>,
}

impl Block {
    /// Create a new block with the given header and transactions
    pub fn new(header: BlockHeader, transactions: Vec<Transaction>) -> Self {
        Self { header, transactions }
    }
    
    /// Get block height
    pub fn height(&self) -> u64 {
        self.header.height
    }
    
    /// Get block round
    pub fn round(&self) -> u64 {
        self.header.round
    }
    
    /// Get transaction count
    pub fn tx_count(&self) -> usize {
        self.transactions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_block_header_structure() {
        let header = BlockHeader {
            height: 1,
            previous_hash: [0u8; 32],
            timestamp: 1704067200,
            state_root: [1u8; 32],
            transactions_root: [2u8; 32],
            proposer_id: "validator-123".to_string(),
            round: 1,
            chain_id: "self-chain-mainnet".to_string(),
            efficiency_score: 1000,
            point_price: 100,
            commit_signatures: vec![],
        };
        
        assert_eq!(header.height, 1);
        assert_eq!(header.chain_id, "self-chain-mainnet");
        assert_eq!(header.efficiency_score, 1000);
        assert_eq!(header.point_price, 100);
    }
    
    #[test]
    fn test_genesis_block_header() {
        let genesis = BlockHeader::genesis("test-chain");
        
        assert_eq!(genesis.height, 0);
        assert_eq!(genesis.previous_hash, [0u8; 32]);
        assert_eq!(genesis.chain_id, "test-chain");
    }
    
    #[test]
    fn test_commit_signature_structure() {
        let sig = CommitSignature {
            validator_id: "validator-456".to_string(),
            signature: [0u8; 64],
        };
        
        assert_eq!(sig.validator_id, "validator-456");
        assert_eq!(sig.signature.len(), 64);
    }
    
    #[test]
    fn test_block_structure() {
        let header = BlockHeader::genesis("test-chain");
        let block = Block::new(header, vec![]);
        
        assert_eq!(block.height(), 0);
        assert_eq!(block.tx_count(), 0);
    }
}
