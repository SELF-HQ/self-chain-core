//! PoAI Node Types
//!
//! This module provides the three node types specified in the PoAI consensus mechanism:
//! - ValidatorNode: Lightweight nodes that vote and validate color markers
//! - BlockBuilderNode: Full nodes that build blocks using the 20/20/50/10 algorithm
//! - CoordinatorNode: Network service that organizes voting rounds

pub mod node_types;

pub use node_types::{
    NodeType, NodeConfig, ValidatorNode, BlockBuilderNode, CoordinatorNode,
    Vote, ValidatorStats, BlockProposal, BlockBuilderStats, VotingRound, VotingResult,
};
