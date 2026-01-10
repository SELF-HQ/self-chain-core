//! PoAI Consensus Module
//!
//! This module implements the Proof-of-AI consensus mechanism for SELF Chain.
//!
//! ## Key Components
//!
//! - **Validator**: Block and transaction validation with color markers
//! - **TransactionSelector**: 20/20/50/10 algorithm for fair block building
//! - **VotingSystem**: Decentralized voting for block selection
//! - **ValidationCache**: Performance optimization through caching
//!
//! ## v1 Spec-Compliant Types
//!
//! The `v1` submodule contains spec-compliant consensus types that match
//! `docs/POAI_SPECIFICATION.md`. See `v1::ConsensusConfig` for protocol constants.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use self_chain_core::consensus::{TransactionSelector, TransactionSelectorConfig};
//!
//! let config = TransactionSelectorConfig::default();
//! let selector = TransactionSelector::new(config);
//! let selected = selector.select_transactions(mempool)?;
//! ```

pub mod v1;

pub mod cache;
pub mod error;
pub mod metrics;
pub mod transaction_selector;
pub mod validator;
pub mod vote;
pub mod voting;

// Re-export key types
pub use cache::{ValidationCache, CacheConfig, CacheEntry};
pub use error::ConsensusError;
pub use metrics::ConsensusMetrics;
pub use transaction_selector::{
    TransactionSelector, TransactionSelectorConfig, TransactionWithMetadata,
    SelectedTransactions, BlockEfficiency,
};
pub use vote::{Vote, VotingResult};
pub use voting::VotingSystem;
