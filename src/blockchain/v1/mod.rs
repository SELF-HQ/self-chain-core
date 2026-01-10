//! PoAI v1 Spec-Compliant Types
//!
//! This module contains the spec-compliant wire format types as defined in
//! `docs/POAI_SPECIFICATION.md`. These types represent the canonical v1
//! protocol structures.
//!
//! ## Comparison with Production Types
//!
//! The production `blockchain/mod.rs` uses simpler types for the current
//! WebSocket-based coordinator system. The v1 types here are spec-compliant
//! and will be used in the decentralized consensus implementation.
//!
//! | Field | Production | v1 Spec |
//! |-------|-----------|---------|
//! | `previous_hash` | `String` | `[u8; 32]` |
//! | `point_price` | Not present | `u64` |
//! | `chain_id` | Not present | `String` |
//! | `nonce` | Not present | `u64` |
//! | `state_root` | Not present | `[u8; 32]` |
//! | Hash algorithm | `DefaultHasher` | `SHA-256` with domain separation |
//!
//! ## Wire Format
//!
//! All types use **bincode** serialization with deterministic field ordering.
//! Signature domain separation prefixes:
//! - Block: `"self-chain-block-header-v1"`
//! - Transaction: `"self-chain-transaction-v1"`
//! - Prevote: `"self-chain-vote-prevote-v1"`
//! - Precommit: `"self-chain-vote-precommit-v1"`
//! - Proposal: `"self-chain-proposal-v1"`

pub mod block;
pub mod transaction;
pub mod vote;
pub mod proposal;

pub use block::{Block, BlockHeader, CommitSignature};
pub use transaction::Transaction;
pub use vote::{Vote, VoteStep};
pub use proposal::BlockProposal;
