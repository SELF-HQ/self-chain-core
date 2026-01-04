//! PoAI Node Types
//!
//! Implements the three node types specified in the PoAI consensus mechanism:
//! 1. Validator Nodes (Lite nodes) - Vote and validate
//! 2. Block Builder Nodes (Full nodes) - Build blocks and earn rewards
//! 3. PoAI Coordinator - Organize voting and generate reference blocks

use crate::blockchain::{Block, BlockHeader, BlockMeta, Transaction};
use crate::consensus::{
    TransactionSelector, TransactionSelectorConfig, ConsensusMetrics, ValidationCache,
};
use crate::consensus::validator::Validator;
use crate::crypto::{MasterKey, ValidatorKey, KeyManager};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Node type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    /// Lite node that votes and validates (every user runs one)
    Validator,

    /// Full node that builds blocks
    BlockBuilder,

    /// Coordinator that organizes voting and generates reference blocks
    Coordinator,
}

/// Configuration for any node type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub node_id: String,
    pub node_type: NodeType,
    pub listen_addr: String,
    pub bootstrap_peers: Vec<String>,
}

/// Validator Node (Lite Node)
///
/// Per PoAI spec, validators are lightweight nodes that:
/// - Store wallet colors (~10MB)
/// - Vote on block proposals
/// - Validate color markers
/// - Don't store full blockchain
pub struct ValidatorNode {
    config: NodeConfig,
    key_manager: KeyManager,
    validator: Arc<Validator>,

    /// Wallet colors (address -> color)
    wallet_colors: HashMap<String, String>,

    /// Voting history
    voting_history: Vec<VoteRecord>,

    /// Validator key for signing votes
    validator_key: Option<ValidatorKey>,
}

impl ValidatorNode {
    pub fn new(config: NodeConfig) -> Result<Self> {
        let registry = prometheus::Registry::new();
        let metrics = Arc::new(ConsensusMetrics::new(&registry)?);
        let cache = Arc::new(ValidationCache::new(metrics.clone()));
        let validator = Arc::new(Validator::new(metrics, cache));

        Ok(Self {
            config,
            key_manager: KeyManager::new(),
            validator,
            wallet_colors: HashMap::new(),
            voting_history: Vec::new(),
            validator_key: None,
        })
    }

    /// Initialize validator with master key
    pub fn initialize_with_master_key(&mut self, master_key: MasterKey) -> Result<()> {
        let address = master_key.address().to_string();
        self.key_manager.import_master_key(master_key.export_private_key())?;

        // Derive validator key
        let nonce = rand::random::<[u8; 32]>();
        let validator_key = master_key.derive_validator_key(&nonce)?;
        self.validator_key = Some(validator_key);

        tracing::info!("Validator initialized for address: {}", address);
        Ok(())
    }

    /// Vote on a block proposal
    pub async fn vote_on_block(&mut self, block: &Block, approve: bool) -> Result<Vote> {
        let validator_key = self.validator_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Validator key not initialized"))?;

        // Validate block first
        let is_valid = self.validate_block(block).await?;

        if !is_valid && approve {
            return Err(anyhow::anyhow!("Cannot approve invalid block"));
        }

        // Sign vote
        let block_hash = block.hash.as_bytes();
        let signature = validator_key.sign_vote(block_hash, approve)?;

        let vote = Vote {
            validator_id: self.config.node_id.clone(),
            block_hash: hex::encode(block_hash),
            approve,
            signature,
            timestamp: Self::current_timestamp(),
        };

        // Record in history
        self.voting_history.push(VoteRecord {
            block_hash: vote.block_hash.clone(),
            approved: approve,
            timestamp: vote.timestamp,
        });

        Ok(vote)
    }

    /// Validate a block using color markers
    pub async fn validate_block(&self, block: &Block) -> Result<bool> {
        for tx in &block.transactions {
            let current_color = self.wallet_colors.get(&tx.sender)
                .cloned()
                .unwrap_or_else(|| "000000".to_string());

            let hex_tx = self.validator.calculate_hex_transaction(tx)?;
            let expected_color = self.validator.calculate_new_color(&current_color, &hex_tx)?;

            if !self.validator.validate_color_transition(&current_color, &expected_color)? {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Update wallet colors after block is accepted
    pub fn update_colors_from_block(&mut self, block: &Block) -> Result<()> {
        for tx in &block.transactions {
            let current_color = self.wallet_colors.get(&tx.sender)
                .cloned()
                .unwrap_or_else(|| "000000".to_string());

            let hex_tx = self.validator.calculate_hex_transaction(tx)?;
            let new_color = self.validator.calculate_new_color(&current_color, &hex_tx)?;

            self.wallet_colors.insert(tx.sender.clone(), new_color);
        }

        Ok(())
    }

    /// Get validator statistics
    pub fn get_stats(&self) -> ValidatorStats {
        let total_votes = self.voting_history.len();
        let approved_votes = self.voting_history.iter().filter(|v| v.approved).count();

        ValidatorStats {
            node_id: self.config.node_id.clone(),
            total_votes,
            approved_votes,
            rejected_votes: total_votes - approved_votes,
            wallet_colors_stored: self.wallet_colors.len(),
        }
    }

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

/// Block Builder Node (Full Node)
///
/// Per PoAI spec, block builders:
/// - Build blocks using 20/20/50/10 algorithm
/// - Compete on efficiency
/// - Earn 90% of block rewards (in default PoAI distribution)
pub struct BlockBuilderNode {
    config: NodeConfig,
    transaction_selector: TransactionSelector,

    /// Mempool of pending transactions
    mempool: Vec<Transaction>,

    /// Blocks built by this builder
    blocks_built: Vec<Block>,

    /// Builder statistics
    stats: BlockBuilderStats,
}

impl BlockBuilderNode {
    pub fn new(config: NodeConfig) -> Self {
        let selector_config = TransactionSelectorConfig::default();
        let transaction_selector = TransactionSelector::new(selector_config);

        Self {
            config,
            transaction_selector,
            mempool: Vec::new(),
            blocks_built: Vec::new(),
            stats: BlockBuilderStats::default(),
        }
    }

    /// Add transaction to mempool
    pub fn add_to_mempool(&mut self, tx: Transaction) {
        self.mempool.push(tx);
    }

    /// Get mempool size
    pub fn mempool_size(&self) -> usize {
        self.mempool.len()
    }

    /// Build a block using PoAI 20/20/50/10 algorithm
    pub fn build_block(&mut self, previous_hash: String) -> Result<BlockProposal> {
        if self.mempool.is_empty() {
            return Err(anyhow::anyhow!("Mempool is empty"));
        }

        // Select transactions using PoAI algorithm
        let selected = self.transaction_selector.select_transactions(self.mempool.clone())?;

        // Calculate efficiency
        let efficiency = self.transaction_selector.calculate_block_efficiency(&selected)?;

        // Create block
        let transactions = selected.into_transactions();
        let block = Block {
            header: BlockHeader {
                index: self.blocks_built.len() as u64,
                timestamp: Self::current_timestamp(),
                previous_hash,
                ai_threshold: 5,
            },
            transactions,
            meta: BlockMeta {
                size: 0,
                tx_count: 0,
                height: 0,
                validator_signature: None,
                validator_id: None,
            },
            hash: String::new(),
        };

        // Remove selected transactions from mempool
        self.mempool.retain(|tx| !block.transactions.contains(tx));

        // Update stats
        self.stats.blocks_built += 1;
        self.stats.total_efficiency += efficiency.efficiency_score;
        self.stats.avg_efficiency = self.stats.total_efficiency / self.stats.blocks_built as f64;

        Ok(BlockProposal {
            builder_id: self.config.node_id.clone(),
            block,
            efficiency: efficiency.efficiency_score,
            timestamp: Self::current_timestamp(),
        })
    }

    /// Get builder statistics
    pub fn get_stats(&self) -> &BlockBuilderStats {
        &self.stats
    }

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

/// PoAI Coordinator Node
///
/// Per PoAI spec, the coordinator:
/// - Organizes voting rounds
/// - Generates reference blocks
/// - Manages timeouts and rewards
/// - Cannot influence voting results
pub struct CoordinatorNode {
    config: NodeConfig,
    transaction_selector: TransactionSelector,

    /// Active voting round
    current_round: Option<VotingRound>,

    /// Completed rounds
    pub completed_rounds: Vec<VotingRound>,

    /// Reference block for current round
    reference_block: Option<Block>,
}

impl CoordinatorNode {
    pub fn new(config: NodeConfig) -> Self {
        let selector_config = TransactionSelectorConfig::default();
        let transaction_selector = TransactionSelector::new(selector_config);

        Self {
            config,
            transaction_selector,
            current_round: None,
            completed_rounds: Vec::new(),
            reference_block: None,
        }
    }

    /// Start a new voting round
    pub fn start_voting_round(
        &mut self,
        proposals: Vec<BlockProposal>,
        mempool: Vec<Transaction>,
        previous_hash: String,
    ) -> Result<VotingRound> {
        // Generate reference block using same algorithm as builders
        let selected = self.transaction_selector.select_transactions(mempool)?;
        let efficiency = self.transaction_selector.calculate_block_efficiency(&selected)?;

        let reference_block = Block {
            header: BlockHeader {
                index: 0,
                timestamp: Self::current_timestamp(),
                previous_hash,
                ai_threshold: 5,
            },
            transactions: selected.into_transactions(),
            meta: BlockMeta::default(),
            hash: String::new(),
        };

        self.reference_block = Some(reference_block.clone());

        let round = VotingRound {
            round_id: self.completed_rounds.len() as u64,
            proposals,
            reference_block,
            reference_efficiency: efficiency.efficiency_score,
            votes: HashMap::new(),
            started_at: Self::current_timestamp(),
            ended_at: None,
            winner: None,
        };

        self.current_round = Some(round.clone());
        Ok(round)
    }

    /// Add vote to current round
    pub fn add_vote(&mut self, vote: Vote) -> Result<()> {
        let round = self.current_round.as_mut()
            .ok_or_else(|| anyhow::anyhow!("No active voting round"))?;

        round.votes.insert(vote.validator_id.clone(), vote);
        Ok(())
    }

    /// End voting round and determine winner
    pub fn end_voting_round(&mut self) -> Result<VotingResult> {
        let mut round = self.current_round.take()
            .ok_or_else(|| anyhow::anyhow!("No active voting round"))?;

        // Count votes for each proposal
        let mut vote_counts: HashMap<String, usize> = HashMap::new();

        for vote in round.votes.values() {
            if vote.approve {
                *vote_counts.entry(vote.block_hash.clone()).or_insert(0) += 1;
            }
        }

        // Find winner (most votes)
        let winner = vote_counts.iter()
            .max_by_key(|(_, count)| *count)
            .map(|(hash, _)| hash.clone());

        round.ended_at = Some(Self::current_timestamp());
        round.winner = winner.clone();

        self.completed_rounds.push(round);

        Ok(VotingResult {
            round_id: self.completed_rounds.len() as u64 - 1,
            winner,
            total_votes: vote_counts.values().sum(),
        })
    }

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

/// Vote from a validator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub validator_id: String,
    pub block_hash: String,
    pub approve: bool,
    pub signature: Vec<u8>,
    pub timestamp: u64,
}

/// Vote record for validator history
#[derive(Debug, Clone)]
struct VoteRecord {
    block_hash: String,
    approved: bool,
    timestamp: u64,
}

/// Validator statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorStats {
    pub node_id: String,
    pub total_votes: usize,
    pub approved_votes: usize,
    pub rejected_votes: usize,
    pub wallet_colors_stored: usize,
}

/// Block proposal from a builder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockProposal {
    pub builder_id: String,
    pub block: Block,
    pub efficiency: f64,
    pub timestamp: u64,
}

/// Block builder statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlockBuilderStats {
    pub blocks_built: u64,
    pub blocks_accepted: u64,
    pub blocks_rejected: u64,
    pub total_efficiency: f64,
    pub avg_efficiency: f64,
}

/// Voting round
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotingRound {
    pub round_id: u64,
    pub proposals: Vec<BlockProposal>,
    pub reference_block: Block,
    pub reference_efficiency: f64,
    pub votes: HashMap<String, Vote>,
    pub started_at: u64,
    pub ended_at: Option<u64>,
    pub winner: Option<String>,
}

/// Voting result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotingResult {
    pub round_id: u64,
    pub winner: Option<String>,
    pub total_votes: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_builder_creation() {
        let config = NodeConfig {
            node_id: "builder1".to_string(),
            node_type: NodeType::BlockBuilder,
            listen_addr: "127.0.0.1:9001".to_string(),
            bootstrap_peers: vec![],
        };

        let node = BlockBuilderNode::new(config);
        assert_eq!(node.config.node_type, NodeType::BlockBuilder);
        assert_eq!(node.mempool_size(), 0);
    }

    #[test]
    fn test_coordinator_creation() {
        let config = NodeConfig {
            node_id: "coordinator1".to_string(),
            node_type: NodeType::Coordinator,
            listen_addr: "127.0.0.1:10001".to_string(),
            bootstrap_peers: vec![],
        };

        let coordinator = CoordinatorNode::new(config);
        assert!(coordinator.current_round.is_none());
        assert!(coordinator.completed_rounds.is_empty());
    }
}
