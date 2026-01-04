//! # SELF Chain Core
//!
//! A Proof-of-AI (PoAI) consensus blockchain framework.
//!
//! ## Overview
//!
//! SELF Chain implements the patent-pending Proof-of-AI consensus mechanism,
//! replacing computational waste (PoW) and wealth concentration (PoS) with
//! AI-powered validation.
//!
//! ## Key Components
//!
//! - **Consensus**: PoAI validation, voting, and block selection
//! - **Crypto**: Hybrid cryptography (classic + post-quantum ready)
//! - **Blockchain**: Block and transaction types
//! - **Node**: Three node types (Validator, Builder, Coordinator)
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use self_chain_core::consensus::{TransactionSelector, TransactionSelectorConfig};
//! use self_chain_core::blockchain::Transaction;
//!
//! // Configure transaction selector with PoAI 20/20/50/10 algorithm
//! let config = TransactionSelectorConfig::default();
//! let selector = TransactionSelector::new(config);
//!
//! // Select transactions for a block
//! let mempool: Vec<Transaction> = vec![/* ... */];
//! let selected = selector.select_transactions(mempool).unwrap();
//! ```
//!
//! ## Constellation Architecture
//!
//! A "Constellation" is an independent deployment of SELF Chain with its own:
//! - Network configuration
//! - Reward mechanism
//! - Token economics
//! - Governance model
//!
//! All Constellations share the same PoAI consensus core.

pub mod blockchain;
pub mod consensus;
pub mod crypto;
pub mod node;

// Re-export commonly used types
pub use blockchain::{Block, BlockHeader, BlockMeta, Transaction, TransactionData};
pub use consensus::{
    TransactionSelector, TransactionSelectorConfig, SelectedTransactions, BlockEfficiency,
    ConsensusError, ConsensusMetrics, ValidationCache,
};
pub use crypto::{
    MasterKey, ValidatorKey, KeyManager, KeyOperation,
    CryptoError, CryptoResult, CryptoAlgorithm,
};
pub use node::{
    NodeType, NodeConfig, ValidatorNode, BlockBuilderNode, CoordinatorNode,
    Vote, ValidatorStats, BlockProposal, BlockBuilderStats, VotingRound,
};

