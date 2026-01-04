//! Blockchain Core Types
//!
//! This module provides the fundamental blockchain types for PoAI consensus:
//! - Block: Container for transactions with PoAI metadata
//! - Transaction: Signed transaction with optional data payload
//! - BlockHeader: Block metadata including AI validation threshold
//! - BlockMeta: Additional block statistics

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Transaction data payload for different transaction types
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TransactionData {
    /// Validator participation record
    ValidatorParticipation {
        user_id: String,
        validator_id: String,
        round: u64,
        activity_score: u32,
    },
    /// Reward distribution record
    RewardDistribution {
        round: u64,
        builder_id: String,
        builder_amount: f64,
        voter_rewards: HashMap<String, f64>,
        proposer_reward: f64,
        network_reward: f64,
    },
    /// Block builder win record
    BlockBuilderWin {
        round: u64,
        builder_id: String,
        block_hash: String,
        efficiency_score: f64,
    },
    /// Generic transfer
    Transfer {
        amount: u64,
        token_address: Option<String>,
    },
}

/// Block header containing essential block metadata
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct BlockHeader {
    /// Block index in the chain
    pub index: u64,
    /// Unix timestamp when block was created
    pub timestamp: u64,
    /// Hash of the previous block
    pub previous_hash: String,
    /// AI validation threshold level (1-10, higher = stricter)
    pub ai_threshold: u32,
}

/// Additional block metadata
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct BlockMeta {
    /// Size of the block in bytes
    pub size: u64,
    /// Number of transactions in the block
    pub tx_count: u64,
    /// Block height (same as index)
    pub height: u64,
    /// Validator's signature over the block
    pub validator_signature: Option<String>,
    /// ID of the validator who signed
    pub validator_id: Option<String>,
}

/// A block in the PoAI blockchain
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Block {
    /// Block header with essential metadata
    pub header: BlockHeader,
    /// Transactions contained in this block
    pub transactions: Vec<Transaction>,
    /// Additional block metadata
    pub meta: BlockMeta,
    /// Hash of this block
    pub hash: String,
}

impl Default for Block {
    fn default() -> Self {
        Self {
            header: BlockHeader::default(),
            transactions: Vec::new(),
            meta: BlockMeta::default(),
            hash: String::new(),
        }
    }
}

impl Block {
    /// Calculate the size of this block
    pub fn calculate_size(&self) -> u64 {
        let mut size = 0u64;
        size += self.header.index.to_string().len() as u64;
        size += self.header.timestamp.to_string().len() as u64;
        size += self.header.previous_hash.len() as u64;
        size += self.header.ai_threshold.to_string().len() as u64;
        size += self.transactions.iter().map(|tx| tx.calculate_size()).sum::<u64>();
        size
    }

    /// Calculate the hash of this block
    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(format!(
            "{}{}{}{}{}{}",
            self.header.index,
            self.header.timestamp,
            self.header.previous_hash,
            self.header.ai_threshold,
            serde_json::to_string(&self.transactions).unwrap_or_default(),
            self.meta.size
        ));
        format!("{:x}", hasher.finalize())
    }

    /// Verify the block structure
    pub fn verify(&self) -> bool {
        !self.hash.is_empty()
            && !self.header.previous_hash.is_empty()
            && self.header.timestamp > 0
    }
}

/// A transaction in the PoAI blockchain
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct Transaction {
    /// Unique transaction identifier
    pub id: String,
    /// Sender's address (public key in hex format)
    pub sender: String,
    /// Receiver's address
    pub receiver: String,
    /// Transaction amount
    pub amount: u64,
    /// Signature of the transaction data
    pub signature: String,
    /// Timestamp when the transaction was created
    pub timestamp: u64,
    /// Optional transaction-specific data payload
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<TransactionData>,
}

impl Transaction {
    /// Create a new transaction
    pub fn new(
        id: String,
        sender: String,
        receiver: String,
        amount: u64,
        signature: String,
        timestamp: u64,
    ) -> Self {
        Transaction {
            id,
            sender,
            receiver,
            amount,
            signature,
            timestamp,
            data: None,
        }
    }

    /// Calculate the size of this transaction
    pub fn calculate_size(&self) -> u64 {
        self.id.len() as u64
            + self.sender.len() as u64
            + self.receiver.len() as u64
            + self.amount.to_string().len() as u64
            + self.signature.len() as u64
            + self.timestamp.to_string().len() as u64
    }

    /// Verify the transaction structure
    pub fn verify(&self) -> bool {
        !self.id.is_empty()
            && !self.sender.is_empty()
            && !self.receiver.is_empty()
            && !self.signature.is_empty()
            && self.timestamp > 0
    }

    /// Generate a hash of this transaction
    pub fn hash(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.id.hash(&mut hasher);
        self.sender.hash(&mut hasher);
        self.receiver.hash(&mut hasher);
        self.amount.hash(&mut hasher);
        self.timestamp.hash(&mut hasher);
        self.signature.hash(&mut hasher);

        format!("{:x}", hasher.finish())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_creation() {
        let tx = Transaction::new(
            "tx_001".to_string(),
            "sender_abc".to_string(),
            "receiver_xyz".to_string(),
            1000,
            "signature_123".to_string(),
            1704067200,
        );

        assert_eq!(tx.id, "tx_001");
        assert!(tx.verify());
    }

    #[test]
    fn test_block_hash() {
        let block = Block {
            header: BlockHeader {
                index: 1,
                timestamp: 1704067200,
                previous_hash: "0000000000".to_string(),
                ai_threshold: 5,
            },
            transactions: vec![],
            meta: BlockMeta {
                size: 100,
                tx_count: 0,
                height: 1,
                validator_signature: None,
                validator_id: None,
            },
            hash: String::new(),
        };

        let hash = block.calculate_hash();
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64); // SHA256 produces 64 hex characters
    }
}

